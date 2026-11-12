Fixed

- **The test wallet retries one slow Esplora UTXO read.**
  `sendtoaddress` used to fail the RPC on a single timed-out
  `GET /address/:addr/utxo`.
