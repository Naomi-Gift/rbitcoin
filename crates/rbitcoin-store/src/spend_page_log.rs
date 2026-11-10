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

struct OpenPage {
    slots: Vec<(u16, [u8; SLOT])>,
    base: Option<Arc<[u8]>>,
}

struct CommittedPage {
    image: Arc<[u8]>,
}

struct Inner {
    open: HashMap<u64, OpenPage>,
    committed: HashMap<u64, CommittedPage>,
    generation: u32,
    /// Snapshot height named by the on-disk commit, when one is loaded.
    commit_height: Option<u32>,
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

    /// Compose open pages into one generation and publish the commit record.
    ///
    /// `read_page` supplies neighbor bytes from `spent.body` when the page has
    /// no base image. An empty open set leaves an existing commit in place.
    pub fn commit_open<F>(
        &self,
        height: u32,
        published: u64,
        mut read_page: F,
    ) -> Result<(), StoreError>
    where
        F: FnMut(u64, usize) -> Result<Vec<u8>, StoreError>,
    {
        let snapshot: Vec<(u64, OpenPage)> = {
            let g = self.lock();
            if g.open.is_empty() {
                return Ok(());
            }
            g.open
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
                .collect()
        };
        let mut pages: Vec<(u64, Arc<[u8]>)> = Vec::with_capacity(snapshot.len());
        for (page_off, page) in &snapshot {
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

        let mut blob: Vec<u8> = Vec::new();
        let mut recs: Vec<(u64, u64, u32)> = Vec::with_capacity(pages.len());
        for (page_off, image) in &pages {
            let log_off = blob.len() as u64;
            blob.extend_from_slice(image);
            recs.push((*page_off, log_off, image.len() as u32));
        }
        let committed_len = blob.len() as u64;
        let pages_path = self.dir.join(SPEND_PAGES_NAME);
        write_synced_tmp_file(&pages_path, |f| {
            use std::io::Write;
            f.write_all(&blob)
                .map_err(|e| StoreError::io(&pages_path, e))
        })?;

        let generation = {
            let g = self.lock();
            g.generation.wrapping_add(1)
        };
        let commit = encode_commit(height, generation, committed_len, &recs);
        write_synced_tmp_rename(&self.dir.join(SPEND_PAGES_COMMIT_NAME), &commit)?;

        let mut g = self.lock();
        g.generation = generation;
        g.commit_height = Some(height);
        for (page_off, image) in pages {
            let unchanged = g.open.get(&page_off).is_some_and(|page| {
                page.slots.len() == snapshot_slots(&snapshot, page_off).len()
                    && page
                        .slots
                        .iter()
                        .all(|slot| snapshot_slots(&snapshot, page_off).contains(slot))
            });
            if unchanged {
                g.open.remove(&page_off);
                g.committed.insert(page_off, CommittedPage { image });
            } else if let Some(page) = g.open.get_mut(&page_off) {
                page.slots
                    .retain(|(rel, meta)| slot_from_image(&image, *rel) != *meta);
                page.base = Some(image.clone());
                if page.slots.is_empty() {
                    g.open.remove(&page_off);
                    g.committed.insert(page_off, CommittedPage { image });
                }
            } else {
                g.committed.insert(page_off, CommittedPage { image });
            }
        }
        Ok(())
    }

    /// Copy committed pages that confirm has not re-dirtied onto `spent.body`.
    ///
    /// Drops the commit record only after `sync` returns and no committed page
    /// remains.
    pub fn fold_into<W, S>(&self, mut write_page: W, mut sync: S) -> Result<(), StoreError>
    where
        W: FnMut(u64, &[u8]) -> Result<(), StoreError>,
        S: FnMut() -> Result<(), StoreError>,
    {
        let quiet: Vec<(u64, Arc<[u8]>)> = {
            let g = self.lock();
            g.committed
                .iter()
                .filter(|(off, _)| !g.open.contains_key(*off))
                .map(|(off, page)| (*off, Arc::clone(&page.image)))
                .collect()
        };
        if quiet.is_empty() {
            return Ok(());
        }
        for (off, image) in &quiet {
            write_page(*off, image)?;
        }
        sync()?;
        let mut g = self.lock();
        for (off, _) in &quiet {
            if !g.open.contains_key(off) {
                g.committed.remove(off);
            }
        }
        if g.committed.is_empty() {
            g.commit_height = None;
            drop(g);
            let _ = std::fs::remove_file(self.dir.join(SPEND_PAGES_COMMIT_NAME));
            let _ = std::fs::remove_file(self.dir.join(SPEND_PAGES_NAME));
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
        g.committed.clear();
        for rec in parsed.recs {
            let end = rec.log_off.saturating_add(u64::from(rec.image_len));
            if rec.log_off > limit || end > limit {
                continue;
            }
            let start = rec.log_off as usize;
            let image: Arc<[u8]> = Arc::from(file[start..end as usize].to_vec().into_boxed_slice());
            g.committed.insert(rec.page_off, CommittedPage { image });
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
        recs,
    })
}
