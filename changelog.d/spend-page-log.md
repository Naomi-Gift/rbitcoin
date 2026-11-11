Fixed

- **A torn historical spend page below `spend_durable` stays spent while `spend.pages.commit` exists.**
  Confirm records every published slot, including the short tail, in a directory
  and does not write that `spent.body` slot. Schema stays 27. A binary from
  before this log opens the datadir and reads `spent.body`, so a slot that is
  not folded yet looks unspent. The spend checkpoint rewrites `spend.pages`
  to the images still named before it advances the marker, and the next
  checkpoint folds a generation the previous one left
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
  different slot on that page. A different page whose slots are all
  still best-chain stays named when this checkpoint rejects one page,
  including beside a zero page that is kept.
  A missing or short image file beside a
  commit that names bytes refuses to open, including a record past the
  committed length. A committed image shorter than the page does not answer
  for a later slot, and the next compose copies that body tail instead of
  padding zeros. Startup replay does not wait on the edge cap when no sync
  thread is bound, and a confirm batch does not wait while it holds the utxo
  view. The next confirm insert waits at 2,000,000 open edges. A 1 GiB log
  asks for one extra checkpoint, not another on every later batch. Neither is a
  knob. Once the commit names a generation, a failed rename of that file still
  keeps the publish, and the next checkpoint writes the next generation
  instead of replacing the file the commit already names. A keep error
  while filtering notes taken during compose leaves those notes in the
  open directory, and the next checkpoint still publishes them. Startup
  replay checkpoints a chunk once the open directory reaches 2,000,000
  edges, and the next checkpoint folds that generation. It does not wait
  for a sync thread.
