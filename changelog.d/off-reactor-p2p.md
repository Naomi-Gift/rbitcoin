Fixed

- **Tip-follow compact serve does not run on a Tokio worker.** A compact `getdata` and an announced wtxid served from the tip block reconstruct on the blocking pool, under the same permit as block decode. Other peers keep reading while that reconstruct runs.
- **A full tip-accept queue parks the sender.** The session waits until the lane takes a job, instead of spinning a Tokio worker.
