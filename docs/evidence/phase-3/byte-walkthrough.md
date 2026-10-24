#S01 — bytes that Wirebox actually processed

Source: [S01.json](S01.json), generated Zig–Rust runner on October 21, 2026. This simulated state model `ethernet-arp-ipv4-icmp-v1`, not device capture. [Model contract](../../wirebox/packets.md) describes protocol/link boundaries.

## Request forwarded by R1

At **34,800 ns**, event **7** (cause **6**) receives frame version 4 at R1:eth0. Decapsulation produces version **5**, route lookup selects R1:eth1 (entity 8), next-hop **10.0.1.2**. Row ordinal **29**, `decrement_ttl`, creates version **6** from version 5. Both have `packet_id=1`; Older versions can still be read.

```text
versi 5 (IPv4 + ICMP, 28 byte)
45 00 00 1c 00 01 00 00 40 01 65 dd 0a 00 00 02 0a 00 01 02 08 00 e5 ca 12 34 00 01

versi 6 (sesudah forwarding)
45 00 00 1c 00 01 00 00 3f 01 66 dd 0a 00 00 02 0a 00 01 02 08 00 e5 ca 12 34 00 01
```

The following offsets are calculated from the start of the IPv4 version, starting at zero:

| Offset bytes | Fields | Before | After | Meaning |
|---|---|---|---|---|
| 0, bits 0..3 | Versions | 4 | 4 | IPv4 |
| 0, bits 4..7 | IHL | 5 | 5 | Header 20 bytes |
| 2..3 | Total length | 0x001c | 0x001c | Datagrams 28 bytes |
| 8 | TTL | 0x40 (64) | 0x3f (63) | The router reduces one hop |
| 10..11 | Header checksum | 0x65dd | 0x66dd | New checksum according to TTL changes; byte 11 remains dd |
| 12..15 | Source IP | 10.0.0.2 | 10.0.0.2 | Source remains PC1 |
| 16..19 | Destination IP | 10.0.1.2 | 10.0.1.2 | Destination remains PC2 |
| 20..27 | ICMP | 0800e5ca12340001 | 0800e5ca12340001 | Type 8/code 0, ID 0x1234, sequence 1; does not change when forwarding |

Only bytes **8** and **10** change. The 20 byte header matches exactly [golden fixture](../../../fixtures/packets/ipv4-forwarding-v1.json) which was frozen before implementation. Rust recalculates header and ICMP checksums separately from the Zig codec.

## ARP and Ethernet next hop

R1 doesn't know MAC 10.0.1.2 yet, so version 6 is buffered. Version 7 ARP requests are sent at 34,800 ns; ARP reply version 8 arrives at 46,400 ns (event 10). The retry timer is canceled and R1 forms a **version 9** frame, still `packet_id=1`, parent version 6:

```text
02 00 00 00 01 02  02 00 00 00 01 01  08 00
45 00 00 1c 00 01 00 00 3f 01 66 dd 0a 00 00 02 0a 00 01 02
08 00 e5 ca 12 34 00 01
00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
```

Destination MAC = **02:00:00:00:01:02** (PC2), source MAC = **02:00:00:00:01:01** (R1:eth1). EtherType 0800 indicates IPv4. 18 byte padding creates a 60 byte frame, without FCS. On the Ethernet version, the IPv4 fields are 14 bytes further: TTL at offset 22 and checksum at offset 24..25.

Transmission **9** sends version 9 via link 13. Start time **46,400 ns**, serialization `60 × 8 / 100 Mb/s = 4.800 ns`, propagation **1,000 ns**, arrives **52,200 ns** as event **11** with cause **10**. Byte, transmission, and event version identities have different functions.

## Reply and results

PC2 generates echo reply version **11**, `packet_id=6`, parent version **10** (request after decapsulation). ICMP changed to `00 00 ed ca 12 34 00 01`: type 0 and new reply checksum, ID/sequence fixed. Reply also bypasses R1 and suffers its own TTL reduction.

PC1 received version **16** at **69,600 ns (69.6 µs)**, event **14**, trace ordinal **63**. Summary: echo_reply, 12 scheduler events executed, 16 byte versions, 12 transmissions, 63 row traces, zero pending events/ARPs. Event IDs 2 and 9 are ARP timers that were canceled so they were not executed. Summary `now_ns=5.000.000.000` comes from horizon run; that's not the ping travel time.

## Read failure in the same way

- [S02](S02.json): alias static route missing on R1; ICMP 3/0 returns to PC1. Quote contains 28 bytes of the original request.
- [S03](S03.json): TTL 1 stops at R1; ICMP 11/0 returns to PC1. Request is not forwarded.
- [S04](S04.json): ARP request R1:eth1 at 34,800, 1,000,034,800, and 2,000,034,800 ns. Timeout in R1 at 3,000,034,800 ns, pending buffer released.
- [S05](S05.json): byte `4500` kept, reason truncated_ipv4; no false frame or pass events.

All runs use seed 20260117 without jitter/loss. Capture Linux comparison and replay of checkpoints are not yet available.
