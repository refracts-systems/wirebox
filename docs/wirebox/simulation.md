# Device-free simulation

Network simulation without physical devices and vendor OS images allows for the behavior we implemented. Virtual devices retain their identity, ports, configuration, protocol state, and processing rules.

As of October 21, 2026, [phase 3 Wyrlex](packets.md) is already running standalone Ethernet/ARP/IPv4/ICMP scenarios over Zig–Rust. [Evidence G3](../evidence/phase-3/README.md) separates the Wyrlex engine results from the PT report and external mini-simulator. Next Studio integration; Real device validation remains required for vendor conformity claims.

## What does Packet Tracer evidence support?

| Findings in notes | Implications for Wyrlex | Conclusion limits |
|---|---|---|
| Device, port, link, PDU, and process objects | The network world can be represented in the program | All vendor behavior has not been restored |
| Heap timer and registry by-ID | Executions can be sequenced by virtual time and timers can be found/cancelled | PT layout and inheritance do not need to be copied |
| Protocol process chain and software routing | The forwarding behavior can be implemented as an algorithm | The class name/step code is not the same as the full specification |
| The Event List GUI is separate from the queue | Execution and display of traces require different components | The Back button does not restore the entire state |
| Local running mini-sim demo | ARP, switching, routing, ICMP, TCP, UDP scenarios can be demonstrated independently | Limited demo model and not PT equality testing |

Detailed status is available at [evidence ledger](../packet-tracer/evidence-ledger.md). RE documents are primarily sourced from user reports; mini-simulator re-execution is a separate local verification.

## Three types of testing are discussed

| Approach | Suitable questions | What needs to be available |
|---|---|---|
| Configuration/reachability analysis | Does the path or policy allow certain traffic? | Configuration and semantic models that support the feature |
| Discrete event simulation | When is the packet sent, the timer expired, or the state changed? | Protocol algorithms, schedulers, delays, and model assumptions |
| OS/vendor Labs | How does the OS implementation behave in this scenario? | Appropriate image/software and its execution environment |

The three can complement each other. Reachability results do not automatically explain TCP retransmissions. Lab OS does not automatically replicate ASICs, optical conditions, or all physical device conditions. See [comparator project](positioning.md).

## Proposed kernel

Basic flow: the application creates a packet → the protocol process forms the bytes → the port calculates the sending time → the scheduler performs reception on the destination port → the protocol state changes → the trace records the results and why.

Recommended starting options:

- Stable sequence `(virtual_time, sequence)` for events with the same time.
- ID timer and cancel mechanism that does not leave active callbacks on deleted objects.
- Limited step APIs such as `step` and `runUntil`; draining the queue until empty is not suitable for all periodic timers.
- Explicit PRNG with saveable seed/state for jitter and loss.
- GUI clock-independent kernel; realtime mode only sets the virtual time playback rate.
- One simulation state owner as starting point. Parallel execution is added when measurable needs justify it.

Convention [ADR-0003](../adr/0003-virtual-time-and-trace.md) is already implemented in [runtime phase 2](runtime.md): tick u64 nanoseconds and `runUntil(t, budget)` advances the clock to t after all due events have been processed. If the budget runs out, time stops at the last execution. [Evidence G2](../evidence/phase-2/README.md) checks scheduler invariants and synthetic counters; [packet model phase 3](packets.md) uses this scheduler for frame arrival and ARP timer. Older mini-sims use different conventions, noted in [prototype verification](../packet-tracer/mini-simulator.md).

The execution queue should not miss events just because the UI limits the number of rows. Journals may have their own retention policies; if restricted, remaining replayability should be clear.

## Coverage and fidelity

Each model needs to state which features are supported, which are simplified, and which are unknown. Example: static IPv4 routing does not promise BGP, ECMP, NAT, fragmentation, or any ICMP behavior.

Traces can explain why the model chose a particular route. The assertion that real routers will select identical results requires comparison across appropriate configurations, versions, and scenarios. Modeling optical errors, hardware queues, congestion, or firmware requires separate work.

For results that depend on features outside the model, display unsupported/unknown along with the cause. Don't conclude a pass just because the feature was skipped.

## Relationship to Packet Tracer format

The Wyrlex engine can use its own versioned scenario format. The `.pkt` importer is a separate job for mapping recognized topologies and configurations.

The latest report corrects the initial `.pkt` encryption hypothesis, but the documented reader still relies on captured keystream. Complete importer, writer, and round-trip have not been proven. Reference [`.pkt` format status](../packet-tracer/pkt-format.md), not the old report snippet.
