Changed

- **Default tests spend less time on the same pins.** A `cfcheckpt` reply
  is checked at the first checkpoint height. Each header-rewind story uses
  one chain past the proved cap. Pending-header cap tests fill the map
  without a clone on every header, on a four-block tip.
