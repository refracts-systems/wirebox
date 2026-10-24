# Hardening Studio — October 24, 2026

Status: advanced implementation, **G4 not yet passed**. The user chooses to defer human acceptance and have the runbook prepared.

Native retry update: [auto click successful](native-retry.md), including draft/Apply, S02 fix, invalid gateway rejection and per link timings. The latest build includes the latest fixes as well as a link label patch. The input interrupt log and old binary below record previous runs, replaced by repeat results for the scope under test.

## Actual results

- Draft configuration with Apply/Discard; static routes per L3 device, administrative port down and timings per link. Apply failed to maintain active session. Writer schema v2, reader v1/v2 with rerun verification.
- Regression uses actual v1 session files from previous evidence. Tests for shared subnet+gateway, LPM route, route removal, invalid/duplicate prefix, port down, timing and roundtrip v2 passed.
- Kernel journal maximum 10,000 rows. The synthetic test wrote 10,257 rows, read in batches of 127, checked ordinals 258–10,257, byte/version remained correct and the scheduler was not affected. This is not a 10,000 event protocol workload.
- Reading ring seek directly to the cursor; The timeline uses 200 rows per page. There are no p95 input-to-visible results yet.
- `./scripts/verify.ps1`: [final log](verification-final.log), 38 Zig tests per Debug/ReleaseSafe mode, 24 Rust unit tests (4 headless + 12 wrappers + 8 Studio), 3 compile-fail doctests, C layout, fmt, Clippy, headless acceptance and GPUI build passed.
- After repairing the draft label, wrapping the form, resetting the filter page and validating the Config key, `cargo test --locked -p wirebox-studio` [passed 8 tests](studio-final.log); `cargo clippy --locked --workspace --all-targets -- -D warnings` [pass](clippy-final.log). Previous native builds did not include the last four fixes until the executable was rebuilt.
- Initial run stuck Clippy type_complexity ([log](verification.log)); fixed with alias record route. The future-incompatibility dependency warning `proc-macro-error2` remains.

## Native and acceptance is still open

The executable resulting from full verification was successfully opened via computer-use. Screenshot shows Routes, Port down, Per-link timing, Apply/Discard draft and pagination. Auto input was rejected twice because user input was detected; no claims new pass editor interaction walkthrough. The window is left for the user, not forced to close.

[Runbook U1–U5](user-acceptance.md) ready, human output not yet populated. Rerun the build after the window is closed, then do a new editor walkthrough.

Target of four nodes/10,000 rows, selection/pause p95 <100 ms, working set ≤1 GiB, and scripted duration of 30 minutes **not yet measured**. Script [measure-studio-memory.ps1](../../../scripts/measure-studio-memory.ps1) records working set/private bytes/Responding every second; The scope does not measure input latency or prove scripted workloads. Use the application's own PID, for example `./scripts/measure-studio-memory.ps1 -StudioProcessId 1234 -Seconds 1800`, while running the recorded runbook/workload. The script does not send UI input.

Engine workload is still limited to 128 package versions and 256 transmissions per run; A journal capacity of 10,000 does not automatically make one ping fixture produce 10,000 rows. Proving performance requires workloads and row→packet relationships that can be accounted for, not doubling rows that are claimed to be real events.

P4-04 still requires a new walkthrough, P4-06 is postponed by users, P4-07 is still measuring acceptance. G4 Decision: **not yet passed**. Old evidence is not changed. The last source hashes run was recorded separately from the October 23 baseline.

5 second memory smoke monitor successful after OrderedDictionary value aggregation fix; [results](memory-smoke.json) recorded a peak of 64.16 MiB. The first run produces a null peak ([initial result](memory-smoke-initial.json)), so it is not considered a valid budget measurement. This short smoke does not allow for a 30 minute soak.

Update after complete run: [G4 technical agent passed](../phase-4-acceptance/README.md) on October 24 in scope ADR-0010; The pending status in this record is the baseline before the acceptance.
