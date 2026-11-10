//! Open spend-slot directory and the sequential page log behind `spent.body`.
//!
//! Confirm records 8-byte slots here, including a short tail page. Each dirty
//! page is composed into one image. A checkpoint rewrites `spend.pages` to the
//! images still named and renames `spend.pages.commit` before `spend_durable`
//! advances. A slot past a short image stays on `spent.body`. Fold copies a
//! cooled image onto `spent.body` after that marker, starting at byte 16 on
//! page 0 so the file header stays put. A commit that names bytes refuses a
//! missing or short image, including `spend.pages.{generation}` left when a
//! crash lands between the commit rename and the pages rename.

use crate::error::StoreError;
use crate::file::{fsync_parent_dir, write_synced_tmp_file, write_synced_tmp_rename};
use crate::tx_table::BODY_PAGE_SIZE;
use rbitcoin_primitives::{schema_file_openable, SCHEMA_VERSION, STORE_MAGIC};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};

pub const SPEND_PAGES_NAME: &str = "spend.pages";
pub const SPEND_PAGES_COMMIT_NAME: &str = "spend.pages.commit";

/// Open slot deltas that make the next confirm insert wait for a checkpoint.
pub const OPEN_EDGE_CAP: u64 = 2_000_000;
/// On-disk `spend.pages` length that asks for a checkpoint before the ten-minute timer.
pub const LOG_IMAGE_CAP: u64 = 1024 * 1024 * 1024;

const SLOT: usize = 8;
const COMMIT_HEAD: usize = 28;
const COMMIT_REC: usize = 24;

/// True when a spend checkpoint should run before the ten-minute timer.
pub fn spend_checkpoint_early(open_edges: u64, image_bytes: u64) -> bool {
    open_edges >= OPEN_EDGE_CAP || image_bytes >= LOG_IMAGE_CAP
}

/// True when a new note should wake the sync thread.
///
/// The image cap arms once. After that follow-up, a still-large file does
/// not arm again. The edge cap still does.
pub fn spend_note_arms(open_edges: u64, image_bytes: u64, followup_done: bool) -> bool {
    open_edges >= OPEN_EDGE_CAP || (image_bytes >= LOG_IMAGE_CAP && !followup_done)
}

/// `(request another checkpoint, follow-up already used for this oversized log)`.
///
/// Crossing the image cap schedules one extra checkpoint so the generation
/// that filled the file can cool and fold. It does not spin while the file
/// stays large. The edge cap keeps requesting until the open directory drains.
pub fn pace_after_checkpoint(
    open_edges: u64,
    image_bytes: u64,
    followup_done: bool,
) -> (bool, bool) {
    if open_edges >= OPEN_EDGE_CAP {
        return (true, followup_done);
    }
    if image_bytes >= LOG_IMAGE_CAP && !followup_done {
        return (true, true);
    }
    (false, image_bytes >= LOG_IMAGE_CAP)
}

/// `(page_off, log_off, image_len)` in one commit record.
type PageRec = (u64, u64, u32);
type OpenSlot = (u16, [u8; SLOT]);
type PageSlots = Vec<OpenSlot>;
type KeptPage = (u64, PageSlots);
/// Condvar `rbtc-spend-sync` waits on. The flag is the thread's stop bit.
pub type SpendSyncWake = Arc<(Mutex<bool>, Condvar)>;

struct OpenPage {
    slots: Vec<(u16, [u8; SLOT])>,
    base: Option<Arc<[u8]>>,
}

/// A slot `keep` rejected from an image that may already have been copied
/// onto `spent.body`.
struct BodyClear {
    page_off: u64,
    rel: u16,
    meta: [u8; SLOT],
}

struct CommittedPage {
    image: Arc<[u8]>,
    log_off: u64,
    generation: u32,
}

type SnapshotPages = Vec<(u64, OpenPage)>;
type PageImages = Vec<(u64, Arc<[u8]>)>;

struct PendingInstall {
    generation: u32,
    height: u32,
    pages: Vec<(u64, Arc<[u8]>, u64)>,
    rejected: Vec<u64>,
    kept: Vec<KeptPage>,
    /// New file offsets for pages this generation carried forward.
    carried_off: Vec<(u64, u64)>,
}

struct CommitSnapshot {
    pages: Vec<(u64, OpenPage)>,
    froze: bool,
}

/// `Clear` writes the commit with the held bit off. RAM still has the old
/// generation until that rename succeeds.
enum PublishHold {
    Sample,
    Clear,
}

struct Inner {
    /// Slots confirm has noted since the last freeze.
    open: HashMap<u64, OpenPage>,
    /// Notes taken while a held commit is filtered. They belong to a later
    /// checkpoint: this one already sampled its height.
    later: HashMap<u64, OpenPage>,
    /// `later` is the directory `note_slots` writes until the snapshot is copied.
    holding_filter: bool,
    /// The generation the sync thread is composing. Readers still see it.
    frozen: HashMap<u64, OpenPage>,
    committed: HashMap<u64, CommittedPage>,
    /// Pages copied onto `spent.body` and not yet dropped from the directory.
    cooling: Vec<(u64, u32)>,
    pending: Option<PendingInstall>,
    generation: u32,
    /// Fold copies a committed page only when its generation is at or below this.
    cool_through: u32,
    /// Snapshot height named by the on-disk commit, when one is loaded.
    commit_height: Option<u32>,
    /// Snapshot height of the compose in `prepare_commit`. Cleared when that
    /// call returns. A disconnect can see it before the commit is installed.
    compose_height: Option<u32>,
    /// Commit height was above the tip. Fold must not copy that image.
    fold_held: bool,
    open_edges: u64,
    log_bytes: u64,
}

struct SpendPace {
    early: AtomicBool,
    image_followup: AtomicBool,
    sync_wake: Mutex<Option<SpendSyncWake>>,
    room: Condvar,
    room_mu: Mutex<()>,
}

impl SpendPace {
    fn new() -> Self {
        Self {
            early: AtomicBool::new(false),
            image_followup: AtomicBool::new(false),
            sync_wake: Mutex::new(None),
            room: Condvar::new(),
            room_mu: Mutex::new(()),
        }
    }

    fn arm(&self) {
        self.early.store(true, Ordering::Release);
        let wake = self
            .sync_wake
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        let Some(wake) = wake else {
            return;
        };
        let (lock, cv) = &*wake;
        let _g = lock.lock().unwrap_or_else(|e| e.into_inner());
        cv.notify_one();
    }

    fn notify_room(&self) {
        let _g = self.room_mu.lock().unwrap_or_else(|e| e.into_inner());
        self.room.notify_all();
    }
}

pub struct SpendPageLog {
    dir: PathBuf,
    inner: Mutex<Inner>,
    /// Serializes `spend.pages.commit` renames and the unlink of a finished
    /// generation. Hold, publish, and delete take this lock before the
    /// directory lock.
    commit_io: Mutex<()>,
    pace: Arc<SpendPace>,
}

/// True when the 8-byte slot lies inside the published body, including the
/// short tail page. Confirm logs that slot. A slot past `published` is cold.
pub fn is_historical(published: u64, abs: u64) -> bool {
    abs.saturating_add(SLOT as u64) <= published
}

impl SpendPageLog {
    pub fn create(dir: &Path) -> Self {
        Self::empty(dir)
    }

    pub fn load(dir: &Path) -> Result<Self, StoreError> {
        let log = Self::empty(dir);
        log.reload()?;
        Ok(log)
    }

    fn empty(dir: &Path) -> Self {
        Self {
            dir: dir.to_path_buf(),
            commit_io: Mutex::new(()),
            inner: Mutex::new(Inner {
                open: HashMap::new(),
                later: HashMap::new(),
                holding_filter: false,
                frozen: HashMap::new(),
                committed: HashMap::new(),
                cooling: Vec::new(),
                pending: None,
                generation: 0,
                cool_through: 0,
                commit_height: None,
                compose_height: None,
                fold_held: false,
                open_edges: 0,
                log_bytes: 0,
            }),
            pace: Arc::new(SpendPace::new()),
        }
    }

    /// `rbtc-spend-sync` waits on `wake`. An edge or image cap notifies it.
    pub fn bind_sync_wake(&self, wake: SpendSyncWake) {
        *self
            .pace
            .sync_wake
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some(wake);
    }

    pub(crate) fn checkpoint_early(&self) -> bool {
        self.pace.early.load(Ordering::Acquire)
    }

    /// After a checkpoint, arm one follow-up when the log is still over the
    /// image cap, and keep arming while the open directory is over the edge cap.
    pub(crate) fn relax_pace(&self) {
        let (edges, images) = {
            let g = self.lock();
            (g.open_edges, g.log_bytes)
        };
        let follow = self.pace.image_followup.load(Ordering::Acquire);
        let (early, follow) = pace_after_checkpoint(edges, images, follow);
        self.pace.image_followup.store(follow, Ordering::Release);
        self.pace.early.store(early, Ordering::Release);
        if early {
            self.pace.arm();
        }
    }

    /// Block while the open directory is still at the edge cap.
    ///
    /// Returns at once when no sync thread is bound. Startup replay runs
    /// before that thread exists, and nothing else would wake this wait.
    /// Confirm calls this before it takes the utxo view. `note_slots` does
    /// not wait, so one batch may pass the cap and the next batch waits
    /// outside the view. The page-log lock is not held across the wait.
    /// `commit_open` notifies once it has frozen the directory.
    pub fn wait_open_edge_room(&self) {
        loop {
            let room = self.pace.room_mu.lock().unwrap_or_else(|e| e.into_inner());
            if self.lock().open_edges < OPEN_EDGE_CAP {
                return;
            }
            let bound = self
                .pace
                .sync_wake
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .is_some();
            if !bound {
                return;
            }
            self.pace.arm();
            drop(self.pace.room.wait(room).unwrap_or_else(|e| e.into_inner()));
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Current 8-byte image for `abs`, from the open directory, the frozen
    /// generation, or a committed page. `None` means the body still holds the slot.
    pub fn slot(&self, abs: u64) -> Option<[u8; SLOT]> {
        let page_off = abs & !(BODY_PAGE_SIZE - 1);
        let rel = ((abs - page_off) / SLOT as u64) as u16;
        let g = self.lock();
        if let Some(meta) = page_hit(g.later.get(&page_off), rel) {
            return Some(meta);
        }
        if let Some(meta) = page_hit(g.open.get(&page_off), rel) {
            return Some(meta);
        }
        if let Some(meta) = page_hit(g.frozen.get(&page_off), rel) {
            return Some(meta);
        }
        g.committed
            .get(&page_off)
            .and_then(|page| slot_in_image(&page.image, rel))
    }

    pub fn note_slots(&self, slots: &[(u64, [u8; SLOT])]) -> Result<(), StoreError> {
        if slots.is_empty() {
            return Ok(());
        }
        let mut added = false;
        {
            let mut g = self.lock();
            for &(abs, meta) in slots {
                let page_off = abs & !(BODY_PAGE_SIZE - 1);
                let rel = ((abs - page_off) / SLOT as u64) as u16;
                if (rel as usize) >= (BODY_PAGE_SIZE as usize / SLOT) {
                    return Err(StoreError::Corrupt("spend page slot past page"));
                }
                let base = if g.holding_filter || g.frozen.contains_key(&page_off) {
                    None
                } else {
                    g.committed.get(&page_off).map(|c| Arc::clone(&c.image))
                };
                let pages = if g.holding_filter {
                    &mut g.later
                } else {
                    &mut g.open
                };
                if insert_note(pages, page_off, rel, meta, base)? {
                    g.open_edges = g.open_edges.saturating_add(1);
                    added = true;
                }
            }
            let follow = self.pace.image_followup.load(Ordering::Acquire);
            added = added && spend_note_arms(g.open_edges, g.log_bytes, follow);
        }
        if added {
            self.pace.arm();
        }
        Ok(())
    }

    /// An installed commit, a compose still in flight, or a publish waiting to
    /// be installed names a height above `tip`. Keep serving those slots, and
    /// do not fold them. Pending does not invent a commit height: only an
    /// installed height is lowered to `tip`.
    pub fn hold_fold_above(&self, tip: u32) -> Result<(), StoreError> {
        let _io = self.commit_io.lock().unwrap_or_else(|e| e.into_inner());
        let path = self.dir.join(SPEND_PAGES_COMMIT_NAME);
        let (installed_above, in_flight_above) = {
            let g = self.lock();
            let in_flight = g.compose_height.is_some_and(|h| h > tip)
                || g.pending.as_ref().is_some_and(|p| p.height > tip);
            (g.commit_height.is_some_and(|h| h > tip), in_flight)
        };
        if !installed_above && !in_flight_above {
            return Ok(());
        }
        if path.exists() {
            let mut buf = std::fs::read(&path).map_err(|e| StoreError::io(&path, e))?;
            if buf.len() >= COMMIT_HEAD {
                buf[6..8].copy_from_slice(&1u16.to_le_bytes());
                if installed_above {
                    buf[8..12].copy_from_slice(&tip.to_le_bytes());
                }
                write_synced_tmp_rename(&path, &buf)?;
            }
        }
        let mut g = self.lock();
        if g.commit_height.is_some_and(|h| h > tip) {
            g.commit_height = Some(tip);
            g.fold_held = true;
        }
        if g.compose_height.is_some_and(|h| h > tip)
            || g.pending.as_ref().is_some_and(|p| p.height > tip)
        {
            g.fold_held = true;
        }
        Ok(())
    }

    /// Compose, publish, and install one generation.
    ///
    /// Callers that must hold the utxo view across the directory swap use
    /// [`Self::prepare_commit`] and [`Self::install_commit`] instead.
    pub fn commit_open<F, K>(
        &self,
        height: u32,
        published: u64,
        read_page: F,
        keep: K,
    ) -> Result<(), StoreError>
    where
        F: FnMut(u64, usize) -> Result<Vec<u8>, StoreError>,
        K: FnMut(&[u8; SLOT]) -> Result<bool, StoreError>,
    {
        self.prepare_commit(height, published, read_page, keep)?;
        self.install_commit()
    }

    /// Compose open pages into one generation and publish the commit record.
    ///
    /// `read_page` supplies neighbor bytes from `spent.body` when the page has
    /// no base image. `keep` drops a slot that is no longer best-chain so a
    /// later fold does not write it. An empty open set leaves an existing
    /// commit in place, unless that commit's height is above this checkpoint.
    /// A restart can load that height with the held bit still clear. Those
    /// slots are filtered before an empty open set marks the image cool.
    ///
    /// A frozen snapshot left by a failed compose is retried together with
    /// every note sitting in `open`. A page in both directories merges slots,
    /// and the later note wins on the same slot. Notes that arrive after this
    /// snapshot is copied stay in `open` for the next generation.
    ///
    /// A later generation rewrites `spend.pages` to the images still named.
    /// The previous file stays until that commit is renamed. A crash between
    /// the commit rename and the pages rename reloads `spend.pages.{generation}`.
    /// A page whose new notes are all rejected still names that older image.
    #[doc(hidden)]
    pub fn prepare_commit<F, K>(
        &self,
        height: u32,
        published: u64,
        mut read_page: F,
        mut keep: K,
    ) -> Result<(), StoreError>
    where
        F: FnMut(u64, usize) -> Result<Vec<u8>, StoreError>,
        K: FnMut(&[u8; SLOT]) -> Result<bool, StoreError>,
    {
        if self.lock().pending.is_some() {
            return Ok(());
        }
        {
            let mut g = self.lock();
            if g.commit_height.is_some_and(|committed| committed > height) {
                g.fold_held = true;
            }
            g.compose_height = Some(height);
            // Notes taken while `keep` walks the held image are not part of
            // this height. The walk does not hold the utxo view, so a
            // replacement block can land here.
            if g.fold_held {
                g.holding_filter = true;
            }
        }
        let _clear_compose = ClearCompose(self);
        let _release_later = ReleaseLater(self);
        let mut clears = self.explode_held(&mut keep)?;
        // Every slot was rejected. The commit file is still the retry record
        // until compose publishes a replacement or decides the body has none.
        let deferred = {
            let g = self.lock();
            g.fold_held && !g.committed.is_empty()
        };
        let stored = self.committed_images();
        let Some(snap) = self.snapshot_commit() else {
            self.scrub_open_notes(&mut keep)?;
            let pages = compose_kept_pages(&mut read_page, published, &[], &clears, &stored)?;
            return self.settle_deferred_clear(height, pages, deferred, &clears);
        };
        if snap.froze {
            self.pace.notify_room();
        }
        let (kept, rejected, more) = filter_kept_pages(&mut keep, snap.pages)?;
        clears.extend(more);
        let pages = compose_kept_pages(&mut read_page, published, &kept, &clears, &stored)?;
        if pages.is_empty() {
            self.drop_frozen_pages(&rejected);
            return Ok(());
        }
        self.scrub_open_notes(&mut keep)?;
        let mut replaced: Vec<u64> = clears.iter().map(|clear| clear.page_off).collect();
        replaced.extend(kept.iter().map(|(off, _)| *off));
        let carried = {
            let g = self.lock();
            let mut carried = carried_pages(&g.committed, &kept);
            if deferred {
                let skip: HashSet<u64> = replaced.iter().copied().collect();
                carried.retain(|(off, _)| !skip.contains(off));
            }
            carried
        };
        let hold = if deferred {
            PublishHold::Clear
        } else {
            PublishHold::Sample
        };
        self.store_pending(height, pages, rejected, kept, &carried, hold)?;
        if deferred {
            self.release_replaced(&replaced);
        }
        Ok(())
    }

    /// Publish a clearing image, or drop a rejected generation the body does not hold.
    ///
    /// A page this checkpoint did not reject stays named. Deleting the file, or
    /// keeping only a zero page, would serve that best-chain spend as unspent.
    /// The commit file stays in place until this returns. A `read_page` error
    /// never reaches here, so a restart still loads the old image.
    fn settle_deferred_clear(
        &self,
        height: u32,
        pages: PageImages,
        deferred: bool,
        clears: &[BodyClear],
    ) -> Result<(), StoreError> {
        let fresh: HashSet<u64> = pages.iter().map(|(off, _)| *off).collect();
        let cleared: HashSet<u64> = clears.iter().map(|clear| clear.page_off).collect();
        let carried = if deferred {
            self.carry_untouched(&fresh, &cleared)
        } else {
            Vec::new()
        };
        if pages.is_empty() && carried.is_empty() {
            if deferred {
                self.drop_rejected_generation();
            }
            return Ok(());
        }
        let hold = if deferred {
            PublishHold::Clear
        } else {
            PublishHold::Sample
        };
        self.store_pending(height, pages, Vec::new(), Vec::new(), &carried, hold)?;
        if deferred {
            let keep: Vec<u64> = carried.iter().map(|(off, _)| *off).collect();
            self.keep_only(&keep);
        }
        Ok(())
    }

    fn forget_held_directory(&self) {
        let mut g = self.lock();
        g.committed.clear();
        g.fold_held = false;
        g.commit_height = None;
    }

    fn drop_rejected_generation(&self) {
        self.forget_held_directory();
        self.delete_generation_files();
    }

    /// Drop pages this publish replaced. Carried pages stay, so a zero image
    /// that was not recomposed is still here for the next fold.
    fn release_replaced(&self, replaced: &[u64]) {
        let replaced: HashSet<u64> = replaced.iter().copied().collect();
        let mut g = self.lock();
        g.committed.retain(|off, _| !replaced.contains(off));
        g.fold_held = false;
    }

    fn keep_only(&self, keep: &[u64]) {
        let keep: HashSet<u64> = keep.iter().copied().collect();
        let mut g = self.lock();
        g.committed.retain(|off, _| keep.contains(off));
        g.fold_held = false;
    }

    fn committed_images(&self) -> HashMap<u64, Arc<[u8]>> {
        let g = self.lock();
        g.committed
            .iter()
            .map(|(off, page)| (*off, Arc::clone(&page.image)))
            .collect()
    }

    /// Committed pages this checkpoint did not recompose and did not reject.
    fn carry_untouched(
        &self,
        fresh: &HashSet<u64>,
        cleared: &HashSet<u64>,
    ) -> Vec<(u64, CommittedPage)> {
        let g = self.lock();
        let mut out = Vec::new();
        for (off, page) in &g.committed {
            if fresh.contains(off) || cleared.contains(off) {
                continue;
            }
            out.push((
                *off,
                CommittedPage {
                    image: Arc::clone(&page.image),
                    log_off: page.log_off,
                    generation: page.generation,
                },
            ));
        }
        out
    }

    /// Publish the composed generation into the directory.
    ///
    /// Call under `hold_utxo_view` so a reader cannot observe the swap. A hold
    /// taken while the generation was composed stays set: this does not clear it.
    #[doc(hidden)]
    pub fn install_commit(&self) -> Result<(), StoreError> {
        let pending = {
            let mut g = self.lock();
            g.pending.take()
        };
        let Some(pending) = pending else {
            return Ok(());
        };
        let kept_by_page: HashMap<u64, HashSet<OpenSlot>> = pending
            .kept
            .iter()
            .map(|(off, slots)| (*off, slots.iter().copied().collect()))
            .collect();
        {
            let mut g = self.lock();
            g.generation = pending.generation;
            if !g.fold_held {
                g.commit_height = Some(pending.height);
                g.cool_through = pending.generation.wrapping_sub(1);
            }
            for page_off in &pending.rejected {
                g.frozen.remove(page_off);
            }
            for (page_off, image, log_off) in pending.pages {
                g.frozen.remove(&page_off);
                let kept = kept_by_page.get(&page_off);
                let unchanged = match g.open.get(&page_off) {
                    None => true,
                    Some(page) => match kept {
                        Some(set) => {
                            page.slots.len() == set.len()
                                && page.slots.iter().all(|slot| set.contains(slot))
                        }
                        None => page.slots.is_empty(),
                    },
                };
                if unchanged {
                    if let Some(page) = g.open.remove(&page_off) {
                        g.open_edges = g.open_edges.saturating_sub(page.slots.len() as u64);
                    }
                } else if let Some(page) = g.open.get_mut(&page_off) {
                    let before = page.slots.len() as u64;
                    page.slots
                        .retain(|(rel, meta)| slot_from_image(&image, *rel) != *meta);
                    page.base = Some(Arc::clone(&image));
                    let left = page.slots.len() as u64;
                    g.open_edges = g.open_edges.saturating_sub(before.saturating_sub(left));
                    if left == 0 {
                        g.open.remove(&page_off);
                    }
                }
                // The published image stays reachable after restart even when
                // confirm noted a newer slot on the same page. Fold skips a
                // page that is still open, so this copy is not written early.
                g.committed.insert(
                    page_off,
                    CommittedPage {
                        image,
                        log_off,
                        generation: pending.generation,
                    },
                );
            }
            for (page_off, log_off) in pending.carried_off {
                if let Some(page) = g.committed.get_mut(&page_off) {
                    page.log_off = log_off;
                }
            }
        }
        self.pace.notify_room();
        Ok(())
    }

    /// A disconnect left this commit above the tip. Do not copy it onto the body.
    ///
    /// `keep` runs before any directory change. A `keep` error leaves the
    /// commit file and the committed images where they were. When a page
    /// still has a rejected slot, live slots are copied into `open` and the
    /// old image stays until that compose's rename returns. A zero image is
    /// folded. An image whose slots are all still best-chain stays in
    /// `committed` and is republished from that image, not from the body.
    fn explode_held<K>(&self, keep: &mut K) -> Result<Vec<BodyClear>, StoreError>
    where
        K: FnMut(&[u8; SLOT]) -> Result<bool, StoreError>,
    {
        let held: Vec<(u64, Arc<[u8]>)> = {
            let g = self.lock();
            if !g.fold_held {
                return Ok(Vec::new());
            }
            g.committed
                .iter()
                .map(|(off, page)| (*off, Arc::clone(&page.image)))
                .collect()
        };
        let mut restored: Vec<KeptPage> = Vec::new();
        let mut cleared: Vec<BodyClear> = Vec::new();
        for (page_off, image) in &held {
            let mut slots = Vec::new();
            let n = image.len() / SLOT;
            for rel in 0..n {
                if *page_off == 0 && rel < 2 {
                    continue;
                }
                let meta = slot_from_image(image, rel as u16);
                if meta == [0u8; SLOT] {
                    continue;
                }
                if !keep(&meta)? {
                    cleared.push(BodyClear {
                        page_off: *page_off,
                        rel: rel as u16,
                        meta,
                    });
                    continue;
                }
                slots.push((rel as u16, meta));
            }
            if !slots.is_empty() {
                restored.push((*page_off, slots));
            }
        }
        let mut g = self.lock();
        if !g.fold_held {
            return Ok(Vec::new());
        }
        Ok(apply_held_explode(&mut g, &held, restored, cleared))
    }

    /// Freeze a new open set, or reuse the snapshot a failed compose left.
    ///
    /// The freeze is a swap. Confirm notes the next generation into `open`
    /// while this one is composed. A leftover frozen map stays put, and the
    /// edges those later notes added stay counted.
    fn snapshot_commit(&self) -> Option<CommitSnapshot> {
        let mut g = self.lock();
        g.holding_filter = false;
        let reuse = !g.frozen.is_empty();
        if !reuse && g.open.is_empty() {
            if !g.fold_held {
                g.cool_through = g.generation;
            }
            merge_later(&mut g, false);
            return None;
        }
        let reset_edges = if !reuse {
            g.frozen = std::mem::take(&mut g.open);
            true
        } else {
            false
        };
        let mut pages: SnapshotPages = g
            .frozen
            .iter()
            .map(|(off, page)| {
                (
                    *off,
                    OpenPage {
                        slots: page.slots.clone(),
                        base: page.base.clone(),
                    },
                )
            })
            .collect();
        fold_open_into_snapshot(&mut pages, &g.open, &g.committed);
        merge_later(&mut g, reset_edges);
        Some(CommitSnapshot {
            pages,
            froze: !reuse,
        })
    }

    fn scrub_open_notes<K>(&self, keep: &mut K) -> Result<(), StoreError>
    where
        K: FnMut(&[u8; SLOT]) -> Result<bool, StoreError>,
    {
        // Slots noted during compose are not in the image. Drop ones `keep`
        // rejects before the directory swap, so a disconnected spend does not
        // become the committed image.
        let mut g = self.lock();
        let offs: Vec<u64> = g.open.keys().copied().collect();
        for off in offs {
            let mut dropped = 0u64;
            let remove = {
                let Some(page) = g.open.get_mut(&off) else {
                    continue;
                };
                let mut live = Vec::new();
                for slot in page.slots.drain(..) {
                    if keep(&slot.1)? {
                        live.push(slot);
                    } else {
                        dropped = dropped.saturating_add(1);
                    }
                }
                page.slots = live;
                page.slots.is_empty() && page.base.is_none()
            };
            g.open_edges = g.open_edges.saturating_sub(dropped);
            if remove {
                g.open.remove(&off);
            }
        }
        Ok(())
    }

    fn store_pending(
        &self,
        height: u32,
        pages: PageImages,
        rejected: Vec<u64>,
        kept: SnapshotPages,
        carried: &[(u64, CommittedPage)],
        hold: PublishHold,
    ) -> Result<(), StoreError> {
        let (generation, recs) = self.publish_generation(height, &pages, carried, hold)?;
        let kept_slots: Vec<KeptPage> = kept
            .iter()
            .map(|(off, page)| (*off, page.slots.clone()))
            .collect();
        let log_off_by_page: HashMap<u64, u64> = recs
            .iter()
            .map(|(off, log_off, _)| (*off, *log_off))
            .collect();
        let pending_pages: Vec<(u64, Arc<[u8]>, u64)> = pages
            .into_iter()
            .map(|(page_off, image)| {
                let log_off = log_off_by_page.get(&page_off).copied().unwrap_or(0);
                (page_off, image, log_off)
            })
            .collect();
        let carried_off: Vec<(u64, u64)> = carried
            .iter()
            .filter_map(|(off, _)| log_off_by_page.get(off).map(|log_off| (*off, *log_off)))
            .collect();
        let mut g = self.lock();
        g.pending = Some(PendingInstall {
            generation,
            height,
            pages: pending_pages,
            rejected,
            kept: kept_slots,
            carried_off,
        });
        Ok(())
    }

    fn drop_frozen_pages(&self, pages: &[u64]) {
        if pages.is_empty() {
            return;
        }
        let mut g = self.lock();
        for off in pages {
            g.frozen.remove(off);
        }
        drop(g);
        self.pace.notify_room();
    }

    /// Rewrite `spend.pages` to the images still named.
    ///
    /// Carried images are written first, in their previous order, then the
    /// new pages. The bytes go to `spend.pages.{generation}` and the commit
    /// names that blob before the generation file is renamed over
    /// `spend.pages`. A crash before the commit rename leaves the old commit,
    /// which does not name the generation file. A crash after it reloads that
    /// file. `commit_io` is held across the whole publish, so a fold hold
    /// cannot rename the commit between the sample and the commit write.
    fn publish_generation(
        &self,
        height: u32,
        pages: &[(u64, Arc<[u8]>)],
        carried: &[(u64, CommittedPage)],
        hold: PublishHold,
    ) -> Result<(u32, Vec<PageRec>), StoreError> {
        let _io = self.commit_io.lock().unwrap_or_else(|e| e.into_inner());
        let fresh: HashSet<u64> = pages.iter().map(|(off, _)| *off).collect();
        let mut carried_imgs: Vec<(u64, u64, Arc<[u8]>)> = carried
            .iter()
            .filter(|(off, _)| !fresh.contains(off))
            .map(|(off, page)| (*off, page.log_off, Arc::clone(&page.image)))
            .collect();
        carried_imgs.sort_by_key(|(_, log_off, _)| *log_off);
        let mut recs: Vec<PageRec> = Vec::new();
        let mut blob: Vec<u8> = Vec::new();
        for (page_off, _, image) in &carried_imgs {
            let log_off = blob.len() as u64;
            blob.extend_from_slice(image);
            recs.push((*page_off, log_off, image.len() as u32));
        }
        for (page_off, image) in pages {
            let log_off = blob.len() as u64;
            blob.extend_from_slice(image);
            recs.push((*page_off, log_off, image.len() as u32));
        }
        let generation = self.lock().generation.wrapping_add(1);
        let gen_path = self.dir.join(format!("{SPEND_PAGES_NAME}.{generation}"));
        let pages_path = self.dir.join(SPEND_PAGES_NAME);
        let commit_path = self.dir.join(SPEND_PAGES_COMMIT_NAME);
        write_synced_tmp_file(&gen_path, |f| {
            use std::io::Write;
            f.write_all(&blob).map_err(|e| StoreError::io(&gen_path, e))
        })?;
        let committed_len = blob.len() as u64;
        let held = {
            let mut g = self.lock();
            g.log_bytes = committed_len;
            match hold {
                PublishHold::Sample => g.fold_held,
                PublishHold::Clear => false,
            }
        };
        let commit = encode_commit(height, generation, committed_len, &recs, held);
        write_synced_tmp_rename(&commit_path, &commit)?;
        std::fs::rename(&gen_path, &pages_path).map_err(|e| StoreError::io(&pages_path, e))?;
        fsync_parent_dir(&self.dir).map_err(|e| StoreError::io(&self.dir, e))?;
        self.remove_page_sidecars();
        Ok((generation, recs))
    }

    fn delete_generation_files(&self) {
        // A hold that already sampled a live compose will rename this commit
        // back if it runs between the two unlinks. Keep the file names and the
        // RAM height behind the same lock the hold uses.
        let _io = self.commit_io.lock().unwrap_or_else(|e| e.into_inner());
        let _ = std::fs::remove_file(self.dir.join(SPEND_PAGES_COMMIT_NAME));
        let _ = std::fs::remove_file(self.dir.join(SPEND_PAGES_NAME));
        self.remove_page_sidecars();
        let mut g = self.lock();
        g.log_bytes = 0;
        g.commit_height = None;
    }

    /// Drop `spend.pages.{generation}` leftovers. The commit file stays.
    fn remove_page_sidecars(&self) {
        let Ok(rd) = std::fs::read_dir(&self.dir) else {
            return;
        };
        for ent in rd.flatten() {
            let name = ent.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            if name.starts_with("spend.pages.") && name != SPEND_PAGES_COMMIT_NAME {
                let _ = std::fs::remove_file(ent.path());
            }
        }
    }

    /// Copy cooled pages, then drop those directory entries.
    ///
    /// Callers that must not hold a reader lock across the body copy use
    /// [`Self::copy_cooled`] and [`Self::finish_cooled`] instead.
    pub fn fold_into<W, S>(&self, write_page: W, sync: S) -> Result<(), StoreError>
    where
        W: FnMut(u64, &[u8]) -> Result<(), StoreError>,
        S: FnMut() -> Result<(), StoreError>,
    {
        self.copy_cooled(write_page, sync)?;
        self.finish_cooled()
    }

    /// Copy committed pages that confirm has not re-dirtied onto `spent.body`.
    ///
    /// A held snapshot is not copied. The directory entries stay until
    /// [`Self::finish_cooled`], so a crash during the copy still serves the log.
    pub(crate) fn copy_cooled<W, S>(&self, mut write_page: W, mut sync: S) -> Result<(), StoreError>
    where
        W: FnMut(u64, &[u8]) -> Result<(), StoreError>,
        S: FnMut() -> Result<(), StoreError>,
    {
        let quiet: Vec<(u64, u32, Arc<[u8]>)> = {
            let mut g = self.lock();
            if g.fold_held {
                g.cooling.clear();
                return Ok(());
            }
            let cool_through = g.cool_through;
            let mut quiet: Vec<(u64, u32, Arc<[u8]>)> = g
                .committed
                .iter()
                .filter(|(off, page)| {
                    page.generation <= cool_through
                        && !g.open.contains_key(*off)
                        && !g.later.contains_key(*off)
                        && !g.frozen.contains_key(*off)
                })
                .map(|(off, page)| (*off, page.generation, Arc::clone(&page.image)))
                .collect();
            quiet.sort_by_key(|(off, _, _)| *off);
            quiet
        };
        if quiet.is_empty() {
            return Ok(());
        }
        let mut wrote: Vec<(u64, u32)> = Vec::new();
        for (off, generation, image) in quiet {
            {
                let g = self.lock();
                if g.fold_held {
                    break;
                }
                if g.open.contains_key(&off)
                    || g.later.contains_key(&off)
                    || g.frozen.contains_key(&off)
                {
                    continue;
                }
            }
            let (write_off, payload) = body_fold_span(off, &image);
            if payload.is_empty() {
                continue;
            }
            write_page(write_off, payload)?;
            wrote.push((off, generation));
        }
        if wrote.is_empty() {
            return Ok(());
        }
        sync()?;
        let mut g = self.lock();
        // Disconnect can hold the fold during the copy. Leave the log in place.
        if g.fold_held {
            g.cooling.clear();
            return Ok(());
        }
        g.cooling = wrote;
        Ok(())
    }

    /// Drop directory entries for pages [`Self::copy_cooled`] already synced.
    ///
    /// Does not delete the files while an open, frozen, pending, or in-flight
    /// compose still names slots. Those maps can be the only copy.
    pub(crate) fn finish_cooled(&self) -> Result<(), StoreError> {
        let mut g = self.lock();
        if g.fold_held {
            g.cooling.clear();
            return Ok(());
        }
        let wrote = std::mem::take(&mut g.cooling);
        if wrote.is_empty() {
            return Ok(());
        }
        for (off, generation) in wrote {
            if g.open.contains_key(&off) || g.frozen.contains_key(&off) {
                continue;
            }
            if g.committed
                .get(&off)
                .is_some_and(|page| page.generation == generation)
            {
                g.committed.remove(&off);
            }
        }
        if g.committed.is_empty()
            && g.open.is_empty()
            && g.later.is_empty()
            && g.frozen.is_empty()
            && g.pending.is_none()
            && g.compose_height.is_none()
        {
            g.commit_height = None;
            g.fold_held = false;
            drop(g);
            self.delete_generation_files();
        }
        Ok(())
    }

    fn reload(&self) -> Result<(), StoreError> {
        let commit_path = self.dir.join(SPEND_PAGES_COMMIT_NAME);
        if !commit_path.exists() {
            let _ = std::fs::remove_file(self.dir.join(SPEND_PAGES_NAME));
            self.remove_page_sidecars();
            return Ok(());
        }
        let buf = std::fs::read(&commit_path).map_err(|e| StoreError::io(&commit_path, e))?;
        let parsed = parse_commit(&buf)?;
        let gen_path = self
            .dir
            .join(format!("{SPEND_PAGES_NAME}.{}", parsed.generation));
        let pages_path = if gen_path.exists() {
            gen_path
        } else {
            self.dir.join(SPEND_PAGES_NAME)
        };
        let file = match std::fs::read(&pages_path) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                if parsed.committed_len == 0 && parsed.recs.is_empty() {
                    Vec::new()
                } else {
                    return Err(StoreError::Corrupt("spend.pages short"));
                }
            }
            Err(e) => return Err(StoreError::io(&pages_path, e)),
        };
        if (file.len() as u64) < parsed.committed_len {
            return Err(StoreError::Corrupt("spend.pages short"));
        }
        let limit = parsed.committed_len;
        for rec in &parsed.recs {
            let end = rec.log_off.saturating_add(u64::from(rec.image_len));
            if rec.log_off > limit || end > limit {
                return Err(StoreError::Corrupt(
                    "spend page commit names bytes past the committed length",
                ));
            }
        }
        let mut g = self.lock();
        g.generation = parsed.generation;
        g.cool_through = 0;
        g.commit_height = Some(parsed.height);
        g.fold_held = parsed.fold_held;
        g.log_bytes = parsed.committed_len;
        g.open_edges = 0;
        g.committed.clear();
        for rec in parsed.recs {
            let end = rec.log_off.saturating_add(u64::from(rec.image_len));
            let start = rec.log_off as usize;
            let image: Arc<[u8]> = Arc::from(file[start..end as usize].to_vec().into_boxed_slice());
            g.committed.insert(
                rec.page_off,
                CommittedPage {
                    image,
                    log_off: rec.log_off,
                    generation: parsed.generation,
                },
            );
        }
        Ok(())
    }
}

/// Clears `compose_height` on every `prepare_commit` return, including errors.
struct ClearCompose<'a>(&'a SpendPageLog);

impl Drop for ClearCompose<'_> {
    fn drop(&mut self) {
        self.0.lock().compose_height = None;
    }
}

/// Puts notes taken during a held filter back into `open` if `snapshot_commit`
/// did not.
struct ReleaseLater<'a>(&'a SpendPageLog);

impl Drop for ReleaseLater<'_> {
    fn drop(&mut self) {
        let mut g = self.0.lock();
        if g.holding_filter || !g.later.is_empty() {
            merge_later(&mut g, false);
        }
    }
}

/// Pull notes taken after a freeze into the snapshot compose will publish.
///
/// A restored page has no base. The stored image is still that base, including
/// a zero fold has not written. Compose zeros a rejected slot in that image
/// only when the body still holds it. The old image stays in `committed`
/// until the replacement rename returns.
fn fold_open_into_snapshot(
    pages: &mut SnapshotPages,
    open: &HashMap<u64, OpenPage>,
    committed: &HashMap<u64, CommittedPage>,
) {
    let mut index: HashMap<u64, usize> = HashMap::new();
    for (i, (off, _)) in pages.iter().enumerate() {
        index.entry(*off).or_insert(i);
    }
    for (off, open_page) in open {
        if let Some(&i) = index.get(off) {
            overlay_slots(&mut pages[i].1.slots, &open_page.slots);
        } else {
            index.insert(*off, pages.len());
            pages.push((
                *off,
                OpenPage {
                    slots: open_page.slots.clone(),
                    base: open_page.base.clone(),
                },
            ));
        }
    }
    for (off, page) in pages.iter_mut() {
        if !committed.contains_key(off) {
            page.base = None;
        } else if page.base.is_none() {
            // A note taken while the filter ran, or a page this checkpoint
            // is clearing, has no base. The stored image is still the page,
            // including a zero fold has not written. Compose zeros a
            // rejected slot in that image only when the body still holds it.
            if let Some(stored) = committed.get(off) {
                page.base = Some(Arc::clone(&stored.image));
            }
        }
    }
}

fn overlay_slots(dst: &mut PageSlots, src: &[(u16, [u8; SLOT])]) {
    let mut at: HashMap<u16, usize> = HashMap::new();
    for (i, (rel, _)) in dst.iter().enumerate() {
        at.entry(*rel).or_insert(i);
    }
    for (rel, meta) in src {
        if let Some(&i) = at.get(rel) {
            dst[i].1 = *meta;
        } else {
            at.insert(*rel, dst.len());
            dst.push((*rel, *meta));
        }
    }
}

/// Name a committed page this generation does not recompose.
///
/// `keep` can reject every new note on a page. The sealed slots stay in the
/// previous image. Leaving that page out of the new commit drops them before
/// fold copies the body.
fn carried_pages(
    committed: &HashMap<u64, CommittedPage>,
    kept: &[(u64, OpenPage)],
) -> Vec<(u64, CommittedPage)> {
    let recomposed: HashSet<u64> = kept.iter().map(|(off, _)| *off).collect();
    let mut out = Vec::new();
    for (off, page) in committed {
        if recomposed.contains(off) {
            continue;
        }
        out.push((
            *off,
            CommittedPage {
                image: Arc::clone(&page.image),
                log_off: page.log_off,
                generation: page.generation,
            },
        ));
    }
    out
}

fn filter_kept_pages<K>(
    keep: &mut K,
    snapshot: SnapshotPages,
) -> Result<(SnapshotPages, Vec<u64>, Vec<BodyClear>), StoreError>
where
    K: FnMut(&[u8; SLOT]) -> Result<bool, StoreError>,
{
    let mut kept = Vec::new();
    let mut rejected = Vec::new();
    let mut clears = Vec::new();
    for (page_off, mut page) in snapshot {
        let from_body = page.base.is_none();
        let mut slots = Vec::new();
        for slot in page.slots.drain(..) {
            if keep(&slot.1)? {
                slots.push(slot);
            } else if from_body {
                clears.push(BodyClear {
                    page_off,
                    rel: slot.0,
                    meta: slot.1,
                });
            }
        }
        page.slots = slots;
        if page.slots.is_empty() {
            rejected.push(page_off);
        } else {
            kept.push((page_off, page));
        }
    }
    Ok((kept, rejected, clears))
}

fn compose_kept_pages<F>(
    read_page: &mut F,
    published: u64,
    kept: &[(u64, OpenPage)],
    clears: &[BodyClear],
    stored: &HashMap<u64, Arc<[u8]>>,
) -> Result<PageImages, StoreError>
where
    F: FnMut(u64, usize) -> Result<Vec<u8>, StoreError>,
{
    let mut clear_at: HashMap<u64, Vec<(u16, [u8; SLOT])>> = HashMap::new();
    for clear in clears {
        clear_at
            .entry(clear.page_off)
            .or_default()
            .push((clear.rel, clear.meta));
    }
    let mut offs: Vec<u64> = kept.iter().map(|(off, _)| *off).collect();
    for off in clear_at.keys() {
        if !offs.contains(off) {
            offs.push(*off);
        }
    }
    offs.sort_unstable();
    offs.dedup();
    let kept_at: HashMap<u64, &OpenPage> = kept.iter().map(|(off, page)| (*off, page)).collect();
    let mut pages = Vec::with_capacity(offs.len());
    for page_off in offs {
        if let Some(image) = compose_one_page(
            &mut *read_page,
            published,
            page_off,
            kept_at.get(&page_off).copied(),
            clear_at.get(&page_off),
            stored.get(&page_off).map(Arc::as_ref),
        )? {
            pages.push((page_off, image));
        }
    }
    Ok(pages)
}

/// `None` when the page has no kept slot, the body does not still hold a
/// rejected spend, and the stored image is not hiding a body spend with a
/// zero. A matching body slot is zeroed in the stored image so the next
/// fold clears it. The body is not the base when an image is already stored.
fn compose_one_page<F>(
    read_page: &mut F,
    published: u64,
    page_off: u64,
    kept: Option<&OpenPage>,
    clears: Option<&Vec<(u16, [u8; SLOT])>>,
    stored: Option<&[u8]>,
) -> Result<Option<Arc<[u8]>>, StoreError>
where
    F: FnMut(u64, usize) -> Result<Vec<u8>, StoreError>,
{
    if page_off >= published {
        return Err(StoreError::Corrupt(
            "spend page log composed past the published body",
        ));
    }
    let len = BODY_PAGE_SIZE.min(published - page_off) as usize;
    let image = kept.and_then(|page| page.base.as_deref()).or(stored);
    let short = image.is_some_and(|image| image.len() < len);
    let body = if image.is_none() || clears.is_some() || short {
        Some(fit_page(read_page(page_off, len)?, len))
    } else {
        None
    };
    let mut buf = if let Some(image) = image {
        image_with_body_tail(image, body.as_deref(), len)
    } else {
        body.clone().unwrap_or_else(|| vec![0u8; len])
    };
    if let Some(page) = kept {
        for &(rel, meta) in &page.slots {
            write_slot(&mut buf, rel, &meta)?;
        }
    }
    let mut zeroed = false;
    if let Some(body) = body.as_deref() {
        if let Some(clears) = clears {
            for &(rel, meta) in clears {
                if kept.is_some_and(|page| page.slots.iter().any(|(r, _)| *r == rel)) {
                    continue;
                }
                if slot_from_image(body, rel) == meta {
                    write_slot(&mut buf, rel, &[0u8; SLOT])?;
                    zeroed = true;
                } else if image.is_some() {
                    let on_body = slot_from_image(body, rel);
                    write_slot(&mut buf, rel, &on_body)?;
                }
            }
        }
    }
    let hides = match (image, body.as_deref()) {
        (Some(image), Some(body)) => image_hides_body(page_off, image, body, len),
        _ => false,
    };
    let keep_page = kept.is_some_and(|page| !page.slots.is_empty()) || zeroed || hides;
    if !keep_page {
        return Ok(None);
    }
    Ok(Some(Arc::from(buf.into_boxed_slice())))
}

fn fit_page(mut buf: Vec<u8>, len: usize) -> Vec<u8> {
    if buf.len() < len {
        buf.resize(len, 0);
    }
    buf.truncate(len);
    buf
}

/// Copy `image`, then the body bytes past it.
///
/// A short tail padded with zeros would hide a later body slot, and the next
/// fold would erase that spend.
fn image_with_body_tail(image: &[u8], body: Option<&[u8]>, len: usize) -> Vec<u8> {
    let take = len.min(image.len());
    let mut buf = image[..take].to_vec();
    if buf.len() < len {
        if let Some(body) = body {
            let from = buf.len().min(body.len());
            let to = len.min(body.len());
            if from < to {
                buf.extend_from_slice(&body[from..to]);
            }
        }
        if buf.len() < len {
            buf.resize(len, 0);
        }
    }
    buf.truncate(len);
    buf
}

/// A zero in the stored image is hiding a spend fold has not written.
///
/// Bytes past the stored image are not part of it. The body still holds
/// those slots.
fn image_hides_body(page_off: u64, image: &[u8], body: &[u8], len: usize) -> bool {
    let n = (image.len() / SLOT).min(len / SLOT);
    (0..n).any(|rel| {
        if page_off == 0 && rel < 2 {
            return false;
        }
        let rel = rel as u16;
        slot_from_image(image, rel) == [0u8; SLOT] && slot_from_image(body, rel) != [0u8; SLOT]
    })
}

fn write_slot(buf: &mut [u8], rel: u16, meta: &[u8; SLOT]) -> Result<(), StoreError> {
    let i = rel as usize * SLOT;
    if i + SLOT > buf.len() {
        return Err(StoreError::Corrupt("spend page slot past published"));
    }
    buf[i..i + SLOT].copy_from_slice(meta);
    Ok(())
}

fn insert_note(
    pages: &mut HashMap<u64, OpenPage>,
    page_off: u64,
    rel: u16,
    meta: [u8; SLOT],
    base_if_new: Option<Arc<[u8]>>,
) -> Result<bool, StoreError> {
    pages.entry(page_off).or_insert_with(|| OpenPage {
        slots: Vec::new(),
        base: base_if_new,
    });
    let Some(page) = pages.get_mut(&page_off) else {
        return Err(StoreError::Corrupt("spend page directory missing"));
    };
    if let Some(slot) = page.slots.iter_mut().find(|(r, _)| *r == rel) {
        slot.1 = meta;
        return Ok(false);
    }
    page.slots.push((rel, meta));
    Ok(true)
}

/// Notes taken during a held filter join `open` after the snapshot is copied.
///
/// `reset_edges` is the freeze path: `open` was just emptied, so the edge
/// count starts at the notes that waited in `later`.
fn merge_later(g: &mut Inner, reset_edges: bool) {
    g.holding_filter = false;
    let later = std::mem::take(&mut g.later);
    if reset_edges {
        g.open_edges = 0;
    }
    for (off, page) in later {
        let n = page.slots.len() as u64;
        if let Some(dst) = g.open.get_mut(&off) {
            let before = dst.slots.len() as u64;
            overlay_slots(&mut dst.slots, &page.slots);
            let added = (dst.slots.len() as u64).saturating_sub(before);
            if reset_edges {
                g.open_edges = g.open_edges.saturating_add(dst.slots.len() as u64);
            } else {
                g.open_edges = g.open_edges.saturating_sub(n.saturating_sub(added));
            }
        } else if reset_edges {
            g.open_edges = g.open_edges.saturating_add(n);
            g.open.insert(off, page);
        } else {
            g.open.insert(off, page);
        }
    }
}

fn page_hit(page: Option<&OpenPage>, rel: u16) -> Option<[u8; SLOT]> {
    let page = page?;
    if let Some((_, meta)) = page.slots.iter().find(|(r, _)| *r == rel) {
        return Some(*meta);
    }
    page.base
        .as_deref()
        .and_then(|base| slot_in_image(base, rel))
}

/// `None` when `rel` sits past the stored image. The body still holds it.
fn slot_in_image(image: &[u8], rel: u16) -> Option<[u8; SLOT]> {
    let i = rel as usize * SLOT;
    if i + SLOT > image.len() {
        return None;
    }
    Some(slot_from_image(image, rel))
}

/// Page 0's first 16 bytes are the file header. Fold never pwrites them.
fn body_fold_span(page_off: u64, image: &[u8]) -> (u64, &[u8]) {
    if page_off == 0 {
        let skip = crate::file::FILE_HEADER_LEN.min(image.len());
        return (skip as u64, &image[skip..]);
    }
    (page_off, image)
}

/// Decide what a held image does at this checkpoint.
///
/// A pure-zero image is folded: the file stays, and nothing is rebuilt from
/// the body. An image that still has a spend, and no rejected slot, moves
/// those slots into `open` but stays in `committed`. Compose copies that
/// image, so a zero on the page is kept, and this checkpoint does not fold
/// it. A rejected slot leaves the old image in place and clears `base` so
/// compose reads the body. A frozen note on the same page keeps the live slots.
fn apply_held_explode(
    g: &mut Inner,
    held: &[(u64, Arc<[u8]>)],
    restored: Vec<KeptPage>,
    cleared: Vec<BodyClear>,
) -> Vec<BodyClear> {
    if cleared.is_empty() {
        let spend_offs: HashSet<u64> = held
            .iter()
            .filter(|(off, image)| image_has_spend(*off, image))
            .map(|(off, _)| *off)
            .collect();
        if spend_offs.is_empty() {
            g.fold_held = false;
            return Vec::new();
        }
        g.fold_held = false;
        g.commit_height = None;
        for (page_off, slots) in restored {
            if spend_offs.contains(&page_off) {
                park_open_slots(g, page_off, slots);
            }
        }
        return Vec::new();
    }
    let cleared_offs: HashSet<u64> = cleared.iter().map(|clear| clear.page_off).collect();
    for off in &cleared_offs {
        if let Some(page) = g.open.get_mut(off) {
            page.base = None;
        }
        if let Some(page) = g.frozen.get_mut(off) {
            page.base = None;
        }
    }
    for (page_off, slots) in restored {
        if !cleared_offs.contains(&page_off) {
            continue;
        }
        if let Some(page) = g.frozen.get_mut(&page_off) {
            overlay_missing_slots(page, slots);
            page.base = None;
        } else {
            park_open_slots(g, page_off, slots);
        }
    }
    cleared
}

/// Copy slots that this page does not already name. A newer note stays.
fn overlay_missing_slots(page: &mut OpenPage, slots: PageSlots) -> u64 {
    let mut have: HashSet<u16> = page.slots.iter().map(|(rel, _)| *rel).collect();
    let mut added = 0u64;
    for (rel, meta) in slots {
        if !have.insert(rel) {
            continue;
        }
        page.slots.push((rel, meta));
        added = added.saturating_add(1);
    }
    added
}

fn park_open_slots(g: &mut Inner, page_off: u64, slots: PageSlots) {
    if let Some(page) = g.open.get_mut(&page_off) {
        let added = overlay_missing_slots(page, slots);
        page.base = None;
        g.open_edges = g.open_edges.saturating_add(added);
    } else {
        g.open_edges = g.open_edges.saturating_add(slots.len() as u64);
        g.open.insert(page_off, OpenPage { slots, base: None });
    }
}

fn image_has_spend(page_off: u64, image: &[u8]) -> bool {
    let n = image.len() / SLOT;
    (0..n).any(|rel| {
        if page_off == 0 && rel < 2 {
            return false;
        }
        slot_from_image(image, rel as u16) != [0u8; SLOT]
    })
}

fn slot_from_image(image: &[u8], rel: u16) -> [u8; SLOT] {
    let i = rel as usize * SLOT;
    let mut out = [0u8; SLOT];
    if i + SLOT <= image.len() {
        out.copy_from_slice(&image[i..i + SLOT]);
    }
    out
}

struct CommitRec {
    page_off: u64,
    log_off: u64,
    image_len: u32,
}

struct ParsedCommit {
    height: u32,
    generation: u32,
    committed_len: u64,
    fold_held: bool,
    recs: Vec<CommitRec>,
}

fn encode_commit(
    height: u32,
    generation: u32,
    committed_len: u64,
    recs: &[(u64, u64, u32)],
    fold_held: bool,
) -> Vec<u8> {
    let mut buf = vec![0u8; COMMIT_HEAD + recs.len() * COMMIT_REC];
    buf[..4].copy_from_slice(&STORE_MAGIC);
    buf[4..6].copy_from_slice(&SCHEMA_VERSION.to_le_bytes());
    if fold_held {
        buf[6..8].copy_from_slice(&1u16.to_le_bytes());
    }
    buf[8..12].copy_from_slice(&height.to_le_bytes());
    buf[12..16].copy_from_slice(&generation.to_le_bytes());
    buf[16..24].copy_from_slice(&committed_len.to_le_bytes());
    buf[24..28].copy_from_slice(&(recs.len() as u32).to_le_bytes());
    for (i, (page_off, log_off, image_len)) in recs.iter().enumerate() {
        let at = COMMIT_HEAD + i * COMMIT_REC;
        buf[at..at + 8].copy_from_slice(&page_off.to_le_bytes());
        buf[at + 8..at + 16].copy_from_slice(&log_off.to_le_bytes());
        buf[at + 16..at + 20].copy_from_slice(&image_len.to_le_bytes());
    }
    buf
}

// llvm-cov drops the counters when this parser is inlined into `reload`.
#[inline(never)]
fn parse_commit(buf: &[u8]) -> Result<ParsedCommit, StoreError> {
    if buf.len() < COMMIT_HEAD {
        return Err(StoreError::Corrupt("spend.pages.commit short"));
    }
    if buf[..4] != STORE_MAGIC {
        return Err(StoreError::BadMagic);
    }
    let ver = u16::from_le_bytes(buf[4..6].try_into().unwrap());
    if !schema_file_openable(ver) {
        return Err(StoreError::BadSchema(ver));
    }
    let fold_held = u16::from_le_bytes(buf[6..8].try_into().unwrap()) != 0;
    let height = u32::from_le_bytes(buf[8..12].try_into().unwrap());
    let generation = u32::from_le_bytes(buf[12..16].try_into().unwrap());
    let committed_len = u64::from_le_bytes(buf[16..24].try_into().unwrap());
    let n = u32::from_le_bytes(buf[24..28].try_into().unwrap()) as usize;
    let need = COMMIT_HEAD.saturating_add(n.saturating_mul(COMMIT_REC));
    if buf.len() < need {
        return Err(StoreError::Corrupt("spend.pages.commit short"));
    }
    let mut recs = Vec::with_capacity(n);
    for i in 0..n {
        let at = COMMIT_HEAD + i * COMMIT_REC;
        recs.push(CommitRec {
            page_off: u64::from_le_bytes(buf[at..at + 8].try_into().unwrap()),
            log_off: u64::from_le_bytes(buf[at + 8..at + 16].try_into().unwrap()),
            image_len: u32::from_le_bytes(buf[at + 16..at + 20].try_into().unwrap()),
        });
    }
    Ok(ParsedCommit {
        height,
        generation,
        committed_len,
        fold_held,
        recs,
    })
}
