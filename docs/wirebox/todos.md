# Wyrlex Todos per phase

Updated October 24, 2026. Sequence and milestones: [roadmap](roadmap.md). Scenario and scorecard: [pilot](pilot.md). Targets on [contract v1](phase-0-contract.md) remain in effect; Actual results must be linked per gate.

`[x]` means artifact/result available; `[ ]` means not yet proven complete. G0/G1 local pass on October 20, 2026; [evidence and scope limits](../evidence/phase-0-1/README.md). G2 passes for graph/scheduler/counter model; [evidence G2](../evidence/phase-2/README.md). G3 passed local Windows x64 debug on October 21, 2026 for a subset of headless packages; [evidence G3](../evidence/phase-3/README.md). G4 technical agent graduated on October 24 within the scope of ADR-0010. Local Phase 5 checkpoint/replay evidence exists, while the G5 Linux lab gate and G6–G9 remain untested; G10 has a gate per epic. Real access inventory is not considered verified by G0.

## Phase 0 — Evidence and readiness threshold

- [x] P0-01 — Save vision, RE sources, corrections, and proof limits in [docs](../README.md).
- [x] P0-02 — Install and check Zig 0.17.0, Rust 1.99.1, and compiler smoke checks; note in [technology](technology.md).
- [x] P0-03 — Note pilot target users: Linux lab, MikroTik RouterOS, ZTE OLT, Huawei ONU and VSOL.
- [x] P0-04 — Note model/firmware/PON mode as well as unknown in [inventory](../../fixtures/inventory/pilot-v1.json) and [sanitized excerpt](../../fixtures/inventory/zte-c320-user-report.txt). Scope [ADR-0005](../adr/0005-pilot-scope.md): access/model not yet available must be equipped before G6/G8, not considered tested.
- [x] P0-05 — Freeze MVP scope, static IPv4 intent subset, as well as unsupported on [contract v1](phase-0-contract.md).
- [x] P0-06 — Freeze four-node/five-case fixture, independent byte/checksum oracle, comparison method, as well as scorecard in [contract v1](phase-0-contract.md).
- [x] P0-07 — Save [reference engine](../evidence/phase-0-1/environment.json), protocol artifact, source manifest, and decision [G0](../evidence/phase-0-1/README.md).

Gate G0: scope, minimum initial lab inventory, scenarios, and expected results can be reviewed before implementation is expanded. ISP inventory can be resumed for G8 without holding the kernel.

## Phase 1 — Build and prove the stack

- [x] P1-01 — Zig kernel structure, headless runner, Rust host, and [ownership limit](ffi.md) available; state queue probe is simply mutated Zig.
- [x] P1-02 — [Cargo build script](../../crates/wirebox-kernel/build.rs) builds Zig 0.17.0 with Rust 1.99.1 and Windows MSVC; dependency locked Cargo.lock.
- [x] P1-03 — Create/destroy as well as batch copy proven; [ABI v1 contract](ffi.md) defines ownership/lifetime/error/thread/layout.
- [x] P1-04 — GPUI 0.2.2 displays window, selected node/link, hex, input, and shutdown; [runtime results and screenshots](../evidence/phase-0-1/README.md).
- [x] P1-05 — Zig/Rust tests check stale/null/error/capacity, copy/lifetime, as well as thread-bound sessions; close UI destroys session; [result](../evidence/phase-0-1/verification.log).
- [x] P1-06 — [Build guide](../../README.md), [local verifier](../../scripts/verify.ps1), new source export with cache, as well as CI workflow available. [Local run pass](../evidence/phase-0-1/README.md); remote CI has not been started.

Gate G1: build and FFI/GPUI smoke check can be repeated. Just installing the compiler does not meet this gate.

## Phase 2 — Graph, state, scheduler, and basic traces

- [x] P2-01 — [Graph Zig](../../kernel/src/graph.zig): monotonic device/port/link ID, relation validation, cascade delete; without UI coordinates. [Test](../../kernel/src/runtime_tests.zig).
- [x] P2-02 — [ADR-0006](../adr/0006-phase-2-runtime.md) completes ticks ns, overflow, tie-break, empty step/horizon, and progress budget/error; implementation [runtime](runtime.md).
- [x] P2-03 — Binary heap, registry timer, eager cancel, run bounded, lifecycle entity/timer in [runtime Zig](../../kernel/src/runtime.zig); [evidence](../evidence/phase-2/README.md).
- [x] P2-04 — SplitMix64-v1 with explicit seed/state and golden tests; [acceptance 5 seeds](../evidence/phase-2/acceptance.json). The clock does not read the wall clock/GUI.
- [x] P2-05 — Queue, ring journal, and split view projection; event/cause ID and execution ordinal; retention 0/1/3/256 does not change state. [Contract](runtime.md).
- [x] P2-06 — Same-time, cancel, stale/deleted owner, empty horizon, bounded periodic timer, overflow, and ABI ownership tested; [full results](../evidence/phase-2/verification.log).

Gate G2: invariant scheduler passes and the sequence can be repeated; trace capacity/retention does not change the simulation results.

## Phase 3 — PoC packet and headless byte tracker

- [x] P3-01 — [Codec Zig](../../kernel/src/codec.zig), immutable version copy, protocol view byte/bit, bounds/checksum, and ABI copy; [contract](packets.md).
- [x] P3-02 — Frame arrival uses heap phase 2, serialization per egress + propagation checked; [link limit](../adr/0007-packet-model.md) and oracle timing are separate.
- [x] P3-03 — [Network Zig](../../kernel/src/network.zig): Ethernet, MAC learning/flood/unicast, ARP cache/lookup/reply/buffer/retry; multiport flooding tested.
- [x] P3-04 — IPv4, connected/static LPM, TTL/checksum, echo and ICMP 3/0 and 11/0; [acceptance](../evidence/phase-3/acceptance.json).
- [x] P3-05 — Trace route/cache/drop, byte version, packet/transmission/event IDs; [walkthrough actual bytes](../evidence/phase-3/byte-walkthrough.md).
- [x] P3-06 — S01–S05 and healthy static alias pass; 36 runs for batch/step/repeat/retention, golden headers as well as separate Rust checksum/field/quote/timing oracles. [Evidence G3](../evidence/phase-3/README.md); Linux capture remains G5.
- [x] P3-07 — Truncation, checksum, unsupported options/fragment/protocol/ICMP/VLAN/MTU, invalid MAC, ICMP error suppression, and resource/ABI failure; [verification result](../evidence/phase-3/verification.log).

Gate G3 / PoC: success and failure flows can be described down to bytes via a headless runner; requires no UI, hardware, or `.pkt`.

## Phase 4 — Workable MVP Studio

- [x] P4-01 — Rust/GPUI connects to Network via controller and Frame consistent copy on owner thread; there is no FFI in rendering bytes. [ADR-0008](../adr/0008-studio-network-controller.md), [controller results](../evidence/phase-4/README.md).
- [x] P4-02 — Viewport fixture with horizontal scroll/card scale, node/link filter, timeline and fields/hex interconnected; click/scroll/field/byte and TTL/checksum Before/After pass native walkthrough. Inspector uses configuration metadata and traces, not snapshots of the entire cache or a free graph editor; [scope and evidence](../evidence/phase-4/README.md).
- [x] P4-03 — Run fast/slow, pause, step, reset and epoch cancellation tested natively: pause on 7 events/2 pending remains stable, Step becomes 8 events, Resume completes, Reset when S04 is active, no old tasks are mutated. The measured latency target remains P4-07.
- [x] P4-04 — Multi-field draft, route table per L3 fixture device, administrative port down and timing per link available; shared subnet+route changes, port SW1:p1 down and persistence v2 tested natively. [Acceptance agent](../evidence/phase-4-acceptance/README.md). Free topology is outside the scope of this editor.
- [x] P4-05 — v1 session schema and bounded rerun verification pass controller as well as native Save → close → new application → Open. False reports are rejected without replacing active sessions; the baseline is also verified. The active checkpoint remains Phase 5.
- [x] P4-06 — U1–U5 executed by agent according to user instructions, including native persistence v2; [scorecard](../evidence/phase-4-acceptance/README.md). [ADR-0010](../adr/0010-agent-acceptance-and-studio-measurement.md) replaces the technical acceptance operator. Human understanding/usability remains unknown, it is not a requirement that users have to carry out the tests themselves.
- [x] P4-07 — 10,000 rows saved on four-node multi-run projection: 900 samples per type, p95 selection 65,093 ms / pause 73,062 ms; monitor 1,830 seconds without failure, peak working set sampled 68.15 MiB. [Method metrics/limits](../evidence/phase-4-acceptance/README.md).

Gate G4 / MVP: diagnosis–change–comparison workflow can be completed via the UI, including reopening saved results.

## Phase 5 — Replay, disruption, and comparison

- [x] P5-01 — Create checkpoint state of protocol/topology, queue/timer, packet, ID, PRNG, external input metadata, and model version; [Phase 5 evidence](../evidence/phase-5/README.md).
- [x] P5-02 — Implement restore, resume execution, and fork checkpoint; home and branch states remain separate.
- [x] P5-03 — Add controlled loss/jitter and record input noise; the same seed produces the same observable run.
- [x] P5-04 — Test checkpoint while ARP/timer/buffer is still active; compare state and continuation with run without restore.
- [ ] P5-05 — Create a Linux lab fixture and capture/comparison probe; the local direct-host fixture exists, while Linux capture remains open. Use containerlab when useful, without making it a kernel prerequisite.
- [ ] P5-06 — The local replay comparator checks summary, packet bytes/ancestry, field maps, transmissions, PRNG, noise, and retained traces; independent lab comparisons and tolerances remain pending P5-05. [Hardening evidence](../evidence/phase-5-hardening/README.md).
- [ ] P5-07 — Bounded checkpoint retention and incomplete-history behavior are documented and tested locally; final G5 report remains pending the lab comparison. [Local Phase 5 evidence](../evidence/phase-5/README.md).

Gate G5: identical replay in the stated environment and behavior according to the comparator in scope. This does not guarantee replay across all engine versions.

## Phase 6 — Observe Linux and MikroTik

- [ ] P6-01 — Complete the Linux fixture/version matrix as well as RouterOS from the available lab tools.
- [ ] P6-02 — Create a discover/observe/capabilities adapter contract with stable identity and provenance.
- [ ] P6-03 — Implement the required topology/interface/address/route reading on Linux and MikroTik; Store selected observations as sanitized fixtures.
- [ ] P6-04 — Handle timeouts, insufficient permissions, stale data, conflicting observations, and unavailable capabilities.
- [ ] P6-05 — Form a snapshot of the observed state, separate the desired/simulated state, then fork it to the simulator along with the assumed uncaptured state.
- [ ] P6-06 — Compare predictions with probe/capture lab; Show the source and time of observation to the inspector.

Gate G6: the second snapshot of the target family can be provenance-checked and used for the stated scenario; Observation adapter does not write configuration.

## Phase 7 — Core pilot: intent → simulate → Verify → execute → read-back

- [ ] P7-01 — Set static IPv4 intent, test endpoint, return path, capability, and operation contract; use lab routes that do not break management.
- [ ] P7-02 — Implement planner to operation IR and pre/postcondition; displays the plan and state diff which can be checked.
- [ ] P7-03 — Set schema shared declarative IR: match/action/order for routes, filters, VLANs, and unsupported stateful features.
- [ ] P7-04 — Drop the stateless simulator to the same IR and maintain Phase 3 oracle results.
- [ ] P7-05 — Implement packet-set verifiers for reachability, isolation, loop/blackhole, and shadowed rules on declared subsets.
- [ ] P7-06 — Add differential/property test simulator-versus-verifier as well as a concrete counterexample that can be replayed down to bytes.
- [ ] P7-07 — Implement semantically the same operation in the simulator, Linux adapter, and RouterOS; save lowering/backend mapping.
- [ ] P7-08 — Add drift check before apply, run/operation identity, deadline, partial/indeterminate state, and state check before retry.
- [ ] P7-09 — Implement read-back as well as result probe; define pilot operating compensation and its limits.
- [ ] P7-10 — Test healthy baseline, routes lost and then recovered, repeating intents without duplication, stale plans, and communication failures on each backend.
- [ ] P7-11 — Compare manual and Wyrlex workflows on the same task; fill in the scorecard without considering the simulation as proof of successful execution.
- [ ] P7-12 — Write G7 results with a truly proven plan, artifact, mismatch, and scope.

Gate G7 / core pilot: semantic contracts and complete workflow proven on Linux as well as MikroTik lab. If only one backend passes, log the partial pilot; cross-backend proof is incomplete.

## Phase 8 — Pilot ISPs: ZTE, Huawei ONU, and VSOL

- [ ] P8-01 — Inventory of ZTE OLT models/firmware, PON mode, Huawei ONU/VSOL models/firmware, and truly tested interoperability pairs.
- [ ] P8-02 — Read PON/ONU identity, state, VLAN/service mapping, uplink, alarm, and available optics; mark fields unsupported/unknown per model.
- [ ] P8-03 — Map ONU → PON → service/VLAN → uplink → MikroTik → test endpoint, along with source dependencies and observations.
- [ ] P8-04 — Select a service and model gap that must be closed: VLAN/tagging, service mapping, or other features used. Add the fixture/model before claiming the path simulation.
- [ ] P8-05 — Select one service change that can be tested along with its precondition, lowering, read-back, and recovery; verify ONU capabilities through the available channels.
- [ ] P8-06 — Healthy workflow tests, controlled service disruptions, simulated branch changes, applies to test services, and connectivity and other service conditions.
- [ ] P8-07 — Repeat on Huawei and VSOL separately; write model/firmware limits and incompatible pairing results.
- [ ] P8-08 — Separate LOS/optical power observations from model predictions; save pilot report and G8 decision.

Gate G8 / ISP pilot: one proven service workflow on tested matrices. If an ONU family has not qualified, that family's support remains unproven.

## Phase 9 — Beta and repeat use

- [ ] P9-01 — Create Windows packages, install/upgrade/uninstall wizards, and verify dependencies and distribution licenses.
- [ ] P9-02 — Implement platform-appropriate credential storage, export/log sanitation, and separation of observe/execute privileges as required by pilots.
- [ ] P9-03 — Test recovery after crash/disconnect, version migration, retention, backup, and diagnostic bundle export.
- [ ] P9-04 — Measure topology/event and memory capacity; add culling/virtualization/batching based on profiler.
- [ ] P9-05 — Run tasks with other engineers using runbooks; record failures, confusion, time, and manual steps.
- [ ] P9-06 — Document the compatibility matrix, support limits, bug report path, as well as the results of separate pilot sessions.
- [ ] P9-07 — Determine core beta or ISP beta scope based on passed gates and usage results.

Gate G9: applications can be installed, used, restored, and retested by people other than the author within the documented scope.

## Phase 10 — Future, selected according to needs

Each of the following tasks is a candidate for epic. Break it down into contracts, checklists, fixtures, and gates before starting; not everything has to be done.

- [ ] P10-01 — Add incremental UDP/TCP: demultiplexing, port unreachable, handshake, data, loss/retransmission, close, then additional fidelity as needed.
- [ ] P10-02 — Expand network models: advanced VLAN, NAT, firewall, PPPoE, IPv6, dynamic routing; test each feature and its interactions.
- [ ] P10-03 — Expand vendor/firmware and ISP services; publish capability and comparison results per adapter.
- [ ] P10-04 — Add reconciliation and repair planning with explicit execution policies; measure results against real drift.
- [ ] P10-05 — Add versioned infrastructure state: diff, branch/merge with domain conflicts, incident bisect, and replay compatibility declared.
- [ ] P10-06 — AI query/diagnosis/planning exploration via the same semantic API; evaluate labeled incidents and maintain operational contracts on the transfer path.
- [ ] P10-07 — Build collaboration, remote runner, CI integration, as well as Linux/macOS if user needs justify it.
- [ ] P10-08 — Evaluation of `.pkt` importer, OpenConfig, and other formats/models; Separate interoperability from fidelity and known reader limits.
- [ ] P10-09 — Optimize scale/parallelism/storage based on scalable bottlenecks while maintaining contractual determinism.
- [ ] P10-10 — Evaluation of server/PLC/UPS/IoT domains via shared primitives with individual domain models and proofs.

Gate G10: set per epic; One demo does not reveal all future capabilities. If the features on this list are required by selected pilots, move the required subset and its tests to the corresponding phase explicitly.

## First job

The execution sequence, output and completion criteria are outlined in [G4 closing plan, 24 October 2026](phase-4-plan.md). This plan has not changed the checklist status or gate results below.

P4-04 and P4-06 have been tested by the agent according to user instructions. P4-07 complete measurement and soak; [evidence acceptance](../evidence/phase-4-acceptance/README.md). G4 technical agent passes the scope of ADR-0010. Phase 5 local checkpoint/replay and impairment are implemented; the next open job is the Linux lab fixture and capture. P0–P3 and local P5 remain local baselines, not vendor G5 benchmarks. Complete access/firmware separately before P6/P8.

To complete the task, add the change/artifact link and verification results. For gates, use the result record in [pilot](pilot.md). This plan does not constitute authorization to change the device outside the lab/service that will be determined later.

Update October 24: draft implementation, schema v2 and retention available. The latest status follows the checklist above and evidence of acceptance; The previous human test delay has been replaced by user instructions for the agent to carry out all technical acceptance.
