#ADR-0002 — Zig kernel, Rust host, and C ABI v1

- Date: 20 October 2026
- Status: Accepted; the probe, Phase 2 graph/scheduler runtime, and [Phase 3 network ABI](../wirebox/packets.md) are implemented. Network-contract expansion follows ADR-0007.

## Context

The user set Zig 0.17 minimum and Rust 1.99.1. A testable boundary is needed before the graph, protocols, and UI grow.

##Decision

Use Zig **0.17.0** as the verified build pin and Rust **1.99.1**. The product baseline remains Zig 0.17 minimum; newer releases require evaluation before changing the pin. The Cargo build script builds the Zig static library for Windows x64 MSVC. `WYRLEX_ZIG` can select a compiler explicitly.

The v1 contract is in the [C header](../../kernel/include/wirebox.h). A session is a `u64` handle with index/generation; zero and stale generations are invalid. The probe registry has 16 sessions, each with a 16-record queue and a 64-byte maximum payload. A full registry returns a capacity status. This is **probe** capacity, not an engine or packet-MVP promise.

Input is copied into Zig and output is copied into caller buffers. The host stores no borrowed pointer and no allocator crosses the language boundary. The ABI uses fixed-width integers, `size_t`, checked extern structs, and `u32` status values; it does not expose cross-ABI booleans, Rust containers, Zig slices, exceptions, or host callbacks.

The caller serialises all registry operations process-wide. The Rust wrapper uses a mutex and a `!Send + !Sync` session. Read results are owned by Rust and may outlive session close. Drop never unwinds across the ABI. A non-null C pointer must still be valid, aligned, and sized according to the contract; the API cannot prove arbitrary-address validity.

## Alternatives and consequences

Rust-only would reduce the toolchain but would ignore the user's kernel preference. Shared native structs/pointer views reduce copying but expand lifetime risk. A bounded-copy probe proves the contract first. The static registry is not the scheduler architecture; simulation API revisions require ABI review and must not equate probe events with network events.

Details: [FFI contract](../wirebox/ffi.md). Linking follows [Cargo build scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html); native syntax follows [Zig 0.17](https://ziglang.org/documentation/0.17.0/).
