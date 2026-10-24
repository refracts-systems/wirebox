# ADR-0008 — Studio packet controller and v1 session file

- Date: 23 October 2026
- Status: Accepted for the initial Phase 4 implementation. G4 acceptance is recorded separately.

## Context

The Phase 1 GPUI only displayed the transport probe. The Phase 3 kernel already has a thread-bound Network, bounded execution, copied packet/field/trace versions, and configuration frozen after a workload. We need a packet-inspection workflow without moving protocol logic into UI or expanding the ABI for rendering.

## Decision

The Rust controller in `crates/wirebox-studio/src/model.rs` owns one Network and builds a Frame containing summary, trace, packet/field data, and transmissions. Commands run on the owner thread. A snapshot is published after all copies complete; no concurrent mutation occurs during collection. This is host-layer consistency under the current thread contract, not a new cross-thread atomic-snapshot API. GPUI renders the Frame without per-byte FFI.

Fast run pace uses batches of four events with a 40 ms event-loop delay; slow pace uses one event per second for inspection. Step uses budget one. Changing pace pauses the run so no continuation uses the old policy. Pause stops the next batch. An epoch token cancels old asynchronous continuations on pause, reset, scenario change, or branch change. The simulation clock remains virtual and does not follow animation delay. The initial horizon is five seconds; reaching the horizon with a pending outcome is not success.

Configuration changes build a new Network from the fixture and copied Config. The original Network is not mutated. One baseline result, configuration, and command are kept for comparison. This reruns with empty caches; it does not fork active state. Initial editor scope is four L3 port addresses, ping destination/TTL, R1 route `198.51.100.2/32` via eth1, PC2 port, and bitrate/propagation for each link. Topology and host routes remain fixture data.

Schema-1 `.wb.json` stores model/fixture, effective configuration, command budgets, the complete copied result, and optionally one baseline result. Handle registries are not stored. Load builds a new Network, replays the bounded command log, and compares the full report with the stored result before replacing the active session. The comparison baseline is also re-verified. Schema, model, configuration, result, or limit mismatches are rejected without replacing the active session. Limits are 2 MiB per file, 1024 commands, and budget 1..8 per command.

## Consequences and alternatives

The kernel ABI remains v1; no engine thread or dependency is added. Studio uses the workspace-pinned `serde_json`. The model/controller can be tested without a window; the Phase 3 headless oracle remains separate.

Snapshots copy data and the initial renderer is not virtualised. Batch count is not a latency guarantee; the 10,000-row/p95 and 30-minute-session targets still require P4-07 measurement. Current engine limits remain.

Persistence is tied to the model/fixture and reproducible result; an engine change can reject an old result. Storing bytes alone as a checkpoint would lose cache, timers, queue, and PRNG. Active-state restore and cross-version compatibility remain Phase 5.

The initial ASCII form supports replacement typing, Backspace, Ctrl+A/Ctrl+V, Enter, and Apply. Free topology editing, arbitrary routes, IME, and general text editing are outside this form's scope.

Verification and limits: [Phase 4 evidence](../evidence/phase-4/README.md). Guide: [Studio](../wirebox/studio.md).
