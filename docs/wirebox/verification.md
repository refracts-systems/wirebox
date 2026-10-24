# Wyrlex Verify Plan

This document notes Phase 5–7 directions for verifying network properties with the same model used by the simulator. The status is **proposed**; no symbolic engine or vendor adapter has been implemented yet.

## Channel

```text
Intent + observed/simulated state
                 |
                 v
       Declarative device IR
          |              |
          v              v
   Concrete simulator   Set verifier
          |              |
          +-------> differential gate
                         |
                         v
                counterexample packet
                         |
                         v
                 byte-level trace
```

Initial IR models stateless rules for IPv4, routes, filters, and VLANs. `match` selects supported fields/prefixes/ranges; `action` forwards, drops, changes fields, or pushes/pops tags; the sequence of rules is stored as data. Simulator and verifier implementations should not have separate copies of rules.

The first query is reachability, customer isolation, loops/blackholes, and rules that are covered by previous rules. Packet-set representations can start with atomic predicates; BDD becomes an option once state size and performance requirements are measured. Connection tracking and stateful NAT return `unsupported` until the contract is proven.

## Minimum proof

1. Lowering static route Phase 3 to IR does not change oracle bytes/trace.
2. Verifier returns the same outcome/action as the simulator on the fixture and the generated package.
3. Each violation produces a concrete packet that can be replayed and linked to trace rows, fields, and bytes.
4. Plan changes are compared as a set of reachability before/after; The mismatch stops at the CI gate.

This proves the equality of the two interpreters of the Wyrlex model on the stated subset. This does not prove the behavior of the firmware, ASIC, or all vendor versions.

## Proposed temporal extension

The user-provided [temporal verification thesis](temporal-verification.md), recorded 24 October 2026, adds properties over histories and plan transitions: when reachability was lost, whether supported intermediate states preserve a service, and which services would be affected by an assumed failure. These capabilities are proposed extensions, not results of the current replay tests or an implemented symbolic engine.

Transition verification needs explicit operation effects, ordering, asynchronous completion and failure semantics, packet-in-flight assumptions, and observation uncertainty. Checking the initial and final configurations, or one selected sequence of snapshots, does not prove safety across all relevant interleavings. The shared IR remains a foundation; the transition model needs its own contract and lab validation. The [behavior corpus proposal](behavior-corpus.md) would link those semantics to reproducible evidence per model and firmware.
