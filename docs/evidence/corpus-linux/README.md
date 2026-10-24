# Initial Linux behavior corpus and WSL comparison

Date: 24 October 2026. Committed source baseline: `bd1d904`; the corpus tooling was uncommitted during capture. The engine ran on Windows x64 MSVC with Zig 0.17.0 and Rust 1.99.1. Its output was compared with independent Linux kernel execution inside WSL2. This is the first limited lab comparison, not a G5 or vendor-pilot pass.

## Target and experiment

[Target identity](../../../corpus/devices/linux/ubuntu-26.04-wsl2-x86_64/6.18.33.2-microsoft-standard-wsl2-7c656fcc/target.json): Ubuntu 26.04.1 LTS, kernel `6.18.33.2-microsoft-standard-WSL2`, x86_64, iproute2 6.19.0, Python 3.14.4, and jsonschema 4.19.2. Kernel/firmware image hashes are unknown and explicitly null. This is not a bare-metal or RouterOS target.

The experiment creates four private network namespaces for PC1, a Linux bridge SW1, forwarding router R1, and PC2. Veth links connect the same synthetic addresses/MACs as `icmp-static-v1`. Each case/repetition gets fresh namespaces, ARP/cache state, routes, and capture workers. No host or physical interface is attached. All created namespaces were deleted after the experiments; cleanup is recorded per repetition.

The Windows headless runner exports original IP input and simulator traces. `IP_HDRINCL` sends that same IP request through Linux. The Linux kernel independently performs ARP, bridging, routing, TTL decrement, response generation, and delivery. AF_PACKET workers capture source and router interfaces to both PCAP and JSONL. Each run archives commands, before/after observations, capture statistics, simulator output, runner sources, and content hashes.

## Results and retained failure

| Case | Expected and observed scoped behavior | Repeated experiment |
|---|---|---|
| S01 | Echo reply; request forwarded with TTL 64→63, exact original/forwarded request bytes, and router Ethernet rewrite | 3/3 pass |
| S02 | Missing R1 route to the alias; source receives ICMP type 3/code 0 with the expected quote; request is not forwarded | 3/3 pass |
| S03 | TTL 1; source receives ICMP type 11/code 0 with the expected quote; request is not forwarded | 3/3 pass |

The [initial record](../../../corpus/devices/linux/ubuntu-26.04-wsl2-x86_64/6.18.33.2-microsoft-standard-wsl2-7c656fcc/runs/linux-forwarding-20261024T073901Z-981dc0dc/record.json) uses comparator revision 1 and remains **fail**: 4 of 9 comparisons flagged unrelated bridge-generated IGMP multicast reports as invalid ICMP input. The captures show IPv4 IHL 6/protocol 2 traffic, including destination 224.0.0.22. Local diagnosis identifies an instrumentation classifier defect; this is not evidence of a Linux forwarding or firmware bug.

The [new record](../../../corpus/devices/linux/ubuntu-26.04-wsl2-x86_64/6.18.33.2-microsoft-standard-wsl2-7c656fcc/runs/linux-forwarding-20261024T074115Z-f8a8526f/record.json) uses scenario `linux-static-ipv4-v2` and comparator revision 2. It explicitly excludes and counts unrelated multicast IGMP reports. It reruns the experiment and passes **9/9 comparisons, comprising 72 invariant checks**. Expected ICMP outcomes were unchanged. Original captures, source snapshots, checksums, and failure status were not rewritten. The validator reproduces both revisions and rejects switching a comparator revision under an existing scenario contract.

Required checks cover fixture outcome, router ingress IP bytes, source-received ICMP type/code, actual IP/ICMP checksums, forwarding or absence of forwarding, egress Ethernet rewrite, exact ICMP payload/error quote, and zero kernel capture drops. Generated response IPv4 IDs are not required to match between stacks; actual checksums must validate. Ethernet padding and timing equality are excluded before interpreting results. These are bounded semantic and byte comparisons, not firmware-wide conformance.

## Commands and validation

```powershell
./scripts/corpus/run-linux-lab.ps1 -Distro Ubuntu-26.04 -Repetitions 3
wsl -d Ubuntu-26.04 -- python3 /mnt/c/Users/ricky/src/wirebox/scripts/corpus/validate.py --repo /mnt/c/Users/ricky/src/wirebox
wsl -d Ubuntu-26.04 -- python3 -m unittest discover -s /mnt/c/Users/ricky/src/wirebox/scripts/corpus -p test_corpus.py -v
```

The launcher built `wyrlex-headless` with `cargo build --locked -p wyrlex-headless`. Capture result and initial validation: [experiment.log](experiment.log). Final validator output: [validation.json](validation.json). Negative controls: [tests.log](tests.log). Final source/schema hashes and artifact references are in [manifest.json](manifest.json). No new packages were installed; the experiment used existing WSL tools. Remote CI and UI interaction were not run.

The validator checks schema, paths, content hashes and sizes, artifact completeness, target/scenario identity, case repetitions, required checks, capture completion/drop counts, PCAP/JSONL byte/timestamp agreement, recorded cleanup, and re-computation of the comparison. Integrity passing does not mean a failed comparison is promoted to success. The known failed record is intentionally retained.

Fourteen tests cover both historical results, corrupted and rehashed PCAP, fabricated reports, missing case repetitions/invariants, unknown schema/revision, path traversal, absent capture, a changed TTL with a recomputed valid checksum, capture drops, classifier revision handling, and safe environment path components. These tests validate the evidence pipeline; they do not establish authenticity of arbitrary external contributions or human understanding.

## Gate status and remaining work

P5-05 now has a reproducible Linux namespace fixture and capture probe. P5-06 has an initial independent comparator for S01–S03; additional negative cases, ARP failure/retry policy, and broader comparison coverage remain open. P5-07 and the overall G5 decision remain open. No Linux/MikroTik observation adapter, vendor firmware record, or Phase 6/7/8 pilot is complete.

The corpus stores behavior evidence and software-component identities, not firmware binaries. Durable simulation checkpoints, cross-version engine restore, optical-physics models, packet scheduling equivalence with Linux, and broad upgrade/interoperability guarantees are outside the result. [Corpus usage and structure](../../../corpus/README.md) and [learning notes](../../learning/phase_5/phase_5_corpus_linux.md) describe the current workflow.
