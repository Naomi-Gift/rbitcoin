Fixed

- **An outspend lookup for a `vout` past the transaction's outputs answers
  unspent.** Esplora's `GET /tx/{txid}/outspend/{vout}` returned HTTP 500 and
  Electrum's `blockchain.outpoint.get_status` returned `corrupt record: spent
  slot OOB` for any `vout` at or beyond the output count, because the spender
  read treats that slot as store corruption. Both now bound `vout` by the
  confirmed transaction's output count, or the mempool transaction's, and answer
  `{"spent": false}` as electrs does.
