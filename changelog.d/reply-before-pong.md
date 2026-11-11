Fixed

- **A later ping's pong no longer overtakes the reply to an earlier message.**
  The peer session queues that pong from the normal ping handler, after
  work for the earlier message has queued its reply. The writer sends the
  pong after replies already waiting on that peer, including a `tx` for an
  earlier `getdata`, a `getblocktxn` for a compact block that is missing
  transactions, and a `getdata` after `headers` from a high-bandwidth peer.
