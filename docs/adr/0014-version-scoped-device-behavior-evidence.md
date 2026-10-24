# ADR-0014: Version-scoped reverse engineering and device behavior evidence

- Status: Accepted as the evidence and investigation policy; vendor implementation remains pending
- Date: 24 October 2026
- Decision authority: explicit user request to formalize the device/firmware reverse-engineering and corpus approach
- Scope: Linux, MikroTik RouterOS, ZTE OLTs, Huawei ONUs, VSOL ONUs, and their tested interoperability combinations
- Related decisions: [ADR-0001](0001-semantic-state.md), [ADR-0005](0005-pilot-scope.md), [ADR-0007](0007-packet-model.md), [ADR-0012](0012-shared-verify-ir.md), and [ADR-0013](0013-phase-5-checkpoint-and-impairment.md)

## Context

Wyrlex needs a defensible connection between its shared semantic model and the behavior of real devices. Packet Tracer reverse-engineering reports informed the architecture, and the first [Linux WSL2 corpus comparison](../evidence/corpus-linux/README.md) provides independent execution evidence for a small packet subset. Neither establishes RouterOS, OLT, or ONU fidelity.

The user requests reverse-engineering evidence for firmware versions, or at least major-version baselines, as part of the [device behavior corpus](../wirebox/behavior-corpus.md). A major version is useful for organizing investigation effort, but it is too broad to identify the artifact actually inspected. A device can contain several independently versioned software components. A successful OLT/ONU pairing also depends on both endpoints and the service configuration.

This ADR defines what to investigate, how to identify targets, how static findings relate to runtime observations, and when evidence may be reused. Acceptance adopts this policy; it does not report new reverse engineering, device access, firmware acquisition, symbolic verification, or a passed vendor gate.

## Terminology

| Term | Meaning |
|---|---|
| Device model | Product/hardware identity, including relevant board and hardware revisions; for example, an OLT chassis and its installed boards |
| Software artifact | Exact bytes inspected: a firmware image, package, extracted executable, library, or other relevant component |
| Firmware/software version | Vendor version/build label for a specified component; a label alone does not uniquely identify its bytes |
| Target identity | The device/platform, architecture, relevant component versions, operating mode, and artifact identities available for the investigation |
| Measurement context | Tools, instrument versions, configuration, topology, input, initial state, clocks, and execution method for one analysis or run |
| Wyrlex semantic model | Explicit rules and state transitions describing supported behavior, such as route selection, VLAN transformation, or an unsupported outcome |
| Behavior claim | A bounded statement about a capability under stated conditions, linked to evidence and unresolved alternatives |
| Interoperability combination | Exact participating target identities, roles, service configuration, and conditions tested together |

The semantic model is an abstraction executed by Wyrlex. It is not the vendor OS, a firmware image, or a complete hardware replica. A commercial product model and a semantic model must not share an ambiguous `model` label in future evidence contracts.

## Decision

### 1. Investigate selected behavior across the pilot device families

Reverse engineering may cover RouterOS components, ZTE OLT components, and Huawei/VSOL ONU firmware. Investigation is driven by a named behavior claim and pilot scenario. It does not require reconstructing every subsystem of every OS.

| Target | Initial questions and candidate investigation scope | Required distinctions |
|---|---|---|
| Linux | Static IPv4 routing, bridging, ARP, TTL, and ICMP behavior relevant to comparison fixtures | Distribution, kernel build, architecture, modules/configuration, virtualization, and offload mode where relevant |
| MikroTik RouterOS | Route selection and packet-processing order; bridge/VLAN and filter behavior as required by the pilot | Physical model versus CHR, architecture, exact RouterOS build, installed packages, and software versus offloaded forwarding |
| ZTE OLT | VLAN/service/GEM/port mapping, forwarding/isolation, ONU state transitions, and relevant OMCI handling | Chassis, board types/revisions, slot roles, component versions, PON mode, and control-plane versus data-plane evidence |
| Huawei ONU | UNI VLAN handling, bridging/service mapping, and response to relevant OMCI configuration | Exact ONU model, hardware revision, firmware build, management mode, and its OLT peer |
| VSOL ONU | The same selected service questions, investigated independently | Exact ONU model, hardware revision, firmware build, management mode, and its OLT peer |

These are investigation candidates, not implemented capabilities or a promise that binary analysis can recover each mechanism. NAT, connection tracking, PPPoE, dynamic routing, timing fidelity, PON scheduling, and optical physics require separate scope and validation before support is claimed. Stateful behavior remains outside the initial stateless Verify proposal in ADR-0012.

For open-source Linux components, matching source/build configuration and symbols may provide clearer mechanism evidence than decompilation. Record the source-to-running-build relationship and any uncertainty. A REA report is not mandatory for an ordinary source investigation.

### 2. Bind evidence to exact targets and artifacts

Major/minor version folders are navigation aids. A claim's actual scope must identify:

- vendor, device model or virtual platform, architecture, and relevant hardware/board revisions;
- exact version/build of each component relevant to the claim, including installed packages where applicable;
- SHA-256 of the original artifact when available and of every extracted component actually analyzed;
- extraction lineage, component path, and known source ranges; decompressed offsets must not be presented as original-image addresses without a demonstrated mapping;
- observed operating mode and forwarding path, including hardware-offload state when relevant;
- the configuration, scenario, initial state, and peer identities used for a runtime claim.

An OLT software identity can be a set of board/component versions. A single chassis-wide version string must not replace that set. Bootloader versions are recorded when relevant; their presence alone does not establish forwarding semantics.

Unknown values remain explicitly unknown. Runtime measurements may be retained when an image hash is unavailable, but cannot be described as bound to a verified firmware image. A downloaded package with a matching version label is not automatically proven to be the software running on a device. Record how the analysis artifact was associated with the runtime target, and retain that association as unknown if it cannot be established.

A content hash establishes artifact identity/integrity. It does not establish authenticity, device origin, measurement honesty, or equivalence of the complete deployed system.

### 3. Keep three complementary evidence layers

| Layer | Required material | Permitted conclusion |
|---|---|---|
| Mechanism analysis | Exact artifact or matching source, tool/provider identity, analysis parameters, references, findings, and limitations | A specified mechanism is observed or inferred in the inspected artifact/source |
| Runtime observation | Exact target context, controlled input/configuration, initial state, probes/captures/read-back, repetitions, and capture-quality limits | The target exhibited the recorded behavior under those conditions |
| Wyrlex comparison | Model/build revision, scenario contract, predefined invariants/normalizations, independent observations, and mismatch details | Wyrlex agrees or disagrees with the observations for that contract |

Static analysis does not observe an executed path. A successful runtime case does not reveal the complete internal algorithm. Agreement with a model does not prove all packet inputs, hardware paths, or firmware builds equivalent.

For a closed-source capability selected for support, maintain a dossier containing all available layers and an explicit status for every missing layer. Mechanism analysis is expected where a relevant artifact and supported method are available. Missing static analysis does not invalidate a reproducible runtime result; it limits its explanation and must remain visible. Conversely, static findings alone cannot pass a runtime or interoperability gate.

### 4. Use REA as the preferred analysis/evidence interface where applicable

REA is the preferred interface for shipped-artifact inspection when the required provider supports the target. Record REA's exact version/build and the actual engine/provider, its version, executable digest where available, architecture/load profile, parameters, and analysis date. Retain evidence identifiers, raw reports, relevant exports, diagnostics, and output hashes.

Firmware inspection/extraction and native program analysis are separate steps. An extraction report must not be described as decompilation. A decompiled function must link to the actual extracted binary and its lineage. Supported tools and providers must be checked for the selected artifact; installing a skill or passing a general diagnostic is not evidence of firmware-analysis readiness.

If REA cannot analyze a relevant target, record the unavailable capability and use an applicable alternative if available. Direct Ghidra work must be labeled as direct Ghidra work. Never relabel an external analysis as a REA execution merely because it used a similar method. Partial extraction, unsupported architectures, encrypted regions, unresolved calls, uncertain types, and unobserved paths remain explicit limitations.

A dossier is evidence-bearing because of its target binding, method, retained outputs, and bounded conclusions. The tool name alone is not a proof of correctness.

### 5. Allocate effort by baseline and change impact

| Situation | Required investigation policy |
|---|---|
| First supported major family/model/architecture | Establish a baseline for selected capabilities: target inventory, relevant mechanism analysis where available, controlled runtime cases, and Wyrlex comparison |
| New minor/patch/build | Register a distinct target/artifact identity, compare relevant components, assess change impact, and rerun the applicable behavior suite before extending the tested claim |
| Changed relevant component, unexplained mismatch, or changed forwarding mode | Reopen mechanism analysis and add discriminating/negative cases; narrow affected claims until resolved |
| Byte-identical analyzed component in another image | Reuse the component-level finding by explicit hash/reference, documenting dependency, configuration, and hardware assumptions; runtime support is assessed separately |
| Version/build without evidence | Mark it untested/unknown; do not inherit support from a matching major version or neighboring release |

Whole-firmware decompilation for every patch is not required. Focused diffs can prioritize work, but unchanged strings, filenames, version labels, or one component hash cannot establish unchanged system behavior. Evidence reuse needs a written scope and rationale; it must never silently promote an untested combination.

No fixed number of packets proves complete equivalence. Scenario contracts define mandatory cases and repetitions before acceptance. Use boundary, negative, and distinguishing inputs that can reveal competing semantic interpretations, not only repeated healthy pings.

### 6. Track single-device behavior and interoperability separately

An interoperability record references each exact target identity and its role. For the ISP pilot this can include:

```text
OLT chassis + control/PON board components + operating mode
    + ONU model/hardware/firmware
    + VLAN/service/GEM/UNI configuration
    + router/uplink/endpoint targets when part of the tested path
    + scenario, initial state, input, and expected service properties
```

ZTE–Huawei and ZTE–VSOL are separate rows. A changed ONU firmware, OLT board build, or relevant service configuration requires impact assessment and new applicable runs. Identical registration status does not prove VLAN behavior, isolation, bidirectional service, throughput, or safe transitions.

Software forwarding, virtual appliances, device simulators, and physical hardware are distinct measurement contexts. A CLI prompt that resembles a device is insufficient to classify it as a physical unit. ASIC/firmware interaction and offload paths need observations on the relevant path before hardware claims are made.

### 7. Link findings to semantic rules and tests

The intended traceability chain is:

```text
target/artifact -> analysis finding + runtime case -> behavior claim
    -> versioned semantic rule -> simulator/Verify interpretation
    -> comparison/regression -> scoped support assessment
```

Every adopted vendor-specific rule must identify its evidence, assumptions, supported target set, and unsupported conditions. Generic protocol rules can link to applicable specifications and independent tests without inventing vendor-specific internals. Static analysis should generate testable hypotheses; controlled tests should distinguish them where possible.

The proposed shared IR in ADR-0012 remains proposed. This ADR does not select a symbolic backend, add a second vendor simulator, or introduce a new FFI. When Verify exists, simulator/verifier differential tests check agreement within their shared model. Independent device comparisons remain necessary to assess whether that model describes the target.

Disagreements must be classified after investigation: model defect, adapter/lowering defect, instrumentation defect, unsupported behavior, observed target difference, or inconclusive result. A vendor-bug or standards-violation claim additionally needs the applicable contract/specification and evidence addressing alternative causes.

### 8. Keep evidence dimensions independent

Future dossiers must expose at least these independent assessments:

| Dimension | Example states and meaning |
|---|---|
| Knowledge classification | Reported, locally verified, inference, proposal, or unknown, following the [existing evidence discipline](../packet-tracer/README.md#knowledge-status) |
| Mechanism analysis | Not attempted, unavailable, partial, or analyzed for a stated scope; each finding still has its own knowledge classification |
| Runtime experiment | Not run, inconclusive, or observed with stated case/repetition coverage |
| Wyrlex comparison | Not run, pass, fail, or unsupported for a specified contract |
| Content integrity | Unchecked, valid, or invalid; independent of whether the comparison passes |
| Review | Unreviewed or reviewed, with reviewer identity/method; agent review must not be presented as independent human review |
| Support assessment | Untested, experimental, or tested for an explicit target/capability/scenario set |

These labels describe future dossier semantics, not newly implemented JSON enums. A single `verified: true` cannot represent the above dimensions. Do not reinterpret the existing Linux `evidence_status: locally_verified` as firmware-image verification or full support.

A record may be structurally valid and contain a failing experiment. A runtime result may be strong while static coverage is partial. A static finding may be well supported while the running-device association is unknown. Conflicts must remain visible rather than being averaged into one confidence score.

### 9. Preserve append-only provenance and explicit schema boundaries

The current corpus layout and historical Linux records remain valid. The following is an intended extension, not an implemented schema or a migration of existing runs:

```text
corpus/
  devices/<family>/<model-platform>/<version-and-identity>/
    target.json
    analysis/<analysis-id>/
      manifest.json           # Artifact binding, method, tools, hashes, and gaps
      findings.md             # Bounded claims and retained evidence references
      evidence/               # Permitted reports/exports with content hashes
    runs/<run-id>/            # Runtime records and observations
  combinations/<combination-id>/
    ...                       # Exact peer references and service-specific runs
```

The existing Linux target identity includes tool versions. Do not rewrite it to retrofit a cleaner separation between target identity and measurement context. A future schema can separate those concepts with explicit versioning and migration/reference rules. The current runtime record schema and validator are Linux-specific; they do not yet admit or validate analysis dossiers or vendor combinations.

Before adding those artifacts, define their schemas, safe path/reference rules, compatibility behavior, immutable IDs, hashes, and validators. Cross-links should use retained artifact/finding/claim IDs and content hashes, not machine-local absolute paths. Do not insert new analysis files into historical run directories whose artifact sets are already frozen.

Corrections create new records referencing the superseded finding/run and their reason. Preserve failed comparisons and tool limitations. Redaction creates a declared derivative with a new hash and lineage to its restricted source; changing historical bytes in place is not a correction.

Firmware images, extracted proprietary executables, credentials, and customer configurations are not added to public Git by this decision. Store permitted reports, sanitized minimal fixtures, metadata, and content hashes; retain restricted inputs outside Git under explicit local/access-controlled references. Sharing, licensing, external disclosure, certification, pricing, and a public contribution program remain separate decisions under the [corpus publication boundaries](../wirebox/behavior-corpus.md#contribution-and-publication-boundaries).

## Investigation workflow and acceptance criteria

For each selected behavior:

1. Write the question and competing interpretations, target identity, relevant hardware/software path, and proposed claim scope.
2. Establish artifact provenance and its association with the runtime target. Record unknowns before analysis.
3. Inspect/extract only the relevant material, retain hashes/lineage, and analyze supported components using REA or a correctly identified alternative.
4. Record findings with concrete references: component digest, function/address or source location, evidence IDs, method, observation/inference, and limitations. Addresses are scoped to the inspected build/load profile.
5. Freeze a discriminating runtime scenario: minimal configuration, initial state, positive/negative inputs, expected invariants, normalizations, repetitions, and capture-quality requirements.
6. Execute in the identified lab context, retain raw observations/read-back, and record completion, capture loss, uncertainty, and failures.
7. Compare the supported Wyrlex behavior; investigate mismatches before altering the expected contract. Retain old results when a comparator or scenario needs correction.
8. Link the reviewed claim to semantic rules and regression cases; publish only the tested support scope and explicit exclusions.

A mechanism dossier is complete for its declared scope when the artifact/tool provenance, retained references, findings, gaps, and assessment can be inspected. A behavior claim is runtime-tested only after its required cases produce usable observations on the stated target. A Wyrlex comparison passes only when all mandatory invariants pass without unresolved critical mismatches. An interoperability claim also requires the exact peer/service context.

Unavailable static analysis is a recorded gap rather than an invented result or an automatic pass. Failed capture/instrumentation can make an experiment inconclusive; it does not establish a device defect. None of these document-level completion conditions automatically passes G5–G8.

## Worked investigation examples

These examples define investigation plans, not findings about an inspected vendor artifact:

- **RouterOS route selection:** choose one exact build/platform and ask which route wins for a packet when prefixes, priorities, or route availability differ within the intended support scope. Record candidate explanations before analysis. Identify relevant code/configuration where possible, create packet inputs that distinguish the candidates, observe selected egress/response on the target, and compare Wyrlex's decision and byte changes. Equal-prefix behavior stays unsupported until its own contract and cases exist. A software/CHR result does not establish hardware-offloaded forwarding.
- **ZTE–ONU VLAN service:** select one OLT/board build, one exact ONU firmware, and one minimal service configuration. Observe tagged/untagged ingress and egress at available boundaries, including reverse direction and isolation negatives. Analyze relevant VLAN/OMCI/service components where accessible. If only endpoint captures are available, record the internal PON/OMCI behavior as unobserved. Repeat for the other ONU family and create a distinct combination record; registration success alone cannot substitute for service tests.

In either example, changing firmware produces a new target context and impact assessment. Existing evidence can guide the next experiment, while the new build remains untested until its required runs are recorded.

## Rollout and phase responsibilities

| Stage | Work | Completion evidence |
|---|---|---|
| Current documentation decision | Adopt this ADR and connect it to corpus guidance/checklists | Consistent English documentation and valid local links; no firmware analysis claimed |
| Phase 5 corpus foundation | Design versioned analysis/claim/combination contracts and validators without rewriting Linux history | Schema/rejection checks and historical-record compatibility when implementation occurs; broader Linux comparison and G5 remain separate |
| Phase 6 Linux/MikroTik | Complete runtime/firmware inventory; select one exact RouterOS 7 baseline and one bounded capability; create mechanism dossiers where feasible | Artifact/running-target association, read-only provenance, retained findings/gaps, and corresponding lab cases |
| Phase 7 semantic integration | Connect adopted claims to rule/operation lowering and supported Verify scope | Independent lab comparisons plus simulator/verifier tests when implemented; unresolved mismatches visible |
| Phase 8 ISP pilot | Establish ZTE board/component identities and separate Huawei/VSOL ONU baselines and combinations | Exact peer/service matrix, mechanism findings/gaps, repeated service/isolation observations, and scoped G8 assessment |
| Later releases | Extend builds and capabilities by change impact | New target records, explicit evidence reuse, regression results, and updated support boundaries |

Inventory and artifact availability can be investigated early without claiming later phases complete. Future firmware/version support is selected by pilot needs and available evidence, not by creating empty directories for all releases. REA/provider installation, firmware download, device mutation, and external publication are not performed by adopting this ADR.

## Current baseline and known gaps

As of 24 October 2026:

- The [Linux corpus](../../corpus/README.md) contains S01–S03 on one WSL2 target. Three repetitions per case pass in the corrected run; an earlier classifier failure is preserved. The kernel image hash is unknown. No REA mechanism dossier for this target exists.
- [Pilot inventory](../../fixtures/inventory/pilot-v1.json) reports RouterOS major version 7; the exact build, hardware model, architecture, access, and analysis artifact remain unknown.
- The same inventory reports a ZTE C320 and component version labels for GTXK, GTGHG, and SMXA boards. The physical-versus-simulated nature of the supplied CLI environment is unverified. No image hashes, analyzed firmware artifacts, or independently verified data-plane behavior are established.
- Huawei and VSOL ONU model/hardware/firmware identities remain incomplete. The reported registration state does not establish those identities or a tested interoperability contract.
- [Packet Tracer evidence](../packet-tracer/README.md) applies to its reported/inspected artifacts and scenarios. It remains a design/comparison reference and cannot serve as evidence for RouterOS, OLT, or ONU firmware internals.
- Vendor analysis/claim/combination schemas, REA automation for this corpus, vendor observation adapters, and shared symbolic verification are not implemented by this ADR. At adoption, G5 and G6–G8 were open. The subsequent [G5 assessment](../evidence/phase-5-gate/README.md) passes scoped replay/Linux comparisons; vendor dossiers, G6–G8, and the analysis/claim/combination schema extensions remain pending.

## Alternatives considered

| Alternative | Reason not selected |
|---|---|
| One report or support flag per vendor/major version | Hides build, hardware, mode, component, and peer differences; evidence cannot justify the implied scope |
| Reverse engineer every release completely before any lab result | Unbounded effort; blocks useful controlled measurements and still does not observe hardware behavior |
| Runtime tests only, with no mechanism investigation | Supports bounded observations but leaves mechanism/order ambiguities harder to explain; retain runtime-only evidence while recording the gap |
| Static/REA reports alone | Cannot establish executed paths, deployment association, or device interoperability |
| Copy the entire vendor OS or emulate all hardware | Exceeds Wyrlex's behavioral-model scope and is unnecessary for the selected scenarios |
| Separate rules for each simulator/verifier | Duplicates semantics and conflicts with the shared-IR direction; vendor differences need explicit applicability and lowering |
| Rewrite historical corpus records when understanding improves | Destroys the provenance of failures, original interpretations, and corrections |

## Consequences

The corpus becomes a set of scoped, explainable behavior claims rather than a collection of firmware names. Investigation effort can follow changed components and pilot needs. Evidence can be reused at component scope without implying unsupported whole-system equivalence. Simulator and future Verify results can expose the provenance and limits of the rules they execute.

The cost is maintaining target/component identity, artifact-to-device association, tooling context, negative cases, and corrections. Some mechanisms will remain inaccessible or ambiguous; hardware observations and unavailable firmware can limit scope. Metadata and hashes do not remove those gaps. Initial delivery can remain runtime-first while making missing mechanism evidence explicit.

This policy does not promise vendor certification, all-version compatibility, full firmware recovery, safe upgrades, or exhaustive correctness. Material changes require a superseding ADR; implementation details must preserve the recorded evidence boundaries.

## References

Repository sources: [corpus structure](../../corpus/README.md), [behavior corpus strategy](../wirebox/behavior-corpus.md), [pilot contract](../wirebox/pilot.md), [roadmap](../wirebox/roadmap.md), and [phase todos](../wirebox/todos.md).

External documentation checked on 24 October 2026:

- [REA overview](https://rea.tools/): code inspection, references, and runtime observations as investigation inputs.
- [REA firmware analysis](https://github.com/morluto/rea/blob/main/docs/firmware-analysis.md): firmware-region inspection, explicit extraction, artifact/tool provenance, partial-result limitations, and separate admission of supported native children. This reference does not establish that any provider was installed or run for a Wyrlex vendor target.

External pages can change. Actual investigations must record the exact tools and retained outputs used at run time rather than relying on this documentation check as toolchain pinning.
