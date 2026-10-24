# ADR-0005 — Pilot v1 scope and staged evidence

- Date: 20 October 2026
- Status: Accepted as the Phase 0 working baseline. Unknown device models, firmware, and access remain unknown.

## Context

The plan must prove the initial idea without waiting for every vendor and protocol. The user provides a Linux VM/server, RouterOS major version 7, a C320 report for GTXK/GTGHG/SMXA boards, and a working GPON ONU example. The ONU output does not identify model/firmware; VSOL details are not available.

## Decision

Freeze scope, fixtures, and expected results in the [Phase 0 contract](../wirebox/phase-0-contract.md). Use four milestones: G3 PoC, G4 MVP, G7 Linux/MikroTik core pilot, and G8 ZTE–Huawei/VSOL ISP pilot. Unsupported features return an explicit status and do not count as connectivity passes.

G0 can pass for the **local development scope** once fixtures, reference machine, known/unknown inventory, and test matrix exist. Complete Linux/RouterOS access remains a G6 dependency; ISP-pair details remain a G8 dependency. This freezes scope before acceptance runs; it does not verify hardware.

Device information from chat is stored as a sanitised user-reported fixture. A prompt with an SAAS-SIM suffix does not prove physical hardware or simulator fidelity. Device-internal timestamps are not used as collection time.

## Alternatives and consequences

Requiring every real device before the build would delay independent kernel proof. Implementing every vendor feature at once would expand assumptions without evidence. Staged proof is chosen, with gate reports stating scope, unknowns, and limits.

The Phase 1 implementation gate proves only the stack and data contract. It does not pass simulation, real ping, or device mutation. Pilot numeric targets are frozen as the acceptance baseline; later revisions must be recorded before a rerun.
