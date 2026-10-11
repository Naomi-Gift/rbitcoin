//! Pins: getdata→BQ does not take lookup's decoded-stash lock, and queue
//! ids stay unique in enqueue order.

use std::panic::AssertUnwindSafe;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use super::Query;

fn open() -> Arc<Query> {
    static N: AtomicU64 = AtomicU64::new(0);
    let id = N.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("rbtc-bq-lock-{id}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    Arc::new(Query::open_or_create_tiny(&dir).unwrap())
}

fn assert_independent(reason: &str, hold: impl Sized, work: impl FnOnce() + Send + 'static) {
    let (tx, rx) = mpsc::channel();
    let worker = thread::spawn(move || {
        let result = std::panic::catch_unwind(AssertUnwindSafe(work));
        let _ = tx.send(result);
    });
    let timed = rx.recv_timeout(Duration::from_secs(2));
    drop(hold);
    if timed.is_err() {
        let _ = rx.recv_timeout(Duration::from_secs(2));
    }
    worker.join().unwrap();
    match timed {
        Ok(Ok(())) => {}
        Ok(Err(payload)) => std::panic::resume_unwind(payload),
        Err(_) => panic!("{reason}"),
    }
}

#[test]
fn getdata_enqueue_does_not_wait_on_lookup_resolved() {
    let q = open();
    let hold = q.bq_resolved.lock().unwrap();
    let q2 = Arc::clone(&q);
    assert_independent(
        "getdata enqueue waited on lookup's decoded stash",
        hold,
        move || {
            q2.block_queue_enqueue(7, [7u8; 32], 1, b"payload").unwrap();
            assert!(q2.block_queue_has_hash(&[7u8; 32]));
        },
    );
}

#[test]
fn lookup_read_does_not_wait_on_decoded_stash() {
    let q = open();
    q.block_queue_enqueue(4, [4u8; 32], 1, b"raw").unwrap();
    let hold = q.bq_resolved.lock().unwrap();
    let q2 = Arc::clone(&q);
    assert_independent(
        "lookup row read waited on the decoded stash",
        hold,
        move || {
            assert!(q2.block_queue_hash_at_height(4).is_some());
            assert!(q2.block_queue_raw_payload(4).unwrap().is_some());
            let intake = q2.block_queue_wave_intake(&[4]);
            assert_eq!(intake.raw.len(), 1);
        },
    );
}

#[test]
fn has_hash_tracks_each_height_until_dequeue() {
    let q = open();
    let hash = [9u8; 32];
    q.block_queue_enqueue(1, hash, 1, b"a").unwrap();
    q.block_queue_enqueue(17, hash, 1, b"b").unwrap();
    assert!(q.block_queue_has_hash(&hash));
    assert_eq!(q.block_queue_dequeue_height(1).unwrap(), 1);
    assert!(q.block_queue_has_hash(&hash));
    assert!(q.block_queue_hash_at_height(17).is_some());
    assert_eq!(q.block_queue_dequeue_height(17).unwrap(), 1);
    assert!(!q.block_queue_has_hash(&hash));
    assert_eq!(q.block_queue_count(), 0);
    assert_eq!(q.block_queue_max_height(), None);
}

#[test]
fn queue_ids_stay_unique_and_list_meta_is_oldest_first() {
    let q = open();
    let older = q.block_queue_enqueue(5, [5u8; 32], 1, b"aa").unwrap();
    let newer = q.block_queue_enqueue(3, [3u8; 32], 1, b"b").unwrap();
    assert_ne!(older, newer);
    let meta = q.block_queue_list_meta();
    assert_eq!(meta.len(), 2);
    assert_eq!(meta[0].id, older);
    assert_eq!(meta[0].height, 5);
    assert_eq!(meta[1].id, newer);
    assert_eq!(meta[1].height, 3);
}

#[test]
fn stats_follow_enqueue_and_dequeue() {
    let q = open();
    q.block_queue_enqueue(5, [5u8; 32], 1, b"aaaaa").unwrap();
    q.block_queue_enqueue(9, [9u8; 32], 1, b"bbb").unwrap();
    let (_stop, bytes, count) = q.block_queue_stats();
    assert_eq!(count, 2);
    assert_eq!(bytes, 8);
    assert_eq!(q.block_queue_max_height(), Some(9));
    q.block_queue_dequeue_height(9).unwrap();
    let (_stop, bytes, count) = q.block_queue_stats();
    assert_eq!(count, 1);
    assert_eq!(bytes, 5);
    assert_eq!(q.block_queue_max_height(), Some(5));
}
