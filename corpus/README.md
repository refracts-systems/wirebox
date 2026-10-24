# Wyrlex device behavior corpus

Updated 24 October 2026. This directory contains reproducible, version-scoped behavior records. The initial target is **Ubuntu 26.04.1 on WSL2, Linux kernel 6.18.33.2-microsoft-standard-WSL2**. No RouterOS, OLT, or ONU behavior record exists yet. The broader [corpus proposal](../docs/wirebox/behavior-corpus.md) remains the strategic direction.

## Layout and identity

```text
corpus/
  schema/                  # Target, scenario, comparison, and record JSON schemas
  scenarios/               # Versioned test contracts and comparison exclusions
  devices/
    linux/
      <distribution-platform-architecture>/
        <kernel-version>-<identity-hash>/
          target.json
          runs/
            <run-id>/
              record.json
              comparison.json
              engine-provenance.json
              scenario.json
              runner-sources/
              S01-r1/      # Commands, observed state, PCAP/JSONL and capture statistics
```

The identity hash covers the model/platform, software components, architecture, and tool versions. A run is a new directory; existing runs and target identities are never overwritten. Software versions alone do not uniquely identify hardware or test instrumentation. Future firmware records should identify board/hardware revisions and list each relevant component in `identity.software`, including exact version/build and an image hash when available. Unknown image hashes remain `null`. Firmware binaries and customer configurations are not stored here.

Interoperability records will reference exact target identities and a service contract when vendor testing becomes available. Empty vendor directories and untested support rows are not evidence.

## Initial records

Target: [Linux identity and tool versions](devices/linux/ubuntu-26.04-wsl2-x86_64/6.18.33.2-microsoft-standard-wsl2-7c656fcc/target.json).

| Run | Scenario | Result and interpretation |
|---|---|---|
| [Initial capture](devices/linux/ubuntu-26.04-wsl2-x86_64/6.18.33.2-microsoft-standard-wsl2-7c656fcc/runs/linux-forwarding-20261024T073901Z-981dc0dc/record.json) | `linux-static-ipv4-v1`, comparator revision 1 | Failure in 4 of 9 comparisons: the classifier treated unrelated bridge IGMP reports as ICMP input. This is an instrumentation defect, not a vendor defect. Artifacts and original results are preserved |
| [Repeated experiment](devices/linux/ubuntu-26.04-wsl2-x86_64/6.18.33.2-microsoft-standard-wsl2-7c656fcc/runs/linux-forwarding-20261024T074115Z-f8a8526f/record.json) | `linux-static-ipv4-v2`, comparator revision 2 | 9 of 9 comparisons pass: healthy ping, missing route, and expired TTL, each repeated three times |

Revision 2 explicitly excludes unrelated multicast IGMP membership reports from the ICMP comparison and reports how many were ignored. It does not change expected ICMP behavior or rewrite the first record. The scenario/comparator revision pair is validated. Archived runner sources describe the tools at capture time; the current validator can replay both comparison revisions without executing arbitrary archived code.

## Reproduce the experiment

The current engine build remains Windows x64 MSVC with Zig 0.17.0 and Rust 1.99.1. Its headless output is compared with independently executed Linux kernel forwarding. A Linux engine build is not implied.

From PowerShell at the repository root:

```powershell
./scripts/corpus/run-linux-lab.ps1 -Distro Ubuntu-26.04 -Repetitions 3
```

The runner builds `wyrlex-headless`, exports S01–S03, and starts a Linux experiment as root inside the specified WSL distribution. It requires Python, iproute2, Linux network namespaces/veth/bridge/AF_PACKET support, and the distro's `jsonschema` package. Tested tool versions are recorded in the target; the runner does not install packages.

Each experiment creates its own four namespaces and private veth/bridge topology. No physical or WSL host interface is attached. Configuration, observations, and captures contain synthetic test-network data. Capture workers stop and the namespaces are deleted on normal completion or a handled exception. A forcibly terminated runner can require cleanup of its recorded namespace names; inspect its `commands.json` before removing anything.

## Validate existing evidence without root

```powershell
wsl -d Ubuntu-26.04 -- python3 /mnt/c/Users/ricky/src/wirebox/scripts/corpus/validate.py --repo /mnt/c/Users/ricky/src/wirebox
wsl -d Ubuntu-26.04 -- python3 -m unittest discover -s /mnt/c/Users/ricky/src/wirebox/scripts/corpus -p test_corpus.py -v
```

Adjust the mounted repository path for another checkout. The validator checks JSON Schema, safe paths, hashes, artifact completeness, case/repetition coverage, required invariants, capture statistics, PCAP/JSONL agreement, namespace cleanup, and reproduction of comparisons. A valid record can contain a failed comparison. `--require-pass --record <absolute-record-path>` additionally requires that specific comparison to pass; using it across the whole corpus will reject the intentionally preserved failed record.

Hashes establish content integrity, not authenticity of an external measurement or human understanding. Contributions still require provenance and review. Existing captures remain historical evidence even if current runner code changes.

## Comparison scope

The same original IPv4 bytes from each simulator scenario are sent through Linux with `IP_HDRINCL`. Linux independently resolves ARP, bridges, routes, decrements TTL, and generates echo replies or ICMP errors. The comparator checks router ingress bytes, the ICMP response received by the source, valid checksums, forwarding or absence of forwarding, Ethernet rewrite, and ICMP payload/error quote.

Ethernet padding is excluded. Generated response IP identification differs between stacks; each actual header checksum must still validate. Linux capture time and simulator virtual time are different clocks, so no timing equality is claimed. ARP retry timing, loss/jitter distributions, malformed-input policy, vendor firmware, and optical behavior are outside this initial contract.

The [evidence report](../docs/evidence/corpus-linux/README.md) records results and remaining G5 work. This corpus does not by itself complete G5, G6, or a vendor pilot.
