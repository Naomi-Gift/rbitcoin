//! Admit applies constant scriptCode and a legacy sighash byte cap.
//! Block connect keeps the consensus flag word.

use bitcoin::absolute::LockTime;
use bitcoin::hashes::Hash;
use bitcoin::transaction::Version;
use bitcoin::{Amount, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Witness};
use rbitcoin_consensus::mine_regtest_paying;
use rbitcoin_consensus::script::interpreter::MAX_UNCONFIRMED_LEGACY_SIGHASH_BYTES;
use rbitcoin_net::chain::tiny_regtest_hub_labeled;
use rbitcoin_net::tx_relay::MempoolHub;
use rbitcoin_primitives::Height;
use std::sync::Arc;

const DUMMY_SIG: [u8; 9] = [0x30, 0x06, 0x02, 0x01, 0x01, 0x02, 0x01, 0x01, 0x01];
/// Compressed secp256k1 generator. It parses, so CHECKSIG reaches sighash.
const PUBKEY: [u8; 33] = [
    0x02, 0x79, 0xBE, 0x66, 0x7E, 0xF9, 0xDC, 0xBB, 0xAC, 0x55, 0xA0, 0x62, 0x95, 0xCE, 0x87, 0x0B,
    0x07, 0x02, 0x9B, 0xFC, 0xDB, 0x2D, 0xCE, 0x28, 0xD9, 0x59, 0xF2, 0x81, 0x5B, 0x16, 0xF8, 0x17,
    0x98,
];

fn op_true() -> ScriptBuf {
    ScriptBuf::from_bytes(vec![0x51])
}

fn push_sig_pubkey() -> Vec<u8> {
    let mut v = Vec::with_capacity(44);
    v.push(DUMMY_SIG.len() as u8);
    v.extend_from_slice(&DUMMY_SIG);
    v.push(PUBKEY.len() as u8);
    v.extend_from_slice(&PUBKEY);
    v
}

/// CHECKSIG runs in the scriptPubKey. The signature stays in scriptSig, so
/// FindAndDelete does not see it in scriptCode.
fn checksig_script(repeats: u64) -> (ScriptBuf, ScriptBuf) {
    let chunk = push_sig_pubkey();
    let mut script_sig = Vec::with_capacity(chunk.len() * repeats as usize);
    let mut spk = Vec::with_capacity(repeats as usize * 2 + 1);
    for _ in 0..repeats {
        script_sig.extend_from_slice(&chunk);
        spk.push(0xac);
        spk.push(0x75);
    }
    spk.push(0x51);
    assert!(script_sig.len() <= 10_000 && spk.len() <= 10_000);
    (
        ScriptBuf::from_bytes(script_sig),
        ScriptBuf::from_bytes(spk),
    )
}

fn spend(prev: &Transaction, script_sig: ScriptBuf) -> Transaction {
    let value = prev.output[0].value.to_sat() - 20_000;
    Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint {
                txid: prev.compute_txid(),
                vout: 0,
            },
            script_sig,
            sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
            witness: Witness::new(),
        }],
        output: vec![TxOut {
            value: Amount::from_sat(value),
            script_pubkey: op_true(),
        }],
    }
}

fn coinbase_at(hub: &rbitcoin_net::ChainHub, height: u32) -> Transaction {
    hub.query
        .reconstruct_block_at_height(Height(height))
        .unwrap()
        .txdata
        .into_iter()
        .next()
        .unwrap()
}

/// Smallest repeat count whose `n * tx.total_size()` exceeds the admit cap.
fn over_cap_repeats(prev: &Transaction) -> u64 {
    let mut repeats = 1u64;
    loop {
        let (script_sig, _) = checksig_script(repeats);
        let cost = repeats * spend(prev, script_sig).total_size() as u64;
        if cost > MAX_UNCONFIRMED_LEGACY_SIGHASH_BYTES {
            return repeats;
        }
        repeats += 1;
        assert!(
            repeats <= 200,
            "legacy sighash cap is not reachable under the script size cap"
        );
    }
}

#[test]
fn admit_rejects_codeseparator_and_legacy_sighash_work_block_connect_accepts() {
    let (dir, hub) = tiny_regtest_hub_labeled("unconf-script");
    hub.ensure_genesis().unwrap();
    let genesis = hub.tip_hash().unwrap();
    let t0 = hub.tip_header().unwrap().time;
    let probe = Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint {
                txid: bitcoin::Txid::from_byte_array([1u8; 32]),
                vout: 0,
            },
            script_sig: ScriptBuf::new(),
            sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
            witness: Witness::new(),
        }],
        output: vec![TxOut {
            value: Amount::from_sat(50_0000_0000),
            script_pubkey: op_true(),
        }],
    };
    let over_n = over_cap_repeats(&probe);
    let (under_sig, under_spk) = checksig_script(1);
    let (over_sig, over_spk) = checksig_script(over_n);
    let codesep = ScriptBuf::from_bytes(vec![0xab, 0x51]);
    let b1 = mine_regtest_paying(genesis, t0.saturating_add(1), 1, codesep, vec![]);
    hub.accept_block(b1.clone()).unwrap();
    let b2 = mine_regtest_paying(b1.block_hash(), t0.saturating_add(2), 2, under_spk, vec![]);
    hub.accept_block(b2.clone()).unwrap();
    let b3 = mine_regtest_paying(b2.block_hash(), t0.saturating_add(3), 3, over_spk, vec![]);
    hub.accept_block(b3).unwrap();
    hub.confirm_empty_regtest(100).unwrap();
    assert_eq!(hub.tip_height(), Some(103));

    let mp = MempoolHub::open(dir.join("mp"), Arc::clone(&hub.query)).unwrap();
    mp.set_relay_enabled(true);

    let codesep_cb = coinbase_at(&hub, 1);
    let codesep_spend = spend(&codesep_cb, ScriptBuf::new());
    let err = mp
        .accept_tx(&codesep_spend)
        .expect_err("constant scriptCode rejects OP_CODESEPARATOR");
    assert!(err.to_string().contains("OP_CODESEPARATOR"), "admit: {err}");
    assert!(!mp.contains(&codesep_spend.compute_txid()));

    let prev = hub.tip_hash().unwrap();
    let time = hub.tip_header().unwrap().time.saturating_add(1);
    let height = hub.tip_height().unwrap().saturating_add(1);
    let block = mine_regtest_paying(prev, time, height, op_true(), vec![codesep_spend]);
    hub.accept_block(block)
        .expect("consensus flags still accept OP_CODESEPARATOR OP_TRUE");
    assert_eq!(hub.tip_height(), Some(104));

    let under_cb = coinbase_at(&hub, 2);
    let under = spend(&under_cb, under_sig);
    let under_cost = under.total_size() as u64;
    assert!(under_cost <= MAX_UNCONFIRMED_LEGACY_SIGHASH_BYTES);
    mp.accept_tx(&under)
        .expect("one legacy sighash of a small tx stays under the cap");
    assert!(mp.contains(&under.compute_txid()));

    let over_cb = coinbase_at(&hub, 3);
    let over = spend(&over_cb, over_sig);
    let err = mp
        .accept_tx(&over)
        .expect_err("repeated full-tx legacy sighashes exceed the cap");
    assert!(
        err.to_string().contains("legacy sighash work"),
        "admit: {err}"
    );
    assert!(!mp.contains(&over.compute_txid()));
    let _ = std::fs::remove_dir_all(dir);
}
