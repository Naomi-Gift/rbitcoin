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
use std::sync::{Arc, Mutex};

pub const SPEND_PAGES_NAME: &str = "spend.pages";
pub const SPEND_PAGES_COMMIT_NAME: &str = "spend.pages.commit";

const SLOT: usize = 8;
const COMMIT_HEAD: usize = 28;
const COMMIT_REC: usize = 24;

/// `(page_off, log_off, image_len)` in one commit record.
type PageRec = (u64, u64, u32);

struct OpenPage {
    slots: Vec<(u16, [u8; SLOT])>,
    base: Option<Arc<[u8]>>,
}

struct CommittedPage {
    image: Arc<[u8]>,
    log_off: u64,
}

struct Inner {
    open: HashMap<u64, OpenPage>,
    committed: HashMap<u64, CommittedPage>,
    generation: u32,
    /// Snapshot height named by the on-disk commit, when one is loaded.
    commit_height: Option<u32>,
    /// Commit height was above the tip. Fold must not copy that image.
    fold_held: bool,
}

pub struct SpendPageLog {
    dir: PathBuf,
    inner: Mutex<Inner>,
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
                committed: HashMap::new(),
                generation: 0,
                commit_height: None,
                fold_held: false,
            }),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Current 8-byte image for `abs`, from the open directory or a committed
    /// page. `None` means the body still holds the slot.
    pub fn slot(&self, abs: u64) -> Option<[u8; SLOT]> {
        let page_off = abs & !(BODY_PAGE_SIZE - 1);
        let rel = ((abs - page_off) / SLOT as u64) as u16;
        let g = self.lock();
        if let Some(page) = g.open.get(&page_off) {
            if let Some((_, meta)) = page.slots.iter().find(|(r, _)| *r == rel) {
                return Some(*meta);
            }
            if let Some(base) = &page.base {
                return Some(slot_from_image(base, rel));
            }
        }
        g.committed
            .get(&page_off)
            .map(|page| slot_from_image(&page.image, rel))
    }

    pub fn note_slots(&self, slots: &[(u64, [u8; SLOT])]) -> Result<(), StoreError> {
        if slots.is_empty() {
            return Ok(());
        }
        let mut g = self.lock();
        for &(abs, meta) in slots {
            let page_off = abs & !(BODY_PAGE_SIZE - 1);
            let rel = ((abs - page_off) / SLOT as u64) as u16;
            if (rel as usize) >= (BODY_PAGE_SIZE as usize / SLOT) {
                return Err(StoreError::Corrupt("spend page slot past page"));
            }
            if !g.open.contains_key(&page_off) {
                let base = g.committed.get(&page_off).map(|c| Arc::clone(&c.image));
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
            }
        }
        Ok(())
    }

    /// The commit names a height above `tip`. Keep serving it, and do not fold.
    pub fn hold_fold_above(&self, tip: u32) -> Result<(), StoreError> {
        let path = self.dir.join(SPEND_PAGES_COMMIT_NAME);
        let should = {
            let g = self.lock();
            g.commit_height.is_some_and(|h| h > tip)
        };
        if !should {
            return Ok(());
        }
        if path.exists() {
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
        Ok(())
    }

    /// Compose open pages into one generation and publish the commit record.
    ///
    /// `read_page` supplies neighbor bytes from `spent.body` when the page has
    /// no base image. `keep` drops a slot that is no longer best-chain so a
    /// later fold does not write it. An empty open set leaves an existing
    /// commit in place, unless a held snapshot has to be filtered first.
    ///
    /// A second generation appends. It does not replace bytes an older commit
    /// still names.
    pub fn commit_open<F, K>(
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
        self.explode_held(&mut keep)?;
        let (snapshot, carried) = {
            let g = self.lock();
            if g.open.is_empty() {
                return Ok(());
            }
            let snapshot: Vec<(u64, OpenPage)> = g
                .open
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
                .filter(|(off, _)| !g.open.contains_key(*off))
                .map(|(off, page)| {
                    (
                        *off,
                        CommittedPage {
                            image: Arc::clone(&page.image),
                            log_off: page.log_off,
                        },
                    )
                })
                .collect();
            (snapshot, carried)
        };
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
            self.drop_open_pages(&rejected)?;
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
                if i + SLOT > buf.len() {
                    return Err(StoreError::Corrupt("spend page poke past image"));
                }
                buf[i..i + SLOT].copy_from_slice(&meta);
            }
            pages.push((*page_off, Arc::from(buf.into_boxed_slice())));
        }
        pages.sort_by_key(|(off, _)| *off);

        let (generation, recs) = self.publish_generation(height, &pages, &carried)?;
        let log_off_of = |page_off: u64| -> u64 {
            recs.iter()
                .find(|(off, _, _)| *off == page_off)
                .map(|(_, log_off, _)| *log_off)
                .unwrap_or(0)
        };

        let mut g = self.lock();
        g.generation = generation;
        g.commit_height = Some(height);
        g.fold_held = false;
        for page_off in &rejected {
            g.open.remove(page_off);
        }
        for (page_off, image) in pages {
            if let Some(page) = g.open.get_mut(&page_off) {
                let mut live = Vec::with_capacity(page.slots.len());
                for slot in page.slots.drain(..) {
                    if keep(&slot.1)? {
                        live.push(slot);
                    }
                }
                page.slots = live;
            }
            let unchanged = g.open.get(&page_off).is_some_and(|page| {
                page.slots.len() == snapshot_slots(&kept, page_off).len()
                    && page
                        .slots
                        .iter()
                        .all(|slot| snapshot_slots(&kept, page_off).contains(slot))
            });
            let log_off = log_off_of(page_off);
            if unchanged {
                g.open.remove(&page_off);
                g.committed
                    .insert(page_off, CommittedPage { image, log_off });
            } else if let Some(page) = g.open.get_mut(&page_off) {
                page.slots
                    .retain(|(rel, meta)| slot_from_image(&image, *rel) != *meta);
                page.base = Some(image.clone());
                if page.slots.is_empty() {
                    g.open.remove(&page_off);
                    g.committed
                        .insert(page_off, CommittedPage { image, log_off });
                }
            } else {
                g.committed
                    .insert(page_off, CommittedPage { image, log_off });
            }
        }
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
            g.open.insert(page_off, OpenPage { slots, base: None });
        }
        drop(g);
        if self.lock().open.is_empty() {
            self.delete_generation_files();
        }
        Ok(())
    }

    fn drop_open_pages(&self, pages: &[u64]) -> Result<(), StoreError> {
        if pages.is_empty() {
            return Ok(());
        }
        let mut g = self.lock();
        for off in pages {
            g.open.remove(off);
        }
        Ok(())
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
        let generation = {
            let g = self.lock();
            g.generation.wrapping_add(1)
        };
        let commit = encode_commit(height, generation, committed_len, &recs);
        write_synced_tmp_rename(&commit_path, &commit)?;
        Ok((generation, recs))
    }

    fn delete_generation_files(&self) {
        let _ = std::fs::remove_file(self.dir.join(SPEND_PAGES_COMMIT_NAME));
        let _ = std::fs::remove_file(self.dir.join(SPEND_PAGES_NAME));
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
            let mut quiet: Vec<(u64, Arc<[u8]>)> = g
                .committed
                .iter()
                .filter(|(off, _)| !g.open.contains_key(*off))
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
                !g.open.contains_key(&off)
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
        g.commit_height = Some(parsed.height);
        g.fold_held = parsed.fold_held;
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
                },
            );
        }
        Ok(())
    }
}

fn snapshot_slots(snapshot: &[(u64, OpenPage)], page_off: u64) -> &[(u16, [u8; SLOT])] {
    snapshot
        .iter()
        .find(|(off, _)| *off == page_off)
        .map(|(_, page)| page.slots.as_slice())
        .unwrap_or(&[])
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
) -> Vec<u8> {
    let mut buf = vec![0u8; COMMIT_HEAD + recs.len() * COMMIT_REC];
    buf[..4].copy_from_slice(&STORE_MAGIC);
    buf[4..6].copy_from_slice(&SCHEMA_VERSION.to_le_bytes());
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

#[cfg(test)]
mod tests {
    use super::*;

    fn dir() -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static N: AtomicU64 = AtomicU64::new(0);
        let n = N.fetch_add(1, Ordering::Relaxed);
        let p =
            std::env::temp_dir().join(format!("rbitcoin-spend-pages-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    fn zeros(off: u64, len: usize) -> Result<Vec<u8>, StoreError> {
        let _ = off;
        Ok(vec![0u8; len])
    }

    #[test]
    fn second_generation_keeps_the_first_image_bytes() {
        let d = dir();
        let log = SpendPageLog::create(&d);
        let slot = [9u8, 1, 2, 3, 4, 5, 6, 7];
        log.note_slots(&[(16, slot)]).unwrap();
        log.commit_open(3, 8192, zeros, |_| Ok(true)).unwrap();
        let first = std::fs::read(d.join(SPEND_PAGES_NAME)).unwrap();
        let first_commit = std::fs::read(d.join(SPEND_PAGES_COMMIT_NAME)).unwrap();
        log.note_slots(&[(4096, slot)]).unwrap();
        log.commit_open(4, 8192, zeros, |_| Ok(true)).unwrap();
        let file = std::fs::read(d.join(SPEND_PAGES_NAME)).unwrap();
        assert_eq!(&file[..first.len()], first.as_slice());
        let old = super::parse_commit(&first_commit).unwrap();
        let rec = &old.recs[0];
        let at = rec.log_off as usize + 16;
        assert_eq!(&file[at..at + 8], &slot);
        let loaded = SpendPageLog::load(&d).unwrap();
        assert_eq!(loaded.slot(16), Some(slot));
        assert_eq!(loaded.slot(4096), Some(slot));
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn record_past_committed_length_is_not_served() {
        let d = dir();
        let log = SpendPageLog::create(&d);
        let slot = [4u8, 1, 2, 3, 4, 5, 6, 7];
        log.note_slots(&[(16, slot)]).unwrap();
        log.commit_open(1, 8192, zeros, |_| Ok(true)).unwrap();
        let path = d.join(SPEND_PAGES_COMMIT_NAME);
        let mut buf = std::fs::read(&path).unwrap();
        buf[16..24].copy_from_slice(&8u64.to_le_bytes());
        std::fs::write(&path, &buf).unwrap();
        let loaded = SpendPageLog::load(&d).unwrap();
        assert_eq!(loaded.slot(16), None);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn held_snapshot_is_not_folded() {
        let d = dir();
        let log = SpendPageLog::create(&d);
        let slot = [5u8, 1, 2, 3, 4, 5, 6, 7];
        log.note_slots(&[(16, slot)]).unwrap();
        log.commit_open(9, 8192, zeros, |_| Ok(true)).unwrap();
        log.hold_fold_above(4).unwrap();
        let mut writes = 0u32;
        log.fold_into(
            |_off, _image| {
                writes += 1;
                Ok(())
            },
            || Ok(()),
        )
        .unwrap();
        assert_eq!(writes, 0);
        assert_eq!(log.slot(16), Some(slot));
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn fold_skips_a_page_dirtied_while_an_earlier_page_is_copied() {
        let d = dir();
        let log = SpendPageLog::create(&d);
        let slot = [6u8, 1, 2, 3, 4, 5, 6, 7];
        let later = [7u8, 1, 2, 3, 4, 5, 6, 7];
        log.note_slots(&[(16, slot), (4096, slot)]).unwrap();
        log.commit_open(2, 8192, zeros, |_| Ok(true)).unwrap();
        let mut wrote = Vec::new();
        log.fold_into(
            |off, image| {
                wrote.push(off);
                if off == 0 {
                    log.note_slots(&[(4096 + 8, later)])?;
                }
                let _ = image;
                Ok(())
            },
            || Ok(()),
        )
        .unwrap();
        assert_eq!(wrote, vec![0]);
        assert_eq!(log.slot(4096 + 8), Some(later));
        assert_eq!(log.slot(4096), Some(slot));
        let _ = std::fs::remove_dir_all(&d);
    }
}
