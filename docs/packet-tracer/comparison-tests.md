# Wyrlex–Packet Tracer benchmark test runbook

The baseline is set for October 21, 2026, before the differential run. [Preliminary audit](../evidence/pt-comparison/README.md) provides binary identity, Ghidra audit, PT smoke, and Wyrlex acceptance. **There is no verdict differential yet.**

## Comparison contracts

Freeze engine and PT version/hash, topology/hash files, device model, interface/route configuration, initial cache, TTL, payload, and delay/jitter mode. Start each case from a copy of the initial condition; do not let warm ARP/MAC change the expected sequence of cold-start cases.

Compare the same decisions and invariants: destination lookup, egress, next-hop, delivery/drop, TTL request before/after router, valid checksum, ICMP type/code and quote. Map MAC/device identity explicitly if PT does not allow the same MAC. The numeric checksums must only be identical if all the headers included are also identical; otherwise, validate the checksum against the respective byte as well as any changes caused to the TTL. Do not remove TTL, next-hop, payload, or error code as normalization.

Timestamp, initial reply TTL, ARP retry policy, padding and link overhead must be compared separately from semantic results. Wyrlex's v1 model only counts MAC frames without FCS/preamble/IFG. The number of Event List rows is not compared to the number of Wyrlex pop schedulers.

## Minimum cases and input access

| ID | Setup and mandatory observations | PT access notes |
|---|---|---|
| S01 | PC1–SW1–R1–PC2, two subnets; echo reply; request TTL drops by one, L2 rewrite, checksum correct, payload/ICMP request remains | Set the initial TTL 64 through the facilities that are actually available; if it cannot be set, compare the TTL deltas and note the different inputs, without claiming byte-identical |
| S02 | Destination aka 198.51.100.2; delete route /32 on R1; ICMP type 3/code 0 and without any forwarding; restore route as healthy control | PC-PT has not been proven to support local aliases. Use loopback-enabled endpoints with written role mappings, or mark control aliases as not yet representable. Do not replace deleting a static route by deleting a connected route |
| S03 | Request TTL=1, not forwarded to R1, ICMP type 11/code 0 with true quote | There must be proof of input TTL, not just a timeout message on the application |
| S04 | R1's egress interface remains up; requested next-hop does not answer; record ARP retry, interval, timeout and buffer release | Turning off the PC2 NIC can drop links and connected routes, so it is not an equivalent ARP timeout test. Can use an empty next-hop in the live subnet for ARP subcases, with documented fixture revisions. Wyrlex's triple retry policy is not a known PT default |
| S05 | IPv4 injection truncated `4500`, incorrect checksum, fragment/options/VLAN unsupported; no crashes or false success | Simple/Complex PDU GUI has not been shown to allow raw malformed bytes. If the input method is not available, note `not_testable_via_gui`; do not consider valid GUI-formed packets as a negative test |

For each case save the pre-run configuration, screenshot/export the ingress/egress PDUs, raw bytes if available, relevant Event Lists, route/ARP/MAC state, application results, and three repetitions of the same initial conditions. Also note the fields that cannot be observed. Protocol labels in the GUI without bytes are not enough to pass the byte test.

## Verdict and claim coverage

Each assertion gets `match`, `mismatch`, `not_observed`, or `not_testable`. The case status is only `match` if all mandatory assertions are observed and matched. Unobserved assertions hold the verdict; it is not considered a match. If PT and Wyrlex are different, check the protocol specifications and Linux comparators before settling on the wrong side. PT behavior is not the only oracle of truth.

Separate the following gates:

1. **Subset truth:** independent fixture/oracle, differential PT, then Linux capture for modeled fields and decisions.
2. **Resilience:** decoder/ABI fuzz, malformed input, allocator/buffer limits, timer cancellation, cache full, horizon run and determinism. Do not send malformed input to production devices as part of this test.
3. **Advantages of the tracker:** users can indicate which TTL/checksum/L2 bytes changed and the cause; results can be exported and repeated. Compare paired tasks and the correctness of the answer, not the number of UI panels.
4. **Performance/workflow advantages:** equivalent workload, separate cold/warm cache, median/p95 response, diagnosis time, help, memory and crash/hang. Freeze workloads before measurement; do not use GUI animation time as engine speed.
5. **Vendor equivalency:** specific model/firmware, state read-back and capture lab. Neither PT nor Ghidra static results replace Linux/RouterOS/ZTE/ONU testing.

One ping successfully passed the smoke test. The broader claim follows that the scope of the assertion actually passes.
