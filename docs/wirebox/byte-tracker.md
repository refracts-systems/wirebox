# Byte tracker and replay

The user's need is to understand the packet byte by byte: its value, the meaning of fields, the process that reads them, and the reason for a change. This capability comes from representation and instrumentation design; Zig or Rust can be implementations.

Status October 23, 2026: [tracker headless phase 3](packets.md) provides immutable byte copy, protocol field mapping, packet/transmission ID, and decision/mutation trace. [Actual walkthrough](../evidence/phase-3/byte-walkthrough.md) uses Zig results. Initial integration [Packet Studio Phase 4](studio.md) links traces, protocol fields and hex bytes via a copy of Rust; [evidence](../evidence/phase-4/README.md) limits G4 state.

## Three interconnected representations

| Representation | Contents | Uses |
|---|---|---|
| Byte buffer | Packet byte order in a version | Hex view, decode, and value comparison |
| Protocol view | Field, byte/bit offset, length, endianness, and validation results | Linking semantic views with buffer |
| Mutation/decision records | Event cause, process, reason, version before/after | Explains changes and decisions without byte changes |

The decoder reads bytes with checked boundaries. The encoder generates the wire format explicitly. Native struct alignment and padding do not become network header definitions.

Phase 3 uses a full copy per version, remaining readable after the engine resumes execution. Flooding can share the same version over multiple transmissions. Copy-on-write or delta is the next optimization option; history should not refer to memories that have changed.

## Identity and causal relationships

Schema phase 3 specifies packet_id, version_id/parent_version, transmission ID, entity, event_id/cause_id, and ordinal trace. The contract and the meaning of each ID are explained in [package model](packets.md).

Flooding can create multiple transmissions of a single packet. Encapsulation, retransmission, and echo reply also require traceable relationships. Don't assume a single ID without a parent/child relationship is sufficient for all such cases.

Decisions such as route lookup, cache hit, drop due to policy, or queue full may not change bytes. The tracker must still record the relevant states/inputs and the reasons for the results.

## Examples of the behavior you want to see

In a simple IPv4 forwarding scenario:

1. The user selects the frame reception event on the router.
2. Inspector displays the frame bytes and Ethernet/IPv4/ICMP mapping that were successfully decoded.
3. The routing step explains the selected route and interface.
4. The next version of the packet shows the changed TTL, the adjusted IPv4 checksum, as well as the Ethernet header for the next hop.
5. Users can go back to reading previous versions and events that caused changes.

Data for this flow is already available on the headless runner; Visual navigation is not yet integrated. Byte frame includes destination MAC to padding, without preamble/SFD/FCS/IFG; [representation limits](packets.md) applies to all results.

## Proposed view

The canvas topology shows the location of the event; timeline selects time/event; inspector shows device state and decisions; hex view highlights a range of fields. Selecting a field highlights the associated byte, and selecting a byte reveals an unrecognized field or range.

The UI should accept snapshots or batch traces with stable IDs. Sending one FFI call for each byte adds overhead and makes display consistency difficult. Contract details are at [technology options](technology.md).

## Replay and time travel

There are two distinct capabilities that should be mentioned clearly: rereading recordings and continuing simulations of historical states. The second capability requires a fairly complete checkpoint or deterministic reconstruction from scratch.

States that need to be considered include configuration/topology, ARP/MAC cache, protocol state, queue and pending timer, packet buffer, counter/ID, PRNG, external input, and engine/model version. Seeds alone are not enough if the order of consumption is random or the input changes.

The initial proposal was checkpoint + event replay with a stable order. Branches from a checkpoint create separate simulation states; branch changes do not change the origin snapshot or the real device.

Production footage is often partial. Wyrlex must distinguish between actually observed, reconstructed, and assumed states. Checkpoint policies, retention, and cross-version replay guarantees are still open.
