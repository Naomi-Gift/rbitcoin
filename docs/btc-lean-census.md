# btc-lean consensus census on rbitcoin

Each row of the vendored consensus census is listed once. A path is a file in this repository that implements the check, tests it, or both. A mapped row is not a claim that a Lean theorem holds of the Rust, and it is not a claim of byte-for-byte agreement with Bitcoin Core.

- pin: `4f5c4f858fb9ca0743042644453cfa92d30dd271`
- url: `https://github.com/Rob1Ham/btc-lean`
- gitlink: `third_party/btc-lean`
- census: `third_party/btc-lean/spec/db/rules.yaml`
- core pin in that census: Bitcoin Core `cce32b168` (their tree). Script corpora in this repo track Bitcoin Core v31.1.

Obsolete rows stay in the list. They have a reason and no path. Wallet rows (`spec/db/rules-wallet.yaml`) are not in this census and are not listed.

Transaction and block wire framing (`TXV-011` through `TXV-016`) is the rust-bitcoin decoder called from `check_block_wire`. The production path is that call. CompactSize and witness-flag rules are not reimplemented beside it. Header work (`INV-141`, `BLK-113`) is rust-bitcoin `Target::to_work`, called from `most_work.rs`. ECDSA and Schnorr group operations are the secp256k1 binding and are unmapped on purpose.

### INV-001
- summary: '∀tx. tx.vin ≠ ∅'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### INV-002
- summary: 'tx.outputs ≠ ∅'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### INV-003
- summary: 'weight(noWitness tx) ≤ 4_000_000'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### INV-004
- summary: '∀o ∈ outs. 0 ≤ o.value'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### INV-005
- summary: 'o.value ≤ 21e6·COIN'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### INV-006
- summary: 'foldl (+) 0 values ≤ MAX_MONEY' (checked incrementally ⇒ prefix sums)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### INV-007
- summary: prevout list is duplicate-free
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### INV-008
- summary: '2 ≤ len(cb.scriptSig) ≤ 100'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### INV-009
- summary: '∀i ∈ inputs. i.prevout ≠ null'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### INV-010
- summary: classification rule underlying INV-008/009
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### INV-011
- summary: 'lt=0 → final'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/finality_tests.rs`

### INV-012
- summary: when 'lt < 500M', compare with height
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/finality_tests.rs`

### INV-013
- summary: when 'lt ≥ 500M' ('LOCKTIME_THRESHOLD' script.h:48), compare with (BIP113-corrected) time
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/finality_tests.rs`

### INV-014
- summary: '∀i. i.seq = 0xffffffff → final regardless'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/finality_tests.rs`

### INV-015
- summary: time comparisons use MTP of prev 11 blocks
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/finality_tests.rs`

### INV-016
- summary: block-level quantifier over INV-011..14
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/finality_tests.rs`

### INV-017
- summary: 'fEnforceBIP68 = version≥2 ∧ flag'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/finality_tests.rs`

### INV-018
- summary: mask semantics
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/finality_tests.rs`

### INV-019
- summary: 'MTP(prev of coinHeight) + (seq & 0xffff)·512 − 1'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/finality_tests.rs`

### INV-020
- summary: 'coinHeight + (seq & 0xffff) − 1'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/finality_tests.rs`

### INV-021
- summary: 'minH < blockHeight ∧ minT < MTP(block.prev)'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/finality_tests.rs`

### INV-022
- summary: script-level finality lock (5-byte CScriptNum; both tx-level finality AND lock satisfied)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-023
- summary: script-level relative lock
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-024
- summary: '0 ≤ v ≤ MAX_MONEY'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### INV-025
- summary: per-input
- census: proven consensus
- unmapped: Output values and the output sum are capped at MAX_MONEY (`exceeds_max_money` and `money_range_out_sum` in crates/rbitcoin-consensus/src/block/mod.rs). A transaction input sum and the block fee accumulator are not capped at MAX_MONEY. This row is that missing cap, so it has no production site.

### INV-026
- summary: prefix sums again
- census: proven consensus
- unmapped: Output values and the output sum are capped at MAX_MONEY (`exceeds_max_money` and `money_range_out_sum` in crates/rbitcoin-consensus/src/block/mod.rs). A transaction input sum and the block fee accumulator are not capped at MAX_MONEY. This row is that missing cap, so it has no production site.

### INV-027
- summary: '∀i. prevout ∈ UTXO'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/facts.rs`
- test: `crates/rbitcoin-test/tests/consensus_rules.rs`

### INV-028
- summary: 'h − coin.nHeight ≥ COINBASE_MATURITY'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/facts.rs`
- test: `crates/rbitcoin-test/tests/consensus_rules.rs`

### INV-029
- summary: 'nValueIn ≥ GetValueOut'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### INV-030
- summary: '0 ≤ fee ≤ MAX_MONEY' (provably unreachable given INV-004..006,025–029 — prove it)
- census: proven consensus
- unmapped: Output values and the output sum are capped at MAX_MONEY (`exceeds_max_money` and `money_range_out_sum` in crates/rbitcoin-consensus/src/block/mod.rs). A transaction input sum and the block fee accumulator are not capped at MAX_MONEY. This row is that missing cap, so it has no production site.

### INV-031
- summary: sum of per-tx fees
- census: proven consensus
- unmapped: Output values and the output sum are capped at MAX_MONEY (`exceeds_max_money` and `money_range_out_sum` in crates/rbitcoin-consensus/src/block/mod.rs). A transaction input sum and the block fee accumulator are not capped at MAX_MONEY. This row is that missing cap, so it has no production site.

### INV-032
- summary: 'cb.GetValueOut ≤ GetBlockSubsidy(h) + Σfees'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### INV-033
- summary: halving schedule
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/mod.rs`

### INV-034
- summary: constants
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### INV-035
- summary: sum over scriptSig+scriptPubKey with 'fAccurate=false'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/sigop_cost_tests.rs`

### INV-036
- summary: only when P2SH flag & prevout IsPayToScriptHash
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/sigop_cost_tests.rs`

### INV-037
- summary: v0: 1 for P2WPKH, redeem-based for P2WSH; v1: validation-weight based
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/sigop_cost_tests.rs`

### INV-038
- summary: 'legacy·WITNESS_SCALE_FACTOR'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/sigop_cost_tests.rs`

### INV-039
- summary: running cost check
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/sigop_cost_tests.rs`

### INV-040
- summary: pre-check before context
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/sigop_cost_tests.rs`

### INV-041
- summary: 'len(script) ≤ 10000'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-042
- summary: 'MAX_SCRIPT_ELEMENT_SIZE'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-043
- summary: 'nOpCount'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-044
- summary: 'MAX_STACK_SIZE'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-045
- summary: OP_CAT..OP_NOP10 subset fail
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-046
- summary: BIP62 rule 3/4
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-047
- summary: balanced conditionals
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-048
- summary: success criterion: last stack elem truthy
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-049
- summary: multisig semantics + off-by-one dummy
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-050
- summary: when flag
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-051
- summary: when flag
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### INV-052
- summary: when DERSIG/STRICTENC
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-053
- summary: s ≤ order/2
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### INV-054
- summary: pubkey encoding check (STRICTENC / tapscript x-only)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-055
- summary: hashtype ∈ defined set
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-056
- summary: Legacy BASE checksig FindAndDelete step, conditional only: if the model's 'findAndDelete scriptCode (pushBytesWire sig)' reports a positive match count and 'constScriptCode = false', the ECDSA checker is invoked on '(findAndDelete ...).2'. No claim is made that the matching signature push is deleted; in the core model a whole-tail match is counted but re-appended ('findAndDelete b b = (1, b)'), whereas Core's FindAndDelete returns '(1, empty)'.
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-057
- summary: codeseparator-pos commit
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-058
- summary: numeric encoding rule
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-059
- summary: IF argument minimal
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-060
- summary: c-hanged multisig replacement
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-061
- summary: future-version escapes
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-062
- summary: validation failure
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-063
- summary: see INV-045; cross-ref
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-064
- summary: no concatenation
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-065
- summary: SIGPUSHONLY (policy-flag) scriptSig pushes only
- census: proven consensus
- unmapped: SCRIPT_VERIFY_SIGPUSHONLY is a policy flag and is not a field of ScriptVerifyFlags. Consensus P2SH still requires a push-only scriptSig; that is SCR-006 / INV-066, not this policy flag.

### INV-066
- summary: P2SH: redeemScript = last scriptSig element, evaluated
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-067
- summary: program shapes
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-068
- summary: SCRIPT_ERR_WITNESS_MALLEATED
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-069
- summary: WITNESS_MALLEATED_P2SH
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-070
- summary: dispatch
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-071
- summary: 3-hash preimage
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-072
- summary: key path spend
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-073
- summary: leaf commitment
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-074
- summary: tapscript leaf version 0xc0; unknown versions discouraged
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-075
- summary: annex parsing + hash into sighash
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-076
- summary: CLEANSTACK (policy-flag; implicit in witness)
- census: obsolete consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: CLEANSTACK (policy-flag; implicit in witness).

### INV-077
- summary: WITNESS implies P2SH; unexpected witness data fails
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-078
- summary: interpreter.cpp:SignatureHash(BASE)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### INV-079
- summary: PoW: hash ≤ target from nBits
- census: obsolete consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: PoW: hash ≤ target from nBits.

### INV-080
- summary: compact encoding sanity
- census: obsolete consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: nBits decodes to valid target (non-neg, non-zero, non-overflow, ≤ powLimit).

### INV-081
- summary: merkle root matches tx list
- census: obsolete consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: merkle root matches tx list.

### INV-082
- summary: merkle mutation detection (dup pairs at every level)
- census: obsolete consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: merkle mutation detection (dup pairs at every level).

### INV-083
- summary: size limits
- census: obsolete consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: block has ≥ 1 tx; count·4 ≤ 4M; serialized·4 ≤ 4M.

### INV-084
- summary: first tx is coinbase; no other
- census: obsolete consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: first tx is coinbase; no other.

### INV-085
- summary: CheckTransaction for every tx (incl. INV-007 dup inputs)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### INV-086
- summary: signet solution check (param-gated)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/signet.rs`
- test: `crates/rbitcoin-consensus/src/signet.rs`

### INV-087
- summary: defensive
- census: obsolete consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: merkle mutation ⇒ block rejected not cached-invalid (design rule in ProcessNewBlock).

### INV-088
- summary: 36-byte OP_RETURN, last matching output
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-test/tests/consensus_rules.rs`

### INV-089
- summary: witness reserved value = 32 bytes
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-test/tests/consensus_rules.rs`

### INV-090
- summary: witness merkle root ⊕ reserved value matches commitment
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-test/tests/consensus_rules.rs`

### INV-091
- summary: no witness data in blocks without commitment
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-test/tests/consensus_rules.rs`

### INV-092
- summary: witness merkle: coinbase leaf = 0
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-test/tests/consensus_rules.rs`

### INV-093
- summary: GetBlockWeight consensus/validation.h:143–146
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-test/tests/consensus_rules.rs`

### INV-094
- summary: difficulty continuity
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### INV-095
- summary: Without the testnet min-difficulty rule, retargeting happens only at interval boundaries: a mid-period block copies the previous block's nBits, and a boundary block's nBits is CalculateNextWorkRequired over the interval window. With fPowAllowMinDifficultyBlocks set, a mid-period block more than 2× spacing late instead receives the pow-limit compact (the min-difficulty exception).
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### INV-096
- summary: timespan clamp [T/4, T·4]
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### INV-097
- summary: new target = old·actual/T, capped by powLimit
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### INV-098
- summary: audit check (VerifyDB level)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### INV-099
- summary: time-too-old
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### INV-100
- summary: testnet4/regtest (mainnet off)
- census: proven consensus
- unmapped: BIP94 is not implemented (testnet4 retarget from the first block of the period, and the 600-second timewarp bound). It does not apply to mainnet, testnet3, signet, or default regtest. There is no production check.

### INV-101
- summary: time-too-new; env-clock param
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### INV-102
- summary: version gates
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### INV-103
- summary: BIP34: coinbase scriptSig begins with CScriptNum(height)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/bip34_tests.rs`

### INV-104
- summary: BIP30: no overwriting existing UTXO (conditional)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### INV-105
- summary: historical carve-out
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### INV-106
- summary: optimization with proof caveats in comments
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### INV-107
- summary: genesis block special case (no tx connection)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/facts.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### INV-108
- summary: flag gate
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/finality_tests.rs`

### INV-109
- summary: chain structure
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/facts.rs`
- test: `crates/rbitcoin-test/tests/consensus_rules.rs`

### INV-110
- summary: deletion from UTXO
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/facts.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### INV-111
- summary: insertion
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/facts.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### INV-112
- summary: unspendable (OP_RETURN / >10k script) outputs never added
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/facts.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### INV-113
- summary: coinbase overwrite allowed (pre-BIP30 duplicates)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/facts.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### INV-114
- summary: data model
- census: proven consensus
- unmapped: Core's Coin type (CTxOut plus height plus coinbase, spent when the out is null) is not a type in this tree. Spends are archive rows resolved in confirm. This row is that C++ data model, not a check we run.

### INV-115
- summary: serialization invariant
- census: proven consensus
- unmapped: Core's on-disk Coin word (31-bit height and a coinbase bit in coins.h) is not the rbitcoin archive layout. Height and coinbase-ness are inputs to maturity and BIP30, which are other rows. This row is that Core serialization, which this tree does not use.

### INV-116
- summary: reorg correctness
- census: proven consensus
- production: `crates/rbitcoin-query/src/connect.rs`
- test: `crates/rbitcoin-test/tests/scenarios.rs`

### INV-117
- summary: reverse transition
- census: proven consensus
- production: `crates/rbitcoin-query/src/connect.rs`
- test: `crates/rbitcoin-test/tests/scenarios.rs`

### INV-118
- summary: impl sentinel
- census: proven consensus
- unmapped: Core's MEMPOOL_HEIGHT sentinel (a height that is not a block height, used so mempool coins fail BIP68) is not a value in this tree. Mempool inputs are checked against the confirmed tip, not against a fake coin height.

### INV-119
- summary: snapshot equality predicate
- census: proven consensus
- unmapped: AssumeUTXO (MuHash of the UTXO set, snapshot hash check) is not implemented.

### INV-120
- summary: statefulness
- census: proven consensus
- unmapped: Core's CCoinsView best-block pointer is not a separate object here. The confirmed tip is the store tip. This row is that coins-view pointer.

### INV-121
- summary: DEFINED→STARTED→LOCKED_IN→ACTIVE / FAILED
- census: proven consensus
- unmapped: Shipped activations are buried height compares on ChainParams (crates/rbitcoin-consensus/src/params.rs). The BIP9 state machine (STARTED, LOCKED_IN, ACTIVE, FAILED, timeout, threshold) is not implemented. No shipped network still activates a rule through it.

### INV-122
- summary: signalling condition: top bits = 001 & bit set
- census: proven consensus
- unmapped: Shipped activations are buried height compares on ChainParams (crates/rbitcoin-consensus/src/params.rs). The BIP9 state machine (STARTED, LOCKED_IN, ACTIVE, FAILED, timeout, threshold) is not implemented. No shipped network still activates a rule through it.

### INV-123
- summary: count over 2016
- census: proven consensus
- unmapped: Shipped activations are buried height compares on ChainParams (crates/rbitcoin-consensus/src/params.rs). The BIP9 state machine (STARTED, LOCKED_IN, ACTIVE, FAILED, timeout, threshold) is not implemented. No shipped network still activates a rule through it.

### INV-124
- summary: STARTED→FAILED at timeout (MTP)
- census: proven consensus
- unmapped: Shipped activations are buried height compares on ChainParams (crates/rbitcoin-consensus/src/params.rs). The BIP9 state machine (STARTED, LOCKED_IN, ACTIVE, FAILED, timeout, threshold) is not implemented. No shipped network still activates a rule through it.

### INV-125
- summary: LOCKED_IN→ACTIVE at min_activation_height
- census: proven consensus
- unmapped: Shipped activations are buried height compares on ChainParams (crates/rbitcoin-consensus/src/params.rs). The BIP9 state machine (STARTED, LOCKED_IN, ACTIVE, FAILED, timeout, threshold) is not implemented. No shipped network still activates a rule through it.

### INV-126
- summary: state cache semantics
- census: proven consensus
- unmapped: Shipped activations are buried height compares on ChainParams (crates/rbitcoin-consensus/src/params.rs). The BIP9 state machine (STARTED, LOCKED_IN, ACTIVE, FAILED, timeout, threshold) is not implemented. No shipped network still activates a rule through it.

### INV-127
- summary: BIP34 227931; BIP65 388381; BIP66 363725; CSV 419328; segwit 481824
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/params.rs`
- test: `crates/rbitcoin-consensus/src/params.rs`

### INV-128
- summary: DERSIG flag gate
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### INV-129
- summary: CLTV flag gate
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### INV-130
- summary: CSV flag gate
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### INV-131
- summary: NULLDUMMY flag gate (with segwit)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### INV-132
- summary: post-activation simplification
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### INV-133
- summary: historical carve-outs
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### INV-134
- summary: BIP113 MTP gate on CSV deployment
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### INV-135
- summary: BIP34 gate via DEPLOYMENT_HEIGHTINCB
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### INV-136
- summary: segwit commitment validation gate
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### INV-137
- summary: warnings, NOT enforcement
- census: obsolete consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: MinBIP9WarningHeight / unknown-version warnings.

### INV-138
- summary: API surface
- census: proven consensus
- unmapped: Core's DeploymentEnabled / DeploymentActiveAt helpers are not a type here. Activation is ChainParams height predicates plus ScriptVerifyFlags::consensus_at. This row is that C++ API surface, including BIP9 nStartTime, which we do not model.

### INV-139
- summary: testnet4
- census: proven consensus
- unmapped: BIP94 is not implemented (testnet4 retarget from the first block of the period, and the 600-second timewarp bound). It does not apply to mainnet, testnet3, signet, or default regtest. There is no production check.

### INV-140
- summary: work comparison
- census: proven consensus
- production: `crates/rbitcoin-net/src/chain.rs`
- test: `crates/rbitcoin-net/src/chain.rs`

### INV-141
- summary: work function
- census: proven consensus
- production: `crates/rbitcoin-net/src/most_work.rs`
- test: `crates/rbitcoin-net/src/most_work.rs`

### INV-142
- summary: GetBlockProofEquivalentTime (for assumevalid 2wk check)
- census: proven consensus
- unmapped: Core assumevalid is not implemented. That predicate skips scripts only with chain membership, minimum chainwork, and a two-week proof-equivalent-time burial. rbitcoin's script skip is Milestone (crates/rbitcoin-consensus/src/milestone.rs), which is a different rule, so this row is not mapped onto that file.

### INV-143
- summary: failure propagation
- census: proven consensus
- production: `crates/rbitcoin-net/src/chain.rs`
- test: `crates/rbitcoin-net/src/chain.rs`

### INV-144
- summary: cached-invalid headers rejected
- census: proven consensus
- production: `crates/rbitcoin-net/src/chain.rs`
- test: `crates/rbitcoin-net/src/chain.rs`

### INV-145
- summary: prev-blk-not-found / bad-prevblk
- census: proven consensus
- production: `crates/rbitcoin-net/src/chain.rs`
- test: `crates/rbitcoin-net/src/chain.rs`

### INV-146
- summary: anti-DoS, NOT consensus
- census: obsolete consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: unrequested blocks: more-or-same-work, not-too-far-ahead, ≥ min chain work.

### INV-147
- summary: anti-DoS, NOT consensus
- census: proven consensus
- production: `crates/rbitcoin-net/src/ibd/header_walk.rs`
- test: `crates/rbitcoin-net/src/ibd/header_walk.rs`

### INV-148
- summary: trust option; other checks still enforced
- census: proven not-consensus
- unmapped: Core assumevalid is not implemented. That predicate skips scripts only with chain membership, minimum chainwork, and a two-week proof-equivalent-time burial. rbitcoin's script skip is Milestone (crates/rbitcoin-consensus/src/milestone.rs), which is a different rule, so this row is not mapped onto that file.

### INV-149
- summary: anti-DoS only
- census: proven not-consensus
- production: `crates/rbitcoin-node/src/config.rs`
- test: `crates/rbitcoin-node/src/config.rs`

### INV-150
- summary: bootstrap trust
- census: obsolete not-consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: assumeutxo snapshot: hash_serialized check before VALIDATED.

### INV-151
- summary: historical note
- census: obsolete not-consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: checkpoints: REMOVED (v30.0).

### INV-152
- summary: cache-equivalence lemma
- census: obsolete not-consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: script execution cache must not alter acceptance.

### INV-153
- summary: cache-equivalence lemma
- census: obsolete not-consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: signature cache must not alter acceptance.

### INV-154
- summary: memoization-equivalence lemma
- census: obsolete not-consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: CBlock::fChecked / m_checked_merkle_root memoization.

### INV-155
- summary: impl detail
- census: obsolete not-consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: fCacheResults=false when actually connecting (no stale-cache pollution).

### INV-156
- summary: IsStandard script types (P2PKH, P2SH, P2WPKH, P2WSH, P2TR, multisig, nulldata…)
- census: obsolete not-consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: IsStandard script types (P2PKH, P2SH, P2WPKH, P2WSH, P2TR, multisig, nulldata…).

### INV-157
- summary: IsStandardTx (version ≤ MAX_STANDARD_VERSION, size, bare multisig permit, nulldata ≤ 80 B)
- census: obsolete not-consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: IsStandardTx (version ≤ MAX_STANDARD_VERSION, size, bare multisig permit, nulldata ≤ 80 B).

### INV-158
- summary: IsWitnessStandard (v0 stack ≤100 items ≤80 B; tapscript ≤ 80 B; annex rules; unknown-version discourage)
- census: obsolete not-consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: IsWitnessStandard (v0 stack ≤100 items ≤80 B; tapscript ≤ 80 B; annex rules; unknown-version discourage).

### INV-159
- summary: dust threshold (GetDustThreshold; 3 sat/vB default)
- census: obsolete not-consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: dust threshold (GetDustThreshold; 3 sat/vB default).

### INV-160
- summary: min relay fee / incremental fee / RBF BIP125
- census: obsolete not-consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: min relay fee / incremental fee / RBF BIP125.

### INV-161
- summary: package/cluster limits (25 anc/desc, cluster 64, 101 kvB)
- census: obsolete not-consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: package/cluster limits (25 anc/desc, cluster 64, 101 kvB).

### INV-162
- summary: MAX_STANDARD_TX_WEIGHT 400k; MIN_STANDARD_TX_NONWITNESS_SIZE 65 (CVE-2017-12842)
- census: obsolete not-consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: MAX_STANDARD_TX_WEIGHT 400k; MIN_STANDARD_TX_NONWITNESS_SIZE 65 (CVE-2017-12842).

### INV-163
- summary: TRUC (BIP431) topological restrictions
- census: obsolete not-consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: TRUC (BIP431) topological restrictions.

### INV-164
- summary: P2A ephemeral anchors / ephemeral dust policy
- census: obsolete not-consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: P2A ephemeral anchors / ephemeral dust policy.

### INV-165
- summary: sigop policy denominators (20 B/sigop; 2500 legacy cap)
- census: obsolete not-consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: sigop policy denominators (20 B/sigop; 2500 legacy cap).

### INV-166
- summary: coinbase additional sigops cap for templates (400)
- census: obsolete not-consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: coinbase additional sigops cap for templates (400).

### INV-167
- summary: orphanage limits (CVE-2012-3789)
- census: obsolete not-consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: orphanage limits (CVE-2012-3789).

### INV-168
- summary: header-sync anti-DoS (headerssync.cpp, 2h window consts)
- census: obsolete not-consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: header-sync anti-DoS (headerssync.cpp, 2h window consts).

### INV-169
- summary: CCoinsViewErrorCatcher leveldb-error shutdown
- census: obsolete not-consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: CCoinsViewErrorCatcher leveldb-error shutdown.

### INV-170
- summary: mempool.check() internal consistency assertions
- census: obsolete not-consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: mempool.check() internal consistency assertions.

### INV-171
- summary: mempool expiry (336h default), trimming
- census: obsolete not-consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: mempool expiry (336h default), trimming.

### INV-172
- summary: fee estimation freshness (3h tip age)
- census: obsolete not-consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: fee estimation freshness (3h tip age).

### INV-173
- summary: pruning (MIN_BLOCKS_TO_KEEP 288, prune locks)
- census: obsolete not-consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: pruning (MIN_BLOCKS_TO_KEEP 288, prune locks).

### INV-174
- summary: TestBlockValidity RPC/mining path (skips header index update)
- census: obsolete not-consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: TestBlockValidity RPC/mining path (skips header index update).

### INV-175
- summary: bloom filter CVE-2013-5700 divide-by-zero
- census: obsolete not-consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: bloom filter CVE-2013-5700 divide-by-zero.

### INV-176
- summary: -checkpoints hidden arg no-op
- census: obsolete not-consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: -checkpoints hidden arg no-op.

### SCR-001
- summary: Sequential evaluation, not concatenation
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-002
- summary: After scriptPubKey eval, stack non-empty and top CastToBool-true
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-003
- summary: Empty/short script evaluates to success with unchanged stack
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-004
- summary: with the WITNESS flag off, a scriptSig leaving one truthy element validates against an empty scriptPubKey; with WITNESS on a non-empty witness still fails WITNESS_UNEXPECTED because an empty scriptPubKey is not a witness program (the cited theorem also assumes SIGPUSHONLY and CLEANSTACK off; the one-element CLEANSTACK-on arm is unproved)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-005
- summary: 23-byte HASH160 script: re-eval last scriptSig stack element as subscript
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-006
- summary: scriptSig must be push-only for P2SH evaluation
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-007
- summary: stackCopy (taken after scriptSig) is restored for subscript eval
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-008
- summary: Exactly 1 element after all evaluation (flag; pol.; implicit in witness)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-009
- summary: Flag combination constraint
- census: proven consensus
- unmapped: CLEANSTACK requiring P2SH and WITNESS is a Core assert on flag combinations. ScriptVerifyFlags does not encode that implication. Witness programs enforce a single-element stack on their own path (interpreter and p2wpkh/p2wsh). This row is the flag-combination assert, which is not a check here.

### SCR-010
- summary: scriptSig push-only before eval (pol.)
- census: proven consensus
- unmapped: SCRIPT_VERIFY_SIGPUSHONLY is a policy flag and is not a field of ScriptVerifyFlags. Consensus P2SH still requires a push-only scriptSig; that is SCR-006 / INV-066, not this policy flag.

### SCR-011
- summary: scriptSig == ∅ for bare witness programs
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-012
- summary: scriptSig == single push of redeemScript
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-013
- summary: Non-null witness without witness program fails
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-014
- summary: Flag-combination constraint
- census: proven consensus
- unmapped: Core asserts that the WITNESS flag implies P2SH. ScriptVerifyFlags does not check that combination. Consensus blocks set both through consensus_at except on the BIP16 exception block, which clears both. The assert itself is not a check.

### SCR-015
- summary: P2SH+WITNESS+TAPROOT always; DERSIG/CLTV/CSV/NULLDUMMY by activation
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### SCR-016
- summary: Two named mainnet blocks get reduced flag sets
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### SCR-017
- summary: Any EvalScript/VerifyWitnessProgram failure ⇒ input invalid
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-018
- summary: script.size() > 10000 ⇒ SCRIPT_SIZE (BASE/WITNESS_V0 only, checked once)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-019
- summary: Guard excludes TAPSCRIPT sigversion
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-020
- summary: Push value >520 ⇒ PUSH_SIZE, even unexecuted
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-021
- summary: witness stack items >520 bytes fail with PUSH_SIZE for base/v0 and for tapscript without OP_SUCCESSx, provided the initial stack is ≤1000 items; a tapscript containing OP_SUCCESSx returns success before the item-size loop and is exempt (BIP342)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-022
- summary: under BASE/WITNESS_V0 an opcode with wire byte > OP_16 (0x60) is charged against the 201-op budget; once the budget is exhausted stepBody returns OP_COUNT before disabled-table and dispatch checks (OP_NOP is the concrete charging witness)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-023
- summary: Count is flow-insensitive
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-024
- summary: condition 'opcode > OP_16' (0x00-0x60 exempt)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-025
- summary: op-count check precedes disabled check
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-026
- summary: CMS key count ∈ [0,20] ⇒ PUBKEY_COUNT
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-027
- summary: nSigsCount ∈ [0, nKeysCount] ⇒ SIG_COUNT
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-028
- summary: nOpCount += nKeysCount; ≤201
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-029
- summary: stack+altstack > 1000 ⇒ STACK_SIZE, checked per opcode
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-030
- summary: combined check
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-031
- summary: Checked before EvalScript (altstack empty)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-032
- summary: CHECKSIG=1; CHECKMULTISIG=20 (inaccurate)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-033
- summary: last opcode OP_1..OP_16 ⇒ DecodeOP_N
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-034
- summary: for a push-only scriptSig (every opcode ≤ OP_16), the last push is the redeem script and its accurate sigops are counted; a non-push scriptSig needs Core's all-ops '> OP_16 ⇒ 0' guard, which the current 'p2shSigOpCount' model does not implement (explicit residual)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-035
- summary: v0-20=1; v0-32=accurate(witness script); v1+=0
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-036
- summary: push-only scriptSig; last element as program
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-037
- summary: cost in weight units
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-038
- summary: cumulative block sigop cost limit
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-039
- summary: mempool policy per-tx limit
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-040
- summary: budget = serialized witness size + 50 WU
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-041
- summary: decrement on non-empty sig regardless of validity; <0 ⇒ TAPSCRIPT_VALIDATION_WEIGHT
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-042
- summary: non-empty sig + upgradable key still decrements
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-043
- summary: tapscript multisig-mimic bound (miniscript-side resource limit)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-044
- summary: lower bound for serialized tx weight
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-045
- summary: pushdata1/2/4 length parsing; truncation ⇒ false (⇒ BAD_OPCODE in EvalScript)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-046
- summary: OP_0→∅; OP_1..16→single byte 0x01..0x10
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-047
- summary: OP_1NEGATE (0x4f) pushes the single byte 0x81: encodeScriptNum (-1) false = serU8 0x81, and an executed OP_1NEGATE pushes it with no MINIMALDATA gate
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-048
- summary: canonical push opcode for each data length (under MINIMALDATA, pol.)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-049
- summary: fExec-gated (unexecuted non-minimal pushes OK)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-050
- summary: top byte &0x7f==0 requires size 1 or next-byte high bit (under fRequireMinimal)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-051
- summary: numeric operands always minimal
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-052
- summary: opcode ≤ OP_16 (incl. OP_RESERVED!) counts as push
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-053
- summary: 4..42 bytes; OP_0/OP_N + exact-length push; program 2..40
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-054
- summary: 23-byte HASH160 pattern
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-055
- summary: OP_1 0x4e73 (4-byte script / 2-byte v1 program)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-056
- summary: CastToBool of top (NOTIF inverts); pop
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-057
- summary: any nonzero byte = true; lone 0x80 = false; ∅ = false
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-058
- summary: IF/NOTIF arg ∈ {∅, 01} else TAPSCRIPT_MINIMALIF
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-059
- summary: same check under flag
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-060
- summary: no stack pop when skipped
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-061
- summary: toggle_top; empty cond-stack ⇒ UNBALANCED_CONDITIONAL
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-062
- summary: empty cond-stack ⇒ UNBALANCED_CONDITIONAL
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-063
- summary: non-empty cond-stack after loop ⇒ fail
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-064
- summary: in [OP_IF..OP_ENDIF] gate
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-065
- summary: BAD_OPCODE even in unexecuted branches
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-066
- summary: BAD_OPCODE when executed; unexecuted OP_VER passes
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-067
- summary: each IF counts 1 opcode
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-068
- summary: pop iff true else VERIFY
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-069
- summary: OP_RETURN error
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-070
- summary: first-byte OP_RETURN or >10k script ⇒ never spendable
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-071
- summary: no-op; DISCOURAGE_UPGRADABLE_NOPS = pol. fails them
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-072
- summary: without flags they are plain NOPs
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-073
- summary: executed TOALTSTACK/FROMALTSTACK with one opcode of budget left move the top item; FROMALTSTACK on an empty altstack ⇒ INVALID_ALTSTACK_OPERATION and TOALTSTACK on an empty main stack ⇒ INVALID_STACK_OPERATION; unexecuted branches skip and budget exhaustion yields OP_COUNT first
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-074
- summary: each op checks depth ⇒ INVALID_STACK_OPERATION
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-075
- summary: ScriptNum(top); after the index is popped, n∈[0, remaining-stack size) (= tail.length) else INVALID_STACK_OPERATION; ROLL erases; the same transition runs executed in BASE/WITNESS_V0/TAPSCRIPT (BASE/WITNESS_V0 also charge one opcode-budget unit, tapscript does not), and the unexecuted case is a skip
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-076
- summary: size as minimal CScriptNum encoding
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-077
- summary: dup iff CastToBool
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-078
- summary: popstack throws ⇒ UNKNOWN_ERROR; all ops pre-check
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-079
- summary: >limit ⇒ scriptnum_error ⇒ SCRIPT_ERR_SCRIPTNUM
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-080
- summary: all numeric opcodes
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-081
- summary: nMaxNumSize=5 special case
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-082
- summary: 0x80 sign bit on top byte; negative zero exists
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-083
- summary: 0x80 → 0; CastToBool(0x80)=false
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-084
- summary: CScriptNum minimal-encoding sign-extension exception: +255 and -255 encode to 0xff00 and 0xff80 (little-endian on the wire) and BOTH are minimal; the exception is required because the extra byte carries the sign (script.h:262-268).
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-085
- summary: only re-interpretation enforces limit
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-086
- summary: clamp to int32 for depth/count uses
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-087
- summary: 1ADD/1SUB/NEGATE/ABS/NOT/0NOTEQUAL
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-088
- summary: ADD/SUB/BOOLAND/BOOLOR/NUMEQUAL/NUMNOTEQUAL/LE/GE/LT/GT/MIN/MAX (wire 0x93, 0x94, 0x9a-0x9c, 0x9e-0xa4) push the binary result; OP_NUMEQUALVERIFY (0x9d) is a separate non-pushing op — false => NUMEQUALVERIFY error (SCR-090, PDW79.lean:94-105), and its success arm (both operands dropped) is not yet proved
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-089
- summary: min ≤ x < max
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-090
- summary: false ⇒ NUMEQUALVERIFY error
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-091
- summary: raw bytes, not numeric
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-092
- summary: 0x8e is OP_2DIV (no OP_NOTEQUAL opcode exists); legacy and witness-v0 execution reject it as a disabled opcode, and tapscript treats it as OP_SUCCESS142
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-093
- summary: 20/20/32
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-094
- summary: RIPEMD160∘SHA256; SHA256d
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-095
- summary: works (∅ hashes fine)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-096
- summary: pbegincodehash .. end
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-097
- summary: only executed CODESEPARATOR updates
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-098
- summary: byte-level removal of serialized sig push from scriptCode
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-099
- summary: found>0 ⇒ SIG_FINDANDDELETE (pol.)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-100
- summary: BASE, even unexecuted (pol.)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-101
- summary: 1-byte opcodes removed from serialization
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2pkh.rs`
- test: `crates/rbitcoin-consensus/src/script/core_sighash.rs`

### SCR-102
- summary: 9..73 bytes; strict DER structure; consensus since BIP66
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-103
- summary: applied from BIP66 height 363725
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-104
- summary: S ≤ order/2 under LOW_S (pol.)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-105
- summary: last byte & ~0x80 ∈ {1,2,3} (pol.)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-106
- summary: 33B 02/03 or 65B 04 (pol.)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-107
- summary: v0 witness pubkeys compressed (pol.)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-108
- summary: ∅ sig bypasses DER checks
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-109
- summary: OP_CHECKSIG pushes FALSE on ∅ sig
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-110
- summary: failed check w/ non-empty sig ⇒ SIG_NULLFAIL (pol.)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-111
- summary: ANYONECANPAY bit passed through
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-112
- summary: invalid CPubKey => the script aborts with the pubkey-encoding error (a script failure, not a false check verdict); the proved theorem covers sv ≠ BASE, so the BASE arm (with its FindAndDelete/CONST_SCRIPTCODE side condition) remains uncovered
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-113
- summary: each executed op counts 1 toward 201
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-114
- summary: nKeys∈[0,20], nSigs∈[0,nKeys], depth checks
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-115
- summary: all nSigsCount sigs FindAndDelete'd before any check (BASE)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-116
- summary: sigs to keys in order; early fail when sigs>keys remain
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-117
- summary: CheckSignatureEncoding+CheckPubKeyEncoding inside loop
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-118
- summary: extra stack item consumed unchecked
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-119
- summary: dummy must be ∅ (cons. post-SegWit)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-120
- summary: false ⇒ CHECKMULTISIGVERIFY error
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-121
- summary: TAPSCRIPT_CHECKMULTISIG
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-122
- summary: negative ⇒ NEGATIVE_LOCKTIME
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-123
- summary: both sides < or ≥ 500000000 (script.h:48)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-124
- summary: operand ≤ tx nLockTime
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-125
- summary: vin[nIn].nSequence ≠ 0xffffffff
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-126
- summary: negative ⇒ NEGATIVE_LOCKTIME
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-127
- summary: operand bit 31 ⇒ NOP
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-128
- summary: else fail
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-129
- summary: else fail
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-130
- summary: mask 0x0040ffff
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-131
- summary: CSV script-level compares raw masked values; granularity in SequenceLocks
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-132
- summary: CheckLockTime/Sequence see only input nIn
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-133
- summary: SHA256(witnessScript) == program
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-134
- summary: empty ⇒ WITNESS_PROGRAM_WITNESS_EMPTY
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-135
- summary: else WITNESS_PROGRAM_MISMATCH
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-136
- summary: DUP HASH160 <20> EQUALVERIFY CHECKSIG
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-137
- summary: else WRONG_LENGTH
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-138
- summary: exactly 1 true element (cons.)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-139
- summary: sigversion≠BASE skips deletion
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-140
- summary: field order per BIP143; amount included
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2pkh.rs`
- test: `crates/rbitcoin-consensus/src/script/core_sighash.rs`

### SCR-141
- summary: hashOutputs = 0 (not uint256::ONE)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-142
- summary: without flag, v1-32 non-P2SH spends succeed silently (activation)
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### SCR-143
- summary: ≥2 items, last non-empty, first byte 0x50
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### SCR-144
- summary: CheckSchnorrSignature(TAPROOT)
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### SCR-145
- summary: 33+32k bytes, k≤128 ⇒ TAPROOT_WRONG_CONTROL_SIZE
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### SCR-146
- summary: TaggedHash(\"TapLeaf\", leaf_ver, CompactSize, script); leaf_ver = control[0]&0xfe
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### SCR-147
- summary: q == tweak(p, merkle_root, parity=control[0]&1)
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### SCR-148
- summary: control[0]&0xfe == 0xc0 ⇒ TAPSCRIPT eval
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### SCR-149
- summary: upgradable; DISCOURAGE flag = pol.
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### SCR-150
- summary: v1 2-byte 4e73 program ⇒ success, no execution
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### SCR-151
- summary: v2-v16 / wrong-size v1 / P2SH-wrapped v1 ⇒ success (upgradable)
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### SCR-152
- summary: {80,98,126-129,131-134,137-138,141-142,149-153,187-254}
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-153
- summary: overrides all later checks (incl. size limits)
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-154
- summary: pol. flag fails them
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-155
- summary: the MAX_OPS budget is not consulted on the tapscript path: an already-over-budget tapscript OP_NOP still succeeds and leaves opCount untouched; on BASE/WITNESS_V0 an over-budget counting opcode instead returns OP_COUNT
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-156
- summary: (sig,num,pk)→num+[sigok]; BAD_OPCODE outside tapscript
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### SCR-157
- summary: CScriptNum(fRequireMinimal)
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### SCR-158
- summary: 0 ⇒ fail; 32 ⇒ schnorr; else upgradable
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### SCR-159
- summary: TAPSCRIPT_EMPTY_PUBKEY (before sig check? after weight?) — after weight decrement
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### SCR-160
- summary: success=false; no schnorr verification; no weight consumed
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### SCR-161
- summary: else SCHNORR_SIG_SIZE
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### SCR-162
- summary: SIGHASH_DEFAULT invalid as explicit byte
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### SCR-163
- summary: ≤0x03 or 0x81..0x83 else fail
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### SCR-164
- summary: tapleaf hash, key_version=0, codeseparator_pos
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/p2pkh.rs`
- test: `crates/rbitcoin-consensus/src/script/core_sighash.rs`

### SCR-165
- summary: only for 64-byte key-path sigs
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/p2pkh.rs`
- test: `crates/rbitcoin-consensus/src/script/core_sighash.rs`

### SCR-166
- summary: in_pos ≥ vout.size ⇒ construction fails ⇒ SCHNORR_SIG_HASHTYPE
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### SCR-167
- summary: Executed OP_CODESEPARATOR records the uint32 instruction index (opcode_pos, 0-based) in m_codeseparator_pos; the field is initialized to 0xFFFFFFFF (interpreter.cpp:444-445,1065). The Lean interpreter's State.codeSep instead records the byte offset op.next and initializes to 0; codesepPosFold is the faithful UInt32 index model.
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### SCR-168
- summary: budget may reach exactly 0
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### SCR-169
- summary: epoch 0; spend_type; annex hash
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/p2pkh.rs`
- test: `crates/rbitcoin-consensus/src/script/core_sighash.rs`

### SCR-170
- summary: CAT,SUBSTR,LEFT,RIGHT,INVERT,AND,OR,XOR,2MUL,2DIV,MUL,DIV,MOD,LSHIFT,RSHIFT ⇒ DISABLED_OPCODE in base/witness-v0, even unexecuted, unless the 201-op budget is already exhausted (OP_COUNT then preempts). Tapscript treats these bytes as OP_SUCCESSx, so they never reach stepBody there.
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-171
- summary: after op-count, before fExec
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-172
- summary: same bytes become success in tapscript
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-173
- summary: name table; unknown ⇒ \"OP_UNKNOWN\
- census: proven not-consensus
- unmapped: GetOpName is Core's opcode debug string. The interpreter matches opcode bytes and does not publish a name table. The row is not a consensus check.

### SCR-174
- summary: default case BAD_OPCODE
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### TXV-001
- summary: 'vin ≠ ∅' else 'bad-txns-vin-empty'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### TXV-002
- summary: 'vout ≠ ∅' else 'bad-txns-vout-empty'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### TXV-003
- summary: 'weight(stripped_serialization) ≤ 4_000_000'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### TXV-004
- summary: '∀ o ∈ vout: o.value ≥ 0'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### TXV-005
- summary: '∀ o ∈ vout: o.value ≤ 21e6·COIN'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### TXV-006
- summary: running sum stays in '[0, MAX_MONEY]'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### TXV-007
- summary: prevouts pairwise distinct
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### TXV-008
- summary: coinbase ⇒ '2 ≤
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### TXV-009
- summary: non-coinbase ⇒ no prevout is '(0,0xffffffff)'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### TXV-010
- summary: coinbase ⇔ '
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### TXV-011
- summary: version·vin·vout·nLockTime layout, LE, CompactSize-counted
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### TXV-012
- summary: minimal-length encoding only
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### TXV-013
- summary: empty-vin ⇒ marker byte + flag byte, then re-read vin/vout
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### TXV-014
- summary: flag&1 ⇒ one stack per input after vout, before nLockTime
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### TXV-015
- summary: flag set but all stacks empty ⇒ decode failure
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### TXV-016
- summary: nonzero flag other than 1 (flag != 0 and flag != 1) => decode failure; on a decode-valid tail, an odd flag whose witness stacks are all empty fails with superfluousWitness (Core checks this before the unknown-flag test), otherwise the failure is unknownFlag; flag = 0 is TXV-013's valid short empty-vin record
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### TXV-017
- summary: 'txid = dsha256(encodeNoWitness(tx))'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/tx_precompute.rs`
- test: `crates/rbitcoin-consensus/src/tx_precompute.rs`

### TXV-018
- summary: 'wtxid = dsha256(encode(tx))'; = txid if no witness
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/tx_precompute.rs`
- test: `crates/rbitcoin-consensus/src/tx_precompute.rs`

### TXV-019
- summary: 'weight = stripped·3 + total'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/tx_precompute.rs`
- test: `crates/rbitcoin-consensus/src/tx_precompute.rs`

### TXV-020
- summary: every prevout resolves to unspent coin
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/facts.rs`
- test: `crates/rbitcoin-test/tests/consensus_rules.rs`

### TXV-021
- summary: 'spendH − coinH ≥ 100' for coinbase coins
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/facts.rs`
- test: `crates/rbitcoin-test/tests/consensus_rules.rs`

### TXV-022
- summary: 'Coin = {out, fCoinBase:1, nHeight:31}'; disk code '(h << 1) | cb' (decoded as 'h = code >> 1', 'cb = code & 1'). The local 'Coin.height' is an unrestricted 'UInt32', so the 31-bit domain is an explicit hypothesis: only the height-half roundtrip under 'h < 2^31' and the (untagged) coinbase-bit roundtrip are local theorems.
- census: proven consensus
- unmapped: Core's on-disk Coin word (31-bit height and a coinbase bit in coins.h) is not the rbitcoin archive layout. Height and coinbase-ness are inputs to maturity and BIP30, which are other rows. This row is that Core serialization, which this tree does not use.

### TXV-023
- summary: per-coin and running input sum in '[0,MAX_MONEY]'
- census: proven consensus
- unmapped: Output values and the output sum are capped at MAX_MONEY (`exceeds_max_money` and `money_range_out_sum` in crates/rbitcoin-consensus/src/block/mod.rs). A transaction input sum and the block fee accumulator are not capped at MAX_MONEY. This row is that missing cap, so it has no production site.

### TXV-024
- summary: 'Σin ≥ Σout' else 'bad-txns-in-belowout'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/facts.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### TXV-025
- summary: 'fee = Σin − Σout ∈ [0, MAX_MONEY]' (implied)
- census: proven consensus
- unmapped: Output values and the output sum are capped at MAX_MONEY (`exceeds_max_money` and `money_range_out_sum` in crates/rbitcoin-consensus/src/block/mod.rs). A transaction input sum and the block fee accumulator are not capped at MAX_MONEY. This row is that missing cap, so it has no production site.

### TXV-026
- summary: block Σfees ∈ MoneyRange
- census: proven consensus
- unmapped: Output values and the output sum are capped at MAX_MONEY (`exceeds_max_money` and `money_range_out_sum` in crates/rbitcoin-consensus/src/block/mod.rs). A transaction input sum and the block fee accumulator are not capped at MAX_MONEY. This row is that missing cap, so it has no production site.

### TXV-027
- summary: 'nLockTime = 0' ⇒ final
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/finality_tests.rs`

### TXV-028
- summary: '< 500_000_000' compares to height, else to time
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/finality_tests.rs`

### TXV-029
- summary: final iff 'nLockTime < cutoff_height_or_time'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/finality_tests.rs`

### TXV-030
- summary: all inputs 0xffffffff ⇒ final regardless
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/finality_tests.rs`

### TXV-031
- summary: time cutoff = MTP(prev block) post-CSV, else block time
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/finality_tests.rs`

### TXV-032
- summary: every tx in block must be final else 'bad-txns-nonfinal'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/finality_tests.rs`

### TXV-033
- summary: evaluate at 'tipH+1', 'MTP(tip)'
- census: proven consensus
- unmapped: Mempool finality uses the next-block height and the tip's median time, which is node policy context around is_final_tx. The consensus predicate is is_final_tx in crates/rbitcoin-consensus/src/block/mod.rs (INV-011). This row is the mempool calling convention, not a second consensus rule.

### TXV-034
- summary: enforce iff 'version ≥ 2 ∧ LOCKTIME_VERIFY_SEQUENCE'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/finality_tests.rs`

### TXV-035
- summary: sequence bit31 set ⇒ no relative lock
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/finality_tests.rs`

### TXV-036
- summary: bit22 set ⇒ 'MTP(coin's parent) + (seq&0xffff)·512 − 1'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/finality_tests.rs`

### TXV-037
- summary: else 'coinHeight + (seq&0xffff) − 1'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/finality_tests.rs`

### TXV-038
- summary: lockpoints express *last invalid* height/time
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/finality_tests.rs`

### TXV-039
- summary: fail iff 'minH ≥ blkH ∨ minT ≥ MTP(blk.prev)'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/finality_tests.rs`

### TXV-040
- summary: BIP68 flag set iff CSV deployment active at block
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/finality_tests.rs`

### TXV-041
- summary: evaluate at 'tipH+1'; mempool-input heights = tip+1
- census: proven consensus
- unmapped: Mempool sequence locks are evaluated against tip+1. The consensus predicate is sequence_locks_satisfied in crates/rbitcoin-consensus/src/block/mod.rs. This row is the mempool height convention.

### TXV-042
- summary: unconfirmed coins carry height 0x7FFFFFFF
- census: proven consensus
- unmapped: Core's MEMPOOL_HEIGHT sentinel (a height that is not a block height, used so mempool coins fail BIP68) is not a value in this tree. Mempool inputs are checked against the confirmed tip, not against a fake coin height.

### TXV-043
- summary: operand and tx.nLockTime same kind (< / ≥ 500M)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### TXV-044
- summary: within one domain ('lockTimeSameDomain') and with this input's sequence not '0xFFFFFFFF', fail iff 'operand > tx.nLockTime'; cross-domain operands and final-sequence inputs also fail
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### TXV-045
- summary: with the same-domain gate passed and 'operand ≤ tx.nLockTime', fail iff this input's sequence is '0xFFFFFFFF'; an above-locktime operand also fails while the sequence is non-final
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### TXV-046
- summary: fail iff 'tx.version < 2'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### TXV-047
- summary: fail iff tx sequence bit31 set
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### TXV-048
- summary: mask both sides (bit22\\
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### TXV-049
- summary: for scripts whose 'parseWire' succeeds, the model's 'txLegacySigOps' is exactly the Σ of the model's non-accurate per-script counts over scriptSigs+scriptPubKeys; malformed scripts contribute 0 here (Core returns the already-counted prefix) and a non-accurate CHECKMULTISIG costs 1 here (Core costs 20) — explicit residual
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/sigop_cost_tests.rs`

### TXV-050
- summary: for P2SH prevouts, count with scriptSig as subscript; coinbase = 0
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/sigop_cost_tests.rs`

### TXV-051
- summary: per-input witness program sigop cost
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/sigop_cost_tests.rs`

### TXV-052
- summary: legacy+P2SH counts ×WITNESS_SCALE_FACTOR
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/sigop_cost_tests.rs`

### TXV-053
- summary: block ΣsigopCost ≤ 80_000 else 'bad-blk-sigops'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/sigop_cost_tests.rs`

### TXV-054
- summary: in ACCURATE mode every per-op CHECKMULTISIG contribution is ≤ 20, with fallback exactly 20 when no OP_1..OP_16 precedes; the non-accurate 20-per-CHECKMULTISIG charge is not modelled (explicit residual)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/sigop_cost_tests.rs`

### TXV-055
- summary: 'flags(blk) = base ∪ active-deployment flags'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### TXV-056
- summary: 2 historical blocks use reduced flag sets (P2SH/Taproot)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### TXV-057
- summary: BIP147 enforced from SegWit activation
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### TXV-058
- summary: each input: VerifyScript(sig, prevout.spk, witness, flags)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/block/block_866342.rs`

### TXV-059
- summary: coinbase inputs never script-verified
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/block_866342.rs`

### TXV-060
- summary: consensus flags ⇒ TX_CONSENSUS; extra ⇒ TX_NOT_STANDARD
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/block/block_866342.rs`

### TXV-061
- summary: mempool re-runs scripts under current-block (consensus) flags
- census: proven consensus
- unmapped: Re-checking mempool transactions when consensus flags change at a buried deployment is a mempool maintenance step. Admission uses the same script verifier. There is no separate flag-epoch rescan to cite.

### TXV-062
- summary: cache keyed by witness hash + flags (optimization, provably transparent)
- census: proven consensus
- unmapped: Core's script execution cache keyed by (wtxid, flags) is an equivalence lemma about a cache. rbitcoin does not keep that cache. Script results are not reused across flag sets by a wtxid map.

### TXV-063
- summary: pre-BIP34(-limit) blocks: no txid may already exist in UTXO set
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/facts.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### TXV-064
- summary: sequential UpdateCoins; first spender in block order wins
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/facts.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### TXV-065
- summary: tx i+1 may spend tx i's outputs (cache-first semantics)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/facts.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### TXV-066
- summary: any spk allowed; value rules only
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/policy.rs`
- test: `crates/rbitcoin-consensus/src/policy.rs`

### TXV-067
- summary: 'value = 0' outputs create spendable-looking UTXOs
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/policy.rs`
- test: `crates/rbitcoin-consensus/src/policy.rs`

### TXV-068
- summary: 'MAX_MONEY = 2.1e15' sat, inclusive
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### TXV-069
- summary: all value arithmetic in i64, checked incrementally
- census: proven consensus
- unmapped: Value arithmetic uses i64 and rejects overflow in fee_from_values / add_value_in (crates/rbitcoin-consensus/src/block/facts.rs), but a summed input value is not rejected for exceeding MAX_MONEY before that overflow check. This row is Core's incremental MoneyRange on every amount fold, which is only partly present, so the row is left unmapped rather than cited as done.

### TXV-070
- summary: coinbase of N passes the maturity gate at caller-supplied spend height S iff N + 100 ≤ S; the block caller uses the block height, and Core's mempool caller passes tip + 1, but the local theorems do not model or prove the mempool caller convention
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/facts.rs`
- test: `crates/rbitcoin-test/tests/consensus_rules.rs`

### TXV-071
- summary: any 32-bit version valid; BIP68 gate at ≥2
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### TXV-072
- summary: coinbase txs never input-checked
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/facts.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### TXV-073
- summary: mempool dedup/conflict by wtxid; txid-equivalence handled separately
- census: proven consensus
- unmapped: Using wtxid as the mempool identity (BIP339) is relay, not consensus validity. The wtxid hash itself is tx_precompute.rs (TXV-018).

### TXV-074
- summary: 'checkTransaction = ok ⇒ GetValueOut' terminates in range (lemma)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### TXV-075
- summary: spent ⇔ absent-from-cache-or-spent-bit; Undo restores on disconnect
- census: proven consensus
- unmapped: Core's Coin type (CTxOut plus height plus coinbase, spent when the out is null) is not a type in this tree. Spends are archive rows resolved in confirm. This row is that C++ data model, not a check we run.

### BLK-001
- summary: A block header is exactly (nVersion, prevHash, merkleRoot, nTime, nBits, nNonce), serialized in that order, 80 bytes
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### BLK-002
- summary: A block is header ‖ varint(txcount) ‖ txs
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### BLK-003
- summary: block hash = SHA256d(80-byte header), interpreted little-endian
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/tx_precompute.rs`
- test: `crates/rbitcoin-consensus/src/tx_precompute.rs`

### BLK-004
- summary: nBits == 0 defines a null header
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### BLK-005
- summary: txid = SHA256d(tx serialized without witness)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/tx_precompute.rs`
- test: `crates/rbitcoin-consensus/src/tx_precompute.rs`

### BLK-006
- summary: wtxid = SHA256d(tx with witness) = txid when no witness
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/tx_precompute.rs`
- test: `crates/rbitcoin-consensus/src/tx_precompute.rs`

### BLK-007
- summary: leaves = txid_i of all vtx in order
- census: proven consensus
- production: `crates/rbitcoin-primitives/src/merkle.rs`
- test: `crates/rbitcoin-primitives/src/merkle.rs`

### BLK-008
- summary: odd-sized level duplicates its last element before pairing
- census: proven consensus
- production: `crates/rbitcoin-primitives/src/merkle.rs`
- test: `crates/rbitcoin-primitives/src/merkle.rs`

### BLK-009
- summary: pair hash = SHA256d(concat of two 32-byte values, in order)
- census: proven consensus
- production: `crates/rbitcoin-primitives/src/merkle.rs`
- test: `crates/rbitcoin-primitives/src/merkle.rs`

### BLK-010
- summary: header.hashMerkleRoot must equal computed root ('bad-txnmrklroot')
- census: proven consensus
- production: `crates/rbitcoin-primitives/src/merkle.rs`
- test: `crates/rbitcoin-primitives/src/merkle.rs`

### BLK-011
- summary: reject if any level contains two adjacent equal hashes ('bad-txns-duplicate')
- census: proven consensus
- production: `crates/rbitcoin-primitives/src/merkle.rs`
- test: `crates/rbitcoin-primitives/src/merkle.rs`

### BLK-012
- summary: 'CheckMerkleRoot' failures — both a root mismatch ('bad-txnmrklroot') and a mutated tree ('bad-txns-duplicate') — are 'BLOCK_MUTATED', and 'InvalidBlockFound' must not mark such a block permanently invalid
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-rpc/src/regtest_chain_ops_journey.rs`

### BLK-013
- summary: empty leaf list ⇒ null root (unreachable for valid blocks due BLK-022)
- census: proven consensus
- production: `crates/rbitcoin-primitives/src/merkle.rs`
- test: `crates/rbitcoin-primitives/src/merkle.rs`

### BLK-014
- summary: leaf for coinbase is 0x00…00; leaves are wtxid_1..n−1
- census: proven consensus
- production: `crates/rbitcoin-primitives/src/merkle.rs`
- test: `crates/rbitcoin-primitives/src/merkle.rs`

### BLK-015
- summary: a commitment output is any coinbase output ≥38 bytes with prefix 6a24aa21a9ed; last one counts
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-test/tests/consensus_rules.rs`

### BLK-016
- summary: GenerateCoinbaseCommitment creates the commitment output with nValue = 0 (generation only); block validation does not require nValue = 0 on an existing commitment output, and the model's 'commitmentValueZero' checks every 38-byte '6a24aa21a9ed'-matching output, not only the last selected one
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-test/tests/consensus_rules.rs`

### BLK-017
- summary: SHA256d(witness_root ‖ reserved_value) == scriptPubKey[6..38]
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-test/tests/consensus_rules.rs`

### BLK-018
- summary: coinbase scriptWitness must be exactly one 32-byte element ('bad-witness-nonce-size'), content unconstrained
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-test/tests/consensus_rules.rs`

### BLK-019
- summary: if no commitment output exists, no tx in the block may have witness data ('unexpected-witness')
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-test/tests/consensus_rules.rs`

### BLK-020
- summary: any tx with witness ⇒ a valid commitment must exist (when segwit active)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-test/tests/consensus_rules.rs`

### BLK-021
- summary: weight check must occur after commitment check (hash-invariance argument)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-test/tests/consensus_rules.rs`

### BLK-022
- summary: vtx non-empty ('bad-cb-missing' via empty check)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### BLK-023
- summary: vtx.size() × 4 ≤ 4,000_000
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### BLK-024
- summary: stripped block size × 4 ≤ 4,000,000 ('bad-blk-length')
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### BLK-025
- summary: weight = 3×stripped_size + total_size
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### BLK-026
- summary: weight ≤ 4,000,000 ('bad-blk-weight'), checked after commitment validation
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### BLK-027
- summary: vtx[0].IsCoinBase() ('bad-cb-missing'); IsCoinBase = single input, null prevout
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### BLK-028
- summary: ∀i≥1: ¬vtx[i].IsCoinBase() ('bad-cb-multiple')
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### BLK-029
- summary: coinbase vin scriptSig size ∈ [2,100] bytes ('bad-cb-length')
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### BLK-030
- summary: every tx passes CheckTransaction (vin/vout nonempty, size, money range, dup inputs, prevout null)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### BLK-031
- summary: no two inputs of one tx share an outpoint ('bad-txns-inputs-duplicate')
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### BLK-032
- summary: stripped tx size × 4 ≤ 4,000,000 ('bad-txns-oversize')
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### BLK-033
- summary: count CHECKSIG(VERIFY)+CHECKMULTISIG(VERIFY) in scriptSig&scriptPubKey, multisig = 20
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/sigop_cost_tests.rs`

### BLK-034
- summary: Σ legacy sigops ×4 ≤ 80,000 ('bad-blk-sigops'), under-counting by design
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/sigop_cost_tests.rs`

### BLK-035
- summary: cost = 4×(legacy + P2SH) + witness sigops; P2SH/witness exclude coinbase
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/sigop_cost_tests.rs`

### BLK-036
- summary: cumulative sigop cost ≤ 80,000 during ConnectBlock ('bad-blk-sigops')
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/sigop_cost_tests.rs`

### BLK-037
- summary: nBits → (mantissa&0x007fffff) << (8×(exp−3)) for exp>3, >>8×(3−exp) else; sign bit = 0x00800000 valid only with nonzero mantissa; overflow iff mantissa≠0 ∧ (exp>34 ∨ (m>0xff ∧ exp>33) ∨ (m>0xffff ∧ exp>32))
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### BLK-038
- summary: decode must be non-negative, non-overflow, nonzero, ≤ powLimit
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### BLK-039
- summary: SHA256d(header) interpreted as LE integer ≤ target ('high-hash')
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### BLK-040
- summary: 0x00000000ffffffff…ff
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### BLK-041
- summary: signet 0x00000377ae00…; regtest 0x7fffffff…
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### BLK-042
- summary: header nBits must equal GetNextWorkRequired(prev, header) ('bad-diffbits')
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### BLK-043
- summary: At a retarget boundary ((height+1) mod interval == 0), nextWorkRequired is CalculateNextWorkRequired over the interval window. Off-boundary, with fPowAllowMinDifficultyBlocks = false, it copies the previous block's nBits. On a min-difficulty chain the pow.cpp:28-49 branch runs first: a block more than 2× spacing late gets the compact powLimit, otherwise the min-difficulty walk-back bits are used; the immediate parent's nBits is not copied.
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### BLK-044
- summary: timespan = time(block at height H) − time(block at H−2015)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### BLK-045
- summary: actualTimespan clamped to [T/4, 4T], T = 1209600 s
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### BLK-046
- summary: new = base_nBits_decoded × clampedTimespan / T, truncated division, then cap at powLimit, re-encode
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### BLK-047
- summary: base = first block of period's nBits iff enforce_BIP94 else last block's
- census: proven consensus
- unmapped: BIP94 is not implemented (testnet4 retarget from the first block of the period, and the 600-second timewarp bound). It does not apply to mainnet, testnet3, signet, or default regtest. There is no production check.

### BLK-048
- summary: the pre-encoding new target is min(newT, powLimit) and therefore capped at powLimit
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### BLK-049
- summary: At a period boundary, fPowNoRetargeting copies the last nBits (through calculateNextWorkRequired). On regtest, which also sets fPowAllowMinDifficultyBlocks, non-boundary heights take the min-difficulty branch instead: powLimit when newTime > last.time + 2×spacing, otherwise the BLK-054 min-difficulty walk-back bits. No-retargeting alone therefore does not imply nBits copied.
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### BLK-050
- summary: with fPowAllowMinDifficultyBlocks = false and the new height on a period boundary ('height % difficultyAdjustmentInterval = 0'), a permitted transition implies the observed new target lies in [min_rounded, max_rounded], where the bounds are the clamped ±4-timespan targets recomputed from old nBits, capped at powLimit, then GetCompact-round-tripped; on min-difficulty chains the check returns true before the boundary arm, so no bounds are enforced
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### BLK-051
- summary: with fPowAllowMinDifficultyBlocks = false and off a period boundary ('height % difficultyAdjustmentInterval ≠ 0'): permittedDifficultyTransition = true iff nBits is unchanged; min-difficulty chains bypass the check, so nBits need not be constant within a period
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### BLK-052
- summary: fPowAllowMinDifficultyBlocks ⇒ always permitted
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### BLK-053
- summary: if the new height is off a period boundary, fPowAllowMinDifficultyBlocks = true, the parent is known, and blockTime > prevTime + 2×600, then nBits = powLimit (testnet3/testnet4/regtest); at a period boundary the retarget arm runs instead, so that case is not covered
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### BLK-054
- summary: the walk skips a step-height k only when k is off a period boundary, its nBits equals powLimit, and its parent exists; hence if it stops at ancestor h+1 (one of those conditions fails) and every height above h+1 up to the top is a skip, the result is that ancestor's nBits — the general nearest-stop search and the height-0 terminal are not proved
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### BLK-055
- summary: on period boundaries: blockTime ≥ prevBlockTime − 600 ('time-timewarp-attack'); testnet4/regtest only
- census: proven consensus
- unmapped: BIP94 is not implemented (testnet4 retarget from the first block of the period, and the 600-second timewarp bound). It does not apply to mainnet, testnet3, signet, or default regtest. There is no production check.

### BLK-056
- summary: MTP is the median of the last <=11 block times including the index itself (validator MTP(prev) excludes the candidate block); definedness, window-determination and the empty-window case are proved
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### BLK-057
- summary: blockTime > MTP(prev) ('time-too-old')
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### BLK-058
- summary: blockTime ≤ node now + 7200 s ('time-too-new')
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### BLK-059
- summary: height = pindexPrev.height + 1 (no separate rule)
- census: proven consensus
- production: `crates/rbitcoin-net/src/chain.rs`
- test: `crates/rbitcoin-net/src/chain.rs`

### BLK-060
- summary: nVersion ≥ 2 when BIP34 active ('bad-version')
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### BLK-061
- summary: nVersion ≥ 3 when BIP66 active
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### BLK-062
- summary: nVersion ≥ 4 when BIP65 active
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/header.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### BLK-063
- summary: prev hash must be a known header ('prev-blk-not-found')
- census: proven consensus
- production: `crates/rbitcoin-net/src/chain.rs`
- test: `crates/rbitcoin-net/src/chain.rs`

### BLK-064
- summary: prev not BLOCK_FAILED_VALID ('bad-prevblk')
- census: proven consensus
- production: `crates/rbitcoin-net/src/chain.rs`
- test: `crates/rbitcoin-net/src/chain.rs`

### BLK-065
- summary: known hash ⇒ return cached verdict; genesis exempt from all header checks
- census: proven consensus
- production: `crates/rbitcoin-net/src/chain.rs`
- test: `crates/rbitcoin-net/src/chain.rs`

### BLK-066
- summary: nLockTime==0 or < cutoff(height if <500M else time) or all SEQUENCE_FINAL
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/finality_tests.rs`

### BLK-067
- summary: finality cutoff = MTP(prev) when CSV active, else block time
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/finality_tests.rs`

### BLK-068
- summary: every tx in block must be final ('bad-txns-nonfinal')
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/finality_tests.rs`

### BLK-069
- summary: coinbase scriptSig starts with CScriptNum(height) minimal encoding ('bad-cb-height')
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/bip34_tests.rs`

### BLK-070
- summary: mainnet: height 227931, hash 000000000000024b89…08b8
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/bip34_tests.rs`

### BLK-071
- summary: BIP65@388381, BIP66@363725, CSV@419328, segwit@481824 (mainnet)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/params.rs`
- test: `crates/rbitcoin-consensus/src/params.rs`

### BLK-072
- summary: deployment active at block ⇔ block height ≥ activation height
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/params.rs`
- test: `crates/rbitcoin-consensus/src/params.rs`

### BLK-073
- summary: bit set iff (version & 0xE0000000)==0x20000000 and deployment bit set
- census: proven consensus
- unmapped: Shipped activations are buried height compares on ChainParams (crates/rbitcoin-consensus/src/params.rs). The BIP9 state machine (STARTED, LOCKED_IN, ACTIVE, FAILED, timeout, threshold) is not implemented. No shipped network still activates a rule through it.

### BLK-074
- summary: state defined per period: evaluate at pindexPrev with (h+1) mod period == 0
- census: proven consensus
- unmapped: Shipped activations are buried height compares on ChainParams (crates/rbitcoin-consensus/src/params.rs). The BIP9 state machine (STARTED, LOCKED_IN, ACTIVE, FAILED, timeout, threshold) is not implemented. No shipped network still activates a rule through it.

### BLK-075
- summary: From state STARTED at a period boundary, count signals among the 'period' blocks ending at that boundary; count >= threshold at the boundary => LOCKED_IN (versionbits.cpp:83-94 @cce32b168).
- census: proven consensus
- unmapped: Shipped activations are buried height compares on ChainParams (crates/rbitcoin-consensus/src/params.rs). The BIP9 state machine (STARTED, LOCKED_IN, ACTIVE, FAILED, timeout, threshold) is not implemented. No shipped network still activates a rule through it.

### BLK-076
- summary: From state STARTED at a period boundary with count < threshold, MTP >= nTimeout => FAILED (versionbits.cpp:95-97 @cce32b168); the count test precedes the timeout test.
- census: proven consensus
- unmapped: Shipped activations are buried height compares on ChainParams (crates/rbitcoin-consensus/src/params.rs). The BIP9 state machine (STARTED, LOCKED_IN, ACTIVE, FAILED, timeout, threshold) is not implemented. No shipped network still activates a rule through it.

### BLK-077
- summary: threshold reached in the final period before timeout still locks in
- census: proven consensus
- unmapped: Shipped activations are buried height compares on ChainParams (crates/rbitcoin-consensus/src/params.rs). The BIP9 state machine (STARTED, LOCKED_IN, ACTIVE, FAILED, timeout, threshold) is not implemented. No shipped network still activates a rule through it.

### BLK-078
- summary: LOCKED_IN → ACTIVE only when next height ≥ min_activation_height
- census: proven consensus
- unmapped: Shipped activations are buried height compares on ChainParams (crates/rbitcoin-consensus/src/params.rs). The BIP9 state machine (STARTED, LOCKED_IN, ACTIVE, FAILED, timeout, threshold) is not implemented. No shipped network still activates a rule through it.

### BLK-079
- summary: nStartTime −1 ⇒ ACTIVE; −2 ⇒ FAILED, no state machine
- census: proven consensus
- unmapped: Shipped activations are buried height compares on ChainParams (crates/rbitcoin-consensus/src/params.rs). The BIP9 state machine (STARTED, LOCKED_IN, ACTIVE, FAILED, timeout, threshold) is not implemented. No shipped network still activates a rule through it.

### BLK-080
- summary: null parent ⇒ DEFINED for every deployment
- census: proven consensus
- unmapped: Shipped activations are buried height compares on ChainParams (crates/rbitcoin-consensus/src/params.rs). The BIP9 state machine (STARTED, LOCKED_IN, ACTIVE, FAILED, timeout, threshold) is not implemented. No shipped network still activates a rule through it.

### BLK-081
- summary: only testdummy: bit 28, NEVER_ACTIVE, threshold 1815/period 2016
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/params.rs`
- test: `crates/rbitcoin-consensus/src/params.rs`

### BLK-082
- summary: testnet4 1512/2016; regtest 108/144 start 0
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/params.rs`
- test: `crates/rbitcoin-consensus/src/params.rs`

### BLK-083
- summary: P2SH
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### BLK-084
- summary: exception blocks: BIP16-exempt 0000…02dc…(NONE), taproot-exempt 0000…0f14…(P2SH
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### BLK-085
- summary: SCRIPT_VERIFY_DERSIG when BIP66 active at block
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### BLK-086
- summary: CHECKLOCKTIMEVERIFY when BIP65 active
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### BLK-087
- summary: CHECKSEQUENCEVERIFY + LOCKTIME_VERIFY_SEQUENCE when CSV active
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### BLK-088
- summary: NULLDUMMY when segwit active (BIP147)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### BLK-089
- summary: Σ coinbase outputs ≤ fees + subsidy ('bad-cb-amount')
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### BLK-090
- summary: subsidy = 50 COIN >> (height/210000); 0 when halvings ≥ 64
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### BLK-091
- summary: Σ fees must be in MoneyRange ('bad-txns-accumulated-fee-outofrange')
- census: proven consensus
- unmapped: Output values and the output sum are capped at MAX_MONEY (`exceeds_max_money` and `money_range_out_sum` in crates/rbitcoin-consensus/src/block/mod.rs). A transaction input sum and the block fee accumulator are not capped at MAX_MONEY. This row is that missing cap, so it has no production site.

### BLK-092
- summary: per input: SpendCoin (record undo); then add all outputs as coins
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/facts.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### BLK-093
- summary: outputs with unspendable scriptPubKey are never inserted into the coin set
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/facts.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### BLK-094
- summary: script larger than MAX_SCRIPT_SIZE (10000), or a nonempty script whose first byte is OP_RETURN (0x6a) ⇒ unspendable; an empty script is NOT unspendable
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/policy.rs`
- test: `crates/rbitcoin-consensus/src/policy.rs`

### BLK-095
- summary: coinbase coins may overwrite (models pre-BIP30 duplicates); others require explicit check
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/facts.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### BLK-096
- summary: any output of the new block already in UTXO ⇒ invalid ('bad-txns-BIP30')
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/facts.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### BLK-097
- summary: blocks 91842 (…0a4d0a…) and 91880 (…0743f1…) are exempt on the canonical chain
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/facts.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### BLK-098
- summary: skip BIP30 once chain's block@BIP34Height has BIP34Hash, except ≥ height 1983702 re-enables
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/facts.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### BLK-099
- summary: blocks 91722 (…0271a2…) and 91812 (…0af0ae…) relax disconnect consistency (height-only predicate); the coinstats-index exclusion of first instances is not modelled and is not claimed
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/facts.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### BLK-100
- summary: coinbase input spendable iff spendHeight − coinHeight ≥ 100 ('bad-txns-premature-spend-of-coinbase')
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/facts.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### BLK-101
- summary: a coin is well formed exactly when its height is in the 31-bit on-disk domain ('height.toNat < 2^31'); every coin inserted by 'addedCoins' and every coin surviving 'updateCoins' is well formed whenever the connection height is below '2^31', decoded 32-bit metadata words always land in the domain, and the predicate is not automatic for an arbitrary 'Coin' record
- census: proven consensus
- unmapped: Core's Coin type (CTxOut plus height plus coinbase, spent when the out is null) is not a type in this tree. Spends are archive rows resolved in confirm. This row is that C++ data model, not a check we run.

### BLK-102
- summary: every prevout present in UTXO view ('bad-txns-inputs-missingorspent')
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/facts.rs`
- test: `crates/rbitcoin-test/tests/consensus_rules.rs`

### BLK-103
- summary: Σin ≥ Σout, both in MoneyRange; fee = Σin − Σout ('bad-txns-in-belowout')
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/facts.rs`
- test: `crates/rbitcoin-test/tests/consensus_rules.rs`

### BLK-104
- summary: BIP68 block gate: every non-coinbase tx is BIP68-final against a caller-supplied 'prevDataOf' (per-input entry height/time pairs); the gate is a conjunct of the proof-local 'connectBlockChecks' and is not wired to the real ConnectBlock router; Core enforces BIP68 both in ConnectBlock and at mempool admission ('CheckSequenceLocksAtTip', 'non-BIP68-final')
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/facts.rs`
- test: `crates/rbitcoin-test/tests/consensus_rules.rs`

### BLK-105
- summary: CTxUndo stores prior Coin per spent input, in input order
- census: proven consensus
- production: `crates/rbitcoin-query/src/connect.rs`
- test: `crates/rbitcoin-test/tests/scenarios.rs`

### BLK-106
- summary: replay undos in reverse; outputs re-checked against coins (mismatch ⇒ unclean)
- census: proven consensus
- production: `crates/rbitcoin-query/src/connect.rs`
- test: `crates/rbitcoin-test/tests/scenarios.rs`

### BLK-107
- summary: unspendable outputs excluded from disconnect verification (symmetry with BLK-093)
- census: proven consensus
- production: `crates/rbitcoin-query/src/connect.rs`
- test: `crates/rbitcoin-test/tests/scenarios.rs`

### BLK-108
- summary: vtxundo.size()+1 == vtx.size() else treat as data corruption
- census: proven consensus
- production: `crates/rbitcoin-query/src/connect.rs`
- test: `crates/rbitcoin-test/tests/scenarios.rs`

### BLK-109
- summary: genesis block: skip tx connection, just set best block
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/facts.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### BLK-110
- summary: genesis 50 BTC output never enters UTXO set (empty-set genesis state)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/facts.rs`
- test: `crates/rbitcoin-query/src/structure_rule_tests.rs`

### BLK-111
- summary: time 1231006505, nonce 2083236893, bits 1d00ffff, coinbase text \"The Times 03/Jan/2009…\", hash 000000000019d668…ce26f, merkle 4a5e1e4b…a33b
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/params.rs`
- test: `crates/rbitcoin-consensus/src/params.rs`

### BLK-112
- summary: P2PK to pubkey 04678afd…b6bf11d5f, 50 COIN
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/params.rs`
- test: `crates/rbitcoin-consensus/src/params.rs`

### BLK-113
- summary: work(nBits) = (~target/(target+1)) + 1; 0 for invalid decode
- census: proven consensus
- production: `crates/rbitcoin-net/src/most_work.rs`
- test: `crates/rbitcoin-net/src/most_work.rs`

### BLK-114
- summary: nChainWork(B) = nChainWork(prev) + work(B.nBits); genesis = own work
- census: proven consensus
- production: `crates/rbitcoin-net/src/chain.rs`
- test: `crates/rbitcoin-net/src/chain.rs`

### BLK-115
- summary: active tip = max nChainWork among valid, data-complete candidates
- census: proven consensus
- production: `crates/rbitcoin-net/src/chain.rs`
- test: `crates/rbitcoin-net/src/chain.rs`

### BLK-116
- summary: equal work ⇒ lower nSequenceId (earlier-received/active-chain) wins, then pointer order (non-consensus detail)
- census: proven consensus
- production: `crates/rbitcoin-net/src/chain.rs`
- test: `crates/rbitcoin-net/src/chain.rs`

### BLK-117
- summary: tip changes only to strictly better work (ties keep current tip)
- census: proven consensus
- production: `crates/rbitcoin-net/src/chain.rs`
- test: `crates/rbitcoin-net/src/chain.rs`

### BLK-118
- summary: switch via DisconnectTip to fork point then ConnectTip forward, undo data per block
- census: proven consensus
- production: `crates/rbitcoin-net/src/chain.rs`
- test: `crates/rbitcoin-net/src/chain.rs`

### BLK-119
- summary: non-mutated failures mark BLOCK_FAILED_VALID; descendants pruned from candidates; pure memoization
- census: proven consensus
- production: `crates/rbitcoin-net/src/chain.rs`
- test: `crates/rbitcoin-net/src/chain.rs`

### BLK-120
- summary: cached-failed block re-submission rejected as 'duplicate-invalid' without re-validation
- census: proven consensus
- production: `crates/rbitcoin-net/src/chain.rs`
- test: `crates/rbitcoin-net/src/chain.rs`

### BLK-121
- summary: track highest-work invalid chain (warning/UI only)
- census: proven consensus
- production: `crates/rbitcoin-net/src/chain.rs`
- test: `crates/rbitcoin-net/src/chain.rs`

### BLK-122
- summary: headers below nMinimumChainWork not indexed pre-sync; anti-DoS, not consensus
- census: proven consensus
- production: `crates/rbitcoin-node/src/config.rs`
- test: `crates/rbitcoin-node/src/config.rs`

### BLK-123
- summary: script checks skippable below assumevalid ancestor with 2-week equivalent-time guard; trust-parameterized
- census: proven consensus
- unmapped: Core assumevalid is not implemented. That predicate skips scripts only with chain membership, minimum chainwork, and a two-week proof-equivalent-time burial. rbitcoin's script skip is Milestone (crates/rbitcoin-consensus/src/milestone.rs), which is a different rule, so this row is not mapped onto that file.

### BLK-124
- summary: script verification parallelizable; results identical to serial
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script_pool.rs`
- test: `crates/rbitcoin-consensus/src/block/block_866342.rs`

### BLK-125
- summary: coinbase-less block with a 64-byte tx ⇒ mutated (relay-path only)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-rpc/src/regtest_chain_ops_journey.rs`

### BLK-126
- summary: signet only: block solution satisfies challenge (BIP325); non-mainnet
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/signet.rs`
- test: `crates/rbitcoin-consensus/src/signet.rs`

### BLK-127
- summary: fChecked/m_checked_* are memoization only, no rule content
- census: proven consensus
- unmapped: Core's fWitnessValidated / witness-cache bits on a block index are not stored. A block is either confirmed or not. This row is that cache flag.

### BLK-128
- summary: mining fills 32-byte zero reserved value when segwit active; construction rule, mining-side
- census: proven consensus
- unmapped: Core auto-fills an empty coinbase witness with a 32-byte reserved value when a witness commitment is present. rbitcoin requires the reserved value on the wire (witness_commitment checks in crates/rbitcoin-consensus/src/block/mod.rs) and does not invent it. This row is the auto-fill, which we do not do.

### BLK-129
- summary: constant-space path algorithm; only append composition is proved (merklePathRoot_append); the verifier is self-referential (merklePathVerify root leaf path = bytesBEq (merklePathRoot leaf path) root), so equivalence with the level-duplication tree (merkleLevel) or a block-header root is not proved
- census: proven consensus
- production: `crates/rbitcoin-primitives/src/merkle.rs`
- test: `crates/rbitcoin-primitives/src/merkle.rs`

### BLK-130
- summary: MAX_FUTURE_BLOCK_TIME=7200 s also defines TIMESTAMP_WINDOW for headers sync
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/clock.rs`
- test: `crates/rbitcoin-consensus/src/header.rs`

### SWT-001
- summary: scriptPubKey (or redeemScript) of 4..42 bytes = (OP_0 or OP_1..OP_16) + direct push of 2..40 bytes whose length byte + 2 equals total size
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SWT-002
- summary: witness version = OP_N decoded value (OP_0→0, OP_1..OP_16→1..16)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SWT-003
- summary: if scriptPubKey is a witness program, scriptSig must be exactly empty
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SWT-004
- summary: if redeemScript is a witness program, scriptSig must be exactly one push of that redeemScript
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SWT-005
- summary: v0 20-byte program requires witness of exactly 2 items
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2wpkh.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_verify.rs`

### SWT-006
- summary: executed script = 'OP_DUP OP_HASH160 <program> OP_EQUALVERIFY OP_CHECKSIG'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2wpkh.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_verify.rs`

### SWT-007
- summary: v0 32-byte program requires non-empty witness
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2wsh.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_verify.rs`

### SWT-008
- summary: SHA256(last witness item) must equal the 32-byte program (single SHA256)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2wsh.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_verify.rs`

### SWT-009
- summary: v0 program of size ∉ {20,32} → WITNESS_PROGRAM_WRONG_LENGTH
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2wsh.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_verify.rs`

### SWT-010
- summary: every initial witness stack item ≤ 520 bytes
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SWT-011
- summary: executed script under WITNESS_V0 ≤ 10,000 bytes
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SWT-012
- summary: witness execution must end with exactly 1 truthy stack item
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SWT-013
- summary: witness v0 programs of length other than 20 or 32 are rejected with WITNESS_PROGRAM_WRONG_LENGTH; only other versions (v>=2, and v1 that is not the native 32-byte shape, i.e. wrong length or P2SH-wrapped) pass through as anyone-can-spend unless DISCOURAGE_UPGRADABLE_WITNESS_PROGRAM is set.
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SWT-014
- summary: optional flag: unknown witness program → error (policy/test hook)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### SWT-015
- summary: non-null witness on a non-witness-program output fails (WITNESS flag on)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SWT-016
- summary: WITNESS flag requires P2SH flag (structural assertion)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SWT-017
- summary: weight = base_size×3 + total_size ≤ 4,000,000
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/sigop_cost_tests.rs`

### SWT-018
- summary: legacy units are weight/4
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/sigop_cost_tests.rs`

### SWT-019
- summary: tx weight = ser(no-witness)×3 + ser(with-witness)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/sigop_cost_tests.rs`

### SWT-020
- summary: vsize = ⌈weight/4⌉ (policy-facing but defined here)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/tx_precompute.rs`
- test: `crates/rbitcoin-consensus/src/tx_precompute.rs`

### SWT-021
- summary: Σ sigop-cost ≤ 80,000 per block
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/sigop_cost_tests.rs`

### SWT-022
- summary: legacy script sigops count 4 cost units each
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/sigop_cost_tests.rs`

### SWT-023
- summary: P2SH redeemScript sigops count 4 cost units each
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/sigop_cost_tests.rs`

### SWT-024
- summary: witness-derived sigops count 1 cost unit each (not ×4)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/sigop_cost_tests.rs`

### SWT-025
- summary: each P2WPKH input counts exactly 1 sigop
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2wpkh.rs`
- test: `crates/rbitcoin-consensus/src/block/sigop_cost_tests.rs`

### SWT-026
- summary: accurate sigop count of the last witness item (witnessScript)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/sigop_cost_tests.rs`

### SWT-027
- summary: P2SH scriptSig's last push, if it is a witness program, yields witness sigops -- but only when the WITNESS flag is on and the scriptSig is push-only; the model's 'p2shWitnessSigOps' carries neither gate (its docstring claims the non-push-only case returns 0, its body does not), so it returns 1 for a non-push-only scriptSig whose last op pushes a v0-20 program, where Core's 'CountWitnessSigOps' returns 0; the gated counter is TXV-051's 'countWitnessSigOps'
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/block/sigop_cost_tests.rs`

### SWT-028
- summary: witness versions >0 contribute 0 block sigops
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/block/sigop_cost_tests.rs`

### SWT-029
- summary: coinbase output 'OP_RETURN 0x24 0xaa 0x21 0xa9 0xed <32B>' selects the commitment
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-test/tests/consensus_rules.rs`

### SWT-030
- summary: committed value = SHA256d(witness_merkle_root ‖ reserved_value), compared to scriptPubKey[6..38]
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-test/tests/consensus_rules.rs`

### SWT-031
- summary: coinbase input witness must be exactly one 32-byte item when commitment present
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-test/tests/consensus_rules.rs`

### SWT-032
- summary: root over wtxids of all non-coinbase txs; coinbase leaf = 0
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-test/tests/consensus_rules.rs`

### SWT-033
- summary: block with no commitment output may contain no witness data
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-test/tests/consensus_rules.rs`

### SWT-034
- summary: blocks after segwit activation must pass CheckWitnessMalleation(expect=true)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-test/tests/consensus_rules.rs`

### SWT-035
- summary: mainnet segwit active from block 481,824 (flags via pindexPrev)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-test/tests/consensus_rules.rs`

### SWT-036
- summary: segwit deployed by BIP9 name \"segwit\", bit 1, start 2016-11-15
- census: proven consensus
- unmapped: Shipped activations are buried height compares on ChainParams (crates/rbitcoin-consensus/src/params.rs). The BIP9 state machine (STARTED, LOCKED_IN, ACTIVE, FAILED, timeout, threshold) is not implemented. No shipped network still activates a rule through it.

### SWT-037
- summary: vin==0 ⇒ next byte is flag; flag&1 ⇒ per-input witness stacks after vout
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/tx_precompute.rs`
- test: `crates/rbitcoin-consensus/src/tx_precompute.rs`

### SWT-038
- summary: witness encoding: dummy byte 0x00 then flag 0x01
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/tx_precompute.rs`
- test: `crates/rbitcoin-consensus/src/tx_precompute.rs`

### SWT-039
- summary: witness flag with all-empty stacks = serialization error
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/tx_precompute.rs`
- test: `crates/rbitcoin-consensus/src/tx_precompute.rs`

### SWT-040
- summary: any flag bit ≠ 1 → error (future flags are hardforks)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/tx_precompute.rs`
- test: `crates/rbitcoin-consensus/src/tx_precompute.rs`

### SWT-041
- summary: txid = SHA256d(ser, no witness); wtxid = SHA256d(ser, with witness)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/tx_precompute.rs`
- test: `crates/rbitcoin-consensus/src/tx_precompute.rs`

### SWT-042
- summary: SHA256d over the 10-field preimage (version, 3 sub-hashes, outpoint, scriptCode, amount, sequence, outputs-hash, locktime, hashtype)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2wpkh.rs`
- test: `crates/rbitcoin-consensus/src/script/core_sighash.rs`

### SWT-043
- summary: SHA256d(all outpoints) iff ¬ANYONECANPAY else zero-vector
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2wpkh.rs`
- test: `crates/rbitcoin-consensus/src/script/core_sighash.rs`

### SWT-044
- summary: SHA256d(all nSequence) iff ¬ACP ∧ ¬(SINGLE∨NONE) else zero
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2wpkh.rs`
- test: `crates/rbitcoin-consensus/src/script/core_sighash.rs`

### SWT-045
- summary: SHA256d(all outputs) iff ¬(SINGLE∨NONE); SINGLE∧in-range ⇒ SHA256d(vout[nIn]); else zero
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2wpkh.rs`
- test: `crates/rbitcoin-consensus/src/script/core_sighash.rs`

### SWT-046
- summary: for WITNESS_V0, SINGLE with nIn ≥ vout.size uses zero hashOutputs (no uint256::ONE early return) — legacy SINGLE bug absent
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2wpkh.rs`
- test: `crates/rbitcoin-consensus/src/script/core_sighash.rs`

### SWT-047
- summary: scriptCode = '0x1976a914{20B}88ac' (CompactSize-prefixed in the digest)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2wpkh.rs`
- test: `crates/rbitcoin-consensus/src/script/core_sighash.rs`

### SWT-048
- summary: scriptCode = witnessScript from last executed OP_CODESEPARATOR onward
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2wpkh.rs`
- test: `crates/rbitcoin-consensus/src/script/core_sighash.rs`

### SWT-049
- summary: signature is removed from scriptCode only for SigVersion::BASE
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2wpkh.rs`
- test: `crates/rbitcoin-consensus/src/script/core_sighash.rs`

### SWT-050
- summary: 8-byte LE spent amount inside the digest
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2wpkh.rs`
- test: `crates/rbitcoin-consensus/src/script/core_sighash.rs`

### SWT-051
- summary: digest ends with 4-byte LE hashtype (appended even on cache hit)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2wpkh.rs`
- test: `crates/rbitcoin-consensus/src/script/core_sighash.rs`

### SWT-052
- summary: BIP143 digest and sub-hashes are SHA256d
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2wpkh.rs`
- test: `crates/rbitcoin-consensus/src/script/core_sighash.rs`

### SWT-053
- summary: last sig byte = hashtype, popped before DER/verify
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2wpkh.rs`
- test: `crates/rbitcoin-consensus/src/script/core_sighash.rs`

### SWT-054
- summary: empty signature fails CHECKSIG (v0/base)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2wpkh.rs`
- test: `crates/rbitcoin-consensus/src/script/core_sighash.rs`

### SWT-055
- summary: WITNESS_V0 sighash requires amount ≥ 0; missing data ⇒ assert (consensus) or fail
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2wpkh.rs`
- test: `crates/rbitcoin-consensus/src/script/core_sighash.rs`

### SWT-056
- summary: compressed-pubkey restriction exists as a flag; **not** set in consensus (policy)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### SWT-057
- summary: Obsolete/non-consensus: witness-reserved-value plumbing is mining-only (node/miner.cpp; rpc/mining.cpp default_witness_commitment); no consensus rule and no Lean declaration, not proven.
- census: obsolete consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: BIP145 getblocktemplate.

### SWT-058
- summary: low-S rule = policy only (flag LOW_S absent from GetBlockScriptFlags)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### SWT-059
- summary: failing sigs must be empty — policy only
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### SWT-060
- summary: defined-hashtype check = policy only
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### SWT-061
- summary: strict-DER sig encoding enforced from height 363,725
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SWT-062
- summary: MINIMALIF flag exists for v0 but is not consensus
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_verify.rs`

### SWT-063
- summary: NULLDUMMY active exactly with segwit (buried height)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SWT-064
- summary: CHECKMULTISIG dummy element must be empty (consensus, post-segwit)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SWT-065
- summary: CHECKMULTISIG consumes the extra element regardless of flags (genesis rule)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SWT-066
- summary: midstate caching must be outcome-invisible (6 cache slots by hashtype class)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2wpkh.rs`
- test: `crates/rbitcoin-consensus/src/script/core_sighash.rs`

### SWT-067
- summary: transaction weight = 3 x no-witness (stripped) serialization + total (with-witness) serialization, where total = stripped + 2-byte marker/flag + witness-stack bytes, i.e. 4 x stripped + 2 + stack bytes; Core's per-input GetTransactionInputWeight adds scriptWitness.stack separately because CTxIn's with-witness serialization omits it
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/tx_precompute.rs`
- test: `crates/rbitcoin-consensus/src/tx_precompute.rs`

### SWT-068
- summary: a single tx may mix BASE/WITNESS_V0/TAPROOT/TAPSCRIPT inputs; per-input dispatch
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/tx_precompute.rs`
- test: `crates/rbitcoin-consensus/src/tx_precompute.rs`

### TPR-001
- summary: witness v1 + 32-byte program, native only (never P2SH-wrapped)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-002
- summary: without SCRIPT_VERIFY_TAPROOT, v1-32 spends succeed
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-003
- summary: P2SH-embedded v1 programs are unencumbered today
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-004
- summary: 'OP_1 <0x4e73>' (v1, 2-byte program) validates unconditionally with any witness
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-005
- summary: ≥2 witness items and last item starts 0x50 ⇒ annex, popped
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-006
- summary: sha_annex = SHA256(CSize(len)‖annex incl. 0x50); enters sighash iff present
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-007
- summary: exactly 1 element (post-annex) ⇒ single BIP340 sig against output key q
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-008
- summary: ≥2 elements ⇒ last = control block, second-to-last = script
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-009
- summary: len = 33 + 32m, 0 ≤ m ≤ 128, else TAPROOT_WRONG_CONTROL_SIZE
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-010
- summary: v = control[0] & 0xfe
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-011
- summary: low bit of control[0] must equal y(Q) mod 2
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-012
- summary: tag_TapLeaf(v ‖ CSize(script.size) ‖ script)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-013
- summary: parent = tag_TapBranch(sorted pair), single SHA256
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-014
- summary: fold path elements left-to-right from leaf hash
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-015
- summary: t = tag_TapTweak(p ‖ merkle_root) (p alone if root absent)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-016
- summary: q = x(P + int(t)·G) with parity match, else WITNESS_PROGRAM_MISMATCH
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-017
- summary: t < n required
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-018
- summary: control-block pubkey must parse as a curve point (lift_x)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-019
- summary: control[0]&0xfe == 0xc0 executes script with SigVersion::TAPSCRIPT
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-020
- summary: non-0xc0 leaves validate as success (softfork hook)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-021
- summary: optional error for future leaf versions
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### TPR-022
- summary: budget = Σ per-item witness serialization (CompactSize(item length) + item bytes) + 50 WU; Core's 'GetSerializeSize(witness.stack)' additionally serializes the stack-count CompactSize (1/3/5/9 WU), which 'witnessSerSize' omits (F-Y15-001)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_verify.rs`

### TPR-023
- summary: each passed non-empty sigop −50; negative ⇒ TAPSCRIPT_VALIDATION_WEIGHT
- census: obsolete consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: Sigop budget decrement.

### TPR-024
- summary: empty signature: success=false, no decrement, script continues
- census: obsolete consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: Empty sig skips budget.

### TPR-025
- summary: pubkey size 0 ⇒ script failure
- census: obsolete consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: Empty pubkey fails.

### TPR-026
- summary: size ∉ {0,32} ⇒ counted but always-successful sigop
- census: obsolete consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: Unknown pubkey types.

### TPR-027
- summary: optional error for unknown pubkey sizes
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### TPR-028
- summary: otherwise SCHNORR_SIG_SIZE
- census: obsolete consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: Sig size ∈ {64,65}.

### TPR-029
- summary: last byte popped as hashtype; 0x00 rejected in 65-byte form
- census: obsolete consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: 65-byte sig hashtype.

### TPR-030
- summary: hash_type ≤ 3 or 0x81..0x83, else failure
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-031
- summary: missing sighash byte (64B sig) ≡ ALL; only as 64-byte sig
- census: obsolete consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: SIGHASH_DEFAULT.

### TPR-032
- summary: message = tag_TapSighash(0x00 ‖ SigMsg(...))
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-033
- summary: hashtype, nVersion, nLockTime, [prevouts/amounts/scriptpubkeys/sequences], [outputs], spend_type, input data, [annex], [single-output]
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-034
- summary: SHA256(all outpoints), iff ¬ACP
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-035
- summary: SHA256(all spent input amounts), iff ¬ACP — commits to the fee
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-036
- summary: SHA256(all spent scriptPubKeys as CTxOut scripts), iff ¬ACP
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-037
- summary: SHA256(all nSequence), iff ¬ACP (incl. NONE/SINGLE!)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-038
- summary: only when output_type = ALL/DEFAULT
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-039
- summary: (ext_flag × 2) + annex_present
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-040
- summary: outpoint ‖ full prevout (amount+scriptPubKey) ‖ nSequence
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-041
- summary: 4-byte LE input index
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-042
- summary: SINGLE with the output present returns 'SHA256(serTxOut(vout[in_pos]))' when the output-hash cache is sound ('none' or already that hash — Core's only write); 'in_pos ≥ vout.size' ⇒ failure. An arbitrary pre-populated cache value is unconstrained in the seam and is an explicit residual
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-043
- summary: TAPROOT⇒0, TAPSCRIPT⇒1; key_version = 0 (tapscript)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-044
- summary: ‖ tapleaf_hash ‖ key_version(0x00) ‖ codesep_pos(4LE)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-045
- summary: codesep_pos seam slot defaults to 0xFFFFFFFF and tapscript ext slices require codesepPosInit; last-executed OP_CODESEPARATOR opcode-index counting (including parsed-but-unexecuted opcodes and multi-byte pushes counting as 1) is not modeled
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-046
- summary: BIP341 message + sub-hashes use single SHA256 (no double)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-047
- summary: SHA256(SHA256(tag)‖SHA256(tag)‖msg)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-048
- summary: e = int(tag_BIP0340/challenge(r‖P‖m)) mod n
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-049
- summary: y = even root of x³+7 mod p; fail if x ≥ p or non-QR
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-050
- summary: fail if int(sig[0:32]) ≥ p
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-051
- summary: fail if int(sig[32:64]) ≥ n
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-052
- summary: R = s·G − e·P; fail if ∞, odd-y, or x(R) ≠ r
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-053
- summary: verify is a total deterministic function of (sig, pk, msg)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-054
- summary: 32-byte x-coordinate; both parities derivable (GetCPubKeys)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### TPR-055
- summary: v0/base sigversions use ECDSA path; TAPSCRIPT uses Schnorr path
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### TPR-056
- summary: (sig num pubkey -- num+{0,1}); 0xba; tapscript-only
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### TPR-057
- summary: OP_CHECKSIGADD (0xba) executed under BASE/WITNESS_V0 with one opcode of budget left ⇒ BAD_OPCODE; at budget exhaustion OP_COUNT preempts, and an unexecuted branch skips the opcode
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### TPR-058
- summary: num operand >4 bytes ⇒ failure (scriptnum exception path)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### TPR-059
- summary: TAPSCRIPT ⇒ TAPSCRIPT_CHECKMULTISIG failure; ignored unexecuted
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### TPR-060
- summary: opcodes {80,98,126-129,131-134,137-138,141-142,149-153,187-254}
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### TPR-061
- summary: unconditional success, overrides all limits incl. initial-stack size
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### TPR-062
- summary: optional error replacing the success short-circuit
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### TPR-063
- summary: OP_IF/NOTIF operand must be ∅ or 0x01 (consensus in tapscript)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### TPR-064
- summary: 10,000-byte limit not applied to TAPSCRIPT
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### TPR-065
- summary: 201-op limit not applied to TAPSCRIPT
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### TPR-066
- summary: MAX_STACK_SIZE applies to the witness stack before execution
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### TPR-067
- summary: retained from legacy, checked after every opcode
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### TPR-068
- summary: pushes during execution ≤ 520 bytes
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### TPR-069
- summary: v1 spends contribute 0 to the 80,000 cost
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### TPR-070
- summary: key-path signature not counted anywhere in sigop accounting
- census: obsolete consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: Key-path spend: no sigop cost.

### TPR-071
- summary: annex included in witness serialization ⇒ weight & budget base
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_verify.rs`

### TPR-072
- summary: ScriptExecutionData handed to the Schnorr checker entry reaches sighashSchnorr unchanged and its digest is passed verbatim to verify; interpreter-side population of tapleafHash/codesepPos/outputHash and script-path annex forwarding are residual (F-TS02-008)
- census: proven consensus
- unmapped: ScriptExecutionData is Core's interpreter scratch struct (annex hash, codeseparator position, validation weight). Those fields exist as locals in the interpreter and p2tr paths, but there is no ScriptExecutionData type to cite as this row's owner.

### TPR-073
- summary: executed OP_CHECKLOCKTIMEVERIFY/OP_CHECKSEQUENCEVERIFY agree with BASE on '(stack, alt, vf, codeSep, vwLeft)' when one opcode of budget is left; tapscript leaves 'opCount' frozen, at budget exhaustion BASE returns OP_COUNT while tapscript executes, and WITNESS_V0 (same arm) is not separately stated
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### TPR-074
- summary: disabled opcodes are OP_SUCCESSx under tapscript (renamed OP_SUCCESS126..153); disabled-failure unreachable
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### TPR-075
- summary: mainnet activated at 709,632 via BIP9 bit 2 (historical; buried/always-on in master)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### TPR-076
- summary: block '…1e395ad' validates with flags P2SH
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### TPR-077
- summary: {P2SH, WITNESS, TAPROOT} enforced from genesis (mod exceptions) in current master
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### TPR-078
- summary: flags apply when 'DeploymentActiveAt(pindex) = DeploymentActiveAfter(pprev)'; in the height abstraction this is 'min_activation_height ≤ pprev.nHeight + 1', i.e. the current block height
- census: proven consensus
- unmapped: DeploymentActiveAt(pprev) is Core's 'active in this block if active after the parent' helper. Buried heights are compared directly in ChainParams. The pprev indirection is not a function here.

### TPR-079
- summary: tapscript rules activate identically with taproot
- census: obsolete consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: BIP341/342 deployment coupling.

### TPR-080
- summary: consensus path verifies each Schnorr sig individually
- census: obsolete consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: Batch verify = non-consensus.

### TPR-081
- summary: nonce/aux derivation outside validation
- census: obsolete consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: Signing/nonce = non-consensus.

### TPR-082
- summary: H = lift_x(0x50929b74…) construction is wallet guidance
- census: obsolete consensus
- unmapped: Retired in the vendored census (`status: obsolete`). The row stays in the 726-row denominator and is not a consensus check this tree claims. Census check: NUMS guidance non-consensus.

### TPR-083
- summary: t = tag_TapTweak(p) with no merkle root (key-only outputs still tweakable)
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/p2tr.rs`
- test: `crates/rbitcoin-consensus/src/script/core_bip341.rs`

### SCR-183
- summary: for SigVersion::TAPSCRIPT minimal-IF and clean-stack are consensus requirements regardless of the policy flags
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

### SCR-175
- summary: SCRIPT_VERIFY_LOW_S is mempool policy, never in GetBlockScriptFlags
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### SCR-176
- summary: SCRIPT_VERIFY_STRICTENC is policy (hybrid/hybrid pubkey rejection)
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### SCR-177
- summary: SCRIPT_VERIFY_MINIMALDATA is policy (push minimality)
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### SCR-178
- summary: SCRIPT_VERIFY_MINIMALIF applies to v0 witness spends only; tapscript re-imports it (SCR-183)
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### SCR-179
- summary: SCRIPT_VERIFY_NULLFAIL is policy (empty sig must fail check)
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### SCR-180
- summary: SCRIPT_VERIFY_CLEANSTACK is a policy flag for base (non-witness) spends; witness-v0 and tapscript clean-stack are enforced implicitly as consensus regardless of the policy bit (the general tapscript face is not yet proved — only the concrete Tapscript.cleanstack_implicit_tapscript)
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### SCR-181
- summary: SCRIPT_VERIFY_CONST_SCRIPTCODE is policy (no OP_CODESEPARATOR evolution)
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### SCR-182
- summary: SCRIPT_VERIFY_WITNESS_PUBKEYTYPE is policy (compressed keys)
- census: proven not-consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### SGN-101
- summary: secp256k1 domain parameters: generator G = (Gx,Gy) satisfies y² = x³ + 7 mod P
- census: proven consensus
- unmapped: The curve, field, and libsecp256k1 parse/verify path are the rust-bitcoin secp256k1 binding, not code in this repository. Signature acceptance is exercised through the script corpora, but this row's Lean proof is about btc-lean's own curve model. There is no local implementation to cite.

### SGN-102
- summary: modular arithmetic toolkit over explicit modulus with canonical-representative invariant
- census: proven consensus
- unmapped: The curve, field, and libsecp256k1 parse/verify path are the rust-bitcoin secp256k1 binding, not code in this repository. Signature acceptance is exercised through the script corpora, but this row's Lean proof is about btc-lean's own curve model. There is no local implementation to cite.

### SGN-103
- summary: kernel-reducible modular exponentiation: expBits m x bs = x^(bitsVal bs) % m
- census: proven consensus
- unmapped: The curve, field, and libsecp256k1 parse/verify path are the rust-bitcoin secp256k1 binding, not code in this repository. Signature acceptance is exercised through the script corpora, but this row's Lean proof is about btc-lean's own curve model. There is no local implementation to cite.

### SGN-104
- summary: the secp256k1 field prime P = 2^256 − 2^32 − 977 is prime
- census: proven consensus
- unmapped: The curve, field, and libsecp256k1 parse/verify path are the rust-bitcoin secp256k1 binding, not code in this repository. Signature acceptance is exercised through the script corpora, but this row's Lean proof is about btc-lean's own curve model. There is no local implementation to cite.

### SGN-105
- summary: ECDSA pubkey parse: 33-byte compressed (02/03 + sqrt y-recovery), 65-byte uncompressed (04), hybrid (06/07 parity-checked); x,y < P and on-curve
- census: proven consensus
- unmapped: The curve, field, and libsecp256k1 parse/verify path are the rust-bitcoin secp256k1 binding, not code in this repository. Signature acceptance is exercised through the script corpora, but this row's Lean proof is about btc-lean's own curve model. There is no local implementation to cite.

### SGN-106
- summary: lax DER overlong-integer overflow: ONE shared flag zeroes BOTH r and s (Core memsets the whole 64-byte compact buffer)
- census: proven consensus
- unmapped: The curve, field, and libsecp256k1 parse/verify path are the rust-bitcoin secp256k1 binding, not code in this repository. Signature acceptance is exercised through the script corpora, but this row's Lean proof is about btc-lean's own curve model. There is no local implementation to cite.

### SGN-107
- summary: Compact signature parsing REJECTS DER r or s >= n: scalar_set_b32 overflow zeroes both scalars (secp256k1.c:407-425) and the r=0/s=0 gate rejects; only the 32-byte message hash is reduced mod n with overflow ignored (secp256k1.c:481). 33-byte DER integers >= n are BIP66-admissible and reach this path. The tree's Verify.verifyImpl instead reduces r,s mod n and continues, a consensus-visible divergence modelled by verifyImplCore.
- census: proven consensus
- unmapped: The curve, field, and libsecp256k1 parse/verify path are the rust-bitcoin secp256k1 binding, not code in this repository. Signature acceptance is exercised through the script corpora, but this row's Lean proof is about btc-lean's own curve model. There is no local implementation to cite.

### SGN-108
- summary: High-S signatures verify: CPubKey::Verify normalizes s (pubkey.cpp:295-296) only because libsecp verify demands !scalar_is_high, and s -> n-s is verdict-invariant; low-S is a POLICY flag (SCRIPT_VERIFY_LOW_S, set only by policy/policy.h:125; interpreter.cpp:219), never a consensus rule.
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/block/mod.rs`
- test: `crates/rbitcoin-consensus/src/script/tests_flag_parity.rs`

### SGN-109
- summary: lax DER signature parse (pre-BIP66 consensus grammar): long-form seq-len skipped unchecked, zero-skip inside long-form int lengths (<4), negative-DER payloads, leading-zero padding, trailing garbage ignored
- census: proven consensus
- unmapped: The curve, field, and libsecp256k1 parse/verify path are the rust-bitcoin secp256k1 binding, not code in this repository. Signature acceptance is exercised through the script corpora, but this row's Lean proof is about btc-lean's own curve model. There is no local implementation to cite.

### SGN-110
- summary: model-level ECDSA-lax verification: 'verifyImpl sig pk msg = true ↔ VerifyEqRaw sig pk msg', where the model reduces r,s mod n, rejects r=0∨s=0, sets w = s⁻¹ mod n, R = (z·w)·G + (r·w)·Q, rejects R = ∞, and accepts iff R.x mod n = r; Core's compact-parse zero-reject of r,s ≥ n and high-S normalization are unmodeled, and the faithful 'verifyImplCore' has only window-agreement and overflow-rejection lemmas, not an end-to-end iff
- census: proven consensus
- unmapped: The curve, field, and libsecp256k1 parse/verify path are the rust-bitcoin secp256k1 binding, not code in this repository. Signature acceptance is exercised through the script corpora, but this row's Lean proof is about btc-lean's own curve model. There is no local implementation to cite.

### SGN-111
- summary: BIP66 IsValidSignatureEncoding — strict DER validator (verifier-side): size 9..73 incl. trailing sighash byte (sig[1] = size−3), minimal encodings, no negatives
- census: proven consensus
- production: `crates/rbitcoin-consensus/src/script/interpreter.rs`
- test: `crates/rbitcoin-consensus/src/script/core_vectors.rs`

Mapped rows: 612. Unmapped rows: 114. Census rows: 726.
