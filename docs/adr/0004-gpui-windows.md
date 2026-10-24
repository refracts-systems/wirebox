# ADR-0004 — Direct GPUI on Windows x64

- Date: 20 October 2026
- Status: Accepted for the Phase 1 spike. Build and interaction results are recorded separately in the gate report.

## Context

The user compared GPUI with GPUix/Solid. A Zig kernel and Rust application need a dedicated topology/byte inspector.

## Decision

Use the published crate **gpui = 0.2.2**, `default-features = false`, with `windows-manifest`, and keep `Cargo.lock`. The platform baseline is `x86_64-pc-windows-msvc`, debug build. The package metadata names upstream commit `69e2130295c2649963eb639fc70b4f2ee8ea1624` with `dirty: true`; therefore build identity uses the **version and package checksum in Cargo.lock**, not an assumption of exact source identity.

The spike shows four selectable static nodes/links, a byte inspector, a command that reads a Zig batch, keyboard shortcuts, and shutdown. Displayed bytes come from the actual transport probe. The UI states that packet simulation is not yet available in this historical spike.

Use the API in the pinned package source. Main-branch documentation may have different crate/API boundaries. Phase 1 does not guarantee release packaging, other platforms, large scale, or all MVP requirements.

## Alternatives and consequences

GPUix/Solid remains an alternative if TSX experience is required or direct GPUI fails its tests. It adds another runtime boundary to the stack. Direct GPUI is a measured experiment, not a claim of benchmark superiority.

The UI build needs more native dependencies than the kernel; default workspace commands build only runner/kernel. `--workspace` or `-p wyrlex-studio` includes the UI. Build concurrency is limited to four on the 16 GB reference machine.

Sources: [GPUI](https://gpui.rs/), [package 0.2.2](https://crates.io/crates/gpui/0.2.2), [project technology](../wirebox/technology.md).
