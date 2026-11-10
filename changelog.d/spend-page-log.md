Fixed

- **A torn historical spend page below `spend_durable` stays spent while `spend.pages.commit` exists.**
  Confirm records every published slot, including the short tail, in a directory
  and does not write that `spent.body` slot. Schema 27: a binary that does not
  read the log refuses the datadir. The spend checkpoint appends the page image
  before it advances
  the marker, and the next checkpoint folds a generation the previous one left
  quiet. A retry of a failed compose writes the frozen pages and every note
  taken after that freeze into the replacement commit, including a newer slot
  on a page the previous commit sealed and a page a disconnect put back in
  the open directory. A page whose new notes are no longer best-chain still
  names that sealed image. Folding the first page does not rewrite the file header, and it does
  not publish a shorter `spent.body` end while confirm is still appending. The
  body copy does not hold the utxo view. A checkpoint below a loaded commit height
  filters that image before fold, even when the held bit was never written.
  Unlinking a finished generation takes the same lock as a fold hold. A spend
  noted while that filter walks the image stays out of the checkpoint height
  already sampled, so a later fold does not seal it at the lower tip. A
  rejected spend that `spent.body` already holds is zeroed in the next image
  instead of being copied back. The old commit stays on disk until that
  replacement is renamed. It is removed when the body does not hold the
  rejected spend, unless a zero already stored on that page is still hiding
  a body spend.
  A page that still has a best-chain slot keeps that image across a failed
  body read, and a zero image already stored is folded after a later
  disconnect instead of being deleted. A zero already stored on that page
  stays in the next image when a new note lands on it, when the page
  still has a best-chain slot, and when this checkpoint rejects a
  different slot on that page.
  A missing or short `spend.pages` file beside a
  commit that names bytes refuses to open. The next confirm insert waits at
  2,000,000 open edges. A 1 GiB log asks for one extra checkpoint. Neither is a
  knob.
