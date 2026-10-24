# ADR-0007 — Phase 3 packet model

- Date: 20 October 2026
- Status: Accepted; implementation and [G3 evidence](../evidence/phase-3/README.md) were available on 21 October 2026.

## Contract before acceptance

Use the Phase 2 graph and heap/timer with two one-shot domain events: frame arrival (100) and ARP retry/timeout (101). The scheduler stores tokens/IDs, not callbacks or pointers. Domain dispatch is in Zig. The earlier counter API still limits actions to 1/2.

Create and configure the Network first; freeze topology/configuration when a ping or injection workload starts. Scenario changes use a new Network. Zig generates and processes all bytes; Rust loads fixtures, runs the workload, checks it with a separate oracle, and exports artifacts.

Packet versions are immutable copies of at most 1600 bytes. `packet_id` identifies a logical datagram/frame; `version_id` identifies a snapshot, and `parent_version` links encapsulation/decapsulation/mutation. Replies/errors get a new packet ID and point to their triggering version. Transmissions have their own IDs: flooding shares byte versions but creates distinct transmissions. A field view stores byte offset, MSB bit index, width, and big-endian value; native structs are not wire format.

The subset follows the Phase 0 contract: Ethernet II, Ethernet/IPv4 ARP, IPv4 IHL=5/MTU=1500 without fragmentation, LPM routes, ICMP echo 8/0 and 0/0, and errors 3/0 and 11/0. Options, VLAN, fragmentation, other protocols, and IP multicast are `unsupported`. Damaged packets become `invalid_packet`. ICMP errors are not generated for ICMP errors, invalid IP, or broadcast/multicast packets.

Full-duplex links use an egress-port serialisation queue, `ceil(frame_bytes*8*1e9/bitrate) + propagation`. Minimum frame size is 60 bytes including zero padding; FCS/preamble/SFD/IFG, collision, congestion/drop queue, jitter/loss, STP, and physical models are omitted. MAC learning is per switch and ARP cache per port; aging is not modelled. Caches start empty. ARP buffers the datagram, requests at 0/1/2 seconds relative to lookup, and times out at 3 seconds. Pending timers are cancelled after resolution.

The domain ring journal is separate from the scheduler queue and packet storage; retention does not change execution. Packet/tx/cache/pending capacity overflow is a visible terminal error. Domain dispatch may have changed some state before a resource failure; the Network latches failed and cannot continue. This does not change the Phase 2 atomic counter-firing contract. A failed run must never be reported as successful ping.

The v1 `wb_net_*` API uses a copied C ABI, the same host mutex, and a separate handle namespace. The top two bits distinguish probe (`00`), counter runtime (`10`), and network (`11`); generation is 30 bits, index is slot+1, and a slot retires when generation is exhausted. Four Networks are allocated and freed by Zig. Initial model limits are 128 entities, 32 static routes, 16 aliases, 64 MAC/ARP entries, 16 pending resolutions (8 packets each), 128 byte versions, 256 transmissions/pending scheduler records, and 1024 domain rows. Historical bytes are never overwritten.

## Evidence

S01–S05 continue to use frozen fixtures. The Phase 0 IPv4-header oracle compares request bytes before/after forwarding. Rust independently verifies checksums, ICMP quote/payload, ID relations, retry/delay timing, and outcomes from the bytes actually received. Static routes are also tested in a healthy alias case, not only after removal. Negative tests cover truncation, checksum, options/fragment/VLAN/protocol, and ICMP-error suppression.

Primary references checked on 20 October 2026: [RFC 791](https://www.rfc-editor.org/rfc/rfc791), [RFC 792](https://www.rfc-editor.org/rfc/rfc792), [RFC 826](https://www.rfc-editor.org/rfc/rfc826), and [RFC 1812 §4.3.2.7](https://www.rfc-editor.org/rfc/rfc1812#section-4.3.2.7). This is an engineering subset, not a claim about every RFC or vendor firmware. Linux comparison capture remains the G5 gate.
