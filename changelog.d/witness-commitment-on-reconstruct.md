Fixed

- **A block rebuilt from the store is checked against its witness commitment
  before it is served.** A block whose witness bytes were damaged on disk was
  returned by `getblock`, the P2P block path, and Esplora's raw block endpoint
  as if it were intact: the existing guard compares the rebuilt header's hash
  to the stored hash, and the header row supplies its own `merkle_root`, so no
  transaction byte reached that comparison. Reconstruction now recomputes the
  wtxids and refuses a block whose coinbase carries a witness commitment that
  no longer matches. A block whose coinbase carries no commitment is unchanged,
  which covers every pre-segwit block.
