# Packet Studio — Phase 4 initial integration

Updated October 24, 2026. Draft and v2 schema follow [ADR-0009](../adr/0009-studio-draft-session-v2.md). The studio consumes the Zig package simulator through a Rust controller. [ADR-0008](../adr/0008-studio-network-controller.md) describes ownership, batch and persistence. [Testing evidence](../evidence/phase-4/README.md) limits results claims.

```powershell
cargo run --locked -p wyrlex-studio
```

Select S01 healthy, S02 route lost, S03 TTL timed out, S04 ARP timeout, or S05 IPv4 truncated. Click Run (Ctrl+R), Pause (Space), Step (F10), or Reset (Ctrl+N). Step processes a maximum of one scheduler event; The initial workload can create multiple synchronous trace rows. The number of rows is not the number of event schedulers. Fast pace runs use four events every 40 ms; The Pace button slows down one event per second so that the flow is easy to inspect. Changing the pace pauses the run. This pause is playback only; virtual time and five second horizon do not change. `now_ns` on horizon is not ping latency.

Click a node/link to filter traces; All remove filters. Scroll horizontally the topology area to pan and use +/− to scale the card. This is a fixture topology viewport, not yet a free graph editor.

Click row trace to see event/cause, packet ID and previous/after versions. Before/After selects the raw bytes of the corresponding version. Click a protocol field to highlight a byte range, or click a byte to view the field that contains it. The bit offset is calculated from the MSB. The amber color marks the difference to the prior on the same layer; layer switching is not a comparable diff offset. Invalid packets still have raw bytes, because they are decoded without false fields.

## Diagnose and change without editing source

1. Select S02 and Run. Filter R1; look for `no_route`, ICMP error and unreachable results.
2. Click Add missing R1 route. Initial gateway `10.0.1.2`; Enter or Save to draft saves the draft; Click Apply draft to create a new configuration branch.
3. Run again. Echo replies now go through static routes. The comparison bar shows the outcome/reason before and after. The original configuration and reports are saved as a baseline, not mutated.
4. To check forwarding bytes, select S01, Run, filter R1 then select `decrement_ttl`. Compare TTL 64→63 and checksum using Before/After and `ipv4.ttl`/`ipv4.checksum` fields.

The address on the port card can be clicked to change the IPv4/prefix. Other buttons change the destination, TTL, gateway alias R1, delete route aliases, turn on/off port PC2, as well as the bitrate and propagation of all links. Type to change values, Backspace to delete, Ctrl+A/Ctrl+V to replace/paste, Enter to save draft, Esc to cancel. Apply draft validates all changes; Discard draft throws it away. The form accepts ASCII printables and spaces (maximum 4096 characters). A configuration error is displayed; Old results remain available if branch creation is rejected.

Routes replaces the fixture route table with a list `device prefix port gateway` separated by `;`. Port down accepts port names separated by spaces. Per-link timing receives `L1 bitrate_bps delay_ns; L2 ...`. See [runbook](../evidence/phase-4-hardening/user-acceptance.md). The timeline uses a maximum page of 50 rows; maximum kernel retention of 10,000 rows.

There is no cache/topology mutation in the middle of the run. Each change creates a new Network with an empty cache; this is not a checkpoint restore. Fixed scope [package model v1](packets.md).

## Save and open results

Save / Ctrl+S opens the `.wb.json` file dialog; Open / Ctrl+O read it again while paused. Writer uses schema v2 and reader still accepts v1. Unapplied drafts are not saved. The session contains configuration, execution commands, traces, packets/fields, transmissions and comparison baselines if any. Load reruns from the beginning and compares reports; results that do not match are rejected. Files of other versions or models are not considered automatically compatible. Limits of 2 MiB and 1024 batches apply before sessions are replaced.

Save private files in a folder of your choice, for example `artifacts/` which Git ignores. The GUI does not write real devices. Controller testing, window interactions, and performance measurements are different evidence; see [task status](todos.md#phase-4--mvp-studio-usable-).

Advanced technical acceptance: [agent and performance results](../evidence/phase-4-acceptance/README.md), method [ADR-0010](../adr/0010-agent-acceptance-and-studio-measurement.md). The WYRLEX_ACCEPTANCE environment mode uses read-only historical projections for measurements; regular sessions still use active Network results.
