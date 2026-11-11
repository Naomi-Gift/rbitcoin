Fixed

- **`gettxspendingprevout` rejects a `vout` above `u32::MAX`.** The value
  was cast to 32 bits, so `4294967296` looked up and echoed output 0. It
  is now `-8` `vout out of range`.
