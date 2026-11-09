Fixed

- **Outbound peers on a low-work chain drop during IBD.** A headers
  message that ends the peer's chain and sits under `-minimumchainwork`
  disconnects outbound full-relay and block-relay peers. Inbound and
  manual peers stay. `feature_minchainwork.py` is run again. Port of
  bitcoin/bitcoin#36426 (#1484).
