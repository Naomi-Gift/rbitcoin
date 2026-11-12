Fixed

- **Electrum outpoint subscribers hear a mempool spend at once.** A
  `blockchain.outpoint.subscribe` connection was told of a spend only at the
  next block, although `blockchain.outpoint.get_status` already reported the
  mempool spender. A mempool accept that spends a subscribed outpoint, or
  frees it by replacing its spender, now pushes that outpoint's status. This
  works with `--sh-index` off. A spender that leaves the mempool by eviction,
  expiry, or a block conflict is still reported at the next tip.
