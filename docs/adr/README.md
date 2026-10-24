# Architecture Decision Records

ADRs record context, decisions, alternatives, consequences, and implementation status. `Accepted` means that the decision is the working choice for an authorised task; it does not claim that every feature is complete. Material changes should add a superseding ADR with a reference to the earlier decision.

| ADR | Decision | Status |
|---|---|---|
| [0001](0001-semantic-state.md) | Shared semantic model and state provenance | Accepted; simulated-state graph exists, observed/desired state remains planned |
| [0002](0002-zig-rust-abi.md) | Zig kernel, Rust host, small C ABI with copied data | Accepted; ABI v1 probe implemented |
| [0003](0003-virtual-time-and-trace.md) | Deterministic virtual time and separate trace journal | Accepted; implemented in Phase 2 |
| [0004](0004-gpui-windows.md) | Direct GPUI on Windows x64 | Accepted for the Phase 1 spike; runtime evidence is recorded by the gate report |
| [0005](0005-pilot-scope.md) | Staged proof and v1 scope | Accepted as the pilot baseline; real networks are not yet tested |
| [0006](0006-phase-2-runtime.md) | Graph/timer lifecycle, budget, PRNG, journal, and runtime ABI | Accepted and implemented; [G2 evidence](../evidence/phase-2/README.md) |
| [0007](0007-packet-model.md) | Protocol subset, byte/transmission versions, link/ARP policy, and network ABI | Accepted and implemented; [G3 evidence](../evidence/phase-3/README.md) |
| [0008](0008-studio-network-controller.md) | Studio controller, copied snapshots, Run epochs, branches, and v1 session files | Accepted 23 October 2026; initial integration, full G4 not passed |
| [0009](0009-studio-draft-session-v2.md) | Draft session schema and retention | Accepted 24 October 2026 |
| [0010](0010-agent-acceptance-and-studio-measurement.md) | Agent acceptance and Studio measurement | Accepted 24 October 2026 for the declared fixture scope |
| [0011](0011-wyrlex-product-rename.md) | Wyrlex product identity with `wb_*` ABI compatibility | Accepted 24 October 2026; migration active |
| [0012](0012-shared-verify-ir.md) | Declarative IR shared by the simulator and Verify | Proposed for Phases 5–7 |
| [0013](0013-phase-5-checkpoint-and-impairment.md) | Value-owned checkpoints and deterministic impairment | Accepted; [G5 passes for scoped replay and Linux comparisons](../evidence/phase-5-gate/README.md) |
| [0014](0014-version-scoped-device-behavior-evidence.md) | Exact-build device/firmware reverse engineering, runtime evidence, and interoperability claims | Accepted 24 October 2026 as policy; vendor dossiers and schema extensions remain pending |

Decision dates are 20–24 October 2026, as recorded in each ADR. Actual results are in [Phase 0–1 evidence](../evidence/phase-0-1/README.md) and [initial Phase 4 integration](../evidence/phase-4/README.md). Scope is in the [Phase 0 contract](../wirebox/phase-0-contract.md), and the work checklist is in [todos](../wirebox/todos.md).

Active product and technical documentation is written in English. `docs/learning/` remains Indonesian by explicit project policy. Raw Packet Tracer source captures remain byte-for-byte historical artifacts; their provenance is documented in [the source index](../packet-tracer/sources/README.md).
