#ADR-0006 — Phase 2 runtime contract

- Date: 20 October 2026
- Status: Accepted; implementation and gate evidence are linked from the Phase 2 report.

Expansion on 21 October 2026: [ADR-0007](0007-packet-model.md) adds packet-domain dispatch and a handle namespace. The atomic counter-firing contract remains valid for counter actions; network-dispatch failures have a separate failed state.

##Decision

The Zig runtime stores **simulated state**. Device, port, and link entities use one monotonic per-world `u64` ID namespace; zero is invalid and IDs are never reused. Host/switch/router role is metadata. Each port has one device owner. A point-to-point link joins two distinct unlinked ports; UI coordinates are not part of the graph.

Deleting a device deletes its ports and links touching those ports. Deleting a port deletes its links. All events/timers owned by a deleted entity are canceled before storage is reused. Deleting a link does not delete endpoints.

The scheduler uses a binary min-heap `(time_ns, event_id)`; event ID is also the sequence. The timer registry keeps one pending event per timer. A zero interval is one-shot; a positive interval schedules the next firing from the previous firing time, with a new event ID and the newly executed event as `cause_id`. Host schedules use cause zero. Cancel removes the event from the heap directly and leaves no capacity-consuming tombstone.

Time follows ADR-0003. `runUntil(t, budget)` advances to `t` only after all due events have finished. If the budget ends while an event is still due, it returns `budget_exhausted` at the last event time; a zero budget is valid. An empty `step` does not advance. Bounded calls may be interleaved with cancel/remove; this phase has no worker or asynchronous interrupt.

Time, ID, counter, and execution-counter overflow are checked. A periodic firing that cannot be rescheduled is rejected **before** dequeue, state change, PRNG use, time change, or journaling. A run may have completed earlier events when a later event fails; progress output and snapshots show the successful prefix.

The PRNG is pinned to **SplitMix64-v1**: initial state equals the seed; wrapping increment `0x9e3779b97f4a7c15`, mixers `0xbf58476d1ce4e5b9` and `0x94d049bb133111eb`, and shifts 30/27/31. This matches the inspected Zig 0.17.0 `lib/std/Random/SplitMix64.zig`. Sampling `[0, upper)` uses rejection with threshold `(0 -% upper) % upper`; zero upper is invalid. Seed/state are inspectable, but complete restore/checkpoint remains Phase 5.

Phase 2 actions are only `add_counter` and `random_add_counter`: typed events prove dispatch/state/PRNG, not network protocols. Randomness is consumed at execution, never during schedule/cancel/read.

The queue, execution journal, and display list are separate. The journal stores ID/time/target/timer/cause/action/arguments and before/after counters. Ring retention is 0–256; overwrite increments dropped count without changing execution. Reads copy without consuming. A high-ID event can execute first when its timestamp is lower; read cursors use the execution ordinal `ordinal`. A cause may refer to a record already evicted by retention. A bounded journal is not a complete checkpoint.

## ABI and initial capacity

The ABI v1 probe remains available. The runtime has a separate ABI `wb_sim_*`, version 1, separate registry handles, copy ownership and the same process-wide mutex on the Rust host. Field, pointer, error, layout, and ownership contracts are in [runtime phase 2](../wirebox/runtime.md).

Explicit limits: 8 worlds, 128 entities per world, 256 pending timers/events, 256 maximum journal records. Zig's storage bounded, no allocator across the ABI. `capacity` is a visible error, not a silent drop. This is the foundation limit for phase 2; need to evaluate capacity before the MVP target is large.

## Alternatives and consequences

A linear registry and O(n) cancellation are sufficient for this limit, with an O(log n) heap on schedule/pop. Dynamic structures/hash indices can be added once scale requirements are measured. Events in the form of records enable the evolution of serialization; C layout is not a persistence format. There are no claims of save/load, restore, vendor fidelity, or package simulation from the results of this gate.
