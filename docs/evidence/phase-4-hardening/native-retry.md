# Studio auto click repetition

October 24, 2026. Method: computer-use skills, `node_repl` + `@oai/sky`, click based on the latest screenshot of each step. This is an agent walkthrough, not a human reception or latency benchmark.

## Results observed

1. Closing the old instance with Ctrl+Q, rebuilding and opening the executable that includes the last fix worked. [Build](build-native-retry.log).
2. Click S02 → Run: `unreachable / no_route`, 8 event schedulers, 40 rows, 8 package versions.
3. Click Add missing R1 route → Save to draft: old results are still available and Apply draft marked `*`.
4. Click Apply draft → Run: `echo_reply`, 12 events, 63 rows, 16 package versions. The fixed comparison bar shows the `unreachable / no_route` baseline.
5. Edit the gateway to `10.0.9.2`, Save to draft → Apply: rejected with `kernel: InvalidArgument`; echo reply results, 63 rows, 16 versions and baseline remain available. Discard draft was successful.
6. Per-link timing, fill in `L1 10000000 5000; L3 100000000 2000`, save to draft then Apply and Run. Echo reply still achieved; detailed transmission time in row 22 changes from 29000 to 170600 ns.
7. Found a label topology bug: the engine uses override timing, but the label still takes the global bitrate. Fixed so that labels read effective bitrate/delay per link.
8. After patch: 8 Studio tests, Clippy and build passed ([test](native-retry-tests.log), [Clippy](native-retry-clippy.log), [build](native-retry-build-final.log)). Application reopened, override timing repeated on S01. Label shows L1 `10000000 bps / 5000 ns`, L2 default `100000000 bps / 1000 ns`, L3 `100000000 bps / 2000 ns`. Run completed echo reply, 12 events, 63 rows, 16 versions.

The last window is left open on the S01 results with the override timing; there are no drafts yet to be implemented. No commit/push.

## Limits of evidence

Screenshots are viewed directly in the tool, not exported as artifact files. This record summarizes observations; separate manifest logs source/binary hashes and build/test logs. UI parser reading is not used as an automatic oracle.

Free route editor, multiple address changes at once, general port down, save/load v2 and pagination across more than one page have not been retested via clicks in this run; The associated test controller remains separate evidence. P4-04 has not been marked completely complete. Human testing remains pending per user instructions. The p95/soak G4 target has not been measured.
