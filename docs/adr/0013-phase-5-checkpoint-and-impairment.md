# ADR-0013: Value-owned Phase 5 checkpoints and deterministic impairment

- Status: Accepted for the Phase 5 implementation
- Date: 24 October 2026
- Scope: Wyrlex packet network simulator

## Context

Phase 4 reruns a changed scenario from the beginning. Phase 5 needs to stop while protocol work is active, resume from that point, and compare a branch with the original run. The checkpoint must include more than the visible packet: topology, protocol caches, pending timers, queue order, packet versions, trace retention, counters, and PRNG state. The simulator also needs controlled loss and jitter that can be replayed from a seed.

## Decision

The Zig network state is a fixed-size, pointer-free value. A checkpoint is an independently allocated byte-for-byte copy of that value behind an opaque handle in a namespace separate from network handles. The API supports:

- create and destroy a checkpoint;
- restore a live network in place;
- fork a new live network from an immutable checkpoint;
- read checkpoint metadata containing model version, external input kind/port, clock, PRNG state, and pending event count.

The copy uses direct byte transfer between heap objects. This avoids materializing the large packet/trace arrays as a temporary on the Windows process stack. The Rust wrapper owns the opaque checkpoint and destroys it on `Drop`; no Zig pointer or internal container crosses the C ABI.

Link impairment is configured before the first workload. Loss is an integer probability in parts per million and jitter is a bounded nanosecond interval. Enabled impairment consumes the seeded SplitMix64 stream, records draw/drop/delay counters, emits a `link_loss` trace reason for dropped transmissions, and is therefore part of checkpoint state. The wall clock is never consulted.

Hardening on 24 October defines loss after serialization: a lost frame reserves its egress interval and creates no arrival event or transmission record. The loss trace is emitted at the sending decision. Surviving frames add independent uniform jitter in `[0, jitter_ns]`, which may reorder arrivals. This is a global network impairment model, not a congestion or physical-layer model. Transmit stages the PRNG and noise counters until time checks and scheduler reservation succeed; a committed loss publishes its draw and egress reservation together. This does not make an entire network dispatch atomic.

A recursive compile-time check rejects pointers, slices, and unsupported aggregate types in `Network`; adding owned heap structures requires replacing the copy implementation. Checkpoint and network registries each allow four live objects. Checkpoints may precede input, contain active work, represent a completed run, or preserve a failed network for diagnostics. Restore preserves that failure marker. A healthy checkpoint can recover a failed target. Handle and metadata outputs are zeroed on validation failure.

Checkpoints are process-local and never exposed as serialized struct bytes. Restore rewinds event, packet, and trace IDs; the host must discard cursors and projections from the replaced timeline. Portable persistence, cross-version migration, UI checkpoint controls, and live topology edits remain outside this implementation.

## Consequences

The same checkpoint can be restored repeatedly or forked into independent branches. A branch comparison can include summary, packet bytes, packet ancestry, and retained trace rows. Retained traces remain bounded; `trace_dropped` reports history that is no longer available. Checkpoint handles have bounded capacity and generation retirement, so stale checkpoint handles cannot alias a later copy.

This decision proves deterministic local replay and controlled disruption. It does not prove replay across engine/model versions, firmware, or a real Linux/MikroTik path. Lab capture and comparator tolerances remain Phase 5 work.

## Verification

The headless command `cargo run --locked -p wyrlex-headless -- --check-phase5` checks active ARP checkpointing, fork equality, restore equality, seeded jitter equality, and an all-loss ARP timeout. Kernel and Rust tests also check layout, ownership, and frozen impairment configuration. The command reports `"lab_comparison": "not_run"` until a lab fixture is captured.
