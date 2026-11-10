Fixed

- **Compact-filter replies and served compact blocks count their real
  payload on the per-peer send budget.** A `cfilter`, `cfheaders`,
  `cfcheckpt`, or `cmpctblock` is charged the consensus encoding of that
  message, not a flat 64 or 1,024 bytes. One reply may still cross the
  4 MiB cap. The next request from that peer waits until the writer drains.
