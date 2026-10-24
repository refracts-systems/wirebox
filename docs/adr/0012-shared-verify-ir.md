# ADR-0012: Shared declarative IR for Verify mode

- Status: Proposed direction for Phase 5–7, 24 October 2026
- Scope: stateless IPv4 reachability, route/filter/VLAN transformations, simulator/verifier agreement

## Context

Wyrlex already runs a concrete packet through the simulator and records the decision and byte changes. The proposed Verify mode must answer whether a property holds for a set of packets, then explain a counterexample through the same concrete trace. Maintaining a separate rule implementation for simulation and verification would allow their semantics to drift.

## Proposal

Represent device behavior as a declarative intermediate representation (IR) shared by both executors. An IR rule has:

- `match`: predicates over header fields such as IPv4 prefix, protocol, port range, VLAN ID, and connection metadata supported by the model;
- `action`: forward, drop, rewrite a field, or push/pop a VLAN tag;
- ordered rules per device and chain, with the evaluation order made explicit.

The concrete simulator interprets the IR for one packet. The verifier interprets the same IR over a packet set using atomic predicates or a BDD-backed representation. The first queries are reachability, isolation, loop/blackhole detection, and shadowed rules. Stateful connection tracking and NAT are unsupported until their state model has a separate contract.

Each verifier counterexample must be serializable as a concrete packet input. Running that input through the simulator must produce a byte-level trace whose rule decisions can be linked back to the failing predicate. This is a semantic link, not a claim that the model is equivalent to every vendor firmware.

## Required gates

1. Define the IR schema, rule ordering, supported field set, and unsupported outcomes.
2. Lower the existing static-route simulator to the IR without changing the Phase 3 oracle results.
3. Implement a small verifier for the same stateless subset.
4. Run differential tests over fixed and generated packets; simulator and verifier must agree on outcome and selected action.
5. Emit a counterexample fixture and replay it through the byte tracker.

The equality gate compares semantic outcome/action and the declared state transitions. It does not compare internal data structures or require a full symbolic proof for unsupported features.

## Consequences

This makes Phase 7 a verifiable extension of the existing model rather than a second simulator. It also moves route/filter/VLAN behavior toward data-driven rules. Planner and vendor adapter lowering still need separate contracts, and a verifier pass does not prove device firmware behavior until a device comparison fixture exists.
