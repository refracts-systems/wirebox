# Wyrlex

Wyrlex is an infrastructure studio and runtime for inspection, simulation, packet tracing, and planned change over a shared semantic model. Read the [product concept](docs/wirebox/README.md), [Phase 1–4 learning notes](docs/learning/README.md), [ADRs](docs/adr/README.md), [roadmap](docs/wirebox/roadmap.md), and [phase todos](docs/wirebox/todos.md).

## Implementation status

Phases 0–3 are available for the local scope: contracts/fixtures, Zig–Rust/GPUI integration, virtual time, and a **headless Ethernet/ARP/IPv4/ICMP simulator with static routing and a byte tracker**. The initial [Packet Studio Phase 4](docs/wirebox/studio.md) integration shows live trace and bytes, Run/Pause/Step/Reset controls, configuration branches, and versioned session save/load. [Phase 4 technical acceptance](docs/evidence/phase-4-acceptance/README.md) passed for the declared fixture and handler-to-render scope; human usability is still unknown. The [packet model](docs/wirebox/packets.md) covers successful ping, missing route, expired TTL, ARP timeout, and invalid input. Wyrlex models network behavior; it does not require a vendor OS image.

Phase 5 adds value-owned checkpoint/fork/restore and deterministic seeded loss/jitter. [G5 passes for the declared replay/Linux scope](docs/evidence/phase-5-gate/README.md): five replay seeds, three repetitions, and independent forwarding/failure/recovery comparisons. Evidence-contract extensions and vendor pilots remain open.

The target is Windows x64 with Zig **0.17.0** as the verified baseline (product minimum 0.17), Rust **1.99.1**, Visual Studio C++ Build Tools, and the Windows SDK. GPUI is pinned at **0.2.2** by `Cargo.lock`. rustup selects Rust through `rust-toolchain.toml`.

## Operate

Run these commands from the repository root in PowerShell:

```powershell
# If zig.exe is not on PATH, set its installation path:
$env:WYRLEX_ZIG = 'C:\Users\ricky\AppData\Local\Programs\Zig\zig-x86_64-windows-0.17.0\zig.exe'

cargo run --locked -p wyrlex-headless -- --probe
cargo run --locked -p wyrlex-headless -- --check-fixtures
cargo run --locked -p wyrlex-headless -- --scheduler-demo
cargo run --locked -p wyrlex-headless -- --check-runtime
cargo run --locked -p wyrlex-headless -- --packet-demo S01
cargo run --locked -p wyrlex-headless -- --check-network
cargo run --locked -p wyrlex-headless -- --check-phase5
cargo run --locked -p wyrlex-headless -- --corpus-demo N01
cargo run --locked -p wyrlex-studio
```

Adjust the Zig path on another machine. The build script also searches the normal `LOCALAPPDATA` installation when available. Cargo builds the kernel into `OUT_DIR`; no manual static-library build is required.

In the Studio window, choose S01–S05, use **Ctrl+R** for Run, **Space** for Pause, **F10** for one scheduler step, and **Ctrl+N** for Reset. Select a node or link to filter the trace, a trace row to inspect its packet, or a field/byte to highlight related data. Before/After shows packet-byte versions. **Ctrl+S/Ctrl+O** saves/opens a session, **Esc** clears selection, and **Ctrl+Q** exits. The topology is still a four-node fixture; the [Studio guide](docs/wirebox/studio.md) documents configuration and the S02 route-repair workflow.

## Verify

Phase 0–3 hardening and the route-validation fix are explained in the [learning notes](docs/learning/hardening_phase_0_3.md), with [test results](docs/evidence/hardening-phase-0-3/README.md).

The first [behavior corpus](corpus/README.md) includes a WSL Linux namespace experiment and version-scoped capture records. [S01–S03 comparison evidence](docs/evidence/corpus-linux/README.md) covers three repetitions per case. The [expanded G5 comparison](docs/evidence/phase-5-gate/README.md) adds ARP failure, route recovery, and checksum rejection; vendor pilots remain open. Run `./scripts/corpus/run-linux-lab.ps1` to create a new local record.

Personal Cognitive Coverage material (HTML guide, quiz, dashboard, and learning progress) is stored **locally only** in the Git-ignored `cognitive-coverage/` directory. It is not available from GitHub or a fresh clone. If it has been generated locally, open `cognitive-coverage/cognitive-coverage-open.html` in a browser. The project skills and Markdown learning notes remain in the repository; the generated material adds no runtime dependency.

```powershell
# Kernel, C layout, wrapper, fixtures, formatter, and Clippy
./scripts/verify.ps1 -CoreOnly

# Default Zig tests: Debug + ReleaseSafe. Add ReleaseFast when required.
./scripts/verify.ps1 -CoreOnly -ZigModes Debug,ReleaseSafe,ReleaseFast

# Include desktop build/test/lint without opening a window
./scripts/verify.ps1

# Build from a clean source copy using the existing dependency/compiler cache
./scripts/verify-clean-export.ps1

```

When the local learning folder exists, optionally run `node scripts/check-cognitive-coverage.mjs` to check source hashes, anchors, and quiz mappings. This is not a build or CI requirement. Source or documentation changes can make the local manifest stale; refresh affected material before treating it as current.

The first GPUI build downloads and compiles many dependencies. The default workspace commands cover kernel/headless; `--workspace` includes desktop. The scripts report the status of each native command. The Windows workflow in `.github/workflows/verify.yml` is configured; the presence of a workflow does not mean that a remote job has run.

Visual inspection, input, and window-close checks are separate runtime verification. Actual results are recorded in [Phase 0–1 evidence](docs/evidence/phase-0-1/README.md), [Phase 2](docs/evidence/phase-2/README.md), [Phase 3](docs/evidence/phase-3/README.md), and [Phase 4 integration](docs/evidence/phase-4/README.md). The application uses a debug build; default kernel tests cover Debug and ReleaseSafe. Agent technical G4 passed on 24 October 2026 within the [ADR-0010 scope](docs/adr/0010-agent-acceptance-and-studio-measurement.md). [G5 passed for scoped replay and Linux comparison](docs/evidence/phase-5-gate/README.md); release distribution and vendor pilots remain open.

## Structure

- `kernel/`: C ABI, probe, graph, scheduler, PRNG, packet codecs, protocols, and the Zig journal.
- Rust package `wyrlex-kernel`: ownership/error/thread wrapper and Zig build integration.
- Rust package `wyrlex-headless`: CLI probes, fixtures, scheduler/packet demos, and the independent acceptance oracle.
- Rust package `wyrlex-studio`: GPUI Packet Studio, copied-snapshot controller, configuration branches, and versioned sessions.
- `fixtures/`: synthetic scenarios, expected bytes, and sanitized user-reported inventory.
- `docs/`: design, decisions, phases, evidence, and historical reverse-engineering sources.

Rust packages live under `crates/`. Source/documentation paths and C header filenames retained for compatibility are documented in [ADR-0011](docs/adr/0011-wyrlex-product-rename.md).

Contribution guidance is in [AGENTS.md](AGENTS.md). Do not commit credentials, unsanitized customer captures, or vendor binaries.
