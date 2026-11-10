Fixed

- **A torn historical spend page below `spend_durable` stays spent while `spend.pages.commit` exists.**
  Confirm records a full historical page in a directory and does not write
  that `spent.body` page. The live tail is an 8-byte slot write. This is not a
  schema bump. The spend checkpoint appends the page image before it advances
  the marker, and the next checkpoint folds a generation the previous one left
  quiet. Folding the first page does not rewrite the file header. The body copy
  does not hold the utxo view. A missing or short `spend.pages` file beside a
  commit that names bytes refuses to open. The next confirm insert waits at
  2,000,000 open edges. A 1 GiB log asks for one extra checkpoint. Neither is a
  knob.
