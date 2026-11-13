//! In-RAM block payload queue for the combined archive/confirm path.
//!
//! **Why RAM (not disk):** peer wire would otherwise be written once to a durable
//! queue and again into Class A on confirm — **double disk write per block**.
//! Keeping the same FIFO / height-index structure in process memory trades
//! **redownload on restart** and peak RAM for a single durable write (Class A).
//!
//! **Lifecycle:** enqueue after peer framing (raw payload + stamped Σ inputs;
//! no full block parse); lookup **takes** raw on emit (`take_raw`);
//! **dequeue only after combined confirm-write** (or permanent reject).
//! Restart does **not** rehydrate payloads (queue starts empty).
//!
//! **Primary capacity** is soft densify assign in the net layer (no hysteresis):
//! under ~100 MiB free densify ahead; over ~100 MiB only the next ~1 min of
//! confirm work at tip rate; at/over the 1 GiB assign-stop (`RBITCOIN_BLOCK_QUEUE_GB`
//! / `RBITCOIN_BLOCK_QUEUE_BYTES`) densify holes only within that ~1 min window **and** not past
//! fetched_hi (do not grow past fetched; do not densify far holes outside the
//! window). This type **always** accepts already-requested payloads (OOM aside).

use crate::error::StoreError;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Raw row removed by [`BlockQueue::take_raw`] (lookup consume).
#[derive(Debug, Clone)]
pub struct TakenRaw {
    pub hash: [u8; 32],
    pub header_fk: u64,
    pub payload: Vec<u8>,
}

/// One queued block. `payload` is empty after [`BlockQueue::promote_wave`].
#[derive(Debug, Clone)]
pub struct QueuedBlock {
    pub id: u64,
    pub height: u32,
    pub hash: [u8; 32],
    pub header_fk: u64,
    pub payload: Vec<u8>,
}

/// Wire bytes or a post-lookup charge (decoded `Block` lives on Query).
#[derive(Debug, Clone)]
#[doc(hidden)]
pub enum QueuedBody {
    /// `Vec` inside the `Arc` so [`BlockQueue::take_raw`] can unwrap the
    /// allocation when lookup holds the last handle. `Arc<[u8]>` cannot
    /// `try_unwrap` (the slice is unsized).
    Raw(Arc<Vec<u8>>),
    Promoted {
        charge: u64,
    },
}

impl QueuedBody {
    #[doc(hidden)]
    pub fn charge(&self) -> u64 {
        match self {
            Self::Raw(v) => v.len() as u64,
            Self::Promoted { charge } => *charge,
        }
    }

    #[doc(hidden)]
    pub fn payload(&self) -> &[u8] {
        match self {
            Self::Raw(v) => v.as_slice(),
            Self::Promoted { .. } => &[],
        }
    }
}

/// Index-only view of a queue entry (no payload clone).
#[derive(Debug, Clone, Copy)]
pub struct QueuedBlockMeta {
    pub id: u64,
    pub height: u32,
    pub hash: [u8; 32],
    pub header_fk: u64,
    /// Raw wire length, or decoded charge after promote.
    pub payload_len: u64,
    /// Σ `tx.input` from [`crate::block_wire_input_count`] at enqueue (`0` if unreadable).
    pub n_inputs: u32,
    /// Lookup finished TipOnly for this height (load may still leftover-stamp).
    pub resolve_complete: bool,
}

/// Height → queue id, from the lowest queued height.
///
/// A hole is an empty slot. `start` is a dropped prefix. It stays until it is
/// at least as long as the live tail, then the tail is moved once.
#[derive(Debug)]
pub struct HeightWindow {
    base: u32,
    start: usize,
    slots: Vec<Option<u64>>,
    live: usize,
}

impl HeightWindow {
    fn new() -> Self {
        Self {
            base: 0,
            start: 0,
            slots: Vec::new(),
            live: 0,
        }
    }

    /// Lowest height still in the window.
    pub fn lowest(&self) -> Option<u32> {
        if self.live == 0 {
            None
        } else {
            Some(self.base + self.start as u32)
        }
    }

    /// Highest height still in the window.
    pub fn highest(&self) -> Option<u32> {
        if self.live == 0 {
            None
        } else {
            Some(self.base + self.slots.len() as u32 - 1)
        }
    }

    pub fn get(&self, height: u32) -> Option<u64> {
        let i = self.index_of(height)?;
        if i < self.start {
            return None;
        }
        self.slots.get(i).copied().flatten()
    }

    pub fn contains(&self, height: u32) -> bool {
        self.get(height).is_some()
    }

    pub fn insert(&mut self, height: u32, id: u64) -> Option<u64> {
        if self.live == 0 {
            self.base = height;
            self.start = 0;
            self.slots.clear();
            self.slots.push(Some(id));
            self.live = 1;
            return None;
        }
        if height < self.base {
            let gap = (self.base - height) as usize;
            self.slots.splice(0..0, std::iter::repeat_n(None, gap));
            self.start += gap;
            self.base = height;
            self.slots[0] = Some(id);
            self.start = 0;
            self.live += 1;
            return None;
        }
        let i = (height - self.base) as usize;
        if i < self.start {
            self.slots[i] = Some(id);
            self.start = i;
            self.live += 1;
            return None;
        }
        if i >= self.slots.len() {
            self.slots.resize(i + 1, None);
        }
        let prev = self.slots[i].replace(id);
        if prev.is_none() {
            self.live += 1;
        }
        prev
    }

    pub fn remove(&mut self, height: u32) -> Option<u64> {
        let i = self.index_of(height)?;
        if i < self.start {
            return None;
        }
        let prev = self.slots.get_mut(i)?.take()?;
        self.live -= 1;
        self.trim();
        Some(prev)
    }

    fn index_of(&self, height: u32) -> Option<usize> {
        height.checked_sub(self.base).map(|d| d as usize)
    }

    fn trim(&mut self) {
        if self.live == 0 {
            self.base = 0;
            self.start = 0;
            self.slots.clear();
            self.slots.shrink_to_fit();
            return;
        }
        while self.start < self.slots.len() && self.slots[self.start].is_none() {
            self.start += 1;
        }
        while self.slots.len() > self.start && self.slots.last().is_some_and(|slot| slot.is_none())
        {
            self.slots.pop();
        }
        let tail = self.slots.len() - self.start;
        if self.start > 0 && self.start >= tail {
            let kept = self.slots.split_off(self.start);
            self.base = self.base.saturating_add(self.start as u32);
            self.start = 0;
            self.slots = kept;
            self.slots.shrink_to_fit();
        }
    }

    fn heights(&self) -> impl Iterator<Item = u32> + '_ {
        let base = self.base;
        self.slots
            .iter()
            .enumerate()
            .skip(self.start)
            .filter_map(move |(i, slot)| slot.map(|_| base + i as u32))
    }
}

/// Heights still queued for one hash. One height stays inline.
#[derive(Clone, Debug)]
pub enum QueuedHeights {
    One(u32),
    Many(Vec<u32>),
}

impl QueuedHeights {
    pub fn is_inline(&self) -> bool {
        matches!(self, Self::One(_))
    }

    fn contains(&self, height: u32) -> bool {
        match self {
            Self::One(h) => *h == height,
            Self::Many(heights) => heights.contains(&height),
        }
    }

    fn push_new(&mut self, height: u32) {
        if self.contains(height) {
            return;
        }
        match self {
            Self::One(h) => *self = Self::Many(vec![*h, height]),
            Self::Many(heights) => heights.push(height),
        }
    }

    fn without(self, height: u32) -> Option<Self> {
        match self {
            Self::One(h) if h == height => None,
            Self::One(h) => Some(Self::One(h)),
            Self::Many(heights) => {
                let heights: Vec<u32> = heights.into_iter().filter(|h| *h != height).collect();
                match heights.len() {
                    0 => None,
                    1 => Some(Self::One(heights[0])),
                    _ => Some(Self::Many(heights)),
                }
            }
        }
    }

    fn to_vec(&self) -> Vec<u32> {
        match self {
            Self::One(h) => vec![*h],
            Self::Many(heights) => heights.clone(),
        }
    }
}

/// Process-local FIFO of block payloads (append + delete-by-id).
///
/// Same shape as the former on-disk queue (`id`, height, hash, header_fk,
/// payload) but **never** writes rec files. `store_dir` is accepted for API
/// compatibility and to optionally ignore/remove a legacy `block_queue/` dir.
pub struct BlockQueue {
    pub next_id: AtomicU64,
    /// id → entry (payload owned). Ascending id is enqueue order.
    pub index: BTreeMap<u64, IndexEntry>,
    /// First-wins height → id. A window from the lowest queued height.
    /// Height APIs must not walk `index.values()`.
    pub height_to_id: HeightWindow,
    /// Hash → heights that still hold that hash. One height stays inline.
    /// `has_hash` is one map lookup.
    pub by_hash: HashMap<[u8; 32], QueuedHeights>,
    pub bytes: u64,
}

#[derive(Debug, Clone)]
#[doc(hidden)]
pub struct IndexEntry {
    pub height: u32,
    pub hash: [u8; 32],
    pub header_fk: u64,
    pub body: QueuedBody,
    pub n_inputs: u32,
    pub resolve_complete: bool,
    /// Caller's id for the peer that sent this wire (opaque here; RAM only).
    pub sender: Option<u64>,
}

impl BlockQueue {
    /// Create an empty in-RAM queue. Legacy on-disk `store_dir/block_queue/` is
    /// not loaded (redownload after restart); best-effort remove stale files.
    pub fn open_or_create(store_dir: &Path) -> Result<Self, StoreError> {
        let legacy = store_dir.join("block_queue");
        if legacy.exists() {
            let _ = std::fs::remove_dir_all(&legacy);
        }
        Ok(Self::empty())
    }

    /// Empty queue with no legacy-directory cleanup.
    pub fn empty() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            index: BTreeMap::new(),
            height_to_id: HeightWindow::new(),
            by_hash: HashMap::new(),
            bytes: 0,
        }
    }

    pub fn bytes(&self) -> u64 {
        self.bytes
    }

    pub fn count(&self) -> usize {
        self.index.len()
    }

    /// Append a block payload in RAM. Returns queue id.
    ///
    /// Always accepts (IBD densify assign-stop is request-limited; never refuse
    /// already-requested bodies here).
    pub fn enqueue(
        &mut self,
        height: u32,
        hash: [u8; 32],
        header_fk: u64,
        payload: &[u8],
    ) -> Result<u64, StoreError> {
        let n_inputs = crate::block_wire_input_count(payload).unwrap_or(0);
        self.enqueue_vec(height, hash, header_fk, payload.to_vec(), n_inputs, None)
    }

    /// Enqueue an already-owned payload (copy happened outside the BQ lock).
    ///
    /// `n_inputs` is [`crate::block_wire_input_count`] from the caller so the
    /// walk can run off the BQ mutex (peer offer). `sender` is the caller's id
    /// for the peer the wire came from, if any.
    pub fn enqueue_vec(
        &mut self,
        height: u32,
        hash: [u8; 32],
        header_fk: u64,
        payload: Vec<u8>,
        n_inputs: u32,
        sender: Option<u64>,
    ) -> Result<u64, StoreError> {
        if let Some(id) = self.height_to_id.get(height) {
            return Ok(id);
        }
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.bytes = self.bytes.saturating_add(payload.len() as u64);
        self.index.insert(
            id,
            IndexEntry {
                height,
                hash,
                header_fk,
                body: QueuedBody::Raw(Arc::new(payload)),
                n_inputs,
                resolve_complete: false,
                sender,
            },
        );
        self.height_to_id.insert(height, id);
        self.remember_hash(hash, height);
        Ok(id)
    }

    #[doc(hidden)]
    pub fn remember_hash(&mut self, hash: [u8; 32], height: u32) {
        match self.by_hash.get_mut(&hash) {
            Some(heights) => heights.push_new(height),
            None => {
                self.by_hash.insert(hash, QueuedHeights::One(height));
            }
        }
    }

    #[doc(hidden)]
    pub fn forget_hash(&mut self, hash: &[u8; 32], height: u32) {
        let Some(heights) = self.by_hash.remove(hash) else {
            return;
        };
        if let Some(heights) = heights.without(height) {
            self.by_hash.insert(*hash, heights);
        }
    }

    /// True if any row still carries `hash`. One map lookup, no row scan.
    pub fn has_hash(&self, hash: &[u8; 32]) -> bool {
        self.by_hash.contains_key(hash)
    }

    /// Heights whose row hash is `hash` (first-wins order).
    pub fn heights_for_hash(&self, hash: &[u8; 32]) -> Vec<u32> {
        self.by_hash
            .get(hash)
            .map(QueuedHeights::to_vec)
            .unwrap_or_default()
    }

    /// Load payload by id.
    pub fn get(&self, id: u64) -> Result<Option<QueuedBlock>, StoreError> {
        let Some(e) = self.index.get(&id) else {
            return Ok(None);
        };
        Ok(Some(QueuedBlock {
            id,
            height: e.height,
            hash: e.hash,
            header_fk: e.header_fk,
            payload: e.body.payload().to_vec(),
        }))
    }

    /// List all ids ascending.
    pub fn ids(&self) -> Vec<u64> {
        self.index.keys().copied().collect()
    }

    /// Remove after confirm-write / permanent reject.
    pub fn dequeue(&mut self, id: u64) -> Result<bool, StoreError> {
        let Some(e) = self.index.remove(&id) else {
            return Ok(false);
        };
        self.bytes = self.bytes.saturating_sub(e.body.charge());
        if self.height_to_id.get(e.height) == Some(id) {
            self.height_to_id.remove(e.height);
        }
        self.forget_hash(&e.hash, e.height);
        Ok(true)
    }

    /// Take raw payload and remove the row. `None` if missing or already promoted.
    pub fn take_raw(&mut self, height: u32) -> Option<TakenRaw> {
        let id = self.height_to_id.get(height)?;
        let e = self.index.get_mut(&id)?;
        let payload = match &mut e.body {
            QueuedBody::Raw(v) => {
                let arc = std::mem::replace(v, Arc::new(Vec::new()));
                Arc::try_unwrap(arc).unwrap_or_else(|shared| (*shared).clone())
            }
            QueuedBody::Promoted { .. } => return None,
        };
        let e = self.index.get(&id)?;
        let out = TakenRaw {
            hash: e.hash,
            header_fk: e.header_fk,
            payload,
        };
        self.bytes = self.bytes.saturating_sub(out.payload.len() as u64);
        let _ = self.dequeue(id);
        Some(out)
    }

    /// Dequeue the row at `height` (0 or 1; a second body at the same height is not stored).
    pub fn dequeue_height(&mut self, height: u32) -> Result<usize, StoreError> {
        let Some(id) = self.height_to_id.get(height) else {
            return Ok(0);
        };
        Ok(usize::from(self.dequeue(id)?))
    }

    /// Contiguous unresolved heights from `path_lo`. Stops at the first height
    /// not on the queue (a hole). Skips resolve-complete and `skip` without
    /// treating those as a gap. Capped at `cap`.
    ///
    /// When `lens` is set, it receives each returned height's wire charge
    /// during this walk. Callers must not lock the queue again per height.
    pub fn unresolved_heights(
        &self,
        path_lo: u32,
        skip: &HashSet<u32>,
        cap: usize,
        mut lens: Option<&mut Vec<u64>>,
    ) -> Vec<u32> {
        if let Some(slot) = lens.as_mut() {
            slot.clear();
        }
        if cap == 0 {
            return Vec::new();
        }
        let mut out: Vec<u32> = Vec::new();
        let mut h = path_lo;
        while out.len() < cap {
            if !self.contains_height(h) {
                break;
            }
            if !self.is_resolve_complete(h) && !skip.contains(&h) {
                if let Some(slot) = lens.as_mut() {
                    let Some(wire) = self
                        .height_to_id
                        .get(h)
                        .and_then(|id| self.index.get(&id))
                        .map(|e| e.body.charge())
                    else {
                        break;
                    };
                    slot.push(wire);
                }
                out.push(h);
            }
            h = match h.checked_add(1) {
                Some(n) => n,
                None => break,
            };
        }
        out
    }

    /// Index-only listing (ascending id).
    pub fn list_meta(&self) -> Vec<QueuedBlockMeta> {
        self.index
            .iter()
            .map(|(&id, e)| QueuedBlockMeta {
                id,
                height: e.height,
                hash: e.hash,
                header_fk: e.header_fk,
                payload_len: e.body.charge(),
                n_inputs: e.n_inputs,
                resolve_complete: e.resolve_complete,
            })
            .collect()
    }

    /// Load payload for a height (confirm load intake). First match by height.
    ///
    /// Does **not** dequeue — confirm-write / permanent reject removes the rec.
    pub fn get_by_height(&self, height: u32) -> Result<Option<QueuedBlock>, StoreError> {
        let Some(id) = self.height_to_id.get(height) else {
            return Ok(None);
        };
        self.get(id)
    }

    /// True if any rec exists for `height`.
    pub fn contains_height(&self, height: u32) -> bool {
        self.height_to_id.contains(height)
    }

    /// Heights currently on the queue (ascending).
    pub fn heights(&self) -> Vec<u32> {
        self.height_to_id.heights().collect()
    }

    /// Highest height among recs (`None` if empty).
    pub fn max_height(&self) -> Option<u32> {
        self.height_to_id.highest()
    }

    /// Lowest height among recs (`None` if empty).
    pub fn min_height(&self) -> Option<u32> {
        self.height_to_id.lowest()
    }

    /// First queue id for `height`, if any.
    pub fn id_for_height(&self, height: u32) -> Option<u64> {
        self.height_to_id.get(height)
    }

    #[doc(hidden)]
    pub fn entry_for_height(&self, height: u32) -> Option<&IndexEntry> {
        self.height_to_id
            .get(height)
            .and_then(|id| self.index.get(&id))
    }

    #[doc(hidden)]
    pub fn entry_mut_for_height(&mut self, height: u32) -> Result<&mut IndexEntry, StoreError> {
        let id = self.height_to_id.get(height).ok_or(StoreError::NotFound)?;
        self.index.get_mut(&id).ok_or(StoreError::NotFound)
    }

    /// Lookup finished this height: every external parent has a hit (or none exist).
    pub fn mark_resolve_complete(&mut self, height: u32) -> Result<(), StoreError> {
        self.entry_mut_for_height(height)?.resolve_complete = true;
        Ok(())
    }

    pub fn is_resolve_complete(&self, height: u32) -> bool {
        self.entry_for_height(height)
            .map(|e| e.resolve_complete)
            .unwrap_or(false)
    }

    /// Hash of the first queue entry at `height`, if any (no payload clone).
    pub fn hash_at_height(&self, height: u32) -> Option<[u8; 32]> {
        self.entry_for_height(height).map(|e| e.hash)
    }

    /// Enqueue-stamped Σ inputs (`None` if height missing).
    pub fn input_count_at(&self, height: u32) -> Option<u32> {
        self.entry_for_height(height).map(|e| e.n_inputs)
    }

    /// Class A header fk stamped at enqueue (`None` if height missing).
    pub fn header_fk_at(&self, height: u32) -> Option<u64> {
        self.entry_for_height(height).map(|e| e.header_fk)
    }

    /// Sender id given at enqueue (`None` if height missing or none given).
    pub fn sender_at(&self, height: u32) -> Option<u64> {
        self.entry_for_height(height).and_then(|e| e.sender)
    }

    /// One-pass height list for a load pack: stored hash + resolve-complete.
    /// Missing heights are omitted (caller treats as body-missing).
    pub fn pack_snapshot(&self, heights: &[u32]) -> Vec<(u32, [u8; 32], bool)> {
        let mut out = Vec::with_capacity(heights.len());
        for &h in heights {
            if let Some(e) = self.entry_for_height(h) {
                out.push((h, e.hash, e.resolve_complete));
            }
        }
        out
    }

    /// True when `height` still holds a raw (unpromoted) payload. No clone.
    pub fn has_raw(&self, height: u32) -> bool {
        matches!(
            self.entry_for_height(height).map(|e| &e.body),
            Some(QueuedBody::Raw(_))
        )
    }

    /// Another handle on the still-raw frame. Promoted / missing → `None`.
    ///
    /// Clones the `Arc`, not the bytes.
    pub fn raw_payload(&self, height: u32) -> Option<Arc<Vec<u8>>> {
        match self.entry_for_height(height).map(|e| &e.body) {
            Some(QueuedBody::Raw(v)) => Some(Arc::clone(v)),
            _ => None,
        }
    }

    /// Raw wire for `heights` still holding payload. One pass; skips promoted.
    ///
    /// Lookup wave must not use this for the unresolved cap — clone via
    /// [`Self::raw_payload`] per height that will actually decode.
    pub fn raw_payloads(&self, heights: &[u32]) -> Vec<(u32, Vec<u8>)> {
        let want: HashSet<u32> = heights.iter().copied().collect();
        let mut out = Vec::new();
        for e in self.index.values() {
            if !want.contains(&e.height) {
                continue;
            }
            if let QueuedBody::Raw(v) = &e.body {
                out.push((e.height, v.as_slice().to_vec()));
            }
        }
        out
    }

    /// Replace raw with a decoded charge for each height. Returns how many
    /// were still raw. Already-promoted rows keep their charge. One pass.
    pub fn promote_wave(&mut self, items: &[(u32, u64)]) -> Result<usize, StoreError> {
        let mut n = 0usize;
        for &(height, charge) in items {
            let old = {
                let Ok(e) = self.entry_mut_for_height(height) else {
                    continue;
                };
                if matches!(e.body, QueuedBody::Promoted { .. }) {
                    continue;
                }
                let old = e.body.charge();
                e.body = QueuedBody::Promoted { charge };
                old
            };
            self.bytes = self.bytes.saturating_sub(old).saturating_add(charge);
            n += 1;
        }
        Ok(n)
    }

    pub fn promoted_count(&self) -> usize {
        self.index
            .values()
            .filter(|e| matches!(e.body, QueuedBody::Promoted { .. }))
            .count()
    }
}
