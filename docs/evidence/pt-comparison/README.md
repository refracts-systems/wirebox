# Packet Tracer initial audit and benchmark baseline

Local run **21 October 2026**, Windows x64. User request: use REA and Ghidra headless against Packet Tracer 9.0.1 installation to start Wirebox benchmark testing.

**Result: targeted static audit and PT runtime smoke test available; S01–S05 equivalence has not been evaluated.** Wirebox passed its own acceptance. The PT example being run has a different topology/config, so these two results cannot be combined into a verdict of equality or superiority.

## Targets and tools

| Items | Identity / results |
|---|---|
| Targets | `C:\Program Files\Cisco Packet Tracer 9.0.1\bin\PacketTracer.exe`, 60,005,312 bytes |
| SHA-256 | `843579cc806a41d57a4ca524d6805b97ee1f91e0ddd02ac09be8461db04b94a1` |
| REA | CLI `rea-agents@6.5.0`; [artifact evidence](rea-artifact.json), [managed check](rea-managed-check.json) |
| Ghidra | 12.1.4, Java 25; local project `PTProj`, program `/PacketTracer.exe`; the project input has an identical hash to the attached target |
| REA native Ghidra | Full import/analysis timeout 330,000 ms; [status](rea-status.json). Does not produce decompilation used as evidence |
| REA native UI | Not available on Windows on active providers; [original error](pt-ui-observe.json) |
| Fallback | Ghidra hands-on, existing project analysis, `-noanalysis`; smoke GUI uses Windows UIAutomation and mouse messages to the PT | window

The REA is not written to the vendor installation folder and the MCP Codex configuration is not changed. REA artifact evidence has ID `ev_7aa5c12de345210113e7021e81edfea0313c1ad129e92c2d1770167ab3dfef66`; managed check `ev_c2093ec66b05da007741c5e466daa6b780434f220a822e6cfb08fa051661b053` states there is no CLI data directory. Evidence artifact only proves the identity/inventory of the PE.

## Directional static checking

[First run log](ghidra-run.log) records four functions being decompiled. The following interpretation comes from reading the local pseudocode; address is valid only for the above target hash.

| Function | Local observations | Conclusion limits |
|---|---|---|
| `0x141d1d3e0` | `Simulation::CScheduler` vftable Assignment; storage via `+0x18`; 16 byte striding element, `2*i+2`/sibling child selection and parent shift, 16 byte end subtraction; Qt destructor call on `+0x48/+0x50/+0x58` | Consistent with binary heap unwinding. There is indirection to storage; don't call the inline vector in `+0x18` a fixed layout. This function itself does not prove the `(time,id)` sequence, RB-tree registry type, or thread lifecycle |
| `0x141d22cf0` | Lookup helper, error `SimulationTimer`, RTTI `CIpcTimer`, and error path `getIpcTimer implementation error` | Supports separate timer lookup/validation. Lookup helper details have not been audited on this run |
| `0x14094c530` | Path with RIB, unreachable host, and ICMP time exceeded diagnostics; construction of message type `0xb`, code `0` on the related path | Supports routing/ICMP software. Haven't recovered all forwarding conditions or proven runtime bytes/checksums |
| `0x142580fc0` | `QMessageBox`, maximum events, Clear Event List and View Previous Events | Event List GUI limits are real in the code. GUI policies are different from heap structures; This run does not measure behavior when the buffer is completely full |

So legacy heap claims and log panels get additional local support. Map-by-ID, time/ID comparator, worker thread, jitter distribution, and complete checkpoint claims still require other functions/observations. The presence of a routing code does not establish protocol conformance.

Repeatable extractor: [WireboxPtAudit.java](../../../scripts/ghidra/WireboxPtAudit.java). It checks the program hash, decompiles four functions, and records marker and body size. Markers only aid review, not behavioral oracles. The full pseudocode output is stored locally in Git's ignored `artifacts/pt-audit/`.

[Extractor results](ghidra-audit.json) and [read-only run log](ghidra-audit-run.log) show four decompilations completed and all selected markers found. The body sizes are 589, 966, 26,901, and 944 address bytes, respectively. Hash verification is performed against program metadata in the Ghidra database, in addition to matching the hashes of project input files and installed binaries.

Restart from the root repo with the analyzed project; this command does not create a new database:

```powershell
$ghidra = 'C:\Users\ricky\Downloads\Compressed\ghidra_12.1.4_PUBLIC_20260921\ghidra_12.1.4_PUBLIC'
$projectDir = 'C:\Users\ricky\AppData\Local\Temp\opencode\ghidra_pt\proj'
$repo = (Get-Location).Path
$sha = (Get-FileHash 'C:\Program Files\Cisco Packet Tracer 9.0.1\bin\PacketTracer.exe' -Algorithm SHA256).Hash
& "$ghidra\support\analyzeHeadless.bat" $projectDir PTProj -process PacketTracer.exe -readOnly -noanalysis -scriptPath "$repo\scripts\ghidra" -postScript WireboxPtAudit.java "$repo\artifacts\pt-audit" $sha
if ($LASTEXITCODE -ne 0) { throw 'Ghidra failed; inspect its log and JSON completeness' }
npx -y rea-agents@6.5.0 inspect-artifact 'C:\Program Files\Cisco Packet Tracer 9.0.1\bin\PacketTracer.exe' --format json
npx -y rea-agents@6.5.0 inspect-managed-artifact 'C:\Program Files\Cisco Packet Tracer 9.0.1\bin\PacketTracer.exe' --format json
./target/debug/wirebox-headless.exe --check-network
```

Java/Ghidra prerequisites must be available; do not use `-noanalysis` on a new database as a replacement for analysis. Ghidra sometimes returns exit 0 even though the script fails; also check that the JSON is complete, all four `decompile_completed` are true, and the hash is true. Changes in pseudocode format can change markers without changing the algorithm; Missing markers require re-examination.

## The runtime that actually ran

[PT Observations](pt-runtime-observations.json) is taken from the **User Created PDU** table, using the default example `HSRP/Hsrp_Simple_Ping_With_Same_Priority.pkt` (hash recorded in artifact):

| PT Probe | GUI Status |
|---|---|
| Simple PDU ICMP PC0 → Router0 | Successful |
| Simple PDU ICMP PC0 → PC1 | Failed |

The cause of the second probe has not been determined. The table value `Time(sec)=0.000` is not used as latency. No raw PDUs, checksums, packet capture, or decision traces were collected for these two probes. The HSRP, two switches, and three routers in this example are different from the Wirebox's four node fixture. This is the PT runtime smoke test, not the S01/S02 results. Session changes are not saved; the sample file hash after closing is the same as before testing.

[New Wirebox Baseline](wirebox-network.json): **36 scenario runs and 37 negative input passes**; S01 echo reply, S02 no route, S03 TTL expired, S04 ARP timeout, S05 truncated IPv4. [Core verification](wirebox-verify.log) also passed. Details of the oracle and model boundaries remain following [G3](../phase-3/README.md). No PT results were entered as false expected data.

## Corpus equivalents that have yet to be executed

Use [normative fixture](../../../fixtures/scenarios/icmp-static-v1.json) and [comparison runbook](../../packet-tracer/comparison-tests.md). No `.pkt` for that fixture was created in this run. The current verdict value is **not_evaluated**, not a pass and not a mismatch.

Before saying it is better than PT, it requires pairwise results on equivalent inputs: packet/state correctness, verifiable byte description, successful diagnosis by the user, and performance on the same machine and workload. Firmware/vendor coverage still requires capture/read-back Linux or specified devices, separate from PT.
