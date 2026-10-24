# Wyrlex documentation

This directory contains Wyrlex concepts, design direction, decisions, and development evidence. Phases 0–3 are available, Packet Studio Phase 4 passed technical agent acceptance for its declared scope, and G5 passes for scoped local checkpoint/replay and independent Linux WSL2 comparisons. Evidence-contract extensions and real-device adapters remain on the roadmap.

## Wyrlex

- [Build and preview commands](../README.md)
- [Architecture Decision Records](adr/README.md)
- [Phase 0 contract and baseline fixtures](wirebox/phase-0-contract.md)
- [FFI v1 contract](wirebox/ffi.md)
- [Handle-generation boundary test: candidates A and B](evidence/handle-generation/README.md)
- [G0/G1 verification results](evidence/phase-0-1/README.md)
- [Phase 2 runtime graph, scheduler, and ABI](wirebox/runtime.md)
- [G2 verification results](evidence/phase-2/README.md)
- [Phase 3 packet simulator, byte tracker, and ABI](wirebox/packets.md)
- [G3 verification results](evidence/phase-3/README.md)
- [Packet Studio controls, inspector, and versioned sessions](wirebox/studio.md)
- [Initial Phase 4 integration and G4 limits](evidence/phase-4/README.md)
- [Phase 5 checkpoint/replay and impairment evidence](evidence/phase-5/README.md)
- [Phase 5 hardening, regressions, and replay matrix](evidence/phase-5-hardening/README.md)
- [G5 assessment: replay, extended Linux comparison, and declared differences](evidence/phase-5-gate/README.md)
- [Initial Linux WSL comparison and corpus evidence](evidence/corpus-linux/README.md)
- [Behavior corpus structure, schemas, and reproduction](../corpus/README.md)
- [G4 completion plan and path to a real pilot](wirebox/phase-4-plan.md)
- [Walkthrough of actual byte changes](evidence/phase-3/byte-walkthrough.md)
- [Phase 1–4 learning notes: code decisions and reasons](learning/README.md)
- [Phase 0–3 hardening results and test limits](evidence/hardening-phase-0-3/README.md)
- [Hardening guide for a backend engineer](learning/hardening_phase_0_3.md)
- [Cognitive Coverage notes](learning/hardening_phase_0_3.md#apa-arti-generated): the private guide, quiz, and dashboard remain in the Git-ignored local directory.
- [What Wyrlex is, its goal, and its scope](wirebox/README.md)
- [Semantic model, state, planner, and runtime](wirebox/architecture.md)
- [Device-free simulation and its Packet Tracer evidence](wirebox/simulation.md)
- [Byte tracker, change explanation, and replay](wirebox/byte-tracker.md)
- [Zig, Rust, GPUI, and GPUix + Solid](wirebox/technology.md)
- [Idea positioning and comparison projects](wirebox/positioning.md)
- [Phase roadmap: PoC, MVP, pilot, beta, and future](wirebox/roadmap.md)
- [Phase todos and work status](wirebox/todos.md)
- [Verify design sharing an IR with the simulator](wirebox/verification.md)
- [Time as a first-class dimension: verifiable network history](wirebox/temporal-verification.md)
- [Device behavior corpus and long-term value hypothesis](wirebox/behavior-corpus.md)
- [ADR-0014: version-scoped device/firmware analysis and evidence policy](adr/0014-version-scoped-device-behavior-evidence.md)
- [Pilot plan, test scenarios, and success measures](wirebox/pilot.md)
- [Decisions, preferences, and open questions](wirebox/decisions.md)

Agent and contributor guidance: [AGENTS.md](../AGENTS.md).

## Cisco Packet Tracer reverse engineering

- [Summary and reading order](packet-tracer/README.md)
- [Simulation architecture](packet-tracer/architecture.md)
- [Evidence ledger and confidence levels](packet-tracer/evidence-ledger.md)
- [REA/Ghidra audit and local PT smoke test](evidence/pt-comparison/README.md)
- [PT/Wyrlex comparison-test runbook](packet-tracer/comparison-tests.md)
- [Mini-simulator and local verification](packet-tracer/mini-simulator.md)
- [.pkt document format and corrected findings](packet-tracer/pkt-format.md)
- [Implications for Wyrlex](packet-tracer/wirebox-implications.md)
- [Original reports and provenance archive](packet-tracer/sources/README.md)

The notes were assembled on 19 October 2026 from the initial user vision, project discussion, RE reports, and local mini-simulator inspection. They do not claim that every Packet Tracer binary path was audited again. Rechecked external technology sources are linked from the relevant documents.

The phase plan and Linux/MikroTik/ZTE/ONU pilot targets were updated on 20 October 2026. Gates and checklists distinguish available readiness from implementation that has not been tested.

[Studio hardening on 24 October and G4 status](evidence/phase-4-hardening/README.md).

[Agent technical G4 results, metrics, and test limits](evidence/phase-4-acceptance/README.md).

[Wyrlex rename migration evidence](evidence/rename-wyrlex/README.md).
