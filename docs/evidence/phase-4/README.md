# Evidence of initial Packet Studio integration — Phase 4

October 23, 2026, Windows x64 MSVC; Zig 0.17.0, Rust 1.99.1, GPUI 0.2.2. Git baseline before changes: `1b4139a`; the implementation of the working tree is recorded in [source-manifest.json](source-manifest.json). **P4-01/02/03/05 completed for noted scope; G4/MVP not yet fully passed.** Scope and results: [Studio](../../wirebox/studio.md), [ADR-0008](../../adr/0008-studio-network-controller.md), [learning notes](../../learning/phase_4/phase_4_controller_snapshot.md).

## Locally verified results

`cargo test --locked -p wirebox-studio -- --nocapture`: [studio-tests.log](studio-tests.log), **6 tests passed**.

| Scenario | Outcomes | Event scheduler | Row trace | Package version |
|---|---|---:|---:|---:|
| S01 | echo_reply | 12 | 63 | 16 |
| S02 | unreachable | 8 | 40 | 8 |
| S03 | time_exceeded | 8 | 40 | 8 |
| S04 | arp_timeout | 12 | 44 | 9 |
| S05 | invalid_packet | 0 | 2 | 1 |

Controllers were compared between budget 8 and budget 1. Packet snapshots, fields/bits, causal traces, transmissions and entity IDs were also compared with S01–S05 Phase 3 artifacts, with explicit JSON field name normalization. Historical artifacts are not changed. This is a regression against the Wirebox baseline; not an independent vendor comparison. Headless Oracle checksum/field/timing is still executed by the workspace verifier.

Another test proves:

- Route alias S02 restored on new Network: unreachable becomes echo_reply, static route selected, source report unchanged.
- Partial and completed sessions are reloaded with identical reports; configuration, bytes, fields, traces and transmissions are also checked.
- Unsupported schema/model, fake reports, fake baselines, invalid budgets, invalid configurations and files that are too large are rejected. Resource sessions remain available.
- TTL 64→63 before/after relationships and checksum changes are tested against bytes and field maps; the four-bit field includes only the exact bytes.
- A 1 bit/sec link results in an ARP timeout of 3 seconds while frames are still scheduled past the 5 second horizon. The controller maintains the failure outcome and does not convert it to success because the run completes on the horizon. Save/load is identical.

`./scripts/verify.ps1`: [verification.log](verification.log), passed on final source including pacing options: **37 Zig results per Debug and ReleaseSafe mode**, **22 Rust unit tests including 6 controllers**, **3 compile-fail doctest**, C layout, format, Clippy workspace, GPUI build and headless acceptance. [Clippy targeted](clippy-final.log) and [Studio build](studio-build-final.log) are also logged. The transitive warning `proc-macro-error2 2.0.1` remains. [Clean export](clean-export.log) checks the build from a source copy with the dependency/compiler cache; this is not a cold build or remote CI.

## Native window interactions

The `Wirebox · Packet Studio` window was opened from the local build and visually inspected. F10 produces one event scheduler at 5,800 ns with 10 row traces and two packet versions. The run then hits echo_reply, 12 scheduler events, 63 row traces, 16 package versions, zero pending. The snapshot appears to contain the actual Ethernet/ARP frame along with fields and hex, not the old UTF-8 probes. Note: [ui-interaction.log](ui-interaction.log).

Test using Computer Use skills via `@oai/sky`. The test was suspended when the user requested not to take focus, then **resumed after the user explicitly requested desktop takeover**. Structured summary and hash of tested executables: [native-qa.json](native-qa.json).

| Workflow | Actual native results | Evidence |
|---|---|---|
| U1 | S01 completed echo_reply, 12 events, 63 traces, 16 package versions | Batch log and screenshot inspector |
| U2 | Filter R1 shows row drop `no_route` and `icmp_error` on S02 | [no-route.jpg](no-route.jpg) |
| U3 | Click byte 8 ↔ `ipv4.ttl`; click field highlights bytes. Before TTL 64/checksum 0x65dd, After TTL 63/checksum 0x66dd | [before](ttl-before.jpg), [after](ttl-after.jpg) |
| U4 | `10.0.9.2` off-link gateway rejected with old results intact; `10.0.1.2` creates a new branch and echo_reply, baseline no_route remains visible | [route-repaired.jpg](route-repaired.jpg), [native-session.wb.json](native-session.wb.json) |
| U5 | Save via native dialog → Ctrl+Q → new application → Open. Reports, bytes, fields and baselines appear again in paused state | [session-reopened.jpg](session-reopened.jpg) |
| Invalid file input | Reports whose outcomes are changed are rejected; previous sessions and bytes remain available | [invalid-session-rejected.jpg](invalid-session-rejected.jpg) |
| Pause/Step/Resume | Slow 1 event/second: pause at 7 events/37 rows/2 pending and t=40600 ns, stable at next check; Step becomes 8 events, Resume complete echo_reply | [paused-pending.jpg](paused-pending.jpg), log |
| Active reset | S04 starts slow then resets after 6 events; t/event/trace/packet returns zero and the old continuation does not change the new owner | [reset-active-run.jpg](reset-active-run.jpg), log |
| Config/viewport | Toggle port PC2 and IP PC1 `10.0.0.3/24` produces echo_reply; pan/zoom and L2 filter successful | [address-branch.jpg](address-branch.jpg) |
| Invalid package | S05 maintains two raw bytes `45 00`, field decode InvalidPacket and outcome truncated_ipv4; reset clears selection | [invalid-packet.jpg](invalid-packet.jpg) |

Ctrl+Q produces `window_closed` and `network_destroyed` in the log. The final test run is closed that way. The initial slow pacing of 250 ms is too fast for intercall inspection/automation steps; final slow option is 1 second/event, default fast remains 40 ms/four events. Pacing does not change the virtual clock or results. The initial Save file is moved byte-identical from root to `native-session.wb.json` after the reopen test; SHA-256 is recorded in native QA. Negative input files are in Git's ignored `artifacts/`.

This walkthrough is carried out by an agent with a runbook, not usability validation by a human engineer or measurement of user time/benefits. Not all combinations of input fields or dialog failures have been tested; P4-07 targets remain separate.

## Test failure and claim limits

[verification-initial.log](verification-initial.log) records an incorrect test horizon assertion expecting pending. The engine actually recorded an ARP timeout of 3 seconds; The test expectation is corrected to outcome 4 with frames still pending, without changes to the Zig algorithm. The previous first test was also hit by a limit of four Network owners during parallel testing; multi-owner tests are now serialized without increasing production capacity.

[verification-exe-locked.log](verification-exe-locked.log) records verifiers that have passed test/lint/acceptance but failed to overwrite Studio executables that are still open. Once the test process is stopped, the complete verifier passes. This is differentiated from a compile or engine failure.

Implementation has not met the target of 10,000 rows/p95 <100 ms and 30 minute sessions/working set ≤1 GiB on the v1 contract. The initial renderer and Frame are limited to 1024 rows; The topology is still a fixture with a scroll viewport/card scale, the inspector is not a snapshot of the entire cache/state, and the route editor is limited to alias R1. G4 remains open until U1–U5 interactions and mandatory measurements are available. There are no claims of active checkpoints, PT/vendor conformance, real adapters, benchmark releases, or remote CI passes.

Private Cognitive Coverage materials created before this UI/controller change are stale and need to be refreshed locally. The folder remains ignored by Git.
