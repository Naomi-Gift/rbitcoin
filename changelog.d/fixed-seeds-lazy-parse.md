Changed

- **Start no longer reads the whole fixed-seed table when it doesn't need
  to.** When the saved peer book already has a row for every net the node
  can dial, the table is not read at all. Otherwise Tor and I2P seeds for
  nets the node won't use are skipped before they are parsed. The seeds
  added are the same as before.
