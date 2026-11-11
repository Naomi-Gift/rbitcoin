Fixed

- **The script pool tests no longer flake on a stolen chunk.** `warn_fault_without_keep_fails_the_wave` published a wave without the lock the steal tests hold, so a steal test could claim its chunks.
