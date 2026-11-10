Fixed

- **A killed first start no longer bricks the datadir.** `open_or_create`
  replaces a partial create that has no chain payload, including on a split
  cold directory, and open drops a leftover scripthash OA head. A store that
  already holds chain bytes stays, and an unreadable table is not replaced.
