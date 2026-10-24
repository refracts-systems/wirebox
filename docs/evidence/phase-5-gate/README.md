# G5 assessment: local replay and bounded Linux behavior comparison

Date: 24 October 2026. Decision: **G5 passes for the declared local replay and Linux WSL2 comparison scope below**. P5-06 and P5-07 are complete. P5-08 is a separate, open evidence-contract extension under ADR-0014; it does not retroactively change this experiment contract. G6–G8 and vendor pilots remain open.

Committed baseline: `b3cad11`; implementation and this evidence were uncommitted during verification. Windows x64 MSVC runs the actual Zig 0.17.0 engine through Rust 1.99.1. Independent Linux execution runs in Ubuntu 26.04.1 WSL2, kernel `6.18.33.2-microsoft-standard-WSL2`, x86_64. This is a technical, agent-operated gate assessment, not a human usability or firmware-wide conformance claim.

## Contract fixed before capture

The new [linux-static-ipv4-v3 contract](../../../corpus/scenarios/linux-static-ipv4-v3.json) defines cases, required checks, normalizations, and exclusions before the experiment. It uses new scenario/comparison schemas v2 and comparator revision 3. The original v1/v2 scenarios, schemas, records, captures, and failure status remain unchanged.

The lab uses four private namespaces and synthetic addresses, with no physical or WSL host interface attached. Each case/repetition gets fresh topology and protocol state. Source and router interfaces are captured as PCAP and JSONL, with packet-drop statistics. Commands, pre/post state, sysctls, probe windows, route-change ordering, exact input bytes, tool sources, and artifact hashes are retained. No new dependency was installed.

## Independent Linux experiment

The [run record](../../../corpus/devices/linux/ubuntu-26.04-wsl2-x86_64/6.18.33.2-microsoft-standard-wsl2-7c656fcc/runs/linux-forwarding-20261024T125629Z-5275018d/record.json) passes **21/21 case repetitions and 186 invariant checks**. Each of the seven cases ran three times.

| Case | Compared behavior | Observed result |
|---|---|---|
| S01 | Healthy forwarding, exact request bytes, TTL decrement, MAC rewrite, ICMP response | 3/3 pass |
| S02 | Missing static route, no request forwarding, ICMP 3/0 with expected quote | 3/3 pass |
| S03 | TTL expiration, no request forwarding, ICMP 11/0 with expected quote | 3/3 pass |
| S04 | Unanswered ARP, no IP request forwarded to the destination, bounded failure with drained model work | 3/3 pass within the explicitly different failure policies below |
| R01 | Missing route first produces ICMP 3/0; add the route; the same input then receives an echo reply | 3/3 pass; probe/change ordering and pre/post route state retained |
| N01 | Corrupt IPv4 checksum at R1 ingress; no response/forwarding during the observation window; corrected packet recovers | 3/3 pass |
| N02 | Valid IPv4 header and corrupt ICMP checksum addressed locally to R1; silence followed by a valid control reply | 3/3 pass |

For N01/N02, AF_PACKET injects Ethernet at SW1:p1. The router capture proves the exact invalid IP bytes arrived at R1:eth0. An AF_INET/IP_HDRINCL sender is unsuitable for this negative case because it can repair the IPv4 header checksum. Silence is measured for at least one second, followed by a corrected packet on the same path. Capture absence alone cannot pass: ingress evidence and the healthy control are mandatory. These cases cover local checksum rejection and the specified IPv4 fault; they do not establish a general malformed-packet policy or corrupted-ICMP forwarding behavior.

R01 observes both failure and recovery in one namespace lifetime with the same input. It is a controlled route experiment, not an implemented planner or real-device adapter. All mutations are confined to the private lab namespaces.

### Declared ARP policy difference

Linux keeps the peer link up with `arp_ignore=8` to make the no-answer condition observable. Wyrlex S04 disables the peer port. The compared condition is absence of an ARP answer, not equivalence of administrative state, carrier behavior, or hardware failure.

Linux emitted three ARP requests without an answer in each repetition, then ICMP type 3/code 1. Wyrlex reports internal `arp_timeout`, drains pending ARP/events, and does not generate that host-unreachable message. Comparator revision 3 requires these stated outcomes separately. Their different response generation is a visible model limitation, not byte equivalence. Exact ARP retry counts/timing and Linux-generated ARP-timeout ICMP equivalence were excluded before the run.

## Replay and retention evidence

The [local replay result](check-phase5.json) meets the existing pilot scorecard's five-seed, three-repetition requirement. The matrix covers six scenarios (S01–S05 and the alias route), seeds `0`, `1`, `42`, `20260117`, and `u64::MAX`; four retention capacities; four loss/jitter settings; and five checkpoint cuts.

- **1,440 uninterrupted runs** and **7,200 checkpoint cases** were checked.
- **21,600 home/fork/restore continuations** matched exactly.
- Comparisons include summary, packet bytes/ancestry, field mappings, transmissions, retained trace, noise counters, and PRNG state; a separate host oracle checks link-time bounds.
- Of 480 distinct baseline configurations, 315 intentionally had incomplete retained histories. Contiguous suffix/count checks passed; lost trace history was never presented as complete history.
- Of those distinct configurations, 72 had unresolved outcome 0 under loss. The model has no echo timeout after ARP succeeds. Their deterministic replay passed; neither successful reachability nor failure conformance is inferred from those outcomes.

The existing checkpoint ownership, capacity, generation retirement, failed-state, cancellation/order, and allocator/FFI tests remain part of core verification. Checkpoints remain process-local values; durable storage, cross-build restore, UI checkpoint controls, and live topology mutation are outside this gate.

## Verification and negative controls

[Core verification](verification.log), `./scripts/verify.ps1 -CoreOnly`, passed: 46 Zig tests in Debug and again in ReleaseSafe; 16 Rust kernel tests; 7 headless tests; 4 compile-fail doctests; C ABI layout compilation; formatting; Clippy; and the headless acceptance commands. The existing dependency future-incompatibility warning is retained. Studio UI/build and remote CI were not rerun for these headless/comparator changes.

[Corpus tests](tests.log), 24 tests, passed. The added controls reject missing invalid ingress, a repaired checksum, a short silence window, a missing healthy control response, missing failure before route recovery, an incorrectly ordered route change, an ARP answer in the no-answer case, and incorrect lab preconditions. Schema tests keep new cases outside the old scenario contract. Earlier hash/PCAP/JSONL, fabricated-report, missing-case/invariant, path, capture-drop, and comparator-revision controls still pass.

[Final validation](validation.json) reproduces all three corpus records: the historical classifier failure remains comparison **fail** with valid integrity; the earlier corrected run and the expanded run pass. [Cleanup](cleanup.json) confirms no experiment namespaces remain. Hashes establish content integrity, not authenticity of arbitrary external measurements.

The independent capture used the engine binary/source hashes in its retained provenance. The replay oracle was subsequently expanded to five seeds and three repetitions, and its status text was scoped to the command's actual work. [Export recheck](export-recheck.json) confirms that all seven current `--corpus-demo` JSON exports still match the captured simulator exports exactly. No Zig engine, C ABI, or Rust wrapper behavior changed in this work. The final source/artifact manifest is [manifest.json](manifest.json).

Commands actually run:

```powershell
./scripts/corpus/run-linux-lab.ps1 -Repetitions 3
./scripts/verify.ps1 -CoreOnly
wsl -d Ubuntu-26.04 -- python3 -m unittest discover -s /mnt/c/Users/ricky/src/wirebox/scripts/corpus -p test_corpus.py -v
wsl -d Ubuntu-26.04 -- python3 /mnt/c/Users/ricky/src/wirebox/scripts/corpus/validate.py --repo /mnt/c/Users/ricky/src/wirebox
wsl -d Ubuntu-26.04 -- ip netns list
```

Paths refer to the measured checkout; use the appropriate WSL mount path elsewhere. The launcher builds and exports the model, captures Linux execution, and validates retained results. Local `--check-phase5` assesses replay only; its output does not discover or certify external corpus records.

## Decision and remaining limits

G5 requires reproducible restoration/replay in the stated environment and an independent lab comparison for a declared supported subset. Those conditions pass with the limits above. This assessment does not claim equality with the complete Linux stack, Packet Tracer, RouterOS, OLT/ONU firmware, NIC/ASIC paths, real loss/jitter distributions, wall-clock latency, or optical physics. The kernel image hash remains unknown. Windows engine execution plus Linux lab execution does not establish a Linux engine build.

P5-08 remains open for versioned analysis/claim/interoperability dossiers and their validators. The v2 schemas added here extend only the Linux runtime experiment. They do not implement firmware analysis, an observation adapter, a verifier, or vendor support. The [ADR-0014](../../adr/0014-version-scoped-device-behavior-evidence.md) evidence policy still applies.

The user deferred exact MikroTik inventory on 24 October 2026. Ask again when beginning the Phase 6 MikroTik baseline: model or CHR, exact RouterOS build, architecture, VM/physical context, and a local firmware/image path if available. Winbox client identity is separate from RouterOS artifact identity. No credentials are needed for the inventory reminder, and no RouterOS account, firmware, or runtime access has been supplied or tested.

The local Cognitive Coverage manifest remains stale; no human-understanding status is promoted. [Learning notes](../../learning/phase_5/phase_5_g5_linux_validation.md) explain the boundary and comparator decisions in Indonesian.
