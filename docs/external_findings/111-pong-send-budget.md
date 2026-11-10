# 111 — Pongs were uncharged, and IBD pings were not rate-limited

**Severity:** medium
**Status:** fixed
**Found by:** private report, 2026-11-10

Every tip-follow ping queued a pong that the send budget did not count.
The writer then subtracted that size anyway, so the counter could fall
below the bytes still queued. IBD answered every ping with no message
cap and no send budget.

Tip-follow and IBD now keep one pong carrying the latest nonce, including
nonce 0. The tip-follow pong is charged, and the write subtracts that
charge. IBD pings use the existing per-peer message window. A requested
block still skips it.

**Regression:** `rbitcoin-net` `peer::one_charged_pong_carries_the_latest_nonce`, `rbitcoin-ibd` `ibd::peer_io::ibd_pings_coalesce_and_overflow_the_message_window`.
