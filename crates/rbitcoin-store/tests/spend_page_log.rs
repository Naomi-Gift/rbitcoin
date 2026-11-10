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
    assert_eq!(file.len(), 4096);
    assert_eq!(&file[16..24], &first);
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
fn compose_rejects_the_live_tail_and_a_keep_error() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let slot = [2u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, slot)]).unwrap();
    let err = log.commit_open(4, 100, zeros, |_| Ok(true)).unwrap_err();
    assert!(matches!(
        err,
        StoreError::Corrupt("spend page log composed the live tail")
    ));
    assert_eq!(log.slot(16), Some(slot));
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
