# A reproducible corpus of device behavior

Recorded 24 October 2026 from a user-provided strategic proposal. Status: **product and research hypothesis**. No shared vendor corpus, certification program, contribution network, commercial offering, or licensing change has been implemented or selected.

## Thesis

Wyrlex's most valuable long-term asset could be accumulated knowledge of how real devices behave under reproducible conditions. Semantic fidelity is currently one of the project's largest risks. Systematically investigating that risk could create a durable advantage: a versioned, testable body of evidence per device model, firmware build, feature, and interoperability combination.

The engine and verifier can be independently reimplemented. Reproducing a well-maintained corpus of lab fixtures, packet captures, negative cases, firmware differences, and explained mismatches requires continued measurement and access to tested devices. That is the proposed competitive advantage; its value and defensibility still depend on corpus quality, adoption, and useful coverage.

The initial focus follows the pilot: Linux, MikroTik RouterOS 7, ZTE C320, and separately tested Huawei and VSOL ONUs. Support is defined by tested capabilities and combinations, not by vendor names alone.

## From mismatch to evidence

Phase 5 plans a simulator-versus-Linux lab comparison; Phase 6 adds observed state and Phase 8 extends the tested device matrix. Every mismatch is a useful investigation input. It is **not automatically a verified fact about firmware or a vendor bug**. The cause might be a simulator defect, unsupported behavior, capture loss, incorrect configuration, timing, or a bad test assumption.

The intended evidence workflow is:

1. Preserve the scenario, target identity, exact versions, relevant sanitized configuration, model version, and raw observations.
2. Minimize the case and reproduce the difference. Record repetitions, instrumentation, and timing conditions.
3. Compare only declared invariants; identify fields that may legitimately differ and explain tolerances before interpreting the result.
4. Classify the result: simulator defect, observed device behavior, supported difference, unsupported case, inconclusive result, or potential vendor issue requiring further investigation.
5. Add a versioned fixture and regression test, with links from the model rule to its evidence. Preserve superseded observations and corrections.

An observed difference becomes a scoped behavioral record after reproduction and review. A standards-conformance or vendor-bug claim additionally needs an applicable specification or documented contract and evidence that competing explanations were addressed.

## What a behavior record should contain

This is a proposed record contract, not an implemented schema:

| Category | Required context |
|---|---|
| Identity | Vendor, model, hardware/board revision, firmware version/build, image hash where available, and relevant operating mode |
| Scope | Capability or rule under test, traffic direction, protocol subset, service topology, and interoperability peers |
| Setup | Minimal sanitized configuration, external dependencies, workload, seed where applicable, and instrumentation |
| Observation | Commands/probes/captures, timestamps and clock assumptions, execution order, repetitions, and raw artifact hashes |
| Comparison | Model version, expected versus observed invariants, predefined tolerances, mismatch details, and counterexample bytes when relevant |
| Assessment | Evidence status, reviewer/method, unresolved alternatives, limitations, and linked model/test changes |
| Governance | Record revision, provenance, sharing consent, redaction review, and applicable redistribution/publication permissions |

Operational inventories and corpus records serve different purposes. An inventory says what was observed in an environment. A corpus record describes reproducible behavior under a bounded set of conditions. Neither should silently imply support for every deployment sharing a model name.

## Capabilities the corpus could unlock

**Firmware-change assessment.** Run the same test suite on RouterOS 7.x and 7.y and explain observed behavioral differences before a production upgrade. Results apply to tested features, configurations, and hardware; a passing suite cannot certify all possible effects of an upgrade.

**Cross-vendor interoperability.** Record whether a specific ONU/firmware and OLT/board/firmware combination satisfies a selected service contract. ZTE–Huawei and ZTE–VSOL need separate evidence rows. Success of registration alone does not establish VLAN behavior, service continuity, throughput, or every management capability.

**Behavioral regression and potential vendor defects.** Detect changes from a prior firmware baseline or a specified behavior. Keep unexpected differences visible even when they do not yet justify a vendor-bug claim.

**Shared improvements.** An ISP could explicitly opt in to contribute reviewed, sanitized behavioral results. Useful contributions could improve model coverage for other users. Adoption and contribution quality are prerequisites; network effects are a hypothesis rather than an existing asset.

**Temporal verification evidence.** The same records could support [safe transition analysis](temporal-verification.md): operation ordering, completion semantics, asynchronous effects, and temporary service states become testable facts for the specified targets. A configuration parser alone cannot supply those semantics.

## Business hypothesis

One possible model is an open engine accompanied by maintained behavioral datasets, reproducible compatibility assessments, and scoped verification reports. Potential customers include ISPs, distributors, and smaller ONU vendors needing independent interoperability evidence. Open-source licensing, dataset access, pricing, certification criteria, and who performs reviews remain undecided.

“Certification” would require a published test contract, exact target/version scope, reproducibility and review requirements, known exclusions, and a policy for expiration or invalidation when firmware or assumptions change. Until such a program exists, Wyrlex should publish test results with their scope rather than imply blanket certification.

## Contribution and publication boundaries

Sharing must be explicit and separate from using the local tool. Proposed contributions contain behavioral evidence, not customer configurations or credentials. Before sharing, review captures, addresses, serial numbers, account data, topology identifiers, and other identifying content; removing names alone does not establish sanitization. Any redaction that changes test meaning must be recorded so that the result remains interpretable.

Owning a device does not establish permission to redistribute its firmware or every associated artifact. Publication and redistribution rights, firmware/license terms, and handling of suspected defects remain matters to resolve before a public corpus or certification program is launched. This proposal makes no general legal conclusion and does not authorize external disclosure or data collection.

## Relationship to the roadmap

The Phase 5 Linux comparator is the first opportunity to establish a behavior-record workflow. Phase 6 can supply reproducible observation provenance. Phase 7 can link operation and Verify semantics to evidence. Phase 8 pilots can become the first narrowly scoped ISP interoperability records and, if participants consent, the first reviewed contributions.

The next design step is to agree on a small record schema and one reproducible lab case after the initial comparison is available. Corpus hosting, automated contribution ingestion, and commercial certification should follow evidence of demand rather than precede it. No new implementation or commercial gate is marked complete by recording this proposal.
