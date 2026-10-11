Fixed

- **IBD block accept no longer scans the body queue under the lock lookup uses.** Hash presence is a map lookup, and lookup's decoded stash is its own lock, so a presence check cannot stall confirm lookup.
