Fixed

- **Nightly mutants gets past its workspace baseline.** Parallel integrity tests named the store directory with the clock alone and deleted that path first. Two tests in the same sample both created `meta`, and the baseline exited 101 before any mutant. A repeated sample now gets its own directory.
- **A failed mutants baseline names the test.** The job log prints the failure lines and the last 80 lines of the baseline log. The full log stays in the artifact.
