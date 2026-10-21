//! Debug counts for store reads taken while the IBD coordinator holds its
//! shared state lock, and for store reads the header thread still performs.
//!
//! Release builds keep the types and turn the counters into nothing.

#[cfg(debug_assertions)]
use std::cell::{Cell, RefCell};
#[cfg(debug_assertions)]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(debug_assertions)]
use std::sync::Arc;

#[cfg(debug_assertions)]
thread_local! {
    static COORD_LOCK_HELD: Cell<bool> = const { Cell::new(false) };
    static READS_UNDER_LOCK: Cell<u64> = const { Cell::new(0) };
    static OBSERVER: RefCell<Option<Arc<AtomicU64>>> = const { RefCell::new(None) };
    static HEADER_THREAD: Cell<bool> = const { Cell::new(false) };
    static HEADER_SLOT: RefCell<Option<Arc<AtomicU64>>> = const { RefCell::new(None) };
}

/// Set while the IBD coordinator task holds `shared`.
#[doc(hidden)]
pub struct CoordSharedLock;

impl CoordSharedLock {
    #[doc(hidden)]
    pub fn enter() -> Self {
        #[cfg(debug_assertions)]
        COORD_LOCK_HELD.with(|held| held.set(true));
        Self
    }
}

impl Drop for CoordSharedLock {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        COORD_LOCK_HELD.with(|held| held.set(false));
    }
}

#[doc(hidden)]
#[inline]
pub fn coord_shared_lock_held() -> bool {
    #[cfg(debug_assertions)]
    {
        COORD_LOCK_HELD.with(|held| held.get())
    }
    #[cfg(not(debug_assertions))]
    {
        false
    }
}

/// Shared with the header thread so a test on the coordinator task can see
/// that thread's store reads after it has joined.
#[derive(Clone)]
#[doc(hidden)]
pub struct HeaderThreadProbe {
    #[cfg(debug_assertions)]
    reads: Arc<AtomicU64>,
}

impl HeaderThreadProbe {
    #[doc(hidden)]
    pub fn new() -> Self {
        Self {
            #[cfg(debug_assertions)]
            reads: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Reset this task's under-lock read count and publish `self` for
    /// [`header_thread_store_reads`].
    #[doc(hidden)]
    pub fn arm() -> Self {
        let probe = Self::new();
        #[cfg(debug_assertions)]
        {
            READS_UNDER_LOCK.with(|slot| slot.set(0));
            let reads = Arc::clone(&probe.reads);
            OBSERVER.with(|slot| *slot.borrow_mut() = Some(reads));
        }
        probe
    }

    #[doc(hidden)]
    pub fn bind_header_thread(&self) {
        #[cfg(debug_assertions)]
        {
            HEADER_THREAD.with(|flag| flag.set(true));
            let reads = Arc::clone(&self.reads);
            HEADER_SLOT.with(|slot| *slot.borrow_mut() = Some(reads));
        }
    }
}

impl Default for HeaderThreadProbe {
    fn default() -> Self {
        Self::new()
    }
}

#[doc(hidden)]
pub fn note_store_read() {
    #[cfg(debug_assertions)]
    {
        if COORD_LOCK_HELD.with(|held| held.get()) {
            READS_UNDER_LOCK.with(|slot| slot.set(slot.get().saturating_add(1)));
        }
        if HEADER_THREAD.with(|flag| flag.get()) {
            HEADER_SLOT.with(|slot| {
                if let Some(reads) = slot.borrow().as_ref() {
                    reads.fetch_add(1, Ordering::Relaxed);
                }
            });
        }
    }
}

#[doc(hidden)]
pub fn coord_lock_store_reads() -> u64 {
    #[cfg(debug_assertions)]
    {
        READS_UNDER_LOCK.with(|slot| slot.get())
    }
    #[cfg(not(debug_assertions))]
    {
        0
    }
}

#[doc(hidden)]
pub fn header_thread_store_reads() -> u64 {
    #[cfg(debug_assertions)]
    {
        OBSERVER.with(|slot| {
            slot.borrow()
                .as_ref()
                .map(|reads| reads.load(Ordering::Relaxed))
                .unwrap_or(0)
        })
    }
    #[cfg(not(debug_assertions))]
    {
        0
    }
}
