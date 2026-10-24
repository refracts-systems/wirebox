# Runtime phase 2 — graph, virtual time, and journal

Implementations: [graph Zig](../../kernel/src/graph.zig), [runtime Zig](../../kernel/src/runtime.zig), [C ABI](../../kernel/include/wirebox_sim.h), and [wrapper Rust](../../crates/wirebox-kernel/src/simulation.rs). Normative decisions: [ADR-0003](../adr/0003-virtual-time-and-trace.md) and [ADR-0006](../adr/0006-phase-2-runtime.md). Test results: [G2](../evidence/phase-2/README.md).

The runtime API in this document runs simulated state and event counters. [Layer network phase 3](packets.md) uses the same graph/heap/timer and adds host/router/switch behavior via separate APIs. Coordinates, display labels, IP configurations, observed/desired states, and adapters are not included in the base graph. The fixture name is mapped to an ID by the host.

## Operate

```powershell
cargo run --locked -p wyrlex-headless -- --scheduler-demo
cargo run --locked -p wyrlex-headless -- --check-runtime
./scripts/verify.ps1
```

Demo of reading graph relations from fixture `icmp-static-v1`: 4 devices, 6 ports, 3 links. The runner schedules increments of counters, periodic random timers, canceled timers, and events on temporary objects that are then deleted. The JSON output contains traces and results; it does not yet run S01–S05 or send ICMP.

With seed 20260117, the event sequence is `2, 1, 3, 4, 7`, time is `5, 10, 10, 10, 15` ns. This sequence shows that the schedule ID and execution ordinal are different. Final counters PC1=1, SW1=104, R1=12, PC2=0. The periodic timer generates samples 33 then 71; event 7 points to cause 4. The run limit stops execution at 15 ns. Once the timer is cancelled, the empty horizon can advance to 100 ns.

## Graph and lifecycle

Entity ID `u64` starts 1, monotonic in one World, not recycled; invalid zero. IDs in other worlds cannot be used as global identities. `kind`: 1 device, 2 ports, 3 links. Role devices: 1 host, 2 switches, 3 routers.

- Device: `owner`, `a`, `b` zero.
- Port: `owner` designate device; `a`, `b`, `role` zero.
- Link: `a` and `b` designate two different ports; one port maximum one link.
- `counter` is a phase 2 synthetic state, not the number of packets/interface statistics.

A deleted device deletes its ports and the links that touch those ports. Deleted ports remove associated links. Removed links retain the endpoint. The timer for each deleted entity is immediately cancelled. Creating a new entity in an empty slot does not animate the old ID.

## Times, events and cancellations

Tick ​​`u64` nanosecond, initial zero. Heap sequence `(time_ns, event_id)`; The event ID starts at 1 and becomes a tie-break sequence. Past timestamps are rejected. The maximum ID is set aside to detect exhaustion before increment; The maximum time value remains valid for a one-shot if the action does not overflow.

`step()` runs one very early event. An empty queue returns reason `empty` without advancing the clock. `run_until(horizon, budget)` fires event `<= horizon`; After all the dues are finished, the clock becomes the horizon. Zero budget is legit. If the budget runs out while the due event is still available, the clock remains at the last event and the reason is `budget_exhausted`. Events after the horizon remain pending.

The timer has its own ID, registry, and one pending event. Zero interval = one-shot. Positive interval = periodic; next firing on `previous_time + interval`, new event ID, cause of previous event. Cancel removes pending events directly from the heap. A finished/canceled timer returns `not_found` when canceled again. The host can insert cancel/remove between bounded runs; there are no interrupt workers yet.

Action 1 `add_counter` adds amount. Action 2 `random_add_counter` adds sample `[0, amount)`, amount must be positive. SplitMix64-v1 with explicit seed/state pinned to ADR-0006; PRNG is only consumed when firing is successful. PRNG state does not depend on real time, GUI speed, thread schedule, or read journal.

Overflow on reset time, sequence/ID, execution count, or counter results in `overflow`. Firing failure does not change queue/state/clock/PRNG/journal. Runs that have previously completed another event retain that prefix; `RunError` Rust contains `error` and `progress`. Failed events can still be checked through the caller's schedule context and then canceled, or the object deleted. There is no queue-wide inspection API yet.

The serialization delay helper uses ceil(bytes × 8 × 1e9 / bitrate) with intermediate u128 and output checked u64. Phase 3 frame delivery uses this helper; wire representation limits are described in [package model](packets.md).

## Queue, journal, and visible list

The heap stores work that has not yet been executed. The ring journal stores successful executions, with `ordinal` starting at 1, event ID/time/target/timer/cause/action/amount, and before/after counters. Cancels, schedules, and deletions from hosts do not become journal records at this stage; Therefore this journal has not yet event-sourced the world or audited all commands.

Retention 0–256 can be selected at create. Full ring overwrites the oldest record and raises `dropped`; zero retention keeps running the event. `trace(after_ordinal, capacity)` copies retained records with an ordinal greater than the cursor, does not consume them. Cursor zero reads from the beginning which is still available. The gap is identified via the first ordinal and snapshot `dropped`; cause can point to records that have been lost.

The visible list is a host projection of the journal copy. Demo reading one row projection; test compares reads each step with reads all at once. [Packet Studio Phase 4](studio.md) uses a Network controller, trace/packet/field copy and bounded execution control. Trace is limited and repeat results from the same seed do not prove checkpoint/restore phase 5.

## Contract C ABI runtime v1

The `wb_sim_*` version 1 API is independent of the `wb_*` v1 probe. Revision [ADR-0007](../adr/0007-packet-model.md) uses the top two bit namespace (probe 00/counter 10/network 11), generation 30 bit, and index+1 to prevent stale/cross registry access; generation exhausted retired. This replaces the one bit/31 bit split on the baseline G2. Registry maximum 8 World. Each World has a maximum of 128 entities, 256 pending events/timers, 256 journal records. This limitation is visible through errors, not silent drop events.

| Function | Behavior |
|---|---|
| `wb_sim_version` | Query version 1 |
| `wb_sim_create` | Version, seed, retention; zero handle on error if output is valid |
| `wb_sim_destroy` | Close World and invalidate generation |
| `wb_sim_add` | Device/port/link; ID zero on failure, invalid input does not take ID |
| `wb_sim_remove` | Delete entity with cascade/cancel |
| `wb_sim_schedule` | Copy spec, return timer ID; one-shot zero interval |
| `wb_sim_cancel` | Cancel one active timer |
| `wb_sim_step`, `wb_sim_run_until` | Copy progress including prefix on error |
| `wb_sim_snapshot` | Copy runtime metadata; not a complete checkpoint |
| `wb_sim_entities` | Copy graph complete or capacity error; Rust sorts them by ID |
| `wb_sim_trace` | Batch copy after ordinal, non-consuming; capacity zero no-op |

`u32` status: 0 ok, 1 invalid_argument, 2 abi_mismatch, 3 invalid_handle, 4 capacity, 5 not_found, 6 conflict, 7 past_time, 8 overflow. Progress reasons: 0 empty, 1 horizon, 2 budget_exhausted, 3 stepped, 4 errors. Errors other than run do not promise progress. Null in the output must be invalid; count/ID output is initialized to zero if the pointer is valid. Snapshots are zero-initialized before handle validation.

All pointers must be valid, aligned, long enough, non-overlapping. Null arrays are only valid when capacity is zero. Maximum entity capacity 128, maximum trace 256. Entity output too small returns capacity without partial copy. All input is copied at call; all output belongs to the caller. No World pointers, allocators, callbacks, or native containers pass through the ABI.

The C caller must serialize all three registry process-wide calls. Rust's wrapper uses the same mutex as Session probe and Network, and World is `!Send + !Sync`. Vec/struct results remain alive after World closes; explicit close or Drop destroys the World once. Lock includes one bounded call; the caller must choose a budget that suits the application response.

Windows x64 layout, alignment 8:

| Records | Byte size | Main field/offset |
|---|---:|---|
| Entities | 48 | id/owner/a/b/counter u64 @0/8/16/24/32; kind/role u32 @40/44 |
| Events | 56 | id/time/target/timer/cause/amount u64 @0..40; action/reserved u32 @48/52 |
| Trace | 80 | Event @0; ordinal/before/after u64 @56/64/72 |
| Schedule | 40 | target/time/interval/amount u64 @0..24; action/reserved u32 @32/36 |
| Progress | 24 | now/processed u64 @0/8; reason/reserved u32 @16/20 |
| Snapshots | 80 | 8 u64 @0..56; 4 counts u32 @64..76 |

Reserved must be zero. This in-process ABI is not a package format, file persistence, or promise of cross-endianness/version compatibility. Snapshot stores now, seed, rng_state, next_event_id, next_timer_id, executed, dropped, last_event_id, entity/pending/timer/journal count; does not load all queues, configurations, or states to restore.
