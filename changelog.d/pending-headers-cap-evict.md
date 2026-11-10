Fixed

- **A full pending-header map keeps a header the store rejects.** A
  connecting batch makes room only after its proof of work meets the chain
  limit and the store accepts it. Headers that do not connect are dropped
  first, then forks with less work than the tip. A heavier fork stays.
