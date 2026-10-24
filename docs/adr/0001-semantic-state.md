# ADR-0001 — Shared semantic model and state provenance

- Date: 20 October 2026
- Status: Accepted; model direction chosen, domain implementation follows in Phases 2/6/7.

## Context

The product vision connects inspection, simulation, diagnosis, and execution. Treating all of them as one “current state” could turn a prediction into an apparently observed fact.

## Decision

Use entity, relation, capability, property, resource, constraint, intent, operation, event, and observation as shared vocabulary. Store raw observations with source/time; separate observed, desired, and simulated state. Derived state references evidence and assumptions. Unknown, stale, and conflict values must not be silently filled from simulation.

The kernel owns mutable simulated state. Host/UI sends commands and receives copied snapshots/traces; display coordinates are not operational connections. Operation IR is shared by simulator and adapters through meaning and pre/postconditions, not identical CLI strings.

## Alternatives and consequences

UI that mutates internal fields directly is simpler for a demo but makes headless testing and replay harder. Completely separate vendor models simplify the first integration but duplicate intent logic. The shared model uses capability extensions, so vendors do not need identical features.

An operation is a logical unit and does not guarantee an atomic cross-device transaction. The detailed schema is not yet a public API; types and relations are added with fixtures and tests.

References: [architecture](../wirebox/architecture.md), [initial vision](../packet-tracer/sources/01-wirebox-vision.txt).
