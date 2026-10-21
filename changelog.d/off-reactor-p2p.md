Fixed

- **Tip-follow compact serve does not run on a Tokio worker.** A compact `getdata` and an announced wtxid served from the tip block reconstruct on the blocking pool, under the same permit as block decode. Other peers keep reading while that reconstruct runs.
- **A full tip-accept queue parks the sender.** The session waits until a slot is free, instead of spinning a Tokio worker. Every session that found the queue full gets a turn when a slot opens.
- **Inbound compact reconstruct does not run on a Tokio worker.** A `cmpctblock` rebuilds on the blocking pool, under the same permit as block decode. The session keeps its follow state and only applies the result.
- **Header store work does not run on a Tokio worker.** A `headers` batch is classified and persisted on the blocking pool, and `getheaders` / `getblocks` replies are built there, under the header-decode permit. The session applies the result.
- **A short headers reply stays on the writer.** Encoding eight headers or fewer does not hop to the blocking pool. A block and a 2,000-header reply still do.
- **A script batch wakes one worker per chunk.** Idle script workers stay parked once those chunks are claimed. A larger batch still runs on the whole pool.
- **Block getdata does not look up headers on a Tokio worker.** A missing header and the block body are both decided on the blocking pool. The session only queues the block or `notfound`.
- **A compact tip announcement does not reconstruct on a Tokio worker.** The block body is loaded on the blocking pool. A missing body still goes out as headers. An unsolicited compact block persists its header on that same pool.
