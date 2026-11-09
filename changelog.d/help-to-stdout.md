Fixed

- **`--help` and `--version` print to stdout.** `rbitcoin-node` and
  `rbitcoin-cli` wrote both to stderr, so `rbitcoin-node --help | less` showed
  nothing. Errors stay on stderr.
- **Node help states the per-peer download cap.** It said 16 blocks in transit
  per peer; the default is 64. The text now reads the IBD constants.
