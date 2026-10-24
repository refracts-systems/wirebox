# Work plan after initial Packet Studio integration

October 24 implementation note: this plan records the sequence before further implementation. The latest result status is at [todos](todos.md) and [acceptance agent](../evidence/phase-4-acceptance/README.md).

Prepared 24 October 2026. Status: **planned, not yet implemented or passed gate**. This document breaks down the work that follows; [todos](todos.md) keeps track of task status, [roadmap](roadmap.md) describes the sequence of phases, and [pilot](pilot.md) defines success measures.

## Current position and nearest destination

Phases 0–3 have local evidence for their respective scopes. Phase 4 has a controller, trace/byte display, execution control, configuration branches, and rerun-based save/load. P4-01/02/03/05 is checked based on [native controller evidence and walkthrough](../evidence/phase-4/README.md). **P4-04, P4-06, P4-07 and gate G4 are still open.**

Immediate goal: users can diagnose packages, fix configurations on branches, compare results, and reopen sessions independently through Studio, with measurable response and retention. This proof is limited to supported Ethernet/ARP/IPv4/ICMP models and static routing.

Phase 4 changes when this plan was made were still in the working tree. Maintain the code baseline and evidence throughout the work. Commit/push following user instructions. Evidence of October 23 and its hash remains a historical record; The next run should produce new evidence.

## Sequence of work to close G4

### 1. Define editor contract and session storage — P4-04

- [ ] Write editor coverage for the existing four-node topology: interface addresses/prefixes, static routes per device, supported port/link status, bitrate and propagation per link. The free topology editor can remain a separate job.
- [ ] Determine the meaning of the link status: disabling a port or releasing a connection are different operations. State pending packet behavior and model limits before adding APIs.
- [ ] Design a draft configuration that can be edited in several fields and then applied once. Validate all drafts, build a new Network, then replace active sessions only if successful. This allows subnet changes without forcing any intermediate values ​​to be a valid configuration.
- [ ] Specify session schema changes before changing the Config form. Proposed: new schema with tested v1 reader/migration; Unknown formats are still rejected with a clear message. Don't change the meaning of schema v1 silently.
- [ ] Record additional decisions in ADR if the editor's contract, ABI, ownership or session format changes.

Output: contract on [Studio](studio.md), session format decisions, examples of valid/invalid configurations, and a list of expected results. The next main implementation files are `crates/wirebox-studio/src/model.rs` and `crates/wirebox-studio/src/main.rs`; kernel changes only if the existing API cannot fulfill the contract.

### 2. Implement editor and test branch — P4-04

- [ ] Add configuration model as well as UI based on contract step 1; display errors near the field and maintain the draft for correction.
- [ ] Ensure failed Apply retains active session, comparison baseline, trace and old bytes. A successful Apply creates a branch with a new runtime identity.
- [ ] Test valid routes, gateways outside the subnet, longest-prefix match, route deletion, multiple address changes at once, and port/link down according to the selected semantics. The expected result must come from the protocol/fixture rules.
- [ ] Test save/load new configurations, v1 session compatibility, unknown versions and manipulated results. Old sessions should not be replaced on load failure.
- [ ] Rerun controller regression as well as workspace build; do a native walkthrough of changes to routes, addresses, links and comparison of results.

Output: editor in written scope, behavioral and failure tests, as well as new evidence. P4-04 completes if the configuration in scope can be changed without source edit and the original scenario can still be checked.

### 3. Measure baseline, solve retention, then optimize — P4-07

- [ ] Log machine, OS, GPU, toolchain, build mode, source revision/hash, configuration and measurement workload. Freeze workloads and methods before acceptance runs.
- [ ] Measure selection and pause the input until the related display/state is updated. Include engine costs, FFI, Frame processing and rendering; report sample size as well as p50/p95/maximum.
- [ ] Decide retention of traces and package versions. Currently the host/engine is limited to 1,024 rows, while [initial contract](phase-0-contract.md) targets 10,000 saved rows. Define storage, batching and virtualization based on a baseline, with clear indication of when the history is truncated.
- [ ] Ensure retained rows still have the referenced bytes/fields, or display their unavailability explicitly. Detect journal gaps before data is overwritten; Packet identities from different runs must not be confused.
- [ ] Separate UI projection stress tests with labeled synthetic data from end-to-end engine testing. Repeating the rows to meet the number 10,000 does not prove that the 10,000 row engines were stored correctly.
- [ ] Run accept four nodes/10,000 rows: selection/pause **p95 <100 ms**. Run scripted sessions **30 minutes** without crashes/hangs with a total working set of applications **≤1 GiB**; Note the memory peak, buffer size and number of rows.
- [ ] If the target is not achieved, save the failure and bottleneck profiles, fix them, then retry the same workload. Target revisions require explicit contract changes prior to acceptance; don't pass G4 by lowering the number secretly.

Output: repeatable harness/method, metric data, profiles if needed, and pass/fail reports. The GUI limit should not truncate the scheduler queue.

### 4. Usability testing with users — P4-06

- [ ] Set up runbook U1–U5: run healthy ping, lost route diagnosis, explain TTL/checksum and associated bytes, repair branch, then save/close/open.
- [ ] Have the user run the task with the runbook without source edit. Record results, time, help, errors and points of confusion per task.
- [ ] Differentiate the results of available walkthrough agents from those received by human users. Don't consider learning artifacts or automated tests as evidence of user understanding.
- [ ] Fix discovered bottlenecks and then retry the affected tasks and associated regressions. Use [pilot scorecard](pilot.md), with claims limited to measured results.

Output: scorecard contains actual results and remaining issues. This stage requires user participation; runbook preparation and instrumentation can be completed in advance.

### 5. Record the G4 decision

- [ ] Unify configuration, version/source hash, environment, commands, test results, metrics and scorecards in a new evidence bundle.
- [ ] Make sure all G4 criteria have pass/fail results or untested reasons. Link bundles from todos, Studio and evidence index.
- [ ] Update learning docs: changed files, invariants, ownership, decision reasons, actual commands, and one teach-back question. Check sourceHash local cognitive coverage material; refresh affected material or flag stale.
- [ ] Declare G4 passed only if editors in scope, U1–U5 and acceptance measurements are met. Save/load reruns are still not claimed as active restore checkpoints.

Output: reviewable G4 decision. The success of G4 proves that MVP Studio in scope is not yet equivalent to a real device or Packet Tracer.

## After G4: initial idea proof sequence

| Phases | Core work | Proof required before proceeding |
|---|---|---|
| 5 | Checkpoint/restore, active replay/fork, controlled loss/jitter; Linux lab comparator for modeled behavior | Restore includes queue, timer, cache, pending packets, ID, time, PRNG and input; replay produces the appropriate state/byte/sequence. Capture Linux is tested against the same protocol invariants. |
| 6 | Linux observation adapter and RouterOS 7; snapshot to the simulation model | Identity, capability, source, time and state mapping proven; unknown/stale/conflict visible. The initial path is read-only. |
| 7 | One IPv4 route operation from intent to simulation, plan, executor Linux/MikroTik and read-back/probe | Test success, drift, no-op, partial failure, unknown results and recovery in the lab. G7 provides the first proof of core flow across simulators and two real backends. |
| 8 | ZTE C320 service workflow → Huawei ONU/VSOL → uplink/endpoint | Inventory of models/firmware, capabilities and services/VLANs used are modeled and tested; the results of each ONU pair are recorded separately. |
| 9 | Packaging, credential storage, recovery and repeated use by other engineers | Install, upgrade, crash/disconnect recovery and proven runbooks for the core beta scope or selected ISP. |
| 10 | TCP/UDP, other network features, interoperability, collaboration and scale | Select epic based on pilot needs and proof of use; Each ability has its own contract and gate. |

Collection of inventory of Linux VMs/servers, RouterOS minor versions, as well as ONU models/firmware can be done from now on. Do not store passwords in the repository. Adapter implementation follows phase dependencies; Device access and lab boundaries are determined before testing that changes the device.

Task details still follow [todos Phase 5–10](todos.md). Packet Tracer/device equivalence requires input and comparison oracles in the same scope; see [comparison runbook](../packet-tracer/comparison-tests.md). No single test proves all firmware or all vendor behavior.

## Learning during implementation

For backend engineers, draft configurations are like requests that are validated before changing active aggregates; Frames are copies of the read model for the UI; epoch limits old tasks from mutating new sessions. This analogy helps read code, but is not a guarantee of rollback for all runtime operations.

The code comments focus on the reasons for bounds, FFI ownership, copy lifetime, and session replacement points. The long flow explanation is placed in [learning docs Phase 4](../learning/phase_4/phase_4_controller_snapshot.md).

Teach-back question: **when traces are stored longer than the engine's packet buffer, what data must be retained so that old row hits still point to the correct bytes and packet version?**
