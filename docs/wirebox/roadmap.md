# Wyrlex roadmap: proof, MVP, pilot and development

Updated October 24, 2026. This is a phased work plan at the request of users. The execution checklist is in [todos](todos.md); scenarios and measures of success are in the [pilot plan](pilot.md). Roadmaps explain sequences, todos record work, and pilots determine evidence of success.

## The results you want to prove

Wyrlex should show a checkable relationship from **infrastructure object → intent → plan → state change → protocol decision/byte → result verification**.

| Milestones | Done after | Claims that may be made |
|---|---|---|
| Technical PoC | Phase 3 | The standalone kernel processes the scenario and describes packets down to bytes |
| MVP Studio | Phase 4 | Users open scenarios, run, diagnose, modify and compare results via the application |
| Wyrlex core pilot | Phase 7 | One operating model is tested in the simulator, run on Linux/MikroTik lab, and verified again |
| ISP domain pilots | Phase 8 | One proven customer workflow on ZTE with tested Huawei/VSOL ONU |

One successful ping fulfills a portion of the PoC. Proving the initial idea that links simulation to reality requires a core pilot; ISP workflow requires an ISP domain pilot. Each claim has stated coverage.

## Current position

G0/G1 pass for local development Windows x64 debug: [gate report](../evidence/phase-0-1/README.md). [G2](../evidence/phase-2/README.md) proves graph/scheduler/timer/PRNG. [G3](../evidence/phase-3/README.md) proves Ethernet PoC/ARP/IPv4/ICMP and headless byte tracker in the stated subset. Initial integration [Packet Studio Phase 4](studio.md) is already available: copy controller, trace/byte, control, branch and session v2. [G4 technical agent](../evidence/phase-4-acceptance/README.md) passes on scope fixture, multi-run history projection as well as handler-to-render response; Human usability studies remain unknown. [Phase 5 local evidence](../evidence/phase-5/README.md) proves value-owned checkpoint/fork/restore and seeded impairment replay; initial [Linux WSL2 corpus comparisons](../evidence/corpus-linux/README.md) now cover S01–S03. The full G5 gate, device drivers, and real vendor pilots remain open.

Target pilot users: **Linux VM/server, MikroTik RouterOS 7, OLT ZTE C320, ONU Huawei, and ONU VSOL**. The C320 board version as well as the GPON state example are listed as user-reported in [inventory](../../fixtures/inventory/pilot-v1.json). ONU model/firmware, RouterOS minor version, Linux distro/kernel, access, as well as unknown compatible pairs remain G6/G8 dependencies.

## Phase sequence

| Phases | Focus and results | Dependencies | Exit / gate conditions |
|---|---|---|---|
| 0 | Success contracts, fixtures, lab inventory, and MVP limits | Foundation docs/toolchain partially available | G0: main scenarios, test matrix and written support limits |
| 1 | Zig/Rust build, FFI test, and Windows GPUI window | G0 | G1: new checkout can be built; Rust uses Zig results; window and shutdown work |
| 2 | Graph, state, virtual time, timer, and journal | G1 | G2: stable sequence, cancellation, and execution limits pass test |
| 3 | Ethernet/ARP/IPv4/ICMP, static routing, and trace bytes | G2 | G3: Headless PoC passes defined success as well as failure cases |
| 4 | Studio: topology, timeline, inspector, save/load | G3 and G1 UI trials | G4: MVP completes the diagnose–change–compare flow of the UI |
| 5 | Checkpoint/replay, controlled disruption, validation against lab | G4 | G5: restore/replay according to state and comparison results in scope |
| 6 | Linux/MikroTik observations and snapshots to the simulator | G5 | G6: provenance, unknown/stale, and proven snapshot mapping |
| 7 | Intent, shared declarative IR, Verify, two executors, apply and read-back Linux/MikroTik | G6 | G7: simulator/verifier agrees on subset of models and core pilots pass scorecard as well as failure scenarios |
| 8 | ISP domain: ZTE → ONU Huawei/VSOL → customer service | G7 and device inventory | G8: ISP pilot passes on tested model/firmware combinations |
| 9 | Beta that other users can repeat and operate | G8 for ISP beta; G7 is sufficient for core beta | G9: installation, recovery, documentation, and proven repeat use |
| 10 | Development based on evidence of use | G7/G8/G9 on track | Gate per ability; select features based on measurable needs |

Equipment inventory and examples of phase 6/8 observations can be collected from phase 0. GPUI/FFI experiments are carried out early so that the risk of the stack is known before the engine enlarges. Feature implementations follow their dependencies.

## Phase 0–3: prove the foundation

Starting from `PC1 — SW1 — R1 — PC2` with Wyrlex's own scenario format. Set time semantics, ID, buffer ownership, operation, and trace limits. Select a comparison reference that is independent of the new implementation.

The PoC describes ARP, MAC learning/flooding, route lookup, TTL/checksum, and relevant drops. The original bytes, field mapping, package version, and cause relationships became part of the initial implementation. Pasting the PT step name in the log does not satisfy this gate.

Zig minimum 0.17.0 and Rust 1.99.1 following user decision. Direct GPUI is a prime candidate for UI experiments according to [technology direction](technology.md). If the trial does not meet needs, note the constraints and evaluate alternatives based on the evidence.

## Phase 4: MVP Studio

MVP is a Windows application for a complete workflow: open scenario → send ping → find cause of failure → check bytes and state → change route on scenario copy → rerun → compare → save results.

Mandatory: four initial nodes, Ethernet/ARP/IPv4/ICMP, static routing, bounded execution control, causal trace, field/byte inspector, versioned save/load, and unsupported flag. A simple change form is sufficient; a complete topology editor is not a prerequisite.

MVP does not yet require real drivers, `.pkt` import, full TCP, AI, firmware emulation, or time travel from arbitrary checkpoints. Changing the scenario and then rerunning from scratch is enough on the MVP; restore active state tested in phase 5.

## Phase 5–7: prove the core idea

Make sure the engine is reproducible and compared to the lab on supported behavior. Insert Linux/MikroTik observations into a shared model, fork snapshots into simulations, and state any state assumptions that are not available.

The initial intent is limited to static IPv4 reachability: the source, test destination, direction, and protocol are stated. Plan produces an operation like `EnsureStaticRoute` with semantically the same pre/postcondition. Route/filter/VLAN rules are then derived into a shared declarative IR so that the concrete simulator and Verify set do not have different copy semantics. Real simulators and adapters implement them through their respective backends. The name/operation scheme is still being drafted; [ADR-0012](../adr/0012-shared-verify-ir.md) has proposed status.

The pilot demonstrates initial failure, diagnosis, simulation changes, plan review, execution in the lab, then observation and re-probe. Stale plan tests, repeated operations, management connection breaks, as well as partial failures. Indeterminate results require a state check before retrying.

**G7 is an early proof of Wyrlex's idea in a limited scope.** Pings on the device alone do not prove the semantic relationship between the simulation plan and real operations. Details are in [pilot plan](pilot.md).

## Phase 8: prove the user's ISP domain

Start with observations of ZTE, PON/ONU, service status, VLAN, and available signals from the device. Test Huawei ONU and VSOL separately; Functional similarity does not guarantee identical fields and operations.

Initial workflow: one test customer service on a controlled VLAN and IP path to an endpoint on the uplink side. Select operations once capabilities are known. Don't make PPPoE, NAT, GEM/TCONT, or WAN ONU changes hidden assumptions; model and test what the selected service actually requires.

The “customer accessible internet” target of the vision is maintained. Testing starts on a controlled endpoint so that dependencies can be observed, then extends to external destinations with additional dependencies noted. The initial optical state is observation; PON physics simulations require their own models and validation.

## Phase 9–10: beta and subsequent development

Beta proves workflow repeatability in a documented environment, including install, credential handling, retention, recovery, and failure diagnosis. Set capacity based on measurements and publish tested model/firmware support limits.

Development path: protocol fidelity, scale/performance, domain/vendor, intent automation, collaboration, and interoperability. Each path requires its own user requirements, contracts, comparison fixtures, and gates. Epic candidates are available at [todos](todos.md).

## Proposed extensions: temporal verification and behavior evidence

User-provided directions recorded on 24 October 2026 are detailed in [verifiable network history](temporal-verification.md) and the [device behavior corpus](behavior-corpus.md). They connect existing phases without marking additional capabilities complete:

| Foundation | Proposed extension | Evidence required before claiming it |
|---|---|---|
| Phase 5 replay and lab comparison | Reproducible history and minimal behavioral records | Complete inputs, model/build provenance, reproducible mismatches, and declared replay limits |
| Phase 6 observations | Queryable observed history | Observation intervals, identity, ordering/clock uncertainty, retention, and visible gaps |
| Phase 7 shared model and Verify | Safe ordering of supported plan transitions | Transition semantics, relevant intermediate states/interleavings, counterexamples, and lab probes |
| Phase 8 ISP service pilots | Multi-vendor service-transition evidence and initial corpus records | Exact OLT/board/ONU/router versions, operation scope, separate pairing tests, and consent for any shared data |
| Later development | Incident queries, conditional impact, firmware comparisons, and possible compatibility assessments | Demonstrated operator demand, validated models, and explicit publication/review contracts |

The immediate priority remains real-system fidelity. Temporal claims depend on that fidelity, and corpus value depends on reproducible evidence. The initial local corpus exists; portable history, symbolic temporal verification, shared corpus hosting/contribution infrastructure, licensing, pricing, and certification remain unimplemented or undecided. The existing gate definitions still apply.

The evidence policy is now accepted in [ADR-0014](../adr/0014-version-scoped-device-behavior-evidence.md). Phase 5 follow-up defines analysis/claim/combination contracts without rewriting existing Linux records. Phase 6 selects an exact RouterOS build and capability for targeted mechanism analysis and runtime tests; Phase 7 links adopted claims to semantic rules and independent comparisons. Phase 8 records ZTE board/component versions and separate Huawei/VSOL ONU combinations. Major-version baselines allocate investigation effort; support remains limited to tested builds, modes, and scenarios. Analysis dossiers and vendor schemas are not implemented, and this policy does not change gate results.

## How to execute the plan

Use one main phase as active work. Mark the task as complete once the artifact/evidence is available; Completion of implementation does not automatically pass the gate. Record the gate results as untested, passed, failed, or held and why.

Priority **Now**: phase 0–4. **Next**: phases 5–7. **Later**: phases 8–9. **Future**: phase 10. This is a sequence of focus, not a date promise. Calendar estimates are made after the pilot's working speed and scope are known.

If the gate fails, explicitly refine or narrow the claim/scope before proceeding. PT findings still refer to the [evidence ledger](../packet-tracer/evidence-ledger.md), not a replacement for the Wyrlex test.
