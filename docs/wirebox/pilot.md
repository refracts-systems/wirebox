# Proving Wyrlex's pilot ideas and plans

Prepared 20 October 2026. This document specifies what needs to be proven, not states the pilot has been successful. Implementation follows [roadmap](roadmap.md) and [todos](todos.md).

Target devices come from users: Linux lab, MikroTik RouterOS, OLT ZTE, ONU Huawei, and ONU VSOL. The success rate below is frozen at [contract phase 0 v1](phase-0-contract.md) as acceptance criteria; is not yet a measurement result or performance promise. [G0/G1 results](../evidence/phase-0-1/README.md) only includes local development and probe stacks.

## Tested hypothesis

| Hypothesis | Evidence sought | Milestones |
|---|---|---|
| H1 — The simulator can run independently and explain the package | Success/failure scenarios, bytes, decisions, and states corresponding to the comparison | G3 / PoC |
| H2 — Explanations are useful in user workflow | Users complete diagnosis, changes, and comparisons via the UI | G4 / MVP |
| H3 — Semantic models can relate simulations to real systems | The same intent/operation semantically runs on the simulator and Linux/MikroTik and then verified | G7 / core pilots |
| H4 — The model can handle ISP customer service | Proven workflow from ZTE/ONU to router/endpoint on a real device matrix | G8 / ISP pilot |

Visual demos support the presentation of results, but successful decisions use artifacts and expected results that can be checked. H1/H2 does not automatically prove H3/H4.

Status October 23, 2026: H1/G3 passed for headless subset with frozen manual fixture and separate Rust oracle; [evidence](../evidence/phase-3/README.md). [Studio Phase 4 initial integration](../evidence/phase-4/README.md) available; U1–U5 native agent walkthrough passes, including route diagnosis, field/byte, repair branch and Save/close/Open. Update October 24: [G4 technical agent passed](../evidence/phase-4-acceptance/README.md) according to user instructions to complete all tests via agent; Human usability studies remain unknown. Capture Linux G5 comparators are not yet available; H3/H4 pilots have not been tested. The success of the simulator has not graduated the pilot of the device.

## Three proof environments

| Environment | Coverage | Dependencies that must be noted |
|---|---|---|
| A — Standalone simulator | PC1–SW1–R1–PC2; Ethernet/ARP/IPv4/ICMP and static routing | Revised engine/model, fixture, seed, clock semantics, byte/link limits |
| B — Linux/MikroTik lab | Controlled endpoints, static IPv4 paths, and one route operation | Linux distro/kernel, RouterOS version/model, interface, return path, access management, capability |
| C — ISP lab | Test customer ONU → ZTE → uplink → MikroTik → test endpoint | OLT model/firmware and both ONU families, PON mode, VLAN/service mapping, available operations |

Environment B can use namespaces/containers/VMs/lab devices according to existing access; containerlab is an orchestration option, not a requirement for Wyrlex's success. The C environment starts with controlled endpoints before testing internet destinations that have additional dependencies.

The inventory stores observation sources, timestamps, and target identities without credentials. The vendor families mentioned by the user are not considered to be compatible with each other. Test Huawei and VSOL as two separate rows of matrices.

## Five user tasks for MVP

| ID | Tasks | Proof completed |
|---|---|---|
| U1 | Open the healthy scenario and run ping | Paths, states, and results can be viewed without editing source |
| U2 | Diagnosis of lost route cases | User indicates correct device, lookup, and drop reason |
| U3 | Explain packet changes on the router | The user finds the TTL, checksum, next hop header, and associated byte ranges |
| U4 | Change the route on the scenario copy and compare | The new result changes according to the expected result; original scenario remains intact |
| U5 | Save, close, reopen results | Configuration, output supported formats, required traces/bytes, and versions can be double-checked |

Users can use runbooks. The need for assistance is noted; success that requires changes to the source or the author's verbal explanation to find results that do not meet the independence targets U1–U5.

## Core pilot demo story

1. Observe the Linux/MikroTik lab and save a snapshot with provenance and unobserved state assumptions.
2. Demonstrate failure conditions due to lost lab routes using probes and independent observations.
3. Create a reachability intent with explicit source, destination, direction, protocol, return path, and capability.
4. Planner generates operation IR; The UI shows targets, changes, preconditions, and expected effects.
5. Fork the snapshot and run the operation on the simulator. Check results, causal traces, and package changes.
6. Check the drift, then run the plan on the specified lab via the appropriate adapter.
7. Read the route again and run a probe from the endpoint. Record success only when the postcondition is met.
8. Run the same intent again: do not generate duplicate routes. Also show the stale plan and communication failures detected.
9. Compare the manual and Wyrlex results, then save a proof bundle.

Repeat operation contracts on Linux and RouterOS. Backends can use different transport/syntax, but the preconditions and result meaning must be the same. The real lab package can be compared with predictions only on the fields/invariants that are actually modeled.

## Minimum scenario matrix

| ID | Case | Expected results | Gate |
|---|---|---|---|
| S01 | Healthy topology | Echo reply and path according to fixture; validated bytes/fields | G3, G4 |
| S02 | Route lost | The reason for failure is correct; route changes on branches restore results | G3, G4, G7 |
| S03 | TTL expired | Drop/error according to ICMP subset; TTL/checksum changes can be checked | G3 |
| S04 | Next-hop doesn't answer | ARP/timer expires according to contract; bounded run completed and reason visible | G3 |
| S05 | Package truncated or unsupported features | Input rejected or result unsupported/unknown; no crashes or fake passes | G3, G4 |
| S06 | Checkpoint when the buffer/timer is active | Restore + continue matches state and event run without restore | G5 |
| S07 | Seed/input is the same, including interference | Canonical traces and results can be repeated on the same build/environment | G5 |
| S08 | Observation stale, timeout, or conflicting | Provenance is visible; state is not tacitly considered current or certain | G6 |
| S09 | Intent route repeated | The postcondition remains true without duplication; no-op/changed results can be distinguished | G7 |
| S10 | State changes after plan | Drift/precondition mismatch is detected before the operation that relies on it | G7 |
| S11 | Apply failed or connection dropped after sending | Status failed/partial/indeterminate according to evidence; read-back before retry; not making up success | G7 |
| S12 | Test customer service/VLAN problem | Diagnosis according to capability; plan, apply, and probe recover the modeled path | G8 |
| S13 | ONU offline/LOS or optical telemetry not available | Evidence and dependency appear; does not predict recovery through surgery that does not resolve the cause | G8 |

Case S12 is selected after the service mapping and operation are known. If using PPPoE/NAT or other features outside the model, close gaps and add tests before declaring the end-to-end simulation passed. Declaring unsupported is correct behavior, but does not count as reachability success for that service.

## Baseline success scorecard v1

| Dimensions | Initial target | How to measure |
|---|---|---|
| PoC Truth | S01–S05 according to expected result; no open critical mismatch | Byte/codec fixtures, comparator probe/capture, and decision reasoning checks |
| Uses of MVP | U1–U5 finished with runbook without source edit | Note the results, time, help and points of confusion of each task |
| Reproducibility | Scenario S06/S07: three repetitions for each of the five selected seeds, with no differences within the same seeds | Compare canonical states and traces; output wall-clock metadata, not virtual events or critical bytes |
| Core pilot consistency | All cases applicable to G7 are true on Linux and RouterOS, three iterations of each | Plan/operation ID, state diff, read-back and independent probe; mismatch has an explanation/issue |
| Honesty of results | Zero false success status on tested failed/unsupported cases | Compare the application status with evidence and postcondition |
| Benefits of workflows | On at least four of five comparable diagnosis/change tasks, time was shorter or manual interaction was reduced without decreasing correctness | Define five tasks and how to calculate interactions on P0-06; record manual baseline, Wyrlex results, as well as help |
| MVP Response | On a four-node fixture and 10,000 rows of stored traces, the p95 selection/pause response is under 100 ms on the reference machine | Measure from input until the relevant display/state is updated; include engine/FFI/render costs, not just UI updates |
| Initial stability | 30 minute scripted session with no crashes/hangs; memory remains within the specified retention budget | Note memory usage and journal/buffer size; budget frozen after initial baseline, before acceptance run |
| ISP Coverage | S12/S13 and service isolation checks passed on listed ZTE–Huawei and ZTE–VSOL pairs | Model/firmware matrix, test service observation and comparison service, and result probe |

The numbers above are small engineering/pilot criteria, not an estimate of the reliability of all production or the equivalence of all vendors. If the machine or fixture changes, re-note the context and do not compare numbers without adjustments.

For manual comparisons, use paired tasks of equal difficulty and note the order of work so that learning effects are not misinterpreted as application benefits. The initial user may be the project owner; validation of other engineers to gate beta G9.

## Evidence package for each gate

Local G0/G1 results are available at [phase-0-1](../evidence/phase-0-1/README.md); invariant graph/scheduler and reproducibility of the counter model in [G2](../evidence/phase-2/README.md). G2 testing has not yet passed S01–S13 or the network pilot scorecard. The next run can use a directory such as `docs/evidence/<gate>/<run-id>/`. Save required summaries and artifacts, not dump credentials or vendor binaries to Git.

Minimum records:

| Fields | Contents |
|---|---|
| Gate/task | G0–G9 or gate epic G10, as well as the associated task ID |
| Environment | Source/build revisions, model/schema versions, toolchain/OS, test machines |
| Targets | Model/firmware, capability, topology/fixture hash, and endpoint |
| Input | Scenario, intent/plan/operation ID, seed, and disruption |
| Expected | The results and benchmark origins are set before run |
| Actual | State, relevant trace/byte/capture, read-back/probe, as well as user/performance metrics |
| Results | Untested / passed / failed / held; mismatch and remaining boundaries |

The fixture/trace hash is used according to the specified canonical format. The number of GUI rows is not considered the number of scheduler callbacks. Lab captures are compared on supported invariants: field values, forwarding decisions, checksums, and results. Do not demand the same timestamp, MAC, initial TTL, or different IDs without first establishing valid mapping and normalization.

## Advanced decision rules

- **Pass** when all mandatory gate cases are correct, evidence can be checked, and user benefit for the associated milestone is measurable. Technical results passed with user benefits not yet achieved are recorded separately; full pilot has not yet graduated.
- **Fail** when there is a critical mismatch, false success, crash that breaks the workflow, or inconsistent state/bytes. Fix it before expanding your claim.
- **Stuck** when access, compatibility, firmware, or capability is not yet available. Work on independent phases that are still useful; do not mark evidence that has not been obtained as passed.
- **Narrow the scope** if necessary, by revising the recorded acceptance criteria before the next acceptance run. Don't write off failed cases silently to make the pilot appear successful.

If the MVP is useful but the bridge to the real system has not passed, the new Wyrlex proves to be a simulation studio. If G7 passes, the core idea is proven for that pilot contract and device. G8 extends proof to the user's ISP workflow.
