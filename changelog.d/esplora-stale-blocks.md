Fixed

- Esplora serves archived stale blocks by hash, including header, raw block,
  transaction lists and summaries. Block status includes the original height
  with `in_best_chain: false` and `next_best: null`. Stale height lookup is
  bounded to 10,000 headers; `asof` pins still require a best-chain ancestor.
