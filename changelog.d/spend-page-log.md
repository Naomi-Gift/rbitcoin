Fixed

- **A torn historical spend page below `spend_durable` stays spent while `spend.pages.commit` exists.**
  Confirm records the slot in a directory and does not read-modify-write that
  `spent.body` page. The spend checkpoint appends the page image before it
  advances the marker, and the next checkpoint folds a generation the previous
  one left quiet. The next confirm insert waits at 2,000,000 open edges. A
  1 GiB log asks for one extra checkpoint. Neither is a knob.
