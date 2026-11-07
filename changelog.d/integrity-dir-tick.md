Fixed

- **Nightly mutants gets past its workspace baseline.** Test datadirs named from the process id and the clock collided when two calls shared a sample, and `create_new` failed. Each call now takes a path from one counter.
- **A failed mutants baseline names the test.** The job log prints the failure lines and the last 80 lines of the baseline log, and still exits with cargo's status when that log has no failure line. The full log stays in the artifact.
