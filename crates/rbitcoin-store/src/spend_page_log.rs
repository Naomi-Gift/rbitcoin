//! Open spend-slot directory and the sequential page log behind `spent.body`.
//!
//! Confirm records 8-byte slots here. A full historical page is composed into
//! `spend.pages` and named by `spend.pages.commit` before `spend_durable`
//! advances. Fold copies cooled images onto `spent.body` after that marker.
//! The live tail page is not logged: Class A still owns those bytes.

use crate::error::StoreError;
use crate::file::{write_synced_tmp_file, write_synced_tmp_rename};
use crate::tx_table::BODY_PAGE_SIZE;
use rbitcoin_primitives::{schema_file_openable, SCHEMA_VERSION, STORE_MAGIC};
use std::collections::HashMap;
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

struct CommittedPage {
    image: Arc<[u8]>,
    log_off: u64,
    generation: u32,
}

struct PendingInstall {
    generation: u32,
    height: u32,
    pages: Vec<(u64, Arc<[u8]>, u64)>,
    rejected: Vec<u64>,
    kept: Vec<KeptPage>,
}

struct Inner {
    /// Slots confirm has noted since the last freeze.
    open: HashMap<u64, OpenPage>,
    /// The generation the sync thread is composing. Readers still see it.
    frozen: HashMap<u64, OpenPage>,
    committed: HashMap<u64, CommittedPage>,
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
    pace: Arc<SpendPace>,
}

/// A page whose end is still inside the published body. The partial last page
/// is the tail Class A appends.
pub fn is_historical(published: u64, abs: u64) -> bool {
    let page_off = abs & !(BODY_PAGE_SIZE - 1);
    page_off.saturating_add(BODY_PAGE_SIZE) <= published
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
            inner: Mutex::new(Inner {
                open: HashMap::new(),
                frozen: HashMap::new(),
                committed: HashMap::new(),
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
    pub(crate) fn bind_sync_wake(&self, wake: SpendSyncWake) {
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
    /// The page-log lock is not held across the wait. `commit_open` notifies
    /// once it has frozen the directory into the generation being composed.
    pub fn wait_open_edge_room(&self) {
        loop {
            let room = self.pace.room_mu.lock().unwrap_or_else(|e| e.into_inner());
            if self.lock().open_edges < OPEN_EDGE_CAP {
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
        if let Some(meta) = page_hit(g.open.get(&page_off), rel) {
            return Some(meta);
        }
        if let Some(meta) = page_hit(g.frozen.get(&page_off), rel) {
            return Some(meta);
        }
        g.committed
            .get(&page_off)
            .map(|page| slot_from_image(&page.image, rel))
    }

    pub fn note_slots(&self, slots: &[(u64, [u8; SLOT])]) -> Result<(), StoreError> {
        if slots.is_empty() {
            return Ok(());
        }
        self.wait_open_edge_room();
        let mut added = false;
        {
            let mut g = self.lock();
            for &(abs, meta) in slots {
                let page_off = abs & !(BODY_PAGE_SIZE - 1);
                let rel = ((abs - page_off) / SLOT as u64) as u16;
                if (rel as usize) >= (BODY_PAGE_SIZE as usize / SLOT) {
                    return Err(StoreError::Corrupt("spend page slot past page"));
                }
                if !g.open.contains_key(&page_off) {
                    let base = if g.frozen.contains_key(&page_off) {
                        None
                    } else {
                        g.committed.get(&page_off).map(|c| Arc::clone(&c.image))
                    };
                    g.open.insert(
                        page_off,
                        OpenPage {
                            slots: Vec::new(),
                            base,
                        },
                    );
                }
                let Some(page) = g.open.get_mut(&page_off) else {
                    return Err(StoreError::Corrupt("spend page directory missing"));
                };
                if let Some(slot) = page.slots.iter_mut().find(|(r, _)| *r == rel) {
                    slot.1 = meta;
                } else {
                    page.slots.push((rel, meta));
                    g.open_edges = g.open_edges.saturating_add(1);
                    added = true;
                }
            }
            added = added && spend_checkpoint_early(g.open_edges, g.log_bytes);
        }
        if added {
            self.pace.arm();
        }
        Ok(())
    }

    /// An installed commit, or a compose still in flight, names a height above
    /// `tip`. Keep serving those slots, and do not fold them. An in-flight
    /// compose does not invent a commit height: the publish that is already
    /// running records the hold.
    pub fn hold_fold_above(&self, tip: u32) -> Result<(), StoreError> {
        let path = self.dir.join(SPEND_PAGES_COMMIT_NAME);
        let (installed_above, composing_above) = {
            let g = self.lock();
            (
                g.commit_height.is_some_and(|h| h > tip),
                g.compose_height.is_some_and(|h| h > tip),
            )
        };
        if !installed_above && !composing_above {
            return Ok(());
        }
        if installed_above && path.exists() {
            let mut buf = std::fs::read(&path).map_err(|e| StoreError::io(&path, e))?;
            if buf.len() >= COMMIT_HEAD {
                buf[6..8].copy_from_slice(&1u16.to_le_bytes());
                buf[8..12].copy_from_slice(&tip.to_le_bytes());
                write_synced_tmp_rename(&path, &buf)?;
            }
        }
        let mut g = self.lock();
        if g.commit_height.is_some_and(|h| h > tip) {
            g.commit_height = Some(tip);
            g.fold_held = true;
        }
        if g.compose_height.is_some_and(|h| h > tip) {
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
    /// commit in place, unless a held snapshot has to be filtered first.
    /// A frozen snapshot left by a failed compose is retried. Notes taken
    /// after that failure stay in `open` and do not replace it.
    ///
    /// A second generation appends. It does not replace bytes an older commit
    /// still names.
    pub(crate) fn prepare_commit<F, K>(
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
            g.compose_height = Some(height);
        }
        let _clear_compose = ClearCompose(self);
        self.explode_held(&mut keep)?;
        let (snapshot, carried, froze) = {
            let mut g = self.lock();
            let reuse = !g.frozen.is_empty();
            if !reuse && g.open.is_empty() {
                if !g.fold_held {
                    g.cool_through = g.generation;
                }
                return Ok(());
            }
            // Freeze is a swap, not a clone of the live map. Confirm notes the
            // next generation into `open` while this snapshot is composed.
            // A leftover frozen map is that snapshot: do not drop it for a
            // newer open set, and do not zero the edges those notes added.
            if !reuse {
                g.frozen = std::mem::take(&mut g.open);
                g.open_edges = 0;
            }
            let snapshot: Vec<(u64, OpenPage)> = g
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
            let carried: Vec<(u64, CommittedPage)> = g
                .committed
                .iter()
                .filter(|(off, _)| !g.frozen.contains_key(*off) && !g.open.contains_key(*off))
                .map(|(off, page)| {
                    (
                        *off,
                        CommittedPage {
                            image: Arc::clone(&page.image),
                            log_off: page.log_off,
                            generation: page.generation,
                        },
                    )
                })
                .collect();
            (snapshot, carried, !reuse)
        };
        if froze {
            self.pace.notify_room();
        }
        let mut kept: Vec<(u64, OpenPage)> = Vec::new();
        let mut rejected: Vec<u64> = Vec::new();
        for (page_off, mut page) in snapshot {
            let mut slots = Vec::new();
            for slot in page.slots.drain(..) {
                if keep(&slot.1)? {
                    slots.push(slot);
                }
            }
            page.slots = slots;
            if page.slots.is_empty() {
                rejected.push(page_off);
            } else {
                kept.push((page_off, page));
            }
        }
        if kept.is_empty() {
            self.drop_frozen_pages(&rejected);
            return Ok(());
        }
        let mut pages: Vec<(u64, Arc<[u8]>)> = Vec::with_capacity(kept.len());
        for (page_off, page) in &kept {
            if page_off.saturating_add(BODY_PAGE_SIZE) > published {
                return Err(StoreError::Corrupt("spend page log composed the live tail"));
            }
            let len = BODY_PAGE_SIZE.min(published - page_off) as usize;
            let mut buf = if let Some(base) = &page.base {
                base[..len.min(base.len())].to_vec()
            } else {
                read_page(*page_off, len)?
            };
            if buf.len() < len {
                buf.resize(len, 0);
            }
            buf.truncate(len);
            for &(rel, meta) in &page.slots {
                let i = rel as usize * SLOT;
                buf[i..i + SLOT].copy_from_slice(&meta);
            }
            pages.push((*page_off, Arc::from(buf.into_boxed_slice())));
        }
        pages.sort_by_key(|(off, _)| *off);

        // Slots noted during compose are not in the image. Drop ones `keep`
        // rejects before the directory swap, so a disconnected spend does not
        // become the committed image.
        {
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
        }

        let (generation, recs) = self.publish_generation(height, &pages, &carried)?;
        let kept_slots: Vec<KeptPage> = kept
            .iter()
            .map(|(off, page)| (*off, page.slots.clone()))
            .collect();
        let pending_pages: Vec<(u64, Arc<[u8]>, u64)> = pages
            .into_iter()
            .map(|(page_off, image)| {
                let log_off = recs
                    .iter()
                    .find(|(off, _, _)| *off == page_off)
                    .map(|(_, log_off, _)| *log_off)
                    .unwrap_or(0);
                (page_off, image, log_off)
            })
            .collect();
        {
            let mut g = self.lock();
            g.pending = Some(PendingInstall {
                generation,
                height,
                pages: pending_pages,
                rejected,
                kept: kept_slots,
            });
        }
        Ok(())
    }

    /// Publish the composed generation into the directory.
    ///
    /// Call under `hold_utxo_view` so a reader cannot observe the swap. A hold
    /// taken while the generation was composed stays set: this does not clear it.
    pub(crate) fn install_commit(&self) -> Result<(), StoreError> {
        let pending = {
            let mut g = self.lock();
            g.pending.take()
        };
        let Some(pending) = pending else {
            return Ok(());
        };
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
                let kept = pending
                    .kept
                    .iter()
                    .find(|(off, _)| *off == page_off)
                    .map(|(_, slots)| slots.as_slice())
                    .unwrap_or(&[]);
                let unchanged = match g.open.get(&page_off) {
                    None => true,
                    Some(page) => {
                        page.slots.len() == kept.len()
                            && page.slots.iter().all(|slot| kept.contains(slot))
                    }
                };
                if unchanged {
                    if let Some(page) = g.open.remove(&page_off) {
                        g.open_edges = g.open_edges.saturating_sub(page.slots.len() as u64);
                    }
                    g.committed.insert(
                        page_off,
                        CommittedPage {
                            image,
                            log_off,
                            generation: pending.generation,
                        },
                    );
                } else {
                    let removed = {
                        let Some(page) = g.open.get_mut(&page_off) else {
                            continue;
                        };
                        let before = page.slots.len() as u64;
                        page.slots
                            .retain(|(rel, meta)| slot_from_image(&image, *rel) != *meta);
                        page.base = Some(Arc::clone(&image));
                        let left = page.slots.len() as u64;
                        (before.saturating_sub(left), left == 0)
                    };
                    g.open_edges = g.open_edges.saturating_sub(removed.0);
                    if removed.1 {
                        g.open.remove(&page_off);
                        g.committed.insert(
                            page_off,
                            CommittedPage {
                                image,
                                log_off,
                                generation: pending.generation,
                            },
                        );
                    }
                }
            }
        }
        self.pace.notify_room();
        Ok(())
    }

    /// A disconnect left this commit above the tip. Do not copy it onto the body.
    fn explode_held<K>(&self, keep: &mut K) -> Result<(), StoreError>
    where
        K: FnMut(&[u8; SLOT]) -> Result<bool, StoreError>,
    {
        let mut g = self.lock();
        if !g.fold_held {
            return Ok(());
        }
        let held: Vec<(u64, Arc<[u8]>)> = g
            .committed
            .drain()
            .map(|(off, page)| (off, page.image))
            .collect();
        g.fold_held = false;
        g.commit_height = None;
        for (page_off, image) in held {
            let mut slots = Vec::new();
            let n = image.len() / SLOT;
            for rel in 0..n {
                if page_off == 0 && rel < 2 {
                    continue;
                }
                let meta = slot_from_image(&image, rel as u16);
                if meta == [0u8; SLOT] || !keep(&meta)? {
                    continue;
                }
                slots.push((rel as u16, meta));
            }
            if slots.is_empty() {
                continue;
            }
            if g.open.contains_key(&page_off) {
                let mut added = 0u64;
                {
                    let page = g
                        .open
                        .get_mut(&page_off)
                        .expect("spend page open after contains");
                    for (rel, meta) in slots {
                        if page.slots.iter().any(|(r, _)| *r == rel) {
                            continue;
                        }
                        page.slots.push((rel, meta));
                        added = added.saturating_add(1);
                    }
                }
                g.open_edges = g.open_edges.saturating_add(added);
            } else {
                g.open_edges = g.open_edges.saturating_add(slots.len() as u64);
                g.open.insert(page_off, OpenPage { slots, base: None });
            }
        }
        let empty = g.open.is_empty() && g.frozen.is_empty();
        drop(g);
        if empty {
            self.delete_generation_files();
        }
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

    /// Append new images when a commit already exists. A fresh file is renamed
    /// into place only when nothing older still names `spend.pages`.
    fn publish_generation(
        &self,
        height: u32,
        pages: &[(u64, Arc<[u8]>)],
        carried: &[(u64, CommittedPage)],
    ) -> Result<(u32, Vec<PageRec>), StoreError> {
        let pages_path = self.dir.join(SPEND_PAGES_NAME);
        let commit_path = self.dir.join(SPEND_PAGES_COMMIT_NAME);
        let mut recs: Vec<PageRec> = Vec::new();
        let committed_len = if commit_path.exists() {
            for (off, page) in carried {
                if pages.iter().any(|(new_off, _)| new_off == off) {
                    continue;
                }
                recs.push((*off, page.log_off, page.image.len() as u32));
            }
            use std::io::{Seek, SeekFrom, Write};
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&pages_path)
                .map_err(|e| StoreError::io(&pages_path, e))?;
            let mut end = f
                .seek(SeekFrom::End(0))
                .map_err(|e| StoreError::io(&pages_path, e))?;
            for (page_off, image) in pages {
                f.write_all(image)
                    .map_err(|e| StoreError::io(&pages_path, e))?;
                recs.push((*page_off, end, image.len() as u32));
                end = end.saturating_add(image.len() as u64);
            }
            f.sync_all().map_err(|e| StoreError::io(&pages_path, e))?;
            end
        } else {
            let mut blob: Vec<u8> = Vec::new();
            for (page_off, image) in pages {
                let log_off = blob.len() as u64;
                blob.extend_from_slice(image);
                recs.push((*page_off, log_off, image.len() as u32));
            }
            write_synced_tmp_file(&pages_path, |f| {
                use std::io::Write;
                f.write_all(&blob)
                    .map_err(|e| StoreError::io(&pages_path, e))
            })?;
            blob.len() as u64
        };
        let (generation, held) = {
            let mut g = self.lock();
            let generation = g.generation.wrapping_add(1);
            g.log_bytes = committed_len;
            (generation, g.fold_held)
        };
        let commit = encode_commit(height, generation, committed_len, &recs, held);
        write_synced_tmp_rename(&commit_path, &commit)?;
        Ok((generation, recs))
    }

    fn delete_generation_files(&self) {
        let _ = std::fs::remove_file(self.dir.join(SPEND_PAGES_COMMIT_NAME));
        let _ = std::fs::remove_file(self.dir.join(SPEND_PAGES_NAME));
        let mut g = self.lock();
        g.log_bytes = 0;
        g.commit_height = None;
    }

    /// Copy committed pages that confirm has not re-dirtied onto `spent.body`.
    ///
    /// Drops the commit record only after `sync` returns and no committed page
    /// remains. A held snapshot is not copied.
    pub fn fold_into<W, S>(&self, mut write_page: W, mut sync: S) -> Result<(), StoreError>
    where
        W: FnMut(u64, &[u8]) -> Result<(), StoreError>,
        S: FnMut() -> Result<(), StoreError>,
    {
        let quiet: Vec<(u64, Arc<[u8]>)> = {
            let g = self.lock();
            if g.fold_held {
                return Ok(());
            }
            let cool_through = g.cool_through;
            let mut quiet: Vec<(u64, Arc<[u8]>)> = g
                .committed
                .iter()
                .filter(|(off, page)| {
                    page.generation <= cool_through
                        && !g.open.contains_key(*off)
                        && !g.frozen.contains_key(*off)
                })
                .map(|(off, page)| (*off, Arc::clone(&page.image)))
                .collect();
            quiet.sort_by_key(|(off, _)| *off);
            quiet
        };
        if quiet.is_empty() {
            return Ok(());
        }
        let mut wrote: Vec<(u64, Arc<[u8]>)> = Vec::new();
        for (off, image) in quiet {
            let still_quiet = {
                let g = self.lock();
                !g.open.contains_key(&off) && !g.frozen.contains_key(&off)
            };
            if !still_quiet {
                continue;
            }
            write_page(off, &image)?;
            wrote.push((off, image));
        }
        if wrote.is_empty() {
            return Ok(());
        }
        sync()?;
        let mut g = self.lock();
        for (off, _) in &wrote {
            if !g.open.contains_key(off) {
                g.committed.remove(off);
            }
        }
        if g.committed.is_empty() {
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
            return Ok(());
        }
        let buf = std::fs::read(&commit_path).map_err(|e| StoreError::io(&commit_path, e))?;
        let parsed = parse_commit(&buf)?;
        let pages_path = self.dir.join(SPEND_PAGES_NAME);
        let file = std::fs::read(&pages_path).unwrap_or_default();
        let limit = parsed.committed_len.min(file.len() as u64);
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
            if rec.log_off > limit || end > limit {
                continue;
            }
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

fn page_hit(page: Option<&OpenPage>, rel: u16) -> Option<[u8; SLOT]> {
    let page = page?;
    if let Some((_, meta)) = page.slots.iter().find(|(r, _)| *r == rel) {
        return Some(*meta);
    }
    page.base.as_ref().map(|base| slot_from_image(base, rel))
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
