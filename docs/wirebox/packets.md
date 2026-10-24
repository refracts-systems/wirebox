# Headless packet and byte tracker simulation — phase 3

Model `ethernet-arp-ipv4-icmp-v1` runs `PC1 — SW1 — R1 — PC2` entirely on Zig. Rust loads the fixture, calls the C ABI, checks the results, and exports JSON. No physical device, OS image, or `.pkt` file is required. Contract: [ADR-0007](../adr/0007-packet-model.md); results and evidentiary limits: [G3](../evidence/phase-3/README.md).

## Execute and read results

```powershell
cargo run --locked -p wyrlex-headless -- --packet-demo S01
cargo run --locked -p wyrlex-headless -- --packet-demo S02
cargo run --locked -p wyrlex-headless -- --check-network
```

The demo accepts `S01`–`S05` and `alias`; anonymously wearing S01. The output contains name→entity ID, summary, decision trace, all byte versions along with fields, and transmission. Complete examples are available at [artifact S01](../evidence/phase-3/S01.json) and [byte walkthrough](../evidence/phase-3/byte-walkthrough.md). As of October 23, 2026, [Packet Studio](studio.md) interfaces trace/field/hex with simulation control via a Rust controller; [evidence of Phase 4 initial integration](../evidence/phase-4/README.md) does not automatically pass all MVP gates.

| Case | Checked behavior |
|---|---|
| S01 | Echo reply from PC2; request/reply passes through switches and routers |
| S02 | The static route alias on R1 is removed; ICMP 3/0 received PC1 |
| S03 | TTL request 1; ICMP 11/0 received by PC1, request not forwarded |
| S04 | PC2 port disabled; three ARP requests, timeout, pending buffer freed |
| S05 | IPv4 input `4500`; invalid_packet with reason truncated_ipv4 |
| aka | Healthy static route to 198.51.100.2/32 selected and echo reply received |

Summary `now_ns` is the requested horizon (5 seconds in the demo), not the ping latency. The actual reception time is in trace `delivered`. The number of row traces is different from the number of event schedulers; a single event can result in multiple decisions and package versions.

## Engine and model coverage

[network.zig](../../kernel/src/network.zig) uses graph, `(time_ns, event_id)` heap, and registry timer [runtime](runtime.md). Event 100 carries the transmission ID for frame reception; event 101 carries the pending ARP ID for retry/timeout. Synchronous initial workload creation at time 0 with `event_id=0`; child events designate the ID of the event that scheduled them as the cause.

One Network is configured before one `ping` or `inject`. After workload starts, graph/address/route/link is frozen; change scenario is created as a new Network. `run_until(horizon, budget)` can be called repeatedly, including a budget of 1 for stepping. Reading the output does not change execution. Phase 5 adds process-local checkpoint/restore/fork and seeded loss/jitter; configuration mutations during a running workload and worker threads remain unavailable. See [ADR-0013](../adr/0013-phase-5-checkpoint-and-impairment.md).

| Section | Implementation and limits |
|---|---|
| Ethernet II | Explicit decode/encode; MAC learning per switch; flood broadcast/unknown destination, forward known unicast. No VLANs, STP, aging, or collisions |
| Links | Full duplex, serialization queue per egress port; surviving-frame arrival = start + ceil(bytes × 8 × 1e9 / bitrate) + propagation + optional jitter. Loss still reserves egress serialization time. Global seeded loss/jitter is configurable before input. No congestion/queue drop model |
| ARP | Ethernet/IPv4 request/reply; cache per port; the package is waiting for resolution. Request at 0/1/2 seconds relative lookup, timeout at 3 seconds; timer is canceled if resolved. Without aging/gratuitous ARP |
| IPv4 | IHL 5, MTU 1500, connected/static route longest-prefix match. Equal prefix selects connected routes first; duplicate static prefix rejected, without ECMP. Forwarding lowers the TTL and recalculates the checksum |
| ICMP | Echo request/reply code 0, identifier/sequence/data retained; error network unreachable 3/0 and time exceeded 11/0 with quote header + initial 8 bytes. Error suppression for error/broadcast/invalid source |
| Address | plain IPv4 unicast; one address per port plus a local host alias. This model does not validate all OS subnet configuration rules. IP multicast/broadcast, loopback and unsupported sources are given explicit reasons |

The resulting encoder frame is a minimum of 60 bytes, including zero padding. Byte starts at destination MAC, does not include preamble, SFD, FCS, or IFG; delay is calculated over the bytes represented. The decoder checks buffer boundaries and protocol fields, not the entire physical Ethernet receiver rules/runt. A switch only processes the Ethernet header; IP/ICMP validation is performed by the L3 endpoint.

[codec.zig](../../kernel/src/codec.zig) distinguishes truncated/invalid/checksum errors from unsupported options, fragment, VLAN, MTU, protocol, and ICMP type/code. No TCP/UDP, NAT, firewalls, dynamic routing, IPv6, or vendor behavior. [Scoped G5 Linux WSL2 comparisons](../evidence/phase-5-gate/README.md) cover S01–S04, route recovery, and the two specified checksum faults; general OS/vendor conformance remains unproven.

## Bytes, versions, and cause relationships

- `packet_id`: logical datagram/frame identity. Encapsulation, decapsulation, and TTL changes preserve it. ARP, echo reply, and ICMP errors acquire new identities.
- `version_id`: immutable byte copy. `parent_version` designates the trigger version; reply/error also links the new packet to the original request.
- `transmission.id`: one delivery on one link. Flooding can use versions of the same byte in several transmissions with different IDs.
- `event_id`/`cause_id`: execution order and scheduling reasons. `ordinal` is the row position in the domain journal, not the event ID.
- `prior`/`version`: versions before/after decisions or changes, if relevant. Route/cache/drop is still logged even if it doesn't create any new bytes.

The field view stores `byte_offset`, `bit_offset_msb`, `bit_width`, and big-endian numeric values. Bit 0 means the most significant bit in the byte. Payload/quote/padding more than 64 bits using `value: null` in JSON; Complete bytes remain available via `hex`. The <=64 bit field is checked again against the original bits by the Rust oracle. Invalid/unsupported packages still retain raw bytes; The fields API returns an error without a partial view (the JSON demo uses an empty field array).

Trace records selected route, next-hop, cache lookup/hit/learn, ARP request/reply, buffering, switch learning/flood/forward, transmit/receive, decapsulation, TTL, ICMP, delivery, drop, and timeout. `detail_a/detail_b` is defined by kind: route = egress/next-hop, MAC learn = MAC/port, TTL = before/after, ICMP = type/code, timeout = next-hop/attempts. The engine code becomes a complete definition of kind/reason; wrapper provides a legible name.

## Ownership and C ABI network v1

[wirebox_net.h](../../kernel/include/wirebox_net.h) and [net_abi.zig](../../kernel/src/net_abi.zig) define `wb_net_*`. [Rust Wrapper](../../crates/wirebox-kernel/src/network.rs) has `Network`, copies all input/output and destroys Zig allocations via Zig. `close`/Drop only closes once; `Network` is `!Send + !Sync`. The same host mutex serializes probes, runtime counters, and networks. C callers are required to enforce process-wide serialization as well as the validity/alignment/length/non-overlap of their own pointers.

The opaque handle has an upper two-bit namespace: probe `00`, checkpoint `01`, counter `10`, network `11`; 30 bit generation and 32 bit index+1. Slots are retired when the generation runs out. This replaces phase 2's internal bit division; handle is only valid as long as the process/owner is alive, it should not be saved in persistence format.

| API | Contract |
|---|---|
| create/destroy | Version 1, seed, retention; maximum 4 networks simultaneously |
| checkpoint/checkpoint_destroy | Up to 4 immutable process-local copies, independent of source lifetime; their handles occupy namespace `01` |
| restore/fork | Restore a live network or allocate a new independent network. Configuration is editable only if the checkpoint precedes workload. Failure state is copied too; discard old timeline projections/cursors after restore |
| checkpoint_info/noise_state | Copy input/model/clock/PRNG metadata or impairment configuration/counters. Invalid checkpoint info output is zeroed |
| noise | Before workload, set loss in `[0,1000000]` parts per million and jitter in `[0,UINT64_MAX-1]` ns; surviving frames sample delay in `[0,jitter_ns]` |
| add/port/link/alias/route | Configuration before workload; IP/MAC is a number, not a native wire struct |
| ping/inject | One workload; inject layer 1=IPv4 or 2=Ethernet, copy up to 1600 bytes |
| run | Horizon/budget, copy progress; queue after horizon remains pending |
| summary/packet/tx | Metadata/byte/transmission copy. Packet/tx searched by ID |
| trace | Non-consuming read after ordinal, batch <=256; retention 0..10,000 (expanded October 24, 2026) |
| fields | <=40 fields, all views or error capacity/invalid_packet/unsupported |

State 0..8 follows [runtime ABI](runtime.md). Additional: 9 states, 10 allocations, 11 invalid_packet, 12 unsupported. Protocol rejection on `inject` is not an ABI error: check summary/trace. `failed=1` means engine/resource failure and defeats any outcome value. Outcome: 0 pending, 1 echo_reply, 2 unreachable, 3 time_exceeded, 4 arp_timeout, 5 invalid_packet, 6 unsupported, 7 dropped. Unreachables can originate from local route failures; to prove ICMP was received, check `delivered` and bytes such as acceptance S02/S03.

Dispatch failure can occur after an event is removed from the heap and some state changes. The network then fails and refuses to continue; the output is still readable. `RunError.progress.processed` counts successful dispatch on that call; `summary.executed` also includes events for which dispatch was initiated and then failed. This is different from atomic counter firing in runtime phase 2. Invalid configuration input is rejected before workload; `past_time` does not turn off the Network.

Layout Windows x64 alignment 8: Packet 1640 bytes (payload at offset 40), Trace 96, Transmission 72, Summary 80, Field 24, Progress 24. C/Zig/Rust tested the layout. The struct is an in-process transport, not a package format or file format that is promised to be stable.

Phase 5 adds Noise (40 bytes, `jitter_ns` at offset 8) and CheckpointInfo (56 bytes, `now_ns` at offset 32). All new calls use the same synchronous ownership, process-wide serialization, pointer alignment, non-overlap, and error-code contracts. The source must remain alive for checkpoint creation; the allocated checkpoint then owns an independent heap copy and is freed by Zig. There is no checkpoint file import/export or cross-version restore. `link_loss` is reason 109; its drop trace records drop count/draw in `detail_a/detail_b`, and creates no Tx or arrival record. Jitter may reorder arrivals. Disabled features consume no PRNG draws; transmit publishes PRNG/counters only after time checks and arrival reservation, or together with a committed loss.

Capacity of each Network: 128 entities, 32 static routes, 16 aliases, 64 ARP entries and 64 MACs, 16 pending ARPs with 8 packets each, 128 byte versions, 256 transmissions, 256 pending events/timers. Storage packets do not overwrite history. Full produces a terminal error when the workload has already started. Only journals use rings; `trace_dropped` and gap ordinal indicate retention. Journal retention 0/1/16/1024 was shown to not change bytes, transmissions, clocks, or throughput.

## Limits of proof

[Acceptance runner](../../crates/wirebox-headless/src/packet_demo.rs) reads frozen fixtures and checks golden headers before/after, separate Rust checksums, fields/bits, ICMP quote/correlation, delay, serialization, ancestry, as well as ARP timing. Each case is compared between batch, single event budget, repeat, and multiple retention. Zig adds multiport flooding, LPM routing, odd payload, ICMP suppression, and ABI/resource limits tests.

These results prove independent PoC in the above subset. The cache state has not been exported as a complete host-readable snapshot; decision trace is not an audit of all states. Phase 4 supplies Studio persistence by rerunning a scenario. Phase 5 supplies opaque active-state checkpoint/replay through kernel/headless APIs, with [hardening evidence](../evidence/phase-5-hardening/README.md). A copied failed state stays failed. ICMP echo timeout is not modeled: loss after ARP resolution can leave outcome 0 with no pending work, which remains unresolved rather than evidence of reachability. [G5 Linux WSL2 comparisons](../evidence/phase-5-gate/README.md) cover S01–S03, ARP failure policy, route recovery, and two checksum faults with healthy controls. Linux ARP-timeout ICMP generation differs from the model and is explicitly scoped; broader OS comparisons and cross-vendor pilots remain unproven.
