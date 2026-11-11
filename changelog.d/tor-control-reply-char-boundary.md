Fixed

- **A Tor control reply with a multi-byte character at byte 3 or 4 no
  longer panics.** The reader sliced the line at those byte indexes, so
  the start, or the onion keeper after a reconnect, panicked. Such a line
  is now an `unexpected reply` error.
