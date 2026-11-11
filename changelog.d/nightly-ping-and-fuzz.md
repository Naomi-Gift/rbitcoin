Fixed

- **Functional tests see a ping-timeout line as the peer drops.** The node
  appends that line to `debug.log` and flushes it before the log call
  returns. The harness used to copy the same bytes off a pipe, and
  `p2p_ping.py` reads the file with no wait.
- **The fuzz nightly does not apt-install clang when it is already on
  PATH.** A DNS failure for `azure.archive.ubuntu.com` was failing every
  matrix job before a target compiled. A missing clang still installs, and
  retries through `archive.ubuntu.com` when that install fails.
