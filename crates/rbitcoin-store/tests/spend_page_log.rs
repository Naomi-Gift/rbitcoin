use rbitcoin_store::error::StoreError;
use rbitcoin_store::spend_page_log::{
    pace_after_checkpoint, spend_checkpoint_early, SpendPageLog, LOG_IMAGE_CAP, OPEN_EDGE_CAP,
    SPEND_PAGES_COMMIT_NAME, SPEND_PAGES_NAME,
};
use std::path::PathBuf;

fn dir() -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    let p = std::env::temp_dir().join(format!("rbitcoin-spend-pages-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

#[test]
fn image_cap_schedules_one_followup_and_the_edge_cap_stays_due() {
    assert!(!spend_checkpoint_early(
        OPEN_EDGE_CAP - 1,
        LOG_IMAGE_CAP - 1
    ));
    assert!(spend_checkpoint_early(OPEN_EDGE_CAP, 0));
    assert!(spend_checkpoint_early(0, LOG_IMAGE_CAP));
    assert_eq!(pace_after_checkpoint(0, LOG_IMAGE_CAP, false), (true, true));
    assert_eq!(pace_after_checkpoint(0, LOG_IMAGE_CAP, true), (false, true));
    assert_eq!(
        pace_after_checkpoint(OPEN_EDGE_CAP, LOG_IMAGE_CAP, true),
        (true, true)
    );
    assert_eq!(pace_after_checkpoint(0, 0, true), (false, false));
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
    log.note_slots(&[(4096, slot)]).unwrap();
    log.commit_open(4, 8192, zeros, |_| Ok(true)).unwrap();
    let file = std::fs::read(d.join(SPEND_PAGES_NAME)).unwrap();
    assert_eq!(&file[..first.len()], first.as_slice());
    assert_eq!(&file[16..24], &slot);
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
    let mut early = 0u32;
    log.fold_into(
        |_off, _image| {
            early += 1;
            Ok(())
        },
        || Ok(()),
    )
    .unwrap();
    assert_eq!(early, 0, "the publishing checkpoint does not fold");
    log.commit_open(3, 8192, zeros, |_| Ok(true)).unwrap();
    let mut wrote = Vec::new();
    log.fold_into(
        |off, image| {
            wrote.push(off);
            if off == 16 {
                log.note_slots(&[(4096 + 8, later)])?;
            }
            let _ = image;
            Ok(())
        },
        || Ok(()),
    )
    .unwrap();
    assert_eq!(wrote, vec![16]);
    assert_eq!(log.slot(4096 + 8), Some(later));
    assert_eq!(log.slot(4096), Some(slot));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn three_batches_on_one_page_are_one_image() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let a = [1u8, 1, 2, 3, 4, 5, 6, 7];
    let b = [2u8, 1, 2, 3, 4, 5, 6, 7];
    let c = [3u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, a)]).unwrap();
    log.note_slots(&[(24, b)]).unwrap();
    log.note_slots(&[(32, c)]).unwrap();
    log.commit_open(4, 8192, zeros, |_| Ok(true)).unwrap();
    let file = std::fs::read(d.join(SPEND_PAGES_NAME)).unwrap();
    assert_eq!(file.len(), 4096);
    assert_eq!(&file[16..24], &a);
    assert_eq!(&file[24..32], &b);
    assert_eq!(&file[32..40], &c);
    assert_eq!(file[40], 0, "an untouched neighbor stays in the image");
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn note_during_compose_stays_out_of_that_image() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let first = [4u8, 1, 2, 3, 4, 5, 6, 7];
    let second = [5u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, first)]).unwrap();
    log.commit_open(
        4,
        8192,
        |off, len| {
            log.note_slots(&[(4096, second)])?;
            zeros(off, len)
        },
        |_| Ok(true),
    )
    .unwrap();
    let file = std::fs::read(d.join(SPEND_PAGES_NAME)).unwrap();
    assert_eq!(file.len(), 4096);
    assert_eq!(log.slot(16), Some(first));
    assert_eq!(log.slot(4096), Some(second));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn hold_during_compose_is_not_folded() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let slot = [6u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, slot)]).unwrap();
    log.commit_open(5, 8192, zeros, |_| Ok(true)).unwrap();
    log.commit_open(5, 8192, zeros, |_| Ok(true)).unwrap();
    log.note_slots(&[(4096, slot)]).unwrap();
    log.commit_open(
        9,
        8192,
        |off, len| {
            log.hold_fold_above(1)?;
            zeros(off, len)
        },
        |_| Ok(true),
    )
    .unwrap();
    let mut writes = 0u32;
    log.fold_into(
        |_off, _image| {
            writes += 1;
            Ok(())
        },
        || Ok(()),
    )
    .unwrap();
    assert_eq!(
        writes, 0,
        "a hold taken while composing must survive publish"
    );
    assert_eq!(log.slot(16), Some(slot));
    assert_eq!(log.slot(4096), Some(slot));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn open_edge_cap_blocks_the_next_insert_until_commit() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    let d = dir();
    let log = Arc::new(SpendPageLog::create(&d));
    let slot = [8u8, 1, 2, 3, 4, 5, 6, 7];
    let mut batch = Vec::with_capacity(512);
    let mut n = 0u64;
    while n < 2_000_000 {
        let page = n / 512;
        let rel = n % 512;
        batch.push((page * 4096 + rel * 8, slot));
        n += 1;
        if batch.len() == 512 {
            log.note_slots(&batch).unwrap();
            batch.clear();
        }
    }
    if !batch.is_empty() {
        log.note_slots(&batch).unwrap();
    }
    let published = 3907 * 4096;
    let log2 = Arc::clone(&log);
    let gate = Arc::new(AtomicBool::new(false));
    let gate2 = Arc::clone(&gate);
    let worker = std::thread::spawn(move || {
        while !gate2.load(Ordering::Acquire) {
            std::thread::yield_now();
        }
        std::thread::sleep(Duration::from_millis(80));
        log2.commit_open(3, published, zeros, |_| Ok(true)).unwrap();
    });
    gate.store(true, Ordering::Release);
    let started = Instant::now();
    log.note_slots(&[(n * 8, slot)]).unwrap();
    let waited = started.elapsed();
    worker.join().unwrap();
    assert!(
        waited >= Duration::from_millis(60),
        "insert at the open-edge cap must wait for a checkpoint, waited {waited:?}"
    );
    assert_eq!(log.slot(n * 8), Some(slot));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn disconnect_during_first_compose_is_not_folded() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let slot = [9u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, slot)]).unwrap();
    log.commit_open(
        9,
        8192,
        |off, len| {
            log.hold_fold_above(1)?;
            zeros(off, len)
        },
        |_| Ok(true),
    )
    .unwrap();
    let mut writes = 0u32;
    log.fold_into(
        |_off, _image| {
            writes += 1;
            Ok(())
        },
        || Ok(()),
    )
    .unwrap();
    assert_eq!(writes, 0, "a hold during the first compose is not folded");
    log.commit_open(1, 8192, zeros, |_| Ok(false)).unwrap();
    writes = 0;
    log.fold_into(
        |_off, _image| {
            writes += 1;
            Ok(())
        },
        || Ok(()),
    )
    .unwrap();
    assert_eq!(writes, 0, "the abandoned spend is not copied later");
    assert_eq!(log.slot(16), None);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn failed_compose_retries_the_frozen_page() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let first = [3u8, 1, 2, 3, 4, 5, 6, 7];
    let second = [4u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, first)]).unwrap();
    let err = log
        .commit_open(
            4,
            8192,
            |_off, _len| Err(StoreError::Corrupt("boom")),
            |_| Ok(true),
        )
        .unwrap_err();
    assert!(matches!(err, StoreError::Corrupt("boom")));
    assert_eq!(log.slot(16), Some(first));
    assert!(!d.join(SPEND_PAGES_COMMIT_NAME).exists());
    log.commit_open(4, 8192, zeros, |_| Ok(true)).unwrap();
    let retried = std::fs::read(d.join(SPEND_PAGES_NAME)).unwrap();
    assert_eq!(&retried[16..24], &first);
    assert_eq!(log.slot(16), Some(first));
    let _ = std::fs::remove_dir_all(&d);
    let d = dir();
    let log = SpendPageLog::create(&d);
    log.note_slots(&[(16, first)]).unwrap();
    log.commit_open(
        4,
        8192,
        |_off, _len| Err(StoreError::Corrupt("boom")),
        |_| Ok(true),
    )
    .unwrap_err();
    log.note_slots(&[(4096, second)]).unwrap();
    log.commit_open(4, 8192, zeros, |_| Ok(true)).unwrap();
    assert_eq!(log.slot(16), Some(first));
    assert_eq!(log.slot(4096), Some(second));
    let file = std::fs::read(d.join(SPEND_PAGES_NAME)).unwrap();
    assert_eq!(file.len(), 8192);
    assert_eq!(&file[16..24], &first);
    assert_eq!(&file[4096..4104], &second);
    drop(log);
    let loaded = SpendPageLog::load(&d).unwrap();
    assert_eq!(loaded.slot(16), Some(first));
    assert_eq!(loaded.slot(4096), Some(second));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn retry_keeps_a_committed_page_that_confirm_reopened() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let sealed = [5u8, 1, 2, 3, 4, 5, 6, 7];
    let newer = [6u8, 1, 2, 3, 4, 5, 6, 7];
    let other = [7u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, sealed)]).unwrap();
    log.commit_open(4, 16384, zeros, |_| Ok(true)).unwrap();
    log.note_slots(&[(4096, other)]).unwrap();
    log.commit_open(
        5,
        16384,
        |_off, _len| Err(StoreError::Corrupt("boom")),
        |_| Ok(true),
    )
    .unwrap_err();
    log.note_slots(&[(24, newer)]).unwrap();
    log.commit_open(5, 16384, zeros, |_| Ok(true)).unwrap();
    assert_eq!(log.slot(16), Some(sealed));
    assert_eq!(log.slot(24), Some(newer));
    drop(log);
    let loaded = SpendPageLog::load(&d).unwrap();
    assert_eq!(loaded.slot(16), Some(sealed));
    assert_eq!(loaded.slot(4096), Some(other));
    assert_eq!(loaded.slot(24), Some(newer));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn retry_after_hold_keeps_the_exploded_page() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let sealed = [5u8, 1, 2, 3, 4, 5, 6, 7];
    let fresh = [6u8, 1, 2, 3, 4, 5, 6, 7];
    let waiting = [7u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, sealed)]).unwrap();
    log.commit_open(9, 16384, zeros, |_| Ok(true)).unwrap();
    log.note_slots(&[(4096, fresh)]).unwrap();
    log.commit_open(
        9,
        16384,
        |_off, _len| Err(StoreError::Corrupt("boom")),
        |_| Ok(true),
    )
    .unwrap_err();
    log.note_slots(&[(8192, waiting)]).unwrap();
    log.hold_fold_above(1).unwrap();
    log.commit_open(4, 16384, zeros, |_| Ok(true)).unwrap();
    assert_eq!(log.slot(8192), Some(waiting));
    drop(log);
    let loaded = SpendPageLog::load(&d).unwrap();
    assert_eq!(loaded.slot(16), Some(sealed));
    assert_eq!(loaded.slot(4096), Some(fresh));
    assert_eq!(loaded.slot(8192), Some(waiting));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn rejected_reopen_keeps_the_sealed_page() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let sealed = [5u8, 1, 2, 3, 4, 5, 6, 7];
    let undone = [4u8, 1, 2, 3, 4, 5, 6, 7];
    let other = [6u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, sealed)]).unwrap();
    log.commit_open(4, 16384, zeros, |_| Ok(true)).unwrap();
    log.note_slots(&[(24, undone), (4096, other)]).unwrap();
    log.commit_open(5, 16384, zeros, |slot| Ok(slot[0] != 4))
        .unwrap();
    assert_eq!(log.slot(16), Some(sealed));
    drop(log);
    let loaded = SpendPageLog::load(&d).unwrap();
    assert_eq!(loaded.slot(16), Some(sealed));
    assert_eq!(loaded.slot(24), Some([0u8; 8]));
    assert_eq!(loaded.slot(4096), Some(other));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn retry_merges_a_later_note_on_the_frozen_page() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let sealed = [5u8, 1, 2, 3, 4, 5, 6, 7];
    let frozen_slot = [6u8, 1, 2, 3, 4, 5, 6, 7];
    let replaced = [8u8, 1, 2, 3, 4, 5, 6, 7];
    let added = [7u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, sealed)]).unwrap();
    log.commit_open(4, 16384, zeros, |_| Ok(true)).unwrap();
    log.note_slots(&[(24, frozen_slot), (4096, frozen_slot)])
        .unwrap();
    log.commit_open(
        5,
        16384,
        |_off, _len| Err(StoreError::Corrupt("boom")),
        |_| Ok(true),
    )
    .unwrap_err();
    log.note_slots(&[(24, replaced), (32, added)]).unwrap();
    log.commit_open(5, 16384, zeros, |_| Ok(true)).unwrap();
    drop(log);
    let loaded = SpendPageLog::load(&d).unwrap();
    assert_eq!(loaded.slot(16), Some(sealed));
    assert_eq!(loaded.slot(24), Some(replaced));
    assert_eq!(loaded.slot(32), Some(added));
    assert_eq!(loaded.slot(4096), Some(frozen_slot));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn retry_after_hold_omits_a_rejected_image_slot() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let keep_slot = [5u8, 1, 2, 3, 4, 5, 6, 7];
    let drop_slot = [4u8, 1, 2, 3, 4, 5, 6, 7];
    let noted = [6u8, 1, 2, 3, 4, 5, 6, 7];
    let fresh = [7u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, keep_slot), (24, drop_slot)]).unwrap();
    log.commit_open(9, 16384, zeros, |_| Ok(true)).unwrap();
    log.note_slots(&[(32, noted), (4096, fresh)]).unwrap();
    log.commit_open(
        9,
        16384,
        |_off, _len| Err(StoreError::Corrupt("boom")),
        |_| Ok(true),
    )
    .unwrap_err();
    log.hold_fold_above(1).unwrap();
    log.commit_open(4, 16384, zeros, |slot| Ok(slot[0] != 4))
        .unwrap();
    drop(log);
    let loaded = SpendPageLog::load(&d).unwrap();
    assert_eq!(loaded.slot(16), Some(keep_slot));
    assert_eq!(loaded.slot(24), Some([0u8; 8]));
    assert_eq!(loaded.slot(32), Some(noted));
    assert_eq!(loaded.slot(4096), Some(fresh));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn explode_keeps_a_strong_slot_and_a_newer_open_note() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let slot = [9u8, 1, 2, 3, 4, 5, 6, 7];
    let newer = [8u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, slot), (24, slot)]).unwrap();
    log.commit_open(9, 8192, zeros, |_| Ok(true)).unwrap();
    log.hold_fold_above(1).unwrap();
    log.note_slots(&[(16, newer)]).unwrap();
    log.commit_open(1, 8192, zeros, |_| Ok(true)).unwrap();
    // The open note at rel 2 wins. The image slot at rel 3 is still strong.
    assert_eq!(log.slot(16), Some(newer));
    assert_eq!(log.slot(24), Some(slot));
    assert_eq!(log.slot(0), Some([0u8; 8]));
    let _ = std::fs::remove_dir_all(&d);

    let d = dir();
    let log = SpendPageLog::create(&d);
    log.note_slots(&[(16, slot)]).unwrap();
    log.commit_open(9, 8192, zeros, |_| Ok(true)).unwrap();
    log.hold_fold_above(1).unwrap();
    log.commit_open(1, 8192, zeros, |_| Ok(true)).unwrap();
    assert_eq!(log.slot(16), Some(slot));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn hold_ignores_a_commit_at_or_below_the_tip() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let slot = [2u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, slot)]).unwrap();
    log.commit_open(3, 8192, zeros, |_| Ok(true)).unwrap();
    log.hold_fold_above(9).unwrap();
    log.commit_open(3, 8192, zeros, |_| Ok(true)).unwrap();
    let mut writes = 0u32;
    log.fold_into(
        |_off, _image| {
            writes += 1;
            Ok(())
        },
        || Ok(()),
    )
    .unwrap();
    assert_eq!(writes, 1);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn short_commit_file_is_not_rewritten_by_the_hold() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let slot = [2u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, slot)]).unwrap();
    log.commit_open(9, 8192, zeros, |_| Ok(true)).unwrap();
    let path = d.join(SPEND_PAGES_COMMIT_NAME);
    std::fs::write(&path, [0u8; 4]).unwrap();
    log.hold_fold_above(1).unwrap();
    assert_eq!(std::fs::read(&path).unwrap().len(), 4);
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
fn short_tail_is_logged_and_a_slot_past_published_is_corrupt() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let slot = [2u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, slot)]).unwrap();
    log.commit_open(4, 100, zeros, |_| Ok(true)).unwrap();
    let file = std::fs::read(d.join(SPEND_PAGES_NAME)).unwrap();
    assert_eq!(file.len(), 100);
    assert_eq!(&file[16..24], &slot);
    let _ = std::fs::remove_dir_all(&d);

    let d = dir();
    let log = SpendPageLog::create(&d);
    log.note_slots(&[(4096, slot)]).unwrap();
    let err = log.commit_open(4, 100, zeros, |_| Ok(true)).unwrap_err();
    assert!(matches!(
        err,
        StoreError::Corrupt("spend page log composed past the published body")
    ));
    let _ = std::fs::remove_dir_all(&d);

    let d = dir();
    let log = SpendPageLog::create(&d);
    log.note_slots(&[(96, slot)]).unwrap();
    let err = log.commit_open(4, 100, zeros, |_| Ok(true)).unwrap_err();
    assert!(matches!(
        err,
        StoreError::Corrupt("spend page slot past published")
    ));
    assert_eq!(log.slot(96), Some(slot));
    let _ = std::fs::remove_dir_all(&d);

    let d = dir();
    let log = SpendPageLog::create(&d);
    log.note_slots(&[(16, slot)]).unwrap();
    let err = log
        .commit_open(4, 8192, zeros, |_| Err(StoreError::Corrupt("keep")))
        .unwrap_err();
    assert!(matches!(err, StoreError::Corrupt("keep")));
    assert_eq!(log.slot(16), Some(slot));
    assert!(!d.join(SPEND_PAGES_COMMIT_NAME).exists());
    log.commit_open(4, 8192, |_off, _len| Ok(vec![7u8; 8]), |_| Ok(true))
        .unwrap();
    let file = std::fs::read(d.join(SPEND_PAGES_NAME)).unwrap();
    assert_eq!(file.len(), 4096);
    assert_eq!(file[0], 7);
    assert_eq!(file[8], 0);
    assert_eq!(&file[16..24], &slot);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn fold_does_not_write_the_spent_file_header() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let slot = [7u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, slot), (4096, slot)]).unwrap();
    log.commit_open(2, 8192, zeros, |_| Ok(true)).unwrap();
    log.commit_open(3, 8192, zeros, |_| Ok(true)).unwrap();
    let mut offs = Vec::new();
    log.fold_into(
        |off, image| {
            offs.push((off, image.len()));
            if off == 16 {
                assert_eq!(&image[..8], &slot);
            }
            Ok(())
        },
        || Ok(()),
    )
    .unwrap();
    assert_eq!(offs, vec![(16, 4096 - 16), (4096, 4096)]);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn hold_after_publish_before_install_is_not_folded() {
    let d = dir();
    let slot = [9u8, 1, 2, 3, 4, 5, 6, 7];
    let log = SpendPageLog::create(&d);
    log.note_slots(&[(16, slot)]).unwrap();
    log.prepare_commit(9, 8192, zeros, |_| Ok(true)).unwrap();
    log.hold_fold_above(1).unwrap();
    drop(log);
    let loaded = SpendPageLog::load(&d).unwrap();
    loaded.commit_open(1, 8192, zeros, |_| Ok(true)).unwrap();
    let mut writes = 0u32;
    loaded
        .fold_into(
            |_off, _image| {
                writes += 1;
                Ok(())
            },
            || Ok(()),
        )
        .unwrap();
    assert_eq!(
        writes, 0,
        "a crash before install must not fold that generation"
    );
    assert_eq!(loaded.slot(16), Some(slot));
    let _ = std::fs::remove_dir_all(&d);

    let d = dir();
    let log = SpendPageLog::create(&d);
    log.note_slots(&[(16, slot)]).unwrap();
    log.prepare_commit(9, 8192, zeros, |_| Ok(true)).unwrap();
    log.hold_fold_above(1).unwrap();
    log.install_commit().unwrap();
    log.commit_open(1, 8192, zeros, |_| Ok(true)).unwrap();
    let mut writes = 0u32;
    log.fold_into(
        |_off, _image| {
            writes += 1;
            Ok(())
        },
        || Ok(()),
    )
    .unwrap();
    assert_eq!(
        writes, 0,
        "a hold before install must survive the directory swap"
    );
    assert_eq!(log.slot(16), Some(slot));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn fold_keeps_the_log_while_a_dirtied_page_is_still_open() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let a = [1u8, 1, 2, 3, 4, 5, 6, 7];
    let b = [2u8, 1, 2, 3, 4, 5, 6, 7];
    let extra = [3u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, a)]).unwrap();
    log.commit_open(2, 8192, zeros, |_| Ok(true)).unwrap();
    log.note_slots(&[(4096, b)]).unwrap();
    log.commit_open(
        3,
        8192,
        |off, len| {
            log.note_slots(&[(4096 + 8, extra)])?;
            zeros(off, len)
        },
        |_| Ok(true),
    )
    .unwrap();
    let mut wrote = Vec::new();
    log.fold_into(
        |off, _image| {
            wrote.push(off);
            Ok(())
        },
        || Ok(()),
    )
    .unwrap();
    assert_eq!(wrote, vec![16]);
    assert!(
        d.join(SPEND_PAGES_COMMIT_NAME).exists(),
        "fold must not delete the log while a published page is still open"
    );
    drop(log);
    let loaded = SpendPageLog::load(&d).unwrap();
    assert_eq!(loaded.slot(4096), Some(b));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn explode_keep_error_keeps_the_image() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let slot = [9u8, 1, 2, 3, 4, 5, 6, 7];
    let other = [8u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, slot)]).unwrap();
    log.commit_open(9, 8192, zeros, |_| Ok(true)).unwrap();
    log.hold_fold_above(1).unwrap();
    let err = log
        .commit_open(1, 8192, zeros, |_| Err(StoreError::Corrupt("keep")))
        .unwrap_err();
    assert!(matches!(err, StoreError::Corrupt("keep")));
    assert_eq!(log.slot(16), Some(slot));
    log.note_slots(&[(4096, other)]).unwrap();
    log.commit_open(1, 8192, zeros, |_| Ok(true)).unwrap();
    assert_eq!(log.slot(16), Some(slot));
    assert_eq!(log.slot(4096), Some(other));
    let loaded = SpendPageLog::load(&d).unwrap();
    assert_eq!(loaded.slot(16), Some(slot));
    assert_eq!(loaded.slot(4096), Some(other));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn pages_shorter_than_the_commit_are_corrupt() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let slot = [4u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, slot)]).unwrap();
    log.commit_open(1, 8192, zeros, |_| Ok(true)).unwrap();
    let pages = d.join(SPEND_PAGES_NAME);
    let bytes = std::fs::read(&pages).unwrap();
    assert!(bytes.len() > 8);
    std::fs::write(&pages, &bytes[..8]).unwrap();
    assert!(matches!(
        SpendPageLog::load(&d),
        Err(StoreError::Corrupt("spend.pages short"))
    ));
    let _ = std::fs::remove_dir_all(&d);

    let d = dir();
    let log = SpendPageLog::create(&d);
    log.note_slots(&[(16, slot)]).unwrap();
    log.commit_open(1, 8192, zeros, |_| Ok(true)).unwrap();
    std::fs::remove_file(d.join(SPEND_PAGES_NAME)).unwrap();
    assert!(matches!(
        SpendPageLog::load(&d),
        Err(StoreError::Corrupt("spend.pages short"))
    ));
    let _ = std::fs::remove_dir_all(&d);

    let d = dir();
    let log = SpendPageLog::create(&d);
    log.note_slots(&[(16, slot)]).unwrap();
    log.commit_open(1, 8192, zeros, |_| Ok(true)).unwrap();
    let commit = d.join(SPEND_PAGES_COMMIT_NAME);
    let mut buf = std::fs::read(&commit).unwrap();
    buf[16..24].copy_from_slice(&0u64.to_le_bytes());
    buf[24..28].copy_from_slice(&0u32.to_le_bytes());
    buf.truncate(28);
    std::fs::write(&commit, &buf).unwrap();
    std::fs::remove_file(d.join(SPEND_PAGES_NAME)).unwrap();
    let loaded = SpendPageLog::load(&d).unwrap();
    assert_eq!(loaded.slot(16), None);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn rejected_open_page_is_not_published() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let slot = [2u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, slot)]).unwrap();
    log.commit_open(4, 8192, zeros, |_| Ok(false)).unwrap();
    assert_eq!(log.slot(16), None);
    assert!(!d.join(SPEND_PAGES_COMMIT_NAME).exists());
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn fold_of_page_zero_skips_the_file_header() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let slot = [9u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, slot)]).unwrap();
    log.commit_open(2, 8192, |_off, len| Ok(vec![0xABu8; len]), |_| Ok(true))
        .unwrap();
    log.commit_open(3, 8192, zeros, |_| Ok(true)).unwrap();
    let mut wrote = Vec::new();
    log.fold_into(
        |off, image| {
            wrote.push((off, image.to_vec()));
            Ok(())
        },
        || Ok(()),
    )
    .unwrap();
    assert_eq!(wrote.len(), 1);
    assert_eq!(
        wrote[0].0, 16,
        "page 0 fold must not pwrite the file header"
    );
    assert_eq!(wrote[0].1.len(), 4096 - 16);
    assert_eq!(&wrote[0].1[..8], &slot);
    assert_ne!(wrote[0].1[0], 0xAB);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn hold_between_prepare_and_install_is_not_folded() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let slot = [6u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, slot)]).unwrap();
    log.prepare_commit(9, 8192, zeros, |_| Ok(true)).unwrap();
    log.hold_fold_above(1).unwrap();
    log.install_commit().unwrap();
    let buf = std::fs::read(d.join(SPEND_PAGES_COMMIT_NAME)).unwrap();
    assert_eq!(u16::from_le_bytes(buf[6..8].try_into().unwrap()), 1);
    let mut writes = 0u32;
    log.fold_into(
        |_off, _image| {
            writes += 1;
            Ok(())
        },
        || Ok(()),
    )
    .unwrap();
    assert_eq!(
        writes, 0,
        "a hold after publish and before install is not folded"
    );
    assert_eq!(log.slot(16), Some(slot));
    let loaded = SpendPageLog::load(&d).unwrap();
    assert_eq!(loaded.slot(16), Some(slot));
    writes = 0;
    loaded
        .fold_into(
            |_off, _image| {
                writes += 1;
                Ok(())
            },
            || Ok(()),
        )
        .unwrap();
    assert_eq!(writes, 0, "reopen keeps the hold");
    log.commit_open(1, 8192, zeros, |_| Ok(false)).unwrap();
    writes = 0;
    log.fold_into(
        |_off, _image| {
            writes += 1;
            Ok(())
        },
        || Ok(()),
    )
    .unwrap();
    assert_eq!(
        writes, 0,
        "the next checkpoint does not copy the abandoned spend"
    );
    assert_eq!(log.slot(16), None);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn redirtied_page_stays_in_the_commit_when_an_older_page_folds() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let slot = [4u8, 1, 2, 3, 4, 5, 6, 7];
    let later = [5u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(4096, slot)]).unwrap();
    log.commit_open(2, 8192, zeros, |_| Ok(true)).unwrap();
    log.commit_open(2, 8192, zeros, |_| Ok(true)).unwrap();
    log.note_slots(&[(16, slot)]).unwrap();
    log.commit_open(
        3,
        8192,
        |off, len| {
            log.note_slots(&[(24, later)])?;
            zeros(off, len)
        },
        |_| Ok(true),
    )
    .unwrap();
    let mut wrote = Vec::new();
    log.fold_into(
        |off, _image| {
            wrote.push(off);
            Ok(())
        },
        || Ok(()),
    )
    .unwrap();
    assert_eq!(wrote, vec![4096]);
    assert!(
        d.join(SPEND_PAGES_COMMIT_NAME).exists(),
        "fold must not delete a commit that still names an unfolded page"
    );
    assert_eq!(log.slot(16), Some(slot));
    assert_eq!(log.slot(24), Some(later));
    let loaded = SpendPageLog::load(&d).unwrap();
    assert_eq!(loaded.slot(16), Some(slot));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn missing_or_short_pages_file_refuses_open() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let slot = [4u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, slot)]).unwrap();
    log.commit_open(1, 8192, zeros, |_| Ok(true)).unwrap();
    let pages = d.join(SPEND_PAGES_NAME);
    let mut bytes = std::fs::read(&pages).unwrap();
    bytes.pop();
    std::fs::write(&pages, &bytes).unwrap();
    assert!(matches!(
        SpendPageLog::load(&d),
        Err(StoreError::Corrupt("spend.pages short"))
    ));
    std::fs::remove_file(&pages).unwrap();
    assert!(matches!(
        SpendPageLog::load(&d),
        Err(StoreError::Corrupt("spend.pages short"))
    ));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn explode_keeps_the_commit_when_keep_fails() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let slot = [9u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, slot)]).unwrap();
    log.commit_open(9, 8192, zeros, |_| Ok(true)).unwrap();
    log.hold_fold_above(1).unwrap();
    let err = log
        .commit_open(1, 8192, zeros, |_| Err(StoreError::Corrupt("keep")))
        .unwrap_err();
    assert!(matches!(err, StoreError::Corrupt("keep")));
    assert_eq!(log.slot(16), Some(slot));
    assert!(d.join(SPEND_PAGES_COMMIT_NAME).exists());
    log.commit_open(1, 8192, zeros, |_| Ok(true)).unwrap();
    assert_eq!(log.slot(16), Some(slot));
    assert!(d.join(SPEND_PAGES_COMMIT_NAME).exists());
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn hold_during_fold_leaves_the_commit() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let slot = [7u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, slot), (4096, slot)]).unwrap();
    log.commit_open(9, 8192, zeros, |_| Ok(true)).unwrap();
    log.commit_open(9, 8192, zeros, |_| Ok(true)).unwrap();
    let mut wrote = Vec::new();
    log.fold_into(
        |off, _image| {
            wrote.push(off);
            log.hold_fold_above(1)?;
            Ok(())
        },
        || Ok(()),
    )
    .unwrap();
    assert_eq!(wrote, vec![16]);
    assert!(d.join(SPEND_PAGES_COMMIT_NAME).exists());
    assert_eq!(log.slot(16), Some(slot));
    assert_eq!(log.slot(4096), Some(slot));
    let mut later = 0u32;
    log.fold_into(
        |_off, _image| {
            later += 1;
            Ok(())
        },
        || Ok(()),
    )
    .unwrap();
    assert_eq!(later, 0);
    let _ = std::fs::remove_dir_all(&d);

    let d = dir();
    let log = SpendPageLog::create(&d);
    log.note_slots(&[(16, slot), (4096, slot)]).unwrap();
    log.commit_open(9, 8192, zeros, |_| Ok(true)).unwrap();
    log.commit_open(9, 8192, zeros, |_| Ok(true)).unwrap();
    let mut wrote = Vec::new();
    log.fold_into(
        |off, _image| {
            wrote.push(off);
            Ok(())
        },
        || {
            log.hold_fold_above(1)?;
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(wrote, vec![16, 4096]);
    assert!(
        d.join(SPEND_PAGES_COMMIT_NAME).exists(),
        "a hold during the body sync must not delete the log"
    );
    assert_eq!(log.slot(16), Some(slot));
    assert_eq!(log.slot(4096), Some(slot));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn reload_drops_pages_without_a_commit_and_serves_a_bad_magic() {
    let d = dir();
    std::fs::write(d.join(SPEND_PAGES_NAME), [1u8; 8]).unwrap();
    let log = SpendPageLog::load(&d).unwrap();
    assert!(!d.join(SPEND_PAGES_NAME).exists());
    assert_eq!(log.slot(16), None);
    std::fs::write(d.join(SPEND_PAGES_COMMIT_NAME), [0u8; 28]).unwrap();
    assert!(matches!(SpendPageLog::load(&d), Err(StoreError::BadMagic)));
    std::fs::write(d.join(SPEND_PAGES_COMMIT_NAME), [0u8; 4]).unwrap();
    assert!(matches!(
        SpendPageLog::load(&d),
        Err(StoreError::Corrupt("spend.pages.commit short"))
    ));
    let _ = std::fs::remove_dir_all(&d);
}

/// Confirm publishes a longer `spent.body` end while fold copies cooled pages.
/// The fold must not store the shorter length it sampled.
#[test]
fn fold_does_not_publish_a_shorter_body_end() {
    use rbitcoin_primitives::Fk;
    use rbitcoin_store::io_backend::WriteIoBackend;
    use rbitcoin_store::Store;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::Arc;

    const PAGE: u64 = 4096;
    let d = dir();
    let store = Arc::new(Store::create_tiny(&d).unwrap());
    let nbytes = (256 * PAGE) as usize;
    let prep = store
        .txs
        .spent
        .prepare_batch_encode(1, nbytes, |_i, buf| {
            buf.resize(buf.len() + nbytes, 0);
        })
        .unwrap()
        .expect("one spent append");
    store
        .txs
        .spent
        .write_body_blob_bulk(prep.start, &prep.body_blob)
        .unwrap();
    store.txs.spent.finish_prepared(prep).unwrap();
    let end = store.txs.spent.body_published_len();
    let mut slots = Vec::new();
    let mut abs = 16u64;
    while abs.saturating_add(8) <= end {
        slots.push(abs);
        let next = (abs & !(PAGE - 1)).saturating_add(PAGE);
        if next <= abs {
            break;
        }
        abs = next;
    }
    assert!(
        slots.len() >= 64,
        "fold needs many pages, got {}",
        slots.len()
    );

    let mut saw_shrink = false;
    for _ in 0..4 {
        let mut edges = Vec::with_capacity(slots.len());
        let mut known = Vec::with_capacity(slots.len());
        for (i, &abs) in slots.iter().enumerate() {
            edges.push((abs, Fk(1), 0, Fk(10 + i as u64), 0));
            known.push((Fk(1), 0u8, 0));
        }
        let cold = store
            .put_spend_batch_by_abs_meta_known(&edges, &known, WriteIoBackend::Pwrite)
            .unwrap();
        assert!(cold.is_empty());
        store.checkpoint_observed_spend().unwrap();

        let stop = Arc::new(AtomicBool::new(false));
        let shrunk = Arc::new(AtomicBool::new(false));
        let base = store.txs.spent.body.logical_len();
        let landed = Arc::new(AtomicU64::new(base));
        let next = Arc::new(AtomicU64::new(base.saturating_add(8)));
        let cap = base.saturating_add(32 * 1024 * 1024);
        let racer = {
            let store = Arc::clone(&store);
            let stop = Arc::clone(&stop);
            let shrunk = Arc::clone(&shrunk);
            let landed = Arc::clone(&landed);
            let next = Arc::clone(&next);
            std::thread::spawn(move || {
                let spent = &store.txs.spent;
                while !stop.load(Ordering::Relaxed) {
                    let n = next.fetch_add(8, Ordering::Relaxed);
                    if n >= cap {
                        if spent.body.logical_len() < cap - 8 {
                            shrunk.store(true, Ordering::Release);
                            break;
                        }
                        std::hint::spin_loop();
                        continue;
                    }
                    let _ = spent.note_body_end(n, true);
                    if spent.body.logical_len() < n {
                        shrunk.store(true, Ordering::Release);
                        break;
                    }
                    landed.fetch_max(n, Ordering::Release);
                }
            })
        };
        let started = next.load(Ordering::Relaxed);
        while next.load(Ordering::Relaxed) == started
            && started < cap
            && !shrunk.load(Ordering::Relaxed)
        {
            std::hint::spin_loop();
        }
        store.checkpoint_observed_spend().unwrap();
        stop.store(true, Ordering::Release);
        racer.join().unwrap();
        let end = store.txs.spent.body.logical_len();
        let posted = landed.load(Ordering::Acquire);
        if shrunk.load(Ordering::Acquire) || end < posted {
            saw_shrink = true;
            break;
        }
        assert!(
            !d.join(SPEND_PAGES_COMMIT_NAME).exists(),
            "the second checkpoint must fold the cooled pages"
        );
    }
    assert!(
        !saw_shrink,
        "fold stored a spent.body end confirm had already grown past"
    );
    let _ = std::fs::remove_dir_all(&d);
}
