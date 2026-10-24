# ADR-0003 — Reproducible virtual time and trace

- Date: 20 October 2026
- Status: Accepted, implemented in Phase 2; budget/PRNG/retention details are in ADR-0006 and the G2 report.

## Context

The PT scheduler and GUI log are reported as separate components. Wyrlex also needs causal ordering and stronger restore semantics than replaying an animation.

## Decision

Use `u64` nanosecond ticks starting at zero, ordered by `(time_ns, sequence)`. Sequence monotonicity and overflow are checked. Scheduling in the past is rejected. `step()` processes one scheduler record; an empty queue does not advance time. `runUntil(t)` executes records timestamped `<= t`, then advances time to `t`; `t < now` is an error. Event/cancel bounds prevent a periodic timer from blocking execution.

Start with one owner of state execution. Pending events use serialisable stable records/IDs rather than native function or closure pointers in a checkpoint. Timers have IDs and a cancellation lifecycle. Pin the PRNG algorithm with the Phase 2 implementation before claiming random determinism; seed and state belong in a checkpoint.

Separate the scheduler queue, journal, and visible event list. GUI retention must not change execution. For serialisation delay, use integer ceiling from bits/bitrate and check overflow. GUI/realtime time only controls pacing.

## Alternatives and consequences

Floating-point time and direct wall-clock use are easy but complicate reproducibility. Arbitrary callbacks help a prototype but make checkpoints difficult. Typed records require explicit dispatch as the cost of inspectable replay.

This is a Wyrlex convention, not an imitation of the old mini-simulator's `runUntil` semantics. Replay is guaranteed only for the declared model/build/environment until cross-version compatibility is demonstrated.

References: [simulation](../wirebox/simulation.md), [mini-simulator and limits](../packet-tracer/mini-simulator.md).
