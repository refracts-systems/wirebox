# Contract proof phase 0 — baseline v1

- Set: 20 October 2026, in the user's duties to carry out phase 0 and 1.
- Status: baseline engineering v1; G0/G1 results are recorded in [evidence](../evidence/phase-0-1/README.md).
- Changes in scope/numbers after this must have a revision note before the next acceptance run.

## MVP scenarios and limits

Normative fixture: [icmp-static-v1.json](../../fixtures/scenarios/icmp-static-v1.json). Four PC1–SW1–R1–PC2 nodes, two IPv4 subnets, three 100 Mb/s links with 1 µs propagation. PC1=10.0.0.2/24, router=10.0.0.1/24 and 10.0.1.1/24, PC2=10.0.1.2/24. The host uses the default route to the router; The router also has a static route to PC2's local alias 198.51.100.2/32 via 10.0.1.2.

S01 uses destination 10.0.1.2. S02 uses the alias 198.51.100.2 to delete the **static** route on R1 resulting in no_route without pretending to delete the connected route that the OS will reinstall. An alias is an additional local address of a host, not an additional node.

MVP supports Ethernet without VLAN, MAC learning/flooding, IPv4 ARP, local address, IPv4 without options/fragmentation, longest-prefix match static routing without ECMP, as well as ICMP echo/type 3 code 0 and type 11 code 0 required by fixtures. Ignoring features outside the subset is not allowed; the result is unsupported/unknown or invalid_packet according to the reason.

Does not include TCP/UDP, NAT/firewall, PPPoE, dynamic routing, IPv6, VLAN, physical optics, or vendor firmware images. The needs of other subsets of ISP pilots require scope expansion and separate testing.

The byte frame starts at the destination MAC until the required payload/padding; preamble/SFD/FCS and physical encoding have not been modeled. IPv4 MTU 1500; larger results in unsupported fragmentation. The header/checksum is generated explicitly, not a reinterpretation of the native struct. Initial TTL fixture 64.

## Expected results and oracles

| Case | Input/mutation | Mandatory results when phase 3 is available |
|---|---|---|
| S01 | Healthy topology, PC2 goals | Echo reply; R1 forwards request TTL 64→63 with correct checksum |
| S02 | Delete static route aka PC2 on R1 | no_route in R1; ICMP network unreachable type 3/code 0; not forwarding to any interface |
| S03 | Request TTL=1 | R1 does not forward the request; ICMP time exceeded type 11/code 0 |
| S04 | PC2 eth0 not responding | ARP three attempts at 0/1/2 seconds from lookup, timeout at 3 seconds; buffering is released and the arp_timeout reason is visible |
| S05 | IPv4 header truncated `4500` | invalid_packet/truncated_ipv4; does not crash or read outside the buffer |

S04 must complete within the 5 second virtual horizon. ARP conditions are v1 model policies, not claims of all vendors' default values. Complete ICMP rules remain subset-limited; errors for errors/broadcasts must be prevented when the protocol implementation is created.

Initial oracle: [IPv4 header before/after](../../fixtures/packets/ipv4-forwarding-v1.json), compiled independently before the codec engine was available. TTL and checksum changes are checked with separate one's-complement checksums. The fixture is not a capture result or a run simulator result.

Specification sources: [IPv4 RFC 791](https://www.rfc-editor.org/rfc/rfc791), [ICMP RFC 792](https://www.rfc-editor.org/rfc/rfc792), and [ARP RFC 826](https://www.rfc-editor.org/rfc/rfc826). The scope quote here is a subset of the model; does not state conformance across RFCs. Linux runtime comparator/capture remains mandatory on the G5, separate from the manual fixture.

## Frozen core pilot intent

`EnsureReachability` v1: explicit IPv4 test source and destination, ICMP echo, return/return direction, main routing table, and known return path. Postcondition pilot: three successful consecutive probes with one second timeout per probe in the lab without intentional loss, accompanied by read-back state route. Probes do not replace capability checks.

`EnsureStaticRoute` is a candidate operation v1: target, prefix, next-hop, interface/egress, and main table. Adding a route that doesn't exist yet; Identical routes result in no-op, conflicts result in plan conflict. Do not change/delete other routes hiddenly. Compensation simply deletes the routes created by the run after checking that the state has not changed. The runtime schema form is designed in phase 7, following this semantic contract.

The ABI phase 1 probe does not execute the intent. As of October 21, 2026, [phase 3](packets.md) provides simulated state routing and byte/trace copy. Executor intent, snapshot observed state, and device mutation are not yet available; The above real pilot contracts have not been tested.

## Frozen acceptance figures v1

Targets in [pilot scorecard](pilot.md) are adopted as baseline v1: U1–U5 standalone with runbook; S01–S05 correct; S06/S07 was repeated three times on each of the five seeds; G7 case three iterations per backend; zero false success status; manual benefits on at least four of the five tasks; p95 input response <100 ms at four nodes/10,000 rows; 30 minute steady session.

The five manual comparison tasks are lost route diagnosis, TTL expired diagnosis, failed next-hop/ARP diagnosis, static route repair, and plan review/revision after drift. Note the time from task start to verified results, number of command/form submissions outside of normal navigation, help, and correctness. Freeze case and sequence variations before G7 experiments to reduce learning effects.

Initial retention budget: visible list 10,000 rows and total application working set ≤1 GiB on four-node acceptance fixture. Old data should not be lost without indication of history limits; queue scheduler is not pruned due to GUI limits. This budget is an engineering target, not a measurement result. If the implementation baseline indicates different requirements, explicitly revise the contract before the acceptance run.

Phase 0/1 reference machine: Windows 11 x64, build 26300, Ryzen 5 5600H, 16 GB RAM, integrated AMD GPU and RTX 3050 Laptop; actual details are in [environment.json](../evidence/phase-0-1/environment.json). The comparison Linux simulator has not been accessed; distro/kernel remains unknown.

## Remaining inventory, access, and dependencies

[pilot-v1.json](../../fixtures/inventory/pilot-v1.json) logs user information and unknowns. [Sanitized Excerpt](../../fixtures/inventory/zte-c320-user-report.txt) preserves board version, GPON state, and provenance limits. No endpoints/credentials are stored.

Scope G0 is currently a local development and synthetic fixture. Linux VM/server and RouterOS 7 are called users; distro, minor version, model, and test access are required before G6. The C320/ONU is from user reports, not proven to be physical hardware; ONU model/firmware details as well as PON pairing are required before G8.

Inventory is considered **listed as unknown**, not all verified access. All local phase 0/1 activities; no network scans, device logins, or device configuration changes.

## Implementation of Phase 4 acceptance by agent

On October 24, 2026 the user requested that all tests be carried out by the agent. [ADR-0010](../adr/0010-agent-acceptance-and-studio-measurement.md) defines technical acceptability U1–U5 via native computer-use as well as handler-to-render methods. The number of 10,000 rows, p95 <100 ms, duration of 30 minutes and budget of 1 GiB is maintained. Understanding/usefulness for humans remains unknown and unclaimed from the agent's results. Actual results are at [new evidence](../evidence/phase-4-acceptance/README.md).
