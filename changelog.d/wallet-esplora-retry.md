Fixed

- **The test wallet retries one slow Esplora UTXO read.**
  `sendtoaddress` used to fail the RPC on a single timed-out
  `GET /address/:addr/utxo`. A read that never answers stops
  inside the harness client timeout instead of retrying for
  about three minutes.
- **The test harness binds Esplora above the ephemeral port range.**
  `rpcport+20000` sat around 36000-41000, so a busy runner could
  already own that port and the node failed to start.
