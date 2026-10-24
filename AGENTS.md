# Wyrlex repository guidance

## Project purpose

Wyrlex is an infrastructure studio and runtime built on a shared semantic model. It connects real-system inspection, simulation, packet and state tracing, change planning, and device-adapter execution.

Read the [documentation index](docs/README.md) and [what Wyrlex is](docs/wirebox/README.md) before changing architecture. For a focused task, read only the relevant documents; do not load the full reverse-engineering archive.

## Status and sources of truth

The active product name is **Wyrlex**. Rust packages are `wyrlex-kernel`, `wyrlex-headless`, and `wyrlex-studio`; build environments use `WYRLEX_*`. The `wb_*` C ABI prefix, legacy header names, checkout path, and historical evidence artifacts remain stable under [ADR-0011](docs/adr/0011-wyrlex-product-rename.md).

As of 24 October 2026, Phases 0–3 are available: contracts/fixtures, Zig–Rust/GPUI, virtual time, and a headless Ethernet/ARP/IPv4/ICMP simulator with static routing and byte tracking. Read the [runtime contract](docs/wirebox/runtime.md), [packet model/ABI](docs/wirebox/packets.md), and [G3 evidence](docs/evidence/phase-3/README.md). Initial [Packet Studio Phase 4](docs/wirebox/studio.md) uses copied trace/packet/field data, batch controls, configuration branches, and versioned sessions; [G4 technical agent acceptance](docs/evidence/phase-4-acceptance/README.md) passed on 24 October 2026 for the four-node fixture, projected multi-run history, and handler-to-render scope (ADR-0010). Local [Phase 5 evidence](docs/evidence/phase-5/README.md) now covers value-owned checkpoint/fork/restore and seeded impairment replay; an initial [Linux WSL2 corpus comparison](docs/evidence/corpus-linux/README.md) now covers S01–S03, while the full G5 gate, real-device drivers, and vendor pilots remain open. Human usability is still unknown. The C++ mini-simulator is an external reference and must not be treated as the Wyrlex engine.

- [Decisions and open questions](docs/wirebox/decisions.md) distinguish user preferences, recommendations, and choices not yet made.
- [ADRs](docs/adr/README.md) record architectural decisions; [G0/G1 evidence](docs/evidence/phase-0-1/README.md) limits claims about actual results. Build and verification commands are in the [README](README.md).
- The [roadmap](docs/wirebox/roadmap.md) describes proposed stages, not completed features or promised dates.
- [Todos](docs/wirebox/todos.md) record phase checklists; the [pilot plan](docs/wirebox/pilot.md) defines scenarios and evidence criteria. When implementing work, link it to verify results and never pass a gate from a code checklist alone.
- [Packet Tracer notes](docs/packet-tracer/README.md) summarizes the RE reports and corrections.
- [Verify design](docs/wirebox/verification.md) and [ADR-0012](docs/adr/0012-shared-verify-ir.md) propose a declarative IR shared by the simulator and verifier; no symbolic engine exists yet.
- [ADR-0014](docs/adr/0014-version-scoped-device-behavior-evidence.md) governs version-scoped device/firmware analysis, runtime evidence, and exact OLT/ONU combinations. Major versions are navigation groups, not evidence inheritance. Vendor dossiers and their schema extensions remain pending.
- `docs/packet-tracer/sources/` is a historical archive. Its claims may be superseded by corrections in thematic notes.

Active-session user instructions control the work. Do not treat a recommendation in the documentation as an irreversible user decision. Update documentation status when implementation or decisions change.

## Working and communication rules

- Use clear Indonesian for discussion and learning explanations. Repository documents committed outside `docs/learning/` must be written in English by project policy. The immutable raw captures under `docs/packet-tracer/sources/` are the provenance exception and must not be translated. Preserve API names, identifiers, protocol terms, and source quotations as needed.
- Check Git status, relevant files, and local instructions before editing. Preserve existing user work.
- Complete authorized work. Routine reversible local changes do not need another confirmation.
- Do not commit, push, or publish without a matching user request or authorization.
- Do not add dependencies, scaffolding, or an engine merely because a document mentions a future plan.
- Report results, actual verification commands, and material limitations. Never claim that a command or test ran when it didn't.

## Design direction to preserve

- Prefer entity, relation, state, behavior, constraint, and time as domain concepts; adapters handle vendor syntax.
- Distinguish raw observations, observed state, desired state, and simulated state. Every result derived needs provenance and a model scope.
- A simulation result applies only to modeled behavior. Do not claim firmware, ASIC, hardware, or all-version Packet Tracer equivalence without comparative evidence.
- The simulation kernel must run without UI or physical devices. UI consumes state/trace and sends commands through an explicit boundary.
- Separate the scheduler queue, execution journal, and GUI-visible event list.
- For replay, record event order, configuration, external input, model version, PRNG state, and timer state. Reverse animation is not proof of complete state restoration.
- Preserve packet bytes, field-to-byte/bit mappings, packet versions, and change causes in the tracker. Do not use a native struct layout as a wire format.

Details: [model and runtime](docs/wirebox/architecture.md), [simulation](docs/wirebox/simulation.md), and [byte tracker](docs/wirebox/byte-tracker.md).

## Languages ​​and boundaries between components

Phase 1–3 use Zig for the engine, Rust for the host/wrapper/oracle, and direct GPUI for UI. Contracts and reasons are in ADR-0002/0004/0007. GPUix + Solid remains historical alternatives, not the active stack. A smoke check is not a benchmark or proof that the full MVP is complete.

- Read [technology choices](docs/wirebox/technology.md) before selecting or changing the stack.
- The user-set baseline is Zig **0.17.0** minimum and Rust **1.99.1**. Rust is pinned in `rust-toolchain.toml`; record the exact Zig version tested. Do not lower the target for an older local toolchain. A newer Zig version requires compatibility testing.
- For Zig–Rust FFI, document ownership, buffer size, lifetime, error, ABI version, and thread contracts before expanding the API.
- Do not expose internal Zig/Rust containers as C ABI. Use ABI-defined representations and opaque handles where appropriate.
- Release memory through the correct allocator/owner. Rust's borrow checker does not verify Zig implementation safety.
- Do not allow panic/unwind across FFI. The UI must not retain an engine pointer that can become invalid after the next simulation step.
- Select and pin toolchain/dependency versions after checking documentation and building on the target platform. Do not invent versions or commands.

## Evidence and documentation discipline

- Distinguish **reported**, **locally verified**, **inference**, **proposal**, and **unknown** using the definitions in the [PT notes](docs/packet-tracer/README.md).
- For a new finding, record target/version/hash when available, method, input, result, date, and observation limits.
- PT function addresses and offsets apply to the reported binary. RTTI names and flow-chart strings do not recover a full algorithm.
- Do not revive the old AES/Mash hypothesis as the final `.pkt` explanation; read the [format correction](docs/packet-tracer/pkt-format.md).
- Preserve archived source copies with their hashes. Add a revised source as a new artifact and update the manifest instead of silently editing historical archives.
- Old evidence/manifest files record the baseline at their run time. Do not rewrite G0/G1 hashes to make later code appear validated; create new evidence.
- Keep current explanations in thematic documents and links to sources. Avoid long duplicate passages that can diverge.
- Use relative Markdown links. Date checks of external technology that may change.

## Learning and cognitive coverage

- The project skill is [.agents/skills/cognitive-coverage/SKILL.md](.agents/skills/cognitive-coverage/SKILL.md), with provenance pinned in [UPSTREAM.md](.agents/skills/cognitive-coverage/UPSTREAM.md). Personal output is in the Git-ignored `cognitive-coverage/` directory and is unavailable in a fresh clone. When generated locally, open `cognitive-coverage/cognitive-coverage-open.html` in a browser.
- For meaningful code changes, explain files changed/generated, invariants and decision reasons, commands actually run, and one teach-back question. Relate to backend concepts where useful without treating every error as transaction rollback.
- Code comments explain why, ownership, bounds, and commit points; Rust `SAFETY` comments state concrete FFI preconditions. Longer tutorials live in [learning docs](docs/learning/README.md).
- Do not mark human understanding as verified because an agent generated material or tests passed. Keep UI/CLI/vendor gaps visible. After source changes, check manifest source hashes; refresh affected artifacts or state that they are stale.
- Optional local artifact checker: `node scripts/check-cognitive-coverage.mjs` when output exists. It is separate from build/CI and engine tests and does not measure human understanding.

## Verification

For documentation-only changes, check local links, status consistency, and archive integrity when touched. Do not create a test suite for process changes.

Windows baseline build: `./scripts/verify.ps1 -CoreOnly` for kernel/runner, or `./scripts/verify.ps1` for the whole workspace. UI interaction remains a separate test. Do not mark remote CI as passed merely because its YAML exists.

When code exists, use commands from the actual manifest/build system. Select tests for the change: byte fixtures for codecs, ordering and cancellation for the scheduler, network scenarios for protocol behavior, and ownership/error paths for FFI. Verify UI and platform builds when those components are added.

Do not use trace-line count as scheduler-event count, demo success as conformance evidence, or another project's benchmark as a Wyrlex benchmark.
