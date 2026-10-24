# Decisions, preferences and open questions

Notes until October 23, 2026. Exploratory conversations do not automatically become final decisions. This document differentiates user guidance, recommendations from discussions, as well as draft additions to clarify next steps.

## Recap of discussion

| ID | Topics | Status | Results and references |
|---|---|---|---|
| WB-01 | Wyrlex Definition | User vision | Studio/runtime infrastructure with a shared model for inspect, simulate, and execute; [summary](README.md) |
| WB-02 | Does the idea make sense? | Design assessment | Worth building in stages; model accuracy and vendor coverage are key challenges; [comparator](positioning.md) |
| WB-03 | Evidence of reverse engineering PT | Reports + limited local verification | The foundation of objects, processes, schedulers, and traces supports the direction of the simulator; [evidence ledger](../packet-tracer/evidence-ledger.md) |
| WB-04 | Simulation without device | Directions explored | Can be without hardware/vendor image for modeled behavior; virtual devices still need state and rules; [simulation](simulation.md) |
| WB-05 | Understanding byte by byte | User requirements | Tracker needs bytes, field mapping, version, and reason for change; [tracker design](byte-tracker.md) |
| WB-06 | Zig Language | User decision; runtime/protocol phase 2–3 available | Product minimum 0.17, build pin 0.17.0; [package model](packets.md) running headless |
| WB-07 | Zig with Rust | Implemented for phases 1–3 | Probe/runtime/network Zig via copy C ABI; [ADR-0002](../adr/0002-zig-rust-abi.md), [ADR-0007](../adr/0007-packet-model.md) |
| WB-08 | GPUI or GPUix + Solid | GPUI selected and tested for spike | GPUI 0.2.2 live; local node/link/hex/input/close proven, benchmark not yet; [ADR-0004](../adr/0004-gpui-windows.md) |
| WB-09 | MVP Sequence | Revised plan 20 October 2026 | Early stack/FFI/UI trials, package PoC, MVP Studio, replay/validation, core pilot, then ISP domain; [roadmap](roadmap.md). Complete TCP/UDP is delayed unless required by the scope pilot |
| WB-10 | Project repository | Performed at the request of the user | Workspace `C:/Users/ricky/src/wirebox`, Git initialized on branch `main`, docs created |
| WB-11 | Agent guide and concept documentation | Performed at the request of the user | [AGENTS.md](../../AGENTS.md) and Wyrlex series of docs added |
| WB-12 | Compiler version | User decision | Minimum Zig **0.17.0**, Rust **1.99.1**; Rust is pinned in [rust-toolchain.toml](../../rust-toolchain.toml); [verification status](technology.md) |
| WB-13 | Plan gradually until the future | User request; details constitute a work proposal | Phase 0–10 with PoC/MVP/pilot milestones, [todos](todos.md), and [acceptance criteria](pilot.md); doesn't mean the gate has passed |
| WB-14 | Pilot device | User guidance, October 20, 2026 | Linux lab, MikroTik RouterOS, ZTE OLT, Huawei ONU and VSOL; model/firmware details and capabilities still need to be inventoried |
| WB-15 | Scope and contract evidence v1 | Determined in the implementation of phase 0 | [Baseline contract](phase-0-contract.md) and [user-reported inventory](../../fixtures/inventory/pilot-v1.json); unknown fixed access/device dependency G6/G8 |
| WB-16 | ADR and implementation of phase 0/1 | User request, done | [ADR](../adr/README.md), G0 local development and G1 Windows debug passed; [evidence](../evidence/phase-0-1/README.md) |
| WB-17 | Continue phase 2 | User request, done | Graph, scheduler, timer, PRNG, journal, and headless; [ADR-0006](../adr/0006-phase-2-runtime.md), [G2](../evidence/phase-2/README.md) |
| WB-18 | Continue phase 3 | User request, done | Ethernet/ARP/IPv4/ICMP, static routing, byte and field version copy, causal trace and oracle; [ADR-0007](../adr/0007-packet-model.md), [G3](../evidence/phase-3/README.md) |
| WB-19 | Initial implementation of Phase 4 | User requests, October 23, 2026 | Packet Studio with controller/snapshot, trace/field/byte, control, configuration branch and v1 session; [ADR-0008](../adr/0008-studio-network-controller.md). Native agent U1–U5 walkthrough completes after user authorizes desktop takeover; G4 has not yet passed the full |
| WB-20 | Product identity | User decisions, October 24, 2026 | Active name becomes Wyrlex; package and environment use `wyrlex-*`/`WYRLEX_*`, while the ABI prefix `wb_*` and old artifact evidence are retained; [ADR-0011](../adr/0011-wyrlex-product-rename.md) |
| WB-21 | Verify model sharing | User suggestions, October 24, 2026 | The simulator and verifier are driven using the same declarative IR, with differential gate and counterexample byte trace; [ADR-0012](../adr/0012-shared-verify-ir.md) |

## Corrections that affect the design

- Rust's borrow checker does not hide packet bytes. Byte visibility is a tracker design requirement, not a technical reason that rules out Rust.
- Mini-sim proves a software scenario can run independently; he has not proven all the behavior of the PT or vendor.
- Event List is a display/log. Queue scheduler and checkpoint replay have different functions and states.
- The initial hypothesis of default AES/Mash passphrase for container `.pkt` was replaced by another document path report. Recorded readers still have a keystream capture limit; [RE material final status](../packet-tracer/pkt-format.md).
- GPUix feature/version claims from old conversations do not become permanent facts. Technology documents separate old context and re-examine sources.

## Undecided matters

| Questions | Required before |
|---|---|
| Windows release/distribution strategy; build debug, Zig/Cargo.lock/GPUI pins, and CI workflow are available | Phase 9; remote CI run still needs to be done |
| Increased capacity from baseline bounded ADR-0006 | Before the MVP scenario exceeds the phase 2 |
| Active state checkpoint format and replay compatibility; v1 rerun session already set ADR-0008 | Phase 5 |
| Cross-thread UI snapshot/command scalability; Early single-thread controllers are already available | Before adding worker engines or MVP capacity |
| Expansion of the ID/byte model beyond ADR-0007, for example TCP retransmission | When the next protocol is added |
| Target scenario size and target latency/memory | Benchmarking and optimization |
| Expansion of protocol subsets beyond phase 0 contracts and ADR-0007 | Claim new feature support |
| Model/firmware, capability, lab access, and comparison data for the WB-14 target; the order of Linux/MikroTik then ISP is the plan | Real adapter |
| Revise scope after baseline v1, if new pilot features are needed | Must be noted before the next acceptance run |
| Authorization models, automatic reconciliation, and compensation | Real device mutations |
| `.pkt` import requirement rather than Wyrlex's own format | Interoperability priorities |

## How to update

When choices are confirmed or implementation changes, record the reasons, dates, artifacts, and test results in the relevant documents. Change the state of this table with a link to the proof. Don't mark a recommendation as a user decision just because it is recorded in the repository.

Thematic documents are syntheses, not verbatim transcripts. The original vision and RE report remain available at [source archive](../packet-tracer/sources/README.md).
