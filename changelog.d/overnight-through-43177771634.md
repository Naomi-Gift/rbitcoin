Fixed

- **Missed mutants through run 43177771634 are caught or removed.** Journeys fail when those operators change a session result. Compares and log counters no input can observe are gone.
- **`ibd: sizes` no longer prints prep-ahead pin counters.** Those counters had no publisher, so the line showed zero. The retain window still uses the live pin snapshot.
