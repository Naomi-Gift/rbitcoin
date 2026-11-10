//! Poisoned confirm-feed wait. A session cannot schedule the panic that
//! poisons the mutex, so this is the pure result of the shipped 20ms wait.
use std::sync::atomic::{AtomicUsize, Ordering};

use rbitcoin_ibd::ibd::confirm::ConfirmFeed;

#[test]
fn poisoned_feed_wait_returns_the_guard() {
    let feed = ConfirmFeed::new();
    feed.inner.lock().unwrap().inflight.insert(1);
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _g = feed.inner.lock().unwrap();
        panic!("poison the confirm feed");
    }));
    let calls = AtomicUsize::new(0);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        feed.wait_claims_below(2, || calls.fetch_add(1, Ordering::SeqCst) > 0);
    }));
    assert!(result.is_ok(), "poisoned feed wait must return the guard");
}
