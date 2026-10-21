//! `--trust-peer` script posture on `accept_and_connect_block`.
use bitcoin::absolute::LockTime;
use bitcoin::hashes::Hash;
use bitcoin::transaction::Version as TxVersion;
use bitcoin::{Amount, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut};
use rbitcoin_consensus::{mine_empty_regtest, prepare_regtest_candidate, ChainParams, Milestone};
use rbitcoin_log::{capture_logs, take_logs};
use rbitcoin_primitives::Height;
use rbitcoin_query::{accept_and_connect_block, pad_empty_from};

fn bad_script_spend(op: OutPoint, sats: u64) -> Transaction {
    Transaction {
        version: TxVersion::ONE,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: op,
            script_sig: ScriptBuf::from_bytes(vec![0x6a]),
            sequence: Sequence::MAX,
            witness: bitcoin::Witness::new(),
        }],
        output: vec![TxOut {
            value: Amount::from_sat(sats),
            script_pubkey: ScriptBuf::from_bytes(vec![0x51]),
        }],
    }
}

fn block_with_spend(
    prev: bitcoin::BlockHash,
    time: u32,
    height: u32,
    tx: Transaction,
) -> bitcoin::Block {
    let mut block = mine_empty_regtest(prev, time, height);
    block.txdata.push(tx);
    prepare_regtest_candidate(&mut block, prev, time);
    block
}

fn log_text() -> String {
    take_logs()
        .into_iter()
        .map(|(_, line)| line)
        .collect::<Vec<_>>()
        .join("\n")
}

/// The production session sets the cell for one batch. These tests do that
/// themselves. `Query::trust_peer` does not.
fn opt_in<R>(body: impl FnOnce() -> R) -> R {
    rbitcoin_query::confirm_run::with_confirm_waive(true, body)
}

fn opt_in_if<R>(trust: bool, body: impl FnOnce() -> R) -> R {
    if trust {
        opt_in(body)
    } else {
        body()
    }
}

/// Above the milestone a bad script warns and connects. Flag off rejects with
/// no `trust-peer:` line. A height-only milestone still skips that script.
#[test]
fn trust_peer_script_above_milestone() {
    let (_dir, q) = rbitcoin_query::testutil::tiny_query_labeled("trust-peer-script");
    let params = ChainParams::regtest();
    let genesis = bitcoin::blockdata::constants::genesis_block(bitcoin::Network::Regtest);
    accept_and_connect_block(&q, &params, Height::GENESIS, &genesis, Milestone::NONE).unwrap();
    let maturity = params.coinbase_maturity();
    let (tip, tip_time, cbs) = pad_empty_from(
        &q,
        &params,
        genesis.block_hash(),
        genesis.header.time,
        1,
        maturity + 1,
        2,
    );
    assert_eq!(q.tip_height(), Some(Height(maturity + 1)));
    let cb1 = cbs[0];
    let cb2 = cbs[1];

    let skip_h = maturity + 2;
    let skip_block = block_with_spend(
        tip,
        tip_time + 600,
        skip_h,
        bad_script_spend(OutPoint { txid: cb2, vout: 0 }, 49_0000_0000),
    );
    let ms_skip = Milestone::height(skip_h);
    assert!(ms_skip.skips_scripts_at(skip_h));
    capture_logs(true);
    accept_and_connect_block(&q, &params, Height(skip_h), &skip_block, ms_skip)
        .expect("height-only milestone still skips a bad script");
    let skipped = log_text();
    assert!(
        !skipped.contains("trust-peer:"),
        "milestone skip must not emit a trust-peer line: {skipped}"
    );
    assert!(
        !skipped.contains("block-script-verify"),
        "skipped scripts must not report a script failure: {skipped}"
    );
    assert_eq!(q.tip_height(), Some(Height(skip_h)));
    let skipped_txid = skip_block.txdata[1].compute_txid();
    assert!(
        q.tx_fk_by_txid(skipped_txid.as_byte_array())
            .unwrap()
            .is_some(),
        "skipped bad script still stores the txid"
    );

    let check_h = skip_h + 1;
    let check_block = block_with_spend(
        skip_block.block_hash(),
        skip_block.header.time + 600,
        check_h,
        bad_script_spend(OutPoint { txid: cb1, vout: 0 }, 48_0000_0000),
    );
    let ms_check = Milestone::height(skip_h);
    assert!(!ms_check.skips_scripts_at(check_h));
    capture_logs(true);
    let err = accept_and_connect_block(&q, &params, Height(check_h), &check_block, ms_check)
        .expect_err("flag off rejects a bad script above the milestone");
    let msg = err.to_string().to_lowercase();
    assert!(
        msg.contains("script") || msg.contains("opcode") || msg.contains("return"),
        "expected a script failure, got: {err}"
    );
    let off = log_text();
    assert!(
        !off.contains("trust-peer:"),
        "flag off must not prefix trust-peer: {off}"
    );
    assert_eq!(q.tip_height(), Some(Height(skip_h)));

    q.set_trust_peer(true);
    capture_logs(true);
    opt_in(|| accept_and_connect_block(&q, &params, Height(check_h), &check_block, ms_check))
        .expect("trust-peer applies a bad script above the milestone");
    let on = log_text();
    let hash = check_block.block_hash();
    let needle = format!("trust-peer: applied block {hash} height {check_h}:");
    assert!(
        on.contains(&needle),
        "missing applied line {needle} in {on}"
    );
    assert!(
        on.contains("block-script-verify-flag-failed"),
        "applied line must keep the script reject reason: {on}"
    );
    assert_eq!(q.tip_height(), Some(Height(check_h)));
    let txid = check_block.txdata[1].compute_txid();
    assert!(
        q.tx_fk_by_txid(txid.as_byte_array()).unwrap().is_some(),
        "applied bad script stores the txid"
    );

    let spend_out = check_block.txdata[1].compute_txid();
    let two_h = check_h + 1;
    let mut two = block_with_spend(
        check_block.block_hash(),
        check_block.header.time + 600,
        two_h,
        bad_script_spend(
            OutPoint {
                txid: spend_out,
                vout: 0,
            },
            48_0000_0000,
        ),
    );
    two.txdata[0].output[0].value = Amount::from_sat(50_0000_0000 + 1);
    prepare_regtest_candidate(
        &mut two,
        check_block.block_hash(),
        check_block.header.time + 600,
    );
    capture_logs(true);
    opt_in(|| accept_and_connect_block(&q, &params, Height(two_h), &two, ms_check))
        .expect("script and subsidy faults both apply");
    let both = log_text();
    let two_hash = two.block_hash();
    let two_needle = format!("trust-peer: applied block {two_hash} height {two_h}:");
    assert_eq!(
        both.matches(&two_needle).count(),
        2,
        "two warnable faults, two lines: {both}"
    );
    assert!(both.contains("block-script-verify-flag-failed"), "{both}");
    assert!(both.contains("bad-cb-amount"), "{both}");
    assert_eq!(q.tip_height(), Some(Height(two_h)));
    capture_logs(false);
}

#[test]
fn trust_peer_missing_prevout_not_applied() {
    let (_dir, q) = rbitcoin_query::testutil::tiny_query_labeled("trust-peer-missing");
    let params = ChainParams::regtest();
    let genesis = bitcoin::blockdata::constants::genesis_block(bitcoin::Network::Regtest);
    accept_and_connect_block(&q, &params, Height::GENESIS, &genesis, Milestone::NONE).unwrap();
    let b1 = mine_empty_regtest(genesis.block_hash(), genesis.header.time + 600, 1);
    accept_and_connect_block(&q, &params, Height(1), &b1, Milestone::NONE).unwrap();
    let phantom = block_with_spend(
        b1.block_hash(),
        b1.header.time + 600,
        2,
        bad_script_spend(
            OutPoint {
                txid: bitcoin::Txid::from_byte_array([0xcd; 32]),
                vout: 0,
            },
            1,
        ),
    );
    capture_logs(true);
    let err = accept_and_connect_block(&q, &params, Height(2), &phantom, Milestone::NONE)
        .expect_err("flag off missing prevout");
    assert!(
        matches!(
            err,
            rbitcoin_query::ConfirmError::Rules(
                rbitcoin_consensus::ConsensusError::MissingPrevout
                    | rbitcoin_consensus::ConsensusError::BadTx("bad-txns-inputs-missingorspent")
            )
        ),
        "flag off: {err}"
    );
    let off = log_text();
    assert!(!off.contains("trust-peer:"), "{off}");
    assert_eq!(q.tip_height(), Some(Height(1)));

    q.set_trust_peer(true);
    capture_logs(true);
    let err =
        opt_in(|| accept_and_connect_block(&q, &params, Height(2), &phantom, Milestone::NONE))
            .expect_err("missing prevout is not applied");
    assert!(
        matches!(
            err,
            rbitcoin_query::ConfirmError::Rules(rbitcoin_consensus::ConsensusError::MissingPrevout)
        ),
        "trust-peer still returns the stop error: {err}"
    );
    let on = log_text();
    let hash = phantom.block_hash();
    assert!(
        on.contains(&format!("trust-peer: not applied block {hash} height 2:")),
        "{on}"
    );
    assert!(on.contains("bad-txns-inputs-missingorspent"), "{on}");
    assert_eq!(q.tip_height(), Some(Height(1)));
    capture_logs(false);
}

fn connect_staged(
    q: &rbitcoin_query::Query,
    params: &ChainParams,
    height: u32,
    block: &bitcoin::Block,
    ms: Milestone,
) -> Result<Vec<rbitcoin_primitives::Fk>, rbitcoin_query::ConfirmError> {
    use rbitcoin_query::confirm_run::{
        confirm_scripts_phase, confirm_wire_load_from_plan, confirm_wire_lookup_stamp,
        confirm_write_phase, ScriptPreverified,
    };
    let items = [(Height(height), std::sync::Arc::new(block.clone()), None)];
    let stamped = confirm_wire_lookup_stamp(q, params, ms, &items, None)?;
    let mat = confirm_wire_load_from_plan(q, params, ms, stamped, None, &ScriptPreverified::new())?;
    let ok = confirm_scripts_phase(mat.batch)?;
    confirm_write_phase(q, params, ms, ok.batch)
}

/// The IBD load, script, and write stages each collect. Two bad scripts in one
/// block are two applied lines and the tip moves.
#[test]
fn trust_peer_staged_two_scripts() {
    use bitcoin::WPubkeyHash;
    let (_dir, q) = rbitcoin_query::testutil::tiny_query_labeled("trust-peer-staged");
    let params = ChainParams::regtest();
    let genesis = bitcoin::blockdata::constants::genesis_block(bitcoin::Network::Regtest);
    accept_and_connect_block(&q, &params, Height::GENESIS, &genesis, Milestone::NONE).unwrap();
    let maturity = params.coinbase_maturity();
    let (tip, tip_time, cbs) = pad_empty_from(
        &q,
        &params,
        genesis.block_hash(),
        genesis.header.time,
        1,
        maturity + 1,
        2,
    );
    let wpkh = WPubkeyHash::from_byte_array([0x11; 20]);
    let p2wpkh = ScriptBuf::new_p2wpkh(&wpkh);
    let fund_h = maturity + 2;
    let fund = block_with_spend(
        tip,
        tip_time + 600,
        fund_h,
        Transaction {
            version: TxVersion::ONE,
            lock_time: LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint {
                    txid: cbs[0],
                    vout: 0,
                },
                script_sig: ScriptBuf::new(),
                sequence: Sequence::MAX,
                witness: bitcoin::Witness::new(),
            }],
            output: vec![TxOut {
                value: Amount::from_sat(49_0000_0000),
                script_pubkey: p2wpkh,
            }],
        },
    );
    accept_and_connect_block(&q, &params, Height(fund_h), &fund, Milestone::NONE)
        .expect("fund p2wpkh");
    let fund_txid = fund.txdata[1].compute_txid();
    let bad_h = fund_h + 1;
    let mut bad = block_with_spend(
        fund.block_hash(),
        fund.header.time + 600,
        bad_h,
        bad_script_spend(
            OutPoint {
                txid: cbs[1],
                vout: 0,
            },
            1_000,
        ),
    );
    bad.txdata.push(Transaction {
        version: TxVersion::TWO,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint {
                txid: fund_txid,
                vout: 0,
            },
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: bitcoin::Witness::new(),
        }],
        output: vec![TxOut {
            value: Amount::from_sat(48_0000_0000),
            script_pubkey: ScriptBuf::from_bytes(vec![0x51]),
        }],
    });
    prepare_regtest_candidate(&mut bad, fund.block_hash(), fund.header.time + 600);
    capture_logs(false);
    capture_logs(true);
    let off = connect_staged(&q, &params, bad_h, &bad, Milestone::NONE).unwrap_err();
    assert!(
        off.to_string().to_lowercase().contains("script"),
        "staged flag off rejects: {off}"
    );
    let off_log = log_text();
    assert!(!off_log.contains("trust-peer:"), "{off_log}");
    assert_eq!(q.tip_height(), Some(Height(fund_h)));

    q.set_trust_peer(true);
    capture_logs(true);
    opt_in(|| connect_staged(&q, &params, bad_h, &bad, Milestone::NONE)).expect("staged apply");
    let on = log_text();
    let hash = bad.block_hash();
    let needle = format!("trust-peer: applied block {hash} height {bad_h}:");
    assert!(
        on.matches(&needle).count() >= 2,
        "two script faults, two lines: {on}"
    );
    assert_eq!(q.tip_height(), Some(Height(bad_h)));
    capture_logs(false);
}

fn same_block_coinbase_spend(
    prev: &bitcoin::Block,
    height: u32,
    cb_value: u64,
    spend_value: u64,
) -> bitcoin::Block {
    let mut block = mine_empty_regtest(prev.block_hash(), prev.header.time + 600, height);
    block.txdata[0].output[0].value = Amount::from_sat(cb_value);
    let cb_txid = block.txdata[0].compute_txid();
    block.txdata.push(Transaction {
        version: TxVersion::ONE,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint {
                txid: cb_txid,
                vout: 0,
            },
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: bitcoin::Witness::new(),
        }],
        output: vec![TxOut {
            value: Amount::from_sat(spend_value),
            script_pubkey: ScriptBuf::from_bytes(vec![0x51]),
        }],
    });
    prepare_regtest_candidate(&mut block, prev.block_hash(), prev.header.time + 600);
    block
}

/// A same-block coinbase spend of that whole output must not hide an excess
/// coinbase. A zero-value spend would make `in == out == 0` and hide a fee
/// reject.
#[test]
fn trust_peer_same_block_coinbase_keeps_subsidy() {
    let (_dir, q) = rbitcoin_query::testutil::tiny_query_labeled("trust-peer-cb");
    let params = ChainParams::regtest();
    let genesis = bitcoin::blockdata::constants::genesis_block(bitcoin::Network::Regtest);
    accept_and_connect_block(&q, &params, Height::GENESIS, &genesis, Milestone::NONE).unwrap();
    let b1 = mine_empty_regtest(genesis.block_hash(), genesis.header.time + 600, 1);
    accept_and_connect_block(&q, &params, Height(1), &b1, Milestone::NONE).unwrap();
    let subsidy = rbitcoin_consensus::block_subsidy(2, &params) as u64;
    let cb_value = subsidy + 1;
    let block = same_block_coinbase_spend(&b1, 2, cb_value, cb_value);
    capture_logs(false);
    capture_logs(true);
    let off =
        accept_and_connect_block(&q, &params, Height(2), &block, Milestone::NONE).unwrap_err();
    assert!(
        off.to_string().contains("premature") || off.to_string().contains("immature"),
        "flag off stops on the coinbase spend: {off}"
    );
    let off_log = log_text();
    assert!(!off_log.contains("trust-peer:"), "{off_log}");
    assert_eq!(q.tip_height(), Some(Height(1)));

    q.set_trust_peer(true);
    capture_logs(true);
    opt_in(|| accept_and_connect_block(&q, &params, Height(2), &block, Milestone::NONE))
        .expect("waived coinbase spend still checks subsidy");
    let on = log_text();
    let hash = block.block_hash();
    let needle = format!("trust-peer: applied block {hash} height 2:");
    assert!(on.contains("bad-txns-premature-spend-of-coinbase"), "{on}");
    assert!(on.contains("bad-cb-amount"), "{on}");
    assert!(
        on.matches(&needle).count() >= 2,
        "premature and subsidy are both lines: {on}"
    );
    assert_eq!(q.tip_height(), Some(Height(2)));
    let fees = q
        .block_txstat_rows(Height(2))
        .unwrap()
        .expect("txstat")
        .rows
        .expect("stamped fees");
    assert_eq!(fees[0].0, 0, "the waived coinbase is not a fee: {fees:?}");
    capture_logs(false);
}

/// Creating sats beyond every input, including the waived coinbase, stays a
/// hard reject. The block is not applied.
#[test]
fn trust_peer_same_block_coinbase_inflation_is_not_applied() {
    let (_dir, q) = rbitcoin_query::testutil::tiny_query_labeled("trust-peer-cb-inflate");
    let params = ChainParams::regtest();
    let genesis = bitcoin::blockdata::constants::genesis_block(bitcoin::Network::Regtest);
    accept_and_connect_block(&q, &params, Height::GENESIS, &genesis, Milestone::NONE).unwrap();
    let b1 = mine_empty_regtest(genesis.block_hash(), genesis.header.time + 600, 1);
    accept_and_connect_block(&q, &params, Height(1), &b1, Milestone::NONE).unwrap();
    let subsidy = rbitcoin_consensus::block_subsidy(2, &params) as u64;
    let block = same_block_coinbase_spend(&b1, 2, subsidy, subsidy + 1);
    capture_logs(false);
    capture_logs(true);
    q.set_trust_peer(true);
    let err = opt_in(|| accept_and_connect_block(&q, &params, Height(2), &block, Milestone::NONE))
        .unwrap_err();
    assert!(
        err.to_string().contains("in < out"),
        "value created above every input stays inflation: {err}"
    );
    let on = log_text();
    let hash = block.block_hash();
    let needle = format!("trust-peer: not applied block {hash} height 2:");
    assert!(on.contains(&needle), "{on}");
    assert!(on.contains("bad-txns-in-belowout"), "{on}");
    assert!(
        !on.contains(&format!("trust-peer: applied block {hash} height 2:")),
        "{on}"
    );
    assert_eq!(q.tip_height(), Some(Height(1)));
    capture_logs(false);
}

/// A body whose transactions do not match the header merkle root is not
/// indexed. The tip stays put and the log says the block was not applied.
#[test]
fn trust_peer_merkle_mismatch_is_not_applied() {
    let (_dir, q) = rbitcoin_query::testutil::tiny_query_labeled("trust-peer-merkle");
    let params = ChainParams::regtest();
    let genesis = bitcoin::blockdata::constants::genesis_block(bitcoin::Network::Regtest);
    accept_and_connect_block(&q, &params, Height::GENESIS, &genesis, Milestone::NONE).unwrap();
    let b1 = mine_empty_regtest(genesis.block_hash(), genesis.header.time + 600, 1);
    accept_and_connect_block(&q, &params, Height(1), &b1, Milestone::NONE).unwrap();
    let mut bad = mine_empty_regtest(b1.block_hash(), b1.header.time + 600, 2);
    // Keep a unique coinbase and a header that still meets its target. Only
    // the committed root is wrong, which is the warn-and-apply merkle arm.
    bad.header.merkle_root = bitcoin::TxMerkleNode::from_byte_array([0x11; 32]);
    for _ in 0..10_000 {
        bad.header.nonce = bad.header.nonce.wrapping_add(1);
        if bad.header.validate_pow(bad.header.target()).is_ok() {
            break;
        }
    }
    assert!(
        bad.header.validate_pow(bad.header.target()).is_ok(),
        "fixture must meet proof-of-work so the reject is the merkle root"
    );
    q.set_trust_peer(true);
    capture_logs(false);
    capture_logs(true);
    let err = opt_in(|| accept_and_connect_block(&q, &params, Height(2), &bad, Milestone::NONE));
    let logs = log_text();
    capture_logs(false);
    assert!(err.is_err(), "merkle mismatch must not connect: {err:?}");
    assert_eq!(q.tip_height(), Some(Height(1)));
    assert!(logs.contains("trust-peer: not applied"), "{logs}");
    assert!(
        logs.contains("bad-txnmrklroot") || logs.contains("merkle"),
        "{logs}"
    );
    assert!(
        !logs.contains("trust-peer: applied"),
        "a merkle miss must not be applied: {logs}"
    );
}

/// A child that burns the forwarded coinbase must not turn that value into a
/// fee. Otherwise subsidy+fee covers the excess coinbase.
#[test]
fn trust_peer_forwarded_coinbase_is_not_a_fee() {
    let (_dir, q) = rbitcoin_query::testutil::tiny_query_labeled("trust-peer-cb-fwd");
    let params = ChainParams::regtest();
    let genesis = bitcoin::blockdata::constants::genesis_block(bitcoin::Network::Regtest);
    accept_and_connect_block(&q, &params, Height::GENESIS, &genesis, Milestone::NONE).unwrap();
    let b1 = mine_empty_regtest(genesis.block_hash(), genesis.header.time + 600, 1);
    accept_and_connect_block(&q, &params, Height(1), &b1, Milestone::NONE).unwrap();
    let subsidy = rbitcoin_consensus::block_subsidy(2, &params) as u64;
    let excess = 1_000u64;
    let cb_value = subsidy + excess;
    let mut block = mine_empty_regtest(b1.block_hash(), b1.header.time + 600, 2);
    block.txdata[0].output[0].value = Amount::from_sat(cb_value);
    let cb_txid = block.txdata[0].compute_txid();
    let spk = ScriptBuf::from_bytes(vec![0x51]);
    block.txdata.push(Transaction {
        version: TxVersion::ONE,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint {
                txid: cb_txid,
                vout: 0,
            },
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: bitcoin::Witness::new(),
        }],
        output: vec![TxOut {
            value: Amount::from_sat(cb_value),
            script_pubkey: spk.clone(),
        }],
    });
    let mid_txid = block.txdata[1].compute_txid();
    block.txdata.push(Transaction {
        version: TxVersion::ONE,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint {
                txid: mid_txid,
                vout: 0,
            },
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: bitcoin::Witness::new(),
        }],
        output: vec![TxOut {
            value: Amount::from_sat(subsidy),
            script_pubkey: spk,
        }],
    });
    prepare_regtest_candidate(&mut block, b1.block_hash(), b1.header.time + 600);
    capture_logs(false);
    capture_logs(true);
    q.set_trust_peer(true);
    opt_in(|| accept_and_connect_block(&q, &params, Height(2), &block, Milestone::NONE))
        .expect("forwarded coinbase still connects");
    let on = log_text();
    let hash = block.block_hash();
    let needle = format!("trust-peer: applied block {hash} height 2:");
    assert!(on.contains("bad-txns-premature-spend-of-coinbase"), "{on}");
    assert!(
        on.contains("bad-cb-amount"),
        "burning the forward must not pay the excess: {on}"
    );
    assert!(on.contains(&needle), "{on}");
    let fees = q
        .block_txstat_rows(Height(2))
        .unwrap()
        .expect("txstat")
        .rows
        .expect("stamped fees");
    assert_eq!(fees[0].0, 0, "direct spend fee: {fees:?}");
    assert_eq!(fees[1].0, 0, "child must not fee the coinbase: {fees:?}");
    capture_logs(false);
}

/// Two non-coinbase script failures, loaded with the waive bit set when
/// `trust` is true. Two jobs, so the wave runs off this thread.
fn loaded_two_bad_scripts(
    label: &str,
    trust: bool,
) -> (impl Sized, rbitcoin_query::confirm_run::LoadedBatch) {
    use bitcoin::WPubkeyHash;
    use rbitcoin_query::confirm_run::{
        confirm_wire_load_from_plan, confirm_wire_lookup_stamp, ScriptPreverified,
    };
    let (dir, q) = rbitcoin_query::testutil::tiny_query_labeled(label);
    let params = ChainParams::regtest();
    let genesis = bitcoin::blockdata::constants::genesis_block(bitcoin::Network::Regtest);
    accept_and_connect_block(&q, &params, Height::GENESIS, &genesis, Milestone::NONE).unwrap();
    let maturity = params.coinbase_maturity();
    let (tip, tip_time, cbs) = pad_empty_from(
        &q,
        &params,
        genesis.block_hash(),
        genesis.header.time,
        1,
        maturity + 1,
        2,
    );
    let wpkh = WPubkeyHash::from_byte_array([0x11; 20]);
    let fund_h = maturity + 2;
    let fund = block_with_spend(
        tip,
        tip_time + 600,
        fund_h,
        Transaction {
            version: TxVersion::ONE,
            lock_time: LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint {
                    txid: cbs[0],
                    vout: 0,
                },
                script_sig: ScriptBuf::new(),
                sequence: Sequence::MAX,
                witness: bitcoin::Witness::new(),
            }],
            output: vec![TxOut {
                value: Amount::from_sat(49_0000_0000),
                script_pubkey: ScriptBuf::new_p2wpkh(&wpkh),
            }],
        },
    );
    accept_and_connect_block(&q, &params, Height(fund_h), &fund, Milestone::NONE).unwrap();
    let fund_txid = fund.txdata[1].compute_txid();
    let bad_h = fund_h + 1;
    let mut bad = block_with_spend(
        fund.block_hash(),
        fund.header.time + 600,
        bad_h,
        bad_script_spend(
            OutPoint {
                txid: cbs[1],
                vout: 0,
            },
            1_000,
        ),
    );
    bad.txdata.push(Transaction {
        version: TxVersion::TWO,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint {
                txid: fund_txid,
                vout: 0,
            },
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: bitcoin::Witness::new(),
        }],
        output: vec![TxOut {
            value: Amount::from_sat(48_0000_0000),
            // The label is not spent. It only makes two chains hash apart
            // so the per-thread trust log does not treat them as one block.
            script_pubkey: ScriptBuf::from_bytes(
                std::iter::once(0x51u8).chain(label.bytes()).collect(),
            ),
        }],
    });
    prepare_regtest_candidate(&mut bad, fund.block_hash(), fund.header.time + 600);
    q.set_trust_peer(trust);
    let ms = Milestone::NONE;
    let items = [(Height(bad_h), std::sync::Arc::new(bad), None)];
    let load = || {
        let stamped = confirm_wire_lookup_stamp(&q, &params, ms, &items, None).unwrap();
        confirm_wire_load_from_plan(&q, &params, ms, stamped, None, &ScriptPreverified::new())
            .unwrap()
    };
    let mat = opt_in_if(trust, load);
    ((dir, q), mat.batch)
}

fn batch_needle(batch: &rbitcoin_query::confirm_run::LoadedBatch) -> String {
    let p = &batch.prepared[0];
    let hash = bitcoin::BlockHash::from_byte_array(p.hash);
    format!("trust-peer: applied block {hash} height {}:", p.height.0)
}

/// A clean height-1 block on its own query, loaded with the waive bit set.
fn loaded_clean_height1(
    label: &str,
) -> (
    impl Sized,
    bitcoin::BlockHash,
    rbitcoin_query::confirm_run::LoadedBatch,
) {
    use rbitcoin_query::confirm_run::{
        confirm_wire_load_from_plan, confirm_wire_lookup_stamp, ScriptPreverified,
    };
    let (dir, q) = rbitcoin_query::testutil::tiny_query_labeled(label);
    let params = ChainParams::regtest();
    let genesis = bitcoin::blockdata::constants::genesis_block(bitcoin::Network::Regtest);
    accept_and_connect_block(&q, &params, Height::GENESIS, &genesis, Milestone::NONE).unwrap();
    let mut b1 = mine_empty_regtest(genesis.block_hash(), genesis.header.time + 600, 1);
    prepare_regtest_candidate(&mut b1, genesis.block_hash(), genesis.header.time + 600);
    let hash = b1.block_hash();
    q.set_trust_peer(true);
    let ms = Milestone::NONE;
    let items = [(Height(1), std::sync::Arc::new(b1), None)];
    let mat = opt_in(|| {
        let stamped = confirm_wire_lookup_stamp(&q, &params, ms, &items, None).unwrap();
        confirm_wire_load_from_plan(&q, &params, ms, stamped, None, &ScriptPreverified::new())
            .unwrap()
    });
    ((dir, q), hash, mat.batch)
}

/// The second in-flight batch still applies after the first batch finishes.
#[test]
fn review_trust_second_inflight_batch_still_applies() {
    use rbitcoin_query::confirm_run::scripts::Inflight;
    let (_ga, batch_a) = loaded_two_bad_scripts("review-trust-a", true);
    let (_gb, batch_b) = loaded_two_bad_scripts("review-trust-b", true);
    let needle_b = batch_needle(&batch_b);
    assert!(
        !rbitcoin_consensus::collecting(),
        "no collector before the stage"
    );
    let a = Inflight::start(batch_a, 0).expect("start a");
    let b = Inflight::start(batch_b, 0).expect("start b");
    capture_logs(false);
    capture_logs(true);
    let ra = a.finish();
    assert!(
        ra.is_ok(),
        "first batch applies: {:?}",
        ra.err().map(|e| e.to_string())
    );
    let rb = b.finish();
    let logs = log_text();
    capture_logs(false);
    assert!(
        rb.is_ok(),
        "second in-flight batch should warn-and-apply, got {:?}\nlogs:\n{logs}",
        rb.err().map(|e| e.to_string())
    );
    assert!(
        logs.contains(&needle_b),
        "second batch logs its own block: {logs}"
    );
}

/// A third batch that starts after the first finishes must not receive the
/// second batch's faults.
#[test]
fn review_trust_overlap_logs_under_own_block() {
    use rbitcoin_query::confirm_run::scripts::Inflight;
    let (_ga, batch_a) = loaded_two_bad_scripts("review-trust-a2", true);
    let (_gb, batch_b) = loaded_two_bad_scripts("review-trust-b2", true);
    let needle_b = batch_needle(&batch_b);
    let b_h = batch_b.prepared[0].height.0;
    let (_gc, c_hash, batch_c) = loaded_clean_height1("review-trust-c2");
    let a = Inflight::start(batch_a, 0).expect("start a");
    let b = Inflight::start(batch_b, 0).expect("start b");
    a.finish().expect("a applies");
    let c = Inflight::start(batch_c, 0).expect("start c");
    capture_logs(false);
    capture_logs(true);
    let rb = b.finish();
    assert!(
        rb.is_ok(),
        "b applies while c is in flight: {:?}",
        rb.err().map(|e| e.to_string())
    );
    c.finish().expect("c applies");
    let logs = log_text();
    capture_logs(false);
    let wrong = format!("trust-peer: applied block {c_hash} height {b_h}:");
    assert!(
        !logs.contains(&wrong),
        "b's faults are logged under c's hash {c_hash}:\n{logs}"
    );
    assert!(
        logs.contains(&needle_b),
        "b's faults name b's block: {logs}"
    );
}

/// The mode bit does not waive this thread. A connect opts in for its batch.
#[test]
fn review_trust_mode_does_not_waive_the_thread() {
    let (_dir, q) = rbitcoin_query::testutil::tiny_query_labeled("waive-default");
    q.set_trust_peer(true);
    assert!(
        !rbitcoin_query::confirm_run::confirm_waive(&q),
        "Query::trust_peer waived this thread without an opt-in"
    );
}

/// A connect that panics must not leave the thread waive bit set for the
/// next caller. That next caller did not come from the trusted session.
#[test]
fn review_trust_waive_override_resets_on_panic() {
    let (_dir, q) = rbitcoin_query::testutil::tiny_query_labeled("waive-panic");
    q.set_trust_peer(false);
    let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        rbitcoin_query::confirm_run::with_confirm_waive(true, || panic!("connect"))
    }));
    assert!(panicked.is_err());
    assert!(
        !rbitcoin_query::confirm_run::confirm_waive(&q),
        "a panicked connect left the waive bit set"
    );
}

/// A batch that is not waived rejects a bad script and logs nothing.
#[test]
fn review_trust_unwaived_inflight_rejects() {
    use rbitcoin_query::confirm_run::scripts::Inflight;
    let (_g, batch) = loaded_two_bad_scripts("review-trust-off", false);
    assert!(!batch.trust_peer);
    let inf = Inflight::start(batch, 0).expect("start");
    capture_logs(false);
    capture_logs(true);
    let err = match inf.finish() {
        Err(e) => e,
        Ok(_) => panic!("unwaived script is an error"),
    };
    let logs = log_text();
    capture_logs(false);
    assert!(err.to_string().to_lowercase().contains("script"), "{err}");
    assert!(!logs.contains("trust-peer:"), "{logs}");
}
