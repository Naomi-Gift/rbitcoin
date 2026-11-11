use rbitcoin_store::error::StoreError;
use rbitcoin_store::spend_page_log::{
    pace_after_checkpoint, spend_checkpoint_early, spend_note_arms, SpendPageLog, LOG_IMAGE_CAP,
    OPEN_EDGE_CAP, SPEND_PAGES_COMMIT_NAME, SPEND_PAGES_NAME,
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
    assert!(spend_note_arms(0, LOG_IMAGE_CAP, false));
    assert!(!spend_note_arms(0, LOG_IMAGE_CAP, true));
    assert!(spend_note_arms(OPEN_EDGE_CAP, LOG_IMAGE_CAP, true));
    assert!(!spend_note_arms(
        OPEN_EDGE_CAP - 1,
        LOG_IMAGE_CAP - 1,
        false
    ));
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
    match SpendPageLog::load(&d) {
        Err(err) => assert!(matches!(
            err,
            StoreError::Corrupt("spend page commit names bytes past the committed length")
        )),
        Ok(_) => panic!("load served a commit record past the committed length"),
    }
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn short_tail_does_not_hide_a_later_body_slot() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let early = [9u8, 1, 2, 3, 4, 5, 6, 7];
    let later = [8u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, early)]).unwrap();
    log.commit_open(1, 100, zeros, |_| Ok(true)).unwrap();
    assert_eq!(log.slot(16), Some(early));
    assert_eq!(
        log.slot(200),
        None,
        "a committed tail shorter than the page leaves the later slot on the body"
    );
    let mut body = vec![0u8; 4096];
    body[200..208].copy_from_slice(&later);
    log.note_slots(&[(24, early)]).unwrap();
    assert_eq!(
        log.slot(200),
        None,
        "an open note does not hide slots past its short base"
    );
    log.commit_open(2, 4096, |_off, len| Ok(body[..len].to_vec()), |_| Ok(true))
        .unwrap();
    let file = std::fs::read(d.join(SPEND_PAGES_NAME)).unwrap();
    assert_eq!(&file[200..208], &later, "compose copies the body tail");
    assert_eq!(log.slot(200), Some(later));
    log.commit_open(3, 4096, |_off, len| Ok(body[..len].to_vec()), |_| Ok(true))
        .unwrap();
    let mut folded = None;
    log.fold_into(
        |off, image| {
            folded = Some((off, image.to_vec()));
            Ok(())
        },
        || Ok(()),
    )
    .unwrap();
    let (off, image) = folded.expect("the extended page folds");
    assert_eq!(off, 16);
    assert_eq!(
        &image[184..192],
        &later,
        "fold writes the body tail, not zeros"
    );
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn checkpoint_rewrites_the_log_down_to_pages_still_named() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let slot = [3u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, slot), (4096, slot)]).unwrap();
    log.commit_open(2, 8192, zeros, |_| Ok(true)).unwrap();
    log.commit_open(3, 8192, zeros, |_| Ok(true)).unwrap();
    log.note_slots(&[(4096 + 8, slot)]).unwrap();
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
    log.commit_open(4, 8192, zeros, |_| Ok(true)).unwrap();
    let file = std::fs::read(d.join(SPEND_PAGES_NAME)).unwrap();
    assert_eq!(file.len(), 4096, "a folded page is not left in the log");
    assert_eq!(&file[..8], &slot);
    assert_eq!(&file[8..16], &slot);
    assert_eq!(log.slot(16), None);
    assert_eq!(log.slot(4096), Some(slot));
    assert_eq!(log.slot(4104), Some(slot));
    let loaded = SpendPageLog::load(&d).unwrap();
    assert_eq!(loaded.slot(4096), Some(slot));
    assert_eq!(loaded.slot(4104), Some(slot));
    assert_eq!(loaded.slot(16), None);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn reload_prefers_a_generation_file_over_a_stale_pages_file() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let slot = [9u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, slot)]).unwrap();
    log.commit_open(3, 8192, zeros, |_| Ok(true)).unwrap();
    let pages = d.join(SPEND_PAGES_NAME);
    let image = std::fs::read(&pages).unwrap();
    let commit = std::fs::read(d.join(SPEND_PAGES_COMMIT_NAME)).unwrap();
    let generation = u32::from_le_bytes(commit[12..16].try_into().unwrap());
    let gen_path = d.join(format!("{SPEND_PAGES_NAME}.{generation}"));
    std::fs::write(&gen_path, &image).unwrap();
    std::fs::write(&pages, vec![0u8; image.len()]).unwrap();
    let loaded = SpendPageLog::load(&d).unwrap();
    assert_eq!(loaded.slot(16), Some(slot));
    std::fs::write(&gen_path, [0u8; 1]).unwrap();
    match SpendPageLog::load(&d) {
        Err(err) => assert!(matches!(err, StoreError::Corrupt("spend.pages short"))),
        Ok(_) => panic!("a short generation file fell back to spend.pages"),
    }
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn a_failed_pages_rename_does_not_publish_that_generation_again() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let first = [9u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, first)]).unwrap();
    // The generation file cannot replace a directory, so the pages rename fails
    // after the commit already names that generation.
    std::fs::create_dir(d.join(SPEND_PAGES_NAME)).unwrap();
    log.commit_open(3, 8192, zeros, |_| Ok(true)).unwrap();
    assert_eq!(log.slot(16), Some(first));
    let commit = std::fs::read(d.join(SPEND_PAGES_COMMIT_NAME)).unwrap();
    let generation = u32::from_le_bytes(commit[12..16].try_into().unwrap());
    let gen_path = d.join(format!("{SPEND_PAGES_NAME}.{generation}"));
    let image = std::fs::read(&gen_path).unwrap();
    assert_eq!(&image[16..24], &first);
    let second = [8u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(4096, second)]).unwrap();
    log.commit_open(4, 8192, zeros, |_| Ok(true)).unwrap();
    assert_eq!(std::fs::read(&gen_path).unwrap(), image);
    let commit = std::fs::read(d.join(SPEND_PAGES_COMMIT_NAME)).unwrap();
    let next = u32::from_le_bytes(commit[12..16].try_into().unwrap());
    assert_ne!(next, generation);
    assert!(d.join(format!("{SPEND_PAGES_NAME}.{next}")).is_file());
    let loaded = SpendPageLog::load(&d).unwrap();
    assert_eq!(loaded.slot(16), Some(first));
    assert_eq!(loaded.slot(4096), Some(second));
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
fn open_edge_cap_waits_only_after_the_sync_thread_is_bound() {
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
    let started = Instant::now();
    log.note_slots(&[(n * 8, slot)]).unwrap();
    log.wait_open_edge_room();
    let unbound = started.elapsed();
    assert!(
        unbound < Duration::from_millis(500),
        "replay must not wait for a sync thread that is not running, waited {unbound:?}"
    );
    log.bind_sync_wake(Arc::new((
        std::sync::Mutex::new(false),
        std::sync::Condvar::new(),
    )));
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
    log.wait_open_edge_room();
    let waited = started.elapsed();
    worker.join().unwrap();
    assert!(
        waited >= Duration::from_millis(60),
        "the next batch waits for a checkpoint once the sync thread is bound, waited {waited:?}"
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
fn loaded_commit_above_the_checkpoint_is_not_folded() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let slot = [5u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, slot)]).unwrap();
    log.commit_open(5, 8192, zeros, |_| Ok(true)).unwrap();
    drop(log);
    let log = SpendPageLog::load(&d).unwrap();
    log.commit_open(4, 8192, zeros, |_| Ok(false)).unwrap();
    let mut writes = 0u32;
    log.fold_into(
        |_off, image| {
            writes += 1;
            assert!(
                !image.windows(8).any(|w| w == slot),
                "a not-strong slot from a higher commit must not reach the body"
            );
            Ok(())
        },
        || Ok(()),
    )
    .unwrap();
    assert_eq!(writes, 0);
    assert_ne!(log.slot(16), Some(slot));
    assert!(!d.join(SPEND_PAGES_COMMIT_NAME).exists());
    assert!(!d.join(SPEND_PAGES_NAME).exists());
    log.hold_fold_above(0).unwrap();
    assert!(!d.join(SPEND_PAGES_COMMIT_NAME).exists());
    let loaded = SpendPageLog::load(&d).unwrap();
    assert_ne!(loaded.slot(16), Some(slot));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn loaded_commit_above_the_checkpoint_keeps_a_strong_slot() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let strong = [1u8, 1, 2, 3, 4, 5, 6, 7];
    let stale = [2u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, strong), (24, stale)]).unwrap();
    log.commit_open(5, 8192, zeros, |_| Ok(true)).unwrap();
    drop(log);
    let log = SpendPageLog::load(&d).unwrap();
    log.commit_open(4, 8192, zeros, |meta| Ok(meta[0] == 1))
        .unwrap();
    let mut writes = 0u32;
    log.fold_into(
        |_off, image| {
            writes += 1;
            assert!(!image.windows(8).any(|w| w == stale));
            Ok(())
        },
        || Ok(()),
    )
    .unwrap();
    assert_eq!(writes, 0);
    assert_eq!(log.slot(16), Some(strong));
    assert_ne!(log.slot(24), Some(stale));
    let loaded = SpendPageLog::load(&d).unwrap();
    assert_eq!(loaded.slot(16), Some(strong));
    assert_ne!(loaded.slot(24), Some(stale));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn loaded_commit_above_the_checkpoint_does_not_carry_a_stale_page() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let stale = [3u8, 1, 2, 3, 4, 5, 6, 7];
    let fresh = [4u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, stale)]).unwrap();
    log.commit_open(5, 8192, zeros, |_| Ok(true)).unwrap();
    drop(log);
    let log = SpendPageLog::load(&d).unwrap();
    log.note_slots(&[(4096, fresh)]).unwrap();
    log.commit_open(4, 8192, zeros, |meta| Ok(meta[0] == 4))
        .unwrap();
    let mut writes = 0u32;
    log.fold_into(
        |_off, image| {
            writes += 1;
            assert!(!image.windows(8).any(|w| w == stale));
            Ok(())
        },
        || Ok(()),
    )
    .unwrap();
    assert_eq!(writes, 0);
    assert_eq!(log.slot(4096), Some(fresh));
    assert_ne!(log.slot(16), Some(stale));
    let loaded = SpendPageLog::load(&d).unwrap();
    assert_eq!(loaded.slot(4096), Some(fresh));
    assert_ne!(loaded.slot(16), Some(stale));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn note_during_explode_is_not_folded_at_the_lower_height() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let old = [5u8, 1, 2, 3, 4, 5, 6, 7];
    let fresh = [6u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, old)]).unwrap();
    log.commit_open(5, 8192, zeros, |_| Ok(true)).unwrap();
    log.hold_fold_above(4).unwrap();
    let mut noted = false;
    log.commit_open(4, 8192, zeros, |meta| {
        if !noted {
            noted = true;
            log.note_slots(&[(4096, fresh)]).unwrap();
        }
        Ok(*meta == fresh)
    })
    .unwrap();
    assert!(noted);
    // The replacement spend is still best-chain. This checkpoint sampled 4.
    assert_eq!(log.slot(4096), Some(fresh));
    log.commit_open(4, 8192, zeros, |_| Ok(false)).unwrap();
    let mut wrote_fresh = false;
    log.fold_into(
        |_off, image| {
            if image.windows(8).any(|w| w == fresh) {
                wrote_fresh = true;
            }
            Ok(())
        },
        || Ok(()),
    )
    .unwrap();
    assert!(
        !wrote_fresh,
        "a spend noted during explode must not fold at the lower checkpoint"
    );
    assert_ne!(log.slot(4096), Some(fresh));
    let _ = std::fs::remove_dir_all(&d);
}

fn body_with(slots: &[(u64, [u8; 8])]) -> impl Fn(u64, usize) -> Result<Vec<u8>, StoreError> + '_ {
    move |off, len| {
        let mut buf = vec![0u8; len];
        for &(abs, meta) in slots {
            if abs < off {
                continue;
            }
            let at = (abs - off) as usize;
            if at + 8 <= buf.len() {
                buf[at..at + 8].copy_from_slice(&meta);
            }
        }
        Ok(buf)
    }
}

#[test]
fn rejected_spend_already_on_the_body_is_not_folded() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let live = [1u8, 1, 2, 3, 4, 5, 6, 7];
    let stale = [2u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, live), (24, stale)]).unwrap();
    log.commit_open(5, 8192, zeros, |_| Ok(true)).unwrap();
    log.hold_fold_above(4).unwrap();
    let planted = [(16, live), (24, stale)];
    let read = body_with(&planted);
    log.commit_open(4, 8192, &read, |meta| Ok(meta[0] == 1))
        .unwrap();
    assert_eq!(log.slot(16), Some(live));
    assert_ne!(log.slot(24), Some(stale));
    log.commit_open(4, 8192, &read, |meta| Ok(meta[0] == 1))
        .unwrap();
    log.fold_into(
        |_off, image| {
            assert!(
                !image.windows(8).any(|w| w == stale),
                "a rejected spend already on the body must not be folded again"
            );
            Ok(())
        },
        || Ok(()),
    )
    .unwrap();
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn rejected_page_already_on_the_body_is_cleared() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let stale = [3u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, stale)]).unwrap();
    log.commit_open(5, 8192, zeros, |_| Ok(true)).unwrap();
    log.hold_fold_above(4).unwrap();
    let planted = [(16, stale)];
    let read = body_with(&planted);
    log.commit_open(4, 8192, &read, |_| Ok(false)).unwrap();
    assert_ne!(log.slot(16), Some(stale));
    log.commit_open(4, 8192, &read, |_| Ok(false)).unwrap();
    let mut cleared = false;
    log.fold_into(
        |off, image| {
            assert!(
                !image.windows(8).any(|w| w == stale),
                "a rejected page already on the body must not be folded again"
            );
            if off == 16 && image.len() >= 8 && image[..8] == [0u8; 8] {
                cleared = true;
            }
            Ok(())
        },
        || Ok(()),
    )
    .unwrap();
    assert!(
        cleared,
        "fold must write zeros over a rejected spend the body already has"
    );
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn rejected_body_survives_a_failed_clear() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let stale = [3u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, stale)]).unwrap();
    log.commit_open(5, 8192, zeros, |_| Ok(true)).unwrap();
    log.hold_fold_above(4).unwrap();
    let planted = [(16, stale)];
    let err = log
        .commit_open(
            4,
            8192,
            |_off, _len| Err(StoreError::Corrupt("body read failed")),
            |_| Ok(false),
        )
        .unwrap_err();
    assert!(matches!(err, StoreError::Corrupt("body read failed")));
    assert!(
        d.join(SPEND_PAGES_COMMIT_NAME).exists(),
        "a failed clear must leave the commit so a restart can retry"
    );
    assert_eq!(log.slot(16), Some(stale));
    let loaded = SpendPageLog::load(&d).unwrap();
    assert_eq!(loaded.slot(16), Some(stale));
    let read = body_with(&planted);
    log.commit_open(4, 8192, &read, |_| Ok(false)).unwrap();
    assert_ne!(log.slot(16), Some(stale));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn rejected_clear_survives_a_partial_keep_read_error() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let live = [1u8, 1, 2, 3, 4, 5, 6, 7];
    let stale = [2u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, live), (24, stale)]).unwrap();
    log.commit_open(5, 8192, zeros, |_| Ok(true)).unwrap();
    log.hold_fold_above(4).unwrap();
    let err = log
        .commit_open(
            4,
            8192,
            |_off, _len| Err(StoreError::Corrupt("body read failed")),
            |meta| Ok(meta[0] == 1),
        )
        .unwrap_err();
    assert!(matches!(err, StoreError::Corrupt("body read failed")));
    assert!(d.join(SPEND_PAGES_COMMIT_NAME).exists());
    assert_eq!(log.slot(16), Some(live));
    assert_eq!(
        log.slot(24),
        Some(stale),
        "a failed clear must keep serving the old image"
    );
    let planted = [(24, stale)];
    let read = body_with(&planted);
    log.commit_open(4, 8192, &read, |meta| Ok(meta[0] == 1))
        .unwrap();
    assert_eq!(log.slot(16), Some(live));
    assert_ne!(log.slot(24), Some(stale));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn rejected_clear_survives_a_lower_checkpoint() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let stale = [3u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, stale)]).unwrap();
    log.commit_open(5, 8192, zeros, |_| Ok(true)).unwrap();
    log.hold_fold_above(4).unwrap();
    let planted = [(16, stale)];
    let read = body_with(&planted);
    log.commit_open(4, 8192, &read, |_| Ok(false)).unwrap();
    assert_eq!(log.slot(16), Some([0u8; 8]));
    log.hold_fold_above(2).unwrap();
    log.commit_open(2, 8192, &read, |_| Ok(false)).unwrap();
    assert!(
        d.join(SPEND_PAGES_COMMIT_NAME).exists(),
        "a later checkpoint must not delete the zero image"
    );
    assert_eq!(log.slot(16), Some([0u8; 8]));
    let mut cleared = false;
    log.fold_into(
        |off, image| {
            assert!(!image.windows(8).any(|w| w == stale));
            if off == 16 && image.len() >= 8 && image[..8] == [0u8; 8] {
                cleared = true;
            }
            Ok(())
        },
        || Ok(()),
    )
    .unwrap();
    assert!(
        cleared,
        "fold must write the zeros the clear already stored"
    );
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn stored_zero_survives_a_note_during_the_hold() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let stale = [3u8, 1, 2, 3, 4, 5, 6, 7];
    let strong = [1u8, 1, 2, 3, 4, 5, 6, 7];
    let fresh = [8u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, stale)]).unwrap();
    log.commit_open(5, 8192, zeros, |_| Ok(true)).unwrap();
    log.hold_fold_above(4).unwrap();
    let planted = [(16, stale)];
    let read = body_with(&planted);
    log.commit_open(4, 8192, &read, |_| Ok(false)).unwrap();
    assert_eq!(log.slot(16), Some([0u8; 8]));
    log.note_slots(&[(4096, strong)]).unwrap();
    log.commit_open(4, 8192, &read, |_| Ok(true)).unwrap();
    assert_eq!(log.slot(4096), Some(strong));
    log.hold_fold_above(2).unwrap();
    let mut noted = false;
    log.commit_open(2, 8192, &read, |meta| {
        if !noted {
            noted = true;
            log.note_slots(&[(24, fresh)]).unwrap();
        }
        let _ = meta;
        Ok(true)
    })
    .unwrap();
    assert!(noted);
    log.commit_open(2, 8192, &read, |_| Ok(true)).unwrap();
    assert_eq!(log.slot(24), Some(fresh));
    assert_eq!(
        log.slot(16),
        Some([0u8; 8]),
        "a note on a zero page must not bring the rejected spend back"
    );
    log.commit_open(2, 8192, &read, |_| Ok(true)).unwrap();
    let mut wrote_stale = false;
    log.fold_into(
        |_off, image| {
            if image.windows(8).any(|w| w == stale) {
                wrote_stale = true;
            }
            Ok(())
        },
        || Ok(()),
    )
    .unwrap();
    assert!(
        !wrote_stale,
        "fold must not write the rejected spend the zero image cleared"
    );
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn stored_zero_survives_a_strong_slot() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let strong = [1u8, 1, 2, 3, 4, 5, 6, 7];
    let stale = [2u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, strong), (24, stale)]).unwrap();
    log.commit_open(5, 8192, zeros, |_| Ok(true)).unwrap();
    log.hold_fold_above(4).unwrap();
    let planted = [(24, stale)];
    let read = body_with(&planted);
    log.commit_open(4, 8192, &read, |meta| Ok(meta[0] == 1))
        .unwrap();
    assert_eq!(log.slot(16), Some(strong));
    assert_eq!(log.slot(24), Some([0u8; 8]));
    log.hold_fold_above(2).unwrap();
    log.commit_open(2, 8192, &read, |_| Ok(true)).unwrap();
    assert_eq!(log.slot(16), Some(strong));
    assert_eq!(
        log.slot(24),
        Some([0u8; 8]),
        "a strong slot must not rebuild the page from the body"
    );
    let mut writes = 0u32;
    log.fold_into(
        |_off, image| {
            writes += 1;
            assert!(!image.windows(8).any(|w| w == stale));
            Ok(())
        },
        || Ok(()),
    )
    .unwrap();
    assert_eq!(writes, 0, "this checkpoint must not fold the held page");
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn prior_zero_survives_a_new_reject() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let live = [1u8, 1, 2, 3, 4, 5, 6, 7];
    let stale = [2u8, 1, 2, 3, 4, 5, 6, 7];
    let fresh = [4u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, live), (24, stale), (32, fresh)])
        .unwrap();
    log.commit_open(5, 8192, zeros, |_| Ok(true)).unwrap();
    log.hold_fold_above(4).unwrap();
    let first_slots = [(24, stale)];
    let first = body_with(&first_slots);
    log.commit_open(4, 8192, &first, |meta| Ok(meta[0] != 2))
        .unwrap();
    assert_eq!(log.slot(24), Some([0u8; 8]));
    assert_eq!(log.slot(32), Some(fresh));
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
    log.hold_fold_above(2).unwrap();
    let planted = [(24, stale), (32, fresh)];
    let read = body_with(&planted);
    log.commit_open(2, 8192, &read, |meta| Ok(meta[0] == 1))
        .unwrap();
    assert_eq!(log.slot(16), Some(live));
    assert_ne!(log.slot(32), Some(fresh));
    assert_eq!(
        log.slot(24),
        Some([0u8; 8]),
        "a stored zero must stay when another slot on the page is rejected"
    );
    log.commit_open(2, 8192, &read, |meta| Ok(meta[0] == 1))
        .unwrap();
    let mut wrote_stale = false;
    log.fold_into(
        |_off, image| {
            if image.windows(8).any(|w| w == stale) {
                wrote_stale = true;
            }
            Ok(())
        },
        || Ok(()),
    )
    .unwrap();
    assert!(
        !wrote_stale,
        "fold must not write the spend the stored zero already cleared"
    );
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn prior_zero_survives_a_reject_the_body_lacks() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let stale = [2u8, 1, 2, 3, 4, 5, 6, 7];
    let fresh = [4u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(24, stale), (32, fresh)]).unwrap();
    log.commit_open(5, 8192, zeros, |_| Ok(true)).unwrap();
    log.hold_fold_above(4).unwrap();
    let first_slots = [(24, stale)];
    let first = body_with(&first_slots);
    log.commit_open(4, 8192, &first, |meta| Ok(meta[0] == 4))
        .unwrap();
    assert_eq!(log.slot(24), Some([0u8; 8]));
    assert_eq!(log.slot(32), Some(fresh));
    log.hold_fold_above(2).unwrap();
    let planted = [(24, stale)];
    let read = body_with(&planted);
    log.commit_open(2, 8192, &read, |_| Ok(false)).unwrap();
    assert!(
        d.join(SPEND_PAGES_COMMIT_NAME).exists(),
        "a stored zero must keep the commit when the new reject is not on the body"
    );
    assert_eq!(log.slot(24), Some([0u8; 8]));
    assert_ne!(log.slot(32), Some(fresh));
    log.commit_open(2, 8192, &read, |_| Ok(false)).unwrap();
    let mut wrote_stale = false;
    log.fold_into(
        |_off, image| {
            if image.windows(8).any(|w| w == stale) {
                wrote_stale = true;
            }
            Ok(())
        },
        || Ok(()),
    )
    .unwrap();
    assert!(!wrote_stale);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn strong_sibling_survives_a_reject_the_body_lacks() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let strong = [1u8, 1, 2, 3, 4, 5, 6, 7];
    let stale = [2u8, 1, 2, 3, 4, 5, 6, 7];
    let reject = [3u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, stale)]).unwrap();
    log.commit_open(6, 12288, zeros, |_| Ok(true)).unwrap();
    log.hold_fold_above(5).unwrap();
    let zero_slots = [(16, stale)];
    let zero_body = body_with(&zero_slots);
    log.commit_open(5, 12288, &zero_body, |_| Ok(false))
        .unwrap();
    assert_eq!(log.slot(16), Some([0u8; 8]));
    log.note_slots(&[(4096, strong), (8192, reject)]).unwrap();
    log.commit_open(5, 12288, &zero_body, |_| Ok(true)).unwrap();
    assert_eq!(log.slot(4096), Some(strong));
    assert_eq!(log.slot(8192), Some(reject));
    log.hold_fold_above(2).unwrap();
    log.commit_open(2, 12288, &zero_body, |meta| Ok(meta[0] == 1))
        .unwrap();
    assert!(
        d.join(SPEND_PAGES_COMMIT_NAME).exists(),
        "a best-chain page must keep the commit when another page is rejected"
    );
    assert_eq!(log.slot(4096), Some(strong));
    assert_eq!(log.slot(16), Some([0u8; 8]));
    assert_ne!(log.slot(8192), Some(reject));
    let loaded = SpendPageLog::load(&d).unwrap();
    assert_eq!(loaded.slot(4096), Some(strong));
    assert_eq!(loaded.slot(16), Some([0u8; 8]));
    assert_ne!(loaded.slot(8192), Some(reject));
    drop(loaded);
    let mut wrote_strong = false;
    let mut wrote_reject = false;
    log.fold_into(
        |_off, image| {
            if image.windows(8).any(|w| w == strong) {
                wrote_strong = true;
            }
            if image.windows(8).any(|w| w == reject) {
                wrote_reject = true;
            }
            Ok(())
        },
        || Ok(()),
    )
    .unwrap();
    assert!(wrote_strong, "fold must write the best-chain sibling");
    assert!(!wrote_reject, "fold must not write the rejected page");
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn strong_sibling_survives_a_reject_already_on_the_body() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let strong = [1u8, 1, 2, 3, 4, 5, 6, 7];
    let stale = [2u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, stale), (4096, strong)]).unwrap();
    log.commit_open(5, 8192, zeros, |_| Ok(true)).unwrap();
    log.hold_fold_above(4).unwrap();
    let planted = [(16, stale)];
    let read = body_with(&planted);
    log.commit_open(4, 8192, &read, |meta| Ok(meta[0] == 1))
        .unwrap();
    assert_eq!(
        log.slot(4096),
        Some(strong),
        "a best-chain page must stay when the rejected page is cleared"
    );
    assert_eq!(log.slot(16), Some([0u8; 8]));
    drop(log);
    let loaded = SpendPageLog::load(&d).unwrap();
    assert_eq!(loaded.slot(4096), Some(strong));
    assert_eq!(loaded.slot(16), Some([0u8; 8]));
    let mut wrote_stale = false;
    loaded
        .fold_into(
            |_off, image| {
                if image.windows(8).any(|w| w == stale) {
                    wrote_stale = true;
                }
                Ok(())
            },
            || Ok(()),
        )
        .unwrap();
    assert!(!wrote_stale);
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
fn keep_error_during_scrub_leaves_the_open_note() {
    let d = dir();
    let log = SpendPageLog::create(&d);
    let first = [9u8, 1, 2, 3, 4, 5, 6, 7];
    let later = [8u8, 1, 2, 3, 4, 5, 6, 7];
    log.note_slots(&[(16, first)]).unwrap();
    let err = log.commit_open(
        3,
        8192,
        |_off, len| {
            log.note_slots(&[(4096, later)]).unwrap();
            Ok(vec![0u8; len])
        },
        |meta| {
            if meta == &later {
                Err(StoreError::Corrupt("keep failed"))
            } else {
                Ok(true)
            }
        },
    );
    assert!(err.is_err());
    assert_eq!(log.slot(4096), Some(later));
    log.commit_open(4, 8192, zeros, |_| Ok(true)).unwrap();
    assert_eq!(log.slot(4096), Some(later));
    let loaded = SpendPageLog::load(&d).unwrap();
    assert_eq!(loaded.slot(4096), Some(later));
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
