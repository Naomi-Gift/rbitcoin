# 107 — A torn spend page below the durable marker stays

**Severity:** medium
**Status:** fixed
**Found by:** bobberb and muse, 2026-10-18

Historical spend annotate inserts the 8-byte slot into a directory. It does not read or pwrite `spent.body`. `rbtc-spend-sync` appends one 4 KiB image per dirty page to `spend.pages`, renames `spend.pages.commit`, and only then advances `spend_durable`. A zeroed body page below that marker stays spent while the commit exists. The next checkpoint folds a generation the previous one left quiet. The live tail page stays a Class A append. Replay still covers only `(A, tip]`. [046](./046-spend-durability.md) fixed the kill-9 gap where the marker advanced without the data sync.

**Regression:** `historical_page_stays_unwritten_until_spend_checkpoint`
