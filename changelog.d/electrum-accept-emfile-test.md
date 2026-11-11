Fixed

- **The Electrum accept-retry test now runs on macOS and checks the retry.** It counted descriptors through `/proc/self/fd`, which macOS lacks, so it set the limit below the open count and failed with "Too many open files". On every platform, reaping the helper process freed two descriptors before `accept` ran, so `accept` never saw EMFILE and the test passed with the retry removed.
