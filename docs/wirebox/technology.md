# Zig, Rust, GPUI, and GPUix

Implementation status as of October 21, 2026: **Zig for simulation engine, Rust for wrapper/host/oracle, live GPUI for UI**. Phase 3 adds Ethernet/ARP/IPv4/ICMP and a headless byte tracker to the phase 2 runtime; [evidence G3](../evidence/phase-3/README.md). GPUI interaction is still spike probe phase 1. Studio integration, benchmarks, and release packaging are not yet available. Decision on [ADR](../adr/README.md); previous results [G0/G1](../evidence/phase-0-1/README.md) and [G2](../evidence/phase-2/README.md).

## Baseline version set

| Components | User settings | Implementation in repository |
|---|---|---|
| Zig | Minimum **0.17.0** | Build probe pinned and verified on 0.17.0 by [build.rs](../../crates/wirebox-kernel/build.rs); newer versions need to be tested before pin changes |
| Rust | **1.99.1** | Pinned on [rust-toolchain.toml](../../rust-toolchain.toml), minimal profile with rustfmt and Clippy |

The probe code has been built with both. Zig's minimum product limits do not guarantee that the next release will be compatible without API/build system changes; The current build implementation uses pin exact 0.17.0. Windows Workflow verifies its download checksum and Cargo.lock locks the Rust dependency.

Official source check on October 19, 2026: [Zig download page](https://ziglang.org/download/) lists 0.17.0, released October 1, 2026. Rust link `releases/latest` when checked leads to [Rust 1.99.1 announcement](https://blog.rust-lang.org/2026/10/15/Rust-1.99.1/), released October 15, 2026. Use a fixed version number for project builds, not the alias `latest` which moves.

## Local installation and verification

On October 19, 2026, at user request, both compilers were installed and tested on Windows x86_64:

| Components | Verification results |
|---|---|
| Zig | `zig version` → `0.17.0` |
| Rust | `rustc --version` → `rustc 1.99.1 (58cf8f9c3 2026-10-13)` |
| Cargo | `cargo --version` → `cargo 1.99.1 (5f94df478 2026-08-27)` |
| rustfmt | `rustfmt --version` → `rustfmt 1.10.0-stable` |
| Clippy | `cargo clippy --version` → `clippy 0.1.99` |

Zig is installed in `C:\Users\ricky\AppData\Local\Programs\Zig\zig-x86_64-windows-0.17.0` and the directory is added to the user's PATH. The Windows x86_64 package is taken from the official Zig download manifest. SHA-256 archive matches manifest: `b5663f69581dcf391293fbf16c06cb80d81d806545ce618b4d0bab7f0eb8c428`.

Rust installed via rustup as `1.99.1-x86_64-pc-windows-msvc`, with rustfmt and Clippy. Pin repository selects this version; The previous global stable default was not changed.

A small program to check the interpretation of two bytes in big-endian order was successfully compiled and run with the respective compilers. The Rust program also proves linking using the available MSVC environment. Source and test executables are stored in a temporary directory outside the repository. This is a compiler installation check, not a test of the Wyrlex engine or Zig–Rust/GPUI integration.

Terminals/applications that are already open may need to be reopened to retrieve the new Zig PATH. Check from root repository:

```powershell
zig version
rustc --version
cargo --version
```

## Zig and Rust for byte visibility

Zig emphasizes control flow as well as explicit allocation. These characteristics match user preferences for understanding buffer management and engine behavior. The language documentation explains the runtime safety philosophy and options. [Zig overview](https://ziglang.org/learn/overview/).

Borrowing Rust manages the use of references and ownership; it does not remove access to the bytes. Byte buffers, slices, explicit codecs, and field mapping can be used in Rust. Tracker transparency still requires recording changes and their reasons in both languages.

Project design considerations:

| Options | Reasons for choosing | Costs that need to be tested |
|---|---|---|
| Zig as kernel | Control buffers, allocators, and representations to suit user preferences | Ownership, lifetime, and invariant engines need to be managed with discipline |
| Rust as a kernel and application | One language and compiler-checked ownership contract in appropriate Rust code | Graph, arena and state mutation modeling need to be designed to be easy to read |
| Zig kernel + Rust application | Preserves Zig preferences and direct access to the Rust UI ecosystem | Two toolchains, FFI, packaging, and cross-language debugging |

This table is a design assessment, not a measurement that one language is always faster or easier. ID/arena based data models can be considered in both languages ​​for long-lived graphs and states.

## Proposed division of responsibilities

```text
Rust application + GPUI
  topology / inspector / timeline / application commands
                    |
          Rust wrapper untuk C ABI
                    |
Zig simulation kernel
  scheduler / packet buffers / codecs / protocol state / trace
```

The division of planner, persistence, and device drivers between modules/languages ​​is not final. The kernel must be testable via a headless runner before relying on the UI. PDUs and protocol state do not need to have copies of the model that can be mutated independently in each language.

## Zig–Rust Contract

The [runtime](runtime.md) and [network](packets.md) APIs now provide opaque handles, pre-run configuration, bounded execution, batch trace reading, as well as package/field version copies. Command UI/persistence and active state changes still require the next phase contract.

The contract must answer:

- Who allocates and releases each buffer/handle?
- How long is the pointer/view valid, and is the engine allowed to run while the view is in use?
- How are length, capacity, alignment, enum, and error status represented?
- How are invalid handles, cancels, and simulation termination handled?
- Which threads have state and does the API allow callbacks/reentrancy?
- How are ABI versions, panic/failure, and build compatibility handled?

Start with simple limits that are easy to verify, such as copying small batches to the application's own buffer. Zero-copy is an optimization after the lifetime contract is clear. Rust wrapper does not prove memory safe in Zig implementations.

## GPUI versus GPUix

[GPUI](https://gpui.rs/) is a Rust UI framework with GPU rendering. The source provides the [canvas for custom painting](https://raw.githubusercontent.com/zed-industries/zed/main/crates/gpui/src/elements/canvas.rs) element. This provides a starting point for drawing the topology; Graph layout, hit testing, pan/zoom, and optimization remain Wyrlex's work.

[GPUix](https://github.com/remorses/gpuix) wraps GPUI for React/Solid with a TypeScript/JavaScript application. The documentation reviewed provides the official Solid adapter, states pre-1.0 status, requests the adapter/native version pinned together, and explains the need for culling by the application on the graph node or timeline. [README GPUix](https://raw.githubusercontent.com/remorses/gpuix/main/README.md).

| Wyrlex Needs | GPUI direct | GPUix + Solid |
|---|---|---|
| Applications that already use Rust | The UI is in the application language | Requires a bridge to the JS | runtime
| UI writing preferences | Rust APIs | TSX components and Solid reactivity |
| Zig Integration | Via the C ABI wrapper in Rust | Still need a native/IPC path to the kernel |
| Custom topology and tracker | Access the GPUI drawing API | Need to check the APIs the bindings expose on the selected version |
| Evidence costs | Build, input, drawing, packaging | Same thing plus binding/runtime integration |

GPUI's recommendation stems from suitability for Rust application direction and specific inspector/canvas needs. This doesn't prove GPUI wins in the Wyrlex benchmark. GPUix is ​​relevant if Solid/TSX productivity is a big enough priority to justify the extra layer.

## Correction notes and due diligence

Previous discussions mentioned specific versions as well as GPUix canvases that are still being planned. The information is a historical snapshot. The rechecked README now also contains pixel input paths for drawing; the current state of the capability needs to be determined from the selected API and demo version. Don't use the old claim about the absence of a canvas as the only reason for choosing a framework.

Basic Windows phase 1 feasibility tests have been carried out: GPUI 0.2.2 window, selectable node/link fixture, hex inspector from Zig batch probe, keyboard input, and close/shutdown. On October 23, 2026, initial integration of [Packet Studio](studio.md) using the package simulator and timeline/field/hex; [evidence Phase 4](../evidence/phase-4/README.md) separates controller results, native interactions and measurements that have not yet been performed. See also [phase 1 results](../evidence/phase-0-1/README.md) and [v1 contract](phase-0-contract.md) for frozen acceptance targets.

Web sources checked on October 19, 2026; `main` branch link may change. As implementation begins, record the actually built revisions and their results in [decisions](decisions.md).
