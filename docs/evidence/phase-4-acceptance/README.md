# Phase 4 technical acceptance by agent

24 October 2026. Operator: agent via native computer-use. The user requests all testing to be carried out by the agent; no human understanding or usability is declared verified. Measurement methods and limits: [ADR-0010](../../adr/0010-agent-acceptance-and-studio-measurement.md).

Final status: **G4 technical agent passed**, within scope ADR-0010. [Final metrics](metrics.json) has pass=true. This replaces the pending G4 status in the previous evidence run; human usability remains unknown.

## Editor walkthrough and U1–U5

| Tasks | Observed results | Evidence |
|---|---|---|
| U1 healthy | S01 and edited subnet produce echo reply, 12 events, 63 rows, 16 versions | Native screenshot in tool; initial session log [native-ui.log](native-ui.log) |
| U2 diagnostic route | S02 produces unreachable/no_route on R1 | [Previous native retry](../phase-4-hardening/native-retry.md) with the same kernel/controller; next change added harness/pagination |
| U3 byte forwarding | Page 2, row 53: v13→v14, TTL byte 8 from 0x40 to 0x3f, checksum byte 10 from 0x65 to 0x66; byte→field shows ipv4.ttl | Native inspector Before/After and logs `trace:53`, `byte:8=3f` |
| U4 configuration | PC1 10.2.0.2/24 and R1 eth0 10.2.0.1/24 plus explicit route table staged together, Apply successful, ping echo reply | [Saved v2 session via GUI](native-session-v2.wb.json) |
| U5 persistence | Save native → Ctrl+Q → new process → Open native returns subnet, route, baseline, 63 rows and 16 versions; load states report verified | Same session files and native observations |
| Common port | After load, `SW1:p1` administrative down → Apply → Run: ARP timeout, 6 events, 18 rows, 4 versions; baseline echo reply persists | Native walkthrough agent; not interpreted as physical carrier simulation |
| Pagination | Next in history 63 rows move page 1→2; row 53 can be selected and the bytes inspected | Native inspector; stress projection uses 200 pages ×50 rows |
| Invalid config | Gateway off-link is rejected and active throughput is maintained | [Native retry](../phase-4-hardening/native-retry.md) and controller regression |

Screenshots are checked directly in the tool, not exported as files. Therefore this record does not claim to provide saved screenshots for each action. The session files as well as the command log support the mentioned observations; test controller assesses bytes/reports automatically.

## Automated testing

[Full verify](verification.log) before pagination optimization passes. After optimization: [CoreOnly](core-final.log), [9 test Studio](studio-final.log), [Clippy workspace](clippy-final.log), [build pagination](pagination-build-final.log) passed. Kernel has 38 tests per Debug/ReleaseSafe mode, Rust has a total of 25 unit tests and 3 compile-fail doctests. Remote CI is not declared passed.

The history test examines 10,000 rows with 2,560 package versions; all version/transmission references as well as packet identity remain valid after rebasing. History comes from 160 independent kernel runs: 10,080 rows created, initial 80 rows truncated with indicators. The packet/tx limit per engine remains in effect.

## Performance and stability methods

`WIREBOX_ACCEPTANCE=1` mode uses this historical projection as well as one active kernel that is stepped, reset upon completion, and saved/loaded every ten pulses. The driver is executed on the GPUI owner thread. It selects a cross-page row and calls pause for 1,800 pulses spaced one second apart. Workload identity/timing and minimum samples are frozen in ADR.

Scripted time starts before event/FFI/snapshot processing and ends at the GPUI callback after rendering. Native selection/Space also has instrumentation and is reported separately. Limit: this is handler-to-render; OS input queue and compositor presentation are not measured. P95 statistics use nearest rank, minimum 100 samples per type, all outliers are still included.

An initial run log of 200 rows/page ([log](soak-visible-ui.log)) shows some responses >100 ms. Pagination is reduced to 50 without decreasing the number of stored rows. The initial hidden process produces no frames and is terminated; [initial log](soak-hidden-initial.log) is not used as UI evidence. The build that was locked by the second process is also saved ([log](pagination-build.log)).

Final run: [UI log](final-ui.log), [binary identity](final-environment.json), [environment](visible-environment.json). The initial environment metadata has the initial PID/build run; final run identity must be retrieved from final-environment, not swapped silently.

The summary is calculated with `node scripts/summarize-studio-acceptance.mjs --require-pass` after the monitor completes. `metrics.json` during the run is not the final result. The memory monitor wrote `final-memory.json` after 1,830 seconds; pass conditions require at least 1,800 seconds, no crashes/hangs, all samples Responding, working set ≤1 GiB, last-pulse render, and p95 <100 ms per type. Gate decisions do not extend to vendor conformance, active checkpoints, or human user studies.

## Run closure

- 1800 completed pulses and 1799 render pulses recorded; 900 scripted selection samples and 900 pauses. P95 selection 65,093 ms, pause 73,062 ms. The maximum selection of 101,056 ms is still included, not trimmed.
- Monitor 1830.14 seconds; all samples responded, without failure. Peak working set sampled was 68.15 MiB. [Raw memory](final-memory.json), [process peak](process-peak.json), [complete screenshot](soak-complete.jpg).
- 14 native selection samples and 1 native pause recorded separately; as long as every native click is not all attributed to the agent. These samples are diagnostic, not basic p95 scripted.
- Repeated save/load of the script is serialization/rerun in memory; dialogs and discs were tested separately on the U5.
- The summary CLI parser briefly read --require-pass as a directory; the parser is fixed and then the same command is passed without changing the raw data.
- Closing command: node scripts/summarize-studio-acceptance.mjs --require-pass. Historical evidence of 22 previous Phase 4 files still matches the original hash.

Graduation includes fixture editor, diagnosis/change/comparison, persistence v2, 10,000 row history on multi-run projections and handler-to-render response on this machine. Does not state p95 OS-input-to-photon, human usability, independent topology, active checkpoint restore or vendor equivalence. Phase 5 remains the next job.
