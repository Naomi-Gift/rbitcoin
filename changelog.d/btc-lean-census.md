Added

- **btc-lean consensus census is vendored and indexed.** `third_party/btc-lean` pins Rob1Ham/btc-lean. `docs/btc-lean-census.md` lists every consensus rule in that pin's `spec/db/rules.yaml` and cites the statement that implements it and the statement that tests it, or why the row has no site. Those statements carry a `// btc-lean:` comment. `scripts/btc-lean-census.sh` fails if an id, a comment, or a line number drifts.
