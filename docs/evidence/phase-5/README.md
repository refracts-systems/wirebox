# Phase 5 evidence — checkpoint, replay, and controlled disruption

Updated 24 October 2026 on Windows x64 MSVC with Zig 0.17.0 and Rust 1.99.1. This report records the local implementation scope. It does not claim G5 or vendor equivalence.

This is the initial implementation baseline. Subsequent fixes, expanded replay coverage, current commands, and source hashes are recorded separately in [Phase 5 hardening](../phase-5-hardening/README.md); the original logs below are preserved.

## Implemented and locally verified

- P5-01: a checkpoint copies the pointer-free network value, including topology, pending scheduler events and timers, packet versions, ARP/MAC caches, trace ring, counters, input metadata, impairment configuration/counters, and PRNG state.
- P5-02: a checkpoint can restore a live network in place or fork an independent network handle. The two runs have equal summary, packet bytes/ancestry, and retained trace rows in the declared fixture.
- P5-03: seeded loss and nanosecond jitter are explicit inputs. Draw, drop, and delay counters are copied by checkpoint and a dropped link emits a `link_loss` trace reason.
- P5-04: the check stops immediately after `ping`, while ARP work and two scheduler events are pending, then compares normal, forked, and restored continuation.
- P5-06: the local comparator checks the complete observable result in scope: summary, packet records, and retained trace records.

## Reproduction

```powershell
cargo test --locked -p wyrlex-kernel --lib
cargo test --locked -p wyrlex-headless
cargo run --locked -p wyrlex-headless -- --check-phase5
```

Captured outputs: [headless Phase 5 JSON](check-phase5.log) and [core verification log](verification.log).

The final command returned:

```json
{
  "phase": 5,
  "result": "pass",
  "checkpoint": {"fork_matches": true, "restore_matches": true, "pending_events": 2},
  "jitter": {"replay_matches": true, "draws": 4, "delayed": 4},
  "loss": {"loss_per_million": 1000000, "dropped": 3, "outcome": 4, "reason": 102},
  "lab_comparison": "not_run"
}
```

The Zig suite passed all 39 tests in Debug mode during this implementation. The Rust kernel suite contains 15 tests and the headless suite contains 5 tests; both passed after the stack-safe copy fix.

## Boundary and limitation

The first standalone headless run exposed a Windows stack overflow caused by assigning the large fixed-size `Network` value as a struct temporary. Checkpoint create, restore, and fork now copy initialized bytes directly between heap objects. This case is retained as a design constraint and is covered by the standalone command.

P5-05 is still open: the Linux lab fixture and capture have not been run. Therefore G5 remains incomplete. The local fixture proves deterministic simulator replay only; it does not establish equality with Linux, MikroTik RouterOS, Packet Tracer, or vendor firmware.
