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
