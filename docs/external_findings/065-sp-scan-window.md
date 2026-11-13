# 065 — Silent-payment scan is bounded per chunk, not per span

**Severity:** medium
**Status:** fixed
**Found by:** Stephan Livera, 2026-10-02 (M2)

A silent-payment scan whose start height was not zero walked from there to the tip in one piece and kept running after the client disconnected. The first fix kept every scan inside the last 256 heights. That also dropped the history a wallet asked for: Sparrow restores from the wallet birthday, and a payment older than 256 blocks was never scanned while `start_height` looked honored.

The span is now the one requested. The work is bounded another way. A scan holds one of three process-wide permits for at most 256 heights, then releases it. It runs beside the session and stops before its next chunk when the connection ends, when the client unsubscribes, or when a new subscribe replaces it. A connection has one scan. The scan uses the tweak index. It does not require `--sh-index`.

**Regression:** `rbitcoin-electrum` `sp_chunk_end_covers_one_height_and_stops_at_the_tip`, `sp_scan_stops_when_the_session_drops_it`, `sp_unsubscribe_drops_a_running_scan`, `sp_timestamp_start_finds_a_payment_older_than_one_chunk`.
