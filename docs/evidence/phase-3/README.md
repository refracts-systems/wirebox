# Exhibit G3 — Headless packet and byte tracker PoC

**Local pass, October 21, 2026**, for subset `ethernet-arp-ipv4-icmp-v1` on Windows x64 debug. The Zig simulator processes bytes, switching, ARP, routes, TTL/checksum, ICMP, and timers via C ABI to Rust. S01–S05 fulfill [phase 0 contract](../../wirebox/phase-0-contract.md); healthy static aka additionally tested. This proves a standalone PoC, not yet a Studio MVP or real device pilot.

## Recorded results

| Artifacts | Outcome/reason | Event scheduler | Byte version | Transmission | Row trace |
|---|---|---:|---:|---:|---:|
| [S01](S01.json) | echo_reply | 12 | 16 | 12 | 63 |
| [S02](S02.json) | unreachable / no_route | 8 | 8 | 8 | 40 |
| [S03](S03.json) | time_exceeded / ttl_expired | 8 | 8 | 8 | 40 |
| [S04](S04.json) | arp_timeout | 12 | 9 | 9 | 44 |
| [S05](S05.json) | invalid_packet / truncated_ipv4 | 0 | 1 | 0 | 2 |
| [Static alias healthy](alias.json) | echo_reply | 12 | 16 | 12 | 63 |

All pending events/ARPs are zero at the end of the 5 second horizon. S01 received a reply at 69,600 ns. Request headers on R1 match golden: TTL 64→63 and checksum 0x65dd→0x66dd, ICMP unchanged. [Walkthrough byte](byte-walkthrough.md) links the version/event/transmission that actually exists in the artifact.

[acceptance.json](acceptance.json) logs **36 scenario runs**: for each of six cases, baseline batch, single event budget, repeat batch, and retention 0/1/16 are compared to a retention baseline of 1024. Bytes/fields/transmissions/results are identical; Limited journal is a suffix with dropped count. This is fresh-run determinism, not checkpoint replay.

**37 negative inputs** via ABI: 28 truncated prefixes, broken IP/ICMP checksum, fragments, non-ICMP protocol, options, VLAN, source MAC multicast, unsupported ICMP type, and excess MTU. Does not produce scheduled events or success status. Zig also tests the limits of each decoder, flooding multiport sharing bytes, LPM routing, odd payloads, ICMP error suppression for errors/broadcasts, as well as failed-states for resource exhaustion.

Oracle Rust uses frozen golden headers and a separate checksum implementation; checking fields/bits against raw bytes, serialization delays and queues, event/transmission correlation, ancestry version, ICMP quote, reply, MAC rewrite, and ARP time. Synthetic fixtures are not OS capture and this oracle does not prove the entire RFC.

## Command and verify

```powershell
./scripts/verify.ps1
./scripts/verify-clean-export.ps1
./target/debug/wirebox-headless.exe --check-network
./target/debug/wirebox-headless.exe --packet-demo S01
```

- [verification.log](verification.log): Rust/Zig format, C layout, **24 Zig test results** (including one import aggregator), **16 Rust unit tests** and **3 compile-fail doctest** passed; Clippy the entire target with `-D warnings`; probe, fixture, acceptance runtime/network; GPUI build passed.
- [clean-export.log](clean-export.log): build workspace offline from a new source copy and probe/fixture/runtime/network pass. Cargo dependency/compiler cache is reused; not cold-cache build or remote CI.
- [environment.json](environment.json): Zig 0.17.0, Rust 1.99.1, OS/build, model, seed and checks not performed.
- [source-manifest.json](source-manifest.json): SHA-256 source, fixture, actual docs, and G3 artifacts. The repository does not yet have a HEAD; results are tied to file hashes, not commits. G0/G1/G2 manifests and PT archives are maintained as historical baselines.
- [integrity.log](integrity.log): Markdown links/fences, source whitespace cleanliness, PT archive hashes, old gate artifacts, and new manifests checked after documentation is complete.

The GPUI build still displays future incompatibility notifications from the transitive dependency `proc-macro-error2 2.0.1`. Build, tests, and Clippy pass on the pinned compiler; This warning has not been fixed with a dependency upgrade.

## Limits and continuations

The complete contracts are in [packet model/ABI](../../wirebox/packets.md) and [ADR-0007](../../adr/0007-packet-model.md). Network/config is frozen when a workload is started; byte/tx/cache capacity is limited and resource exhaustion becomes failed, not overwrite history or success. All byte versions are copied; not yet checkpoint/restore or complete cache snapshot.

No VLAN, TCP/UDP, IPv6, NAT/firewall, dynamic routing, aging, STP, jitter/loss, physical optics, or vendor emulation. Timing includes MAC bytes to padding without FCS/preamble/IFG. Results do not represent the equivalence of Packet Tracer, firmware, ASIC, or real devices.

UI interaction phase 1 is not repeated and the UI is still probe; next work **phase 4, simulator integration into GPUI**. Comparator Linux capture remains G5, Linux/MikroTik adapter G6–G7, and ISP ZTE/ONU G8. No login or device change; remote CI and release distribution have not been run.
