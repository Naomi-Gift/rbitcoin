Changed

- **The milestone header path drops hashes the confirmed tip has passed.**
  Header intake still records each on-path header, and a missing height in
  the range still held is not an ancestor of the milestone. Each load pack
  drops heights at or below the store tip. When nothing remains, lookups
  use the confirmed chain again. Header-tip work is unchanged.
