Fixed

- **Compact-filter replies and served compact blocks count their real
  payload on the per-peer send budget.** A `cfilter`, `cfheaders`,
  `cfcheckpt`, or `cmpctblock` is charged the consensus encoding of that
  message, not a flat 64 or 1,024 bytes. One reply may still cross the
  4 MiB cap. The next request from that peer waits until the writer drains.
  One `getcfilters` stops after the filter that crosses the cap.
  `getcfcheckpt` still returns a filter header every 1,000 heights, from
  one range read.
- **A peer gets one pong, and that pong counts on the send budget.**
  Tip-follow and IBD each keep the latest ping nonce, including nonce 0,
  on a single queued pong. Writing it subtracts that one charge. IBD pings
  past the per-peer message window take the existing misbehavior disconnect.
  A block this node requested still skips that window.
- **IBD stall counts solicited block bytes.** A peer with getdata on the
  wire and a stale block clock is withholding when only pings or headers
  have arrived. Solicited block bytes of 64 KiB since the last progress
  stamp keep that peer. Relative-slow zero-throughput sampling uses the
  same delta. A peer that sends no socket bytes stays the quiet case.
