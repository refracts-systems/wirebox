# G4 human acceptance runbook

Preparation date: October 24, 2026. Result status: **not yet run by user**.

Run `cargo run --locked -p wirebox-studio` from the root repository. Fill in the date, build/hash, tester name, start/finish time and any help required. Do not use agent results as user results.

| Tasks | Steps and expected results | Results / time / assistance |
|---|---|---|
| U1 | Select S01, Run; find echo reply and path PC1–SW1–R1–PC2 | Not tested |
| U2 | Select S02, Run; show R1, lookup/drop no_route and ICMP error | Not tested |
| U3 | S01: select forwarding R1, Before/After; explain TTL 64→63, checksum changes and associated bytes | Not tested |
| U4 | S02: edit gateway alias R1 to 10.0.1.2, Enter for draft, Apply draft, Run; new results echo reply and old baseline remains unreachable | Not tested |
| U5 | Save, close the application, run again, Open; match config, result, baseline and bytes | Not tested |

Editor's additional test: Routes received `PC1 0.0.0.0/0 eth0 10.0.0.1; PC2 0.0.0.0/0 eth0 10.0.1.1; R1 198.51.100.2/32 eth1 10.0.1.2`. Port down accepts space-separated port names; empty deletes the list. Per-link timing receives `L1 10000000 5000; L3 100000000 2000`. All edits must be followed by Enter then Apply draft. Gateways outside the subnet must be denied and active throughput remains available.

Also note whether the draft/active labels are clear enough and whether the format editor needs help. These obstacles are the result of usability that must be corrected, not the tester's error.

The G4 doesn't pass just this table: performance of four nodes/10,000 rows p95 <100 ms, working set ≤1 GiB, and scripted time of 30 minutes should speak for themselves.
