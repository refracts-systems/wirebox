# Cisco Packet Tracer 9.0.1 reverse engineering notes

## Conclusions that can be used

The report shows object-based simulation models of devices, ports, links, PDUs, protocol processes, and virtual time schedulers. The local mini-simulator demonstrates that some of these behaviors can be implemented without a physical device and without a vendor OS image.

Architectural findings are design references. The findings do not yet prove the equivalence of the new implementation to all Packet Tracer behavior or production devices.

## Knowledge status

| Category | Meaning |
|---|---|
| Reported confirmed | Reports mention evidence of RTTI, resource, decompilation, or runtime testing; Evidence of provenance has not been completely re-audited in this repository. |
| Verified local | The mini-simulator source or executable is examined in this documentation session. |
| Inference | An explanation that is consistent with the evidence but has not been demonstrated directly. |
| Proposal | A design decision for Wyrlex, not a result of RE Packet Tracer. |
| Unknown | Evidence is insufficient or implementation has not been successfully reproduced. |

## Corrections that must be read first

1. Scheduler queue and Event List GUI are separate components. The number of trace lines is not the same as the number of scheduler events.
2. Hypothesis of AES/Rijndael + Mash as .pkt container encryption replaced by Twofish/EAX document decoder report. The encrypted attribute path must be distinguished from the document container.
3. Searching the default passphrase of the .pkt container is no longer the final conclusion. The final report calls key and nonce constant for the family of documents tested.
4. Offline readers still depend on the captured keystream; independent cipher reproduction has not been successful. Sample reading does not prove writer or round-trip is available.
5. The 0xBA/0xBE constant pair is reported to be related to uuid/nets metadata, not proven to be a decoder extension.
6. The class name and flow-chart code are not a recovery of the entire algorithm. The available evidence also does not constitute an audit of all OS socket paths.

## Reading order

Starting from [architecture](architecture.md), then [evidence ledger](evidence-ledger.md) and [mini-simulator](mini-simulator.md). Read [.pkt format](pkt-format.md) for interoperability context. [Wyrlex Implications](wirebox-implications.md) separates implementation proposals from findings.

[REA/Ghidra local audit and PT runtime smoke](../evidence/pt-comparison/README.md) noted re-examination October 21, 2026. [Comparison runbook](comparison-tests.md) differentiates smoke, protocol subset equivalence, and superiority claims.

Historical reports are stored as is at [sources](sources/README.md), including older claims that were later corrected. The thematic summaries in this directory serve as a guide to the current status of available materials.
