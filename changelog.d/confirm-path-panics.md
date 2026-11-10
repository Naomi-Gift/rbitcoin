Fixed

- **A poisoned confirm-feed wait returns the guard.** The lookup thread stays up. The success arm is still `wait_timeout`'s guard.
- **A missing queued `tx.head` batch drains as empty.** `Some` still submits that same vec.
