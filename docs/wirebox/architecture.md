# Semantic and runtime models

Status: domain model follows [ADR](../adr/README.md). [Runtime phase 2](runtime.md) provides graph device/port/link, counter, scheduler, timer, and journal. [Phase 3](packets.md) adds simulated ARP/MAC/route state, Ethernet/IPv4/ICMP behavior, as well as byte versioning. The device's observed/desired state, planner, and operation executor still follow the roadmap. [ABI transport probe](ffi.md) phase 1 remains separate.

## Infrastructure as a queryable object

The *Infrastructure DOM* analogy means that objects and their relationships can be explored in a structured manner. Conceptual example: find all ONUs on a PON whose signal is low and whose internet service is not active. This does not require the use of browser DOM or JavaScript.

Ten primitives of early vision:

| Primitives | Meaning | Example |
|---|---|---|
| Entities | Something with a stable identity | Router, interface, ONU, process |
| Capabilities | Capabilities with known scope | IPv4 routing, switching, optical observation |
| Properties | Value on an object | Address, admin state, rx power |
| Relations | Relationships of type | `attached_to`, `member_of`, `depends_on` |
| Resources | Something limited or allocated | IP, VLAN ID, bandwidth, GEM port |
| Constraints | Validity rules in domains | The interface reference must be valid |
| Intent | Conditions to be achieved | Subnet A can access service B |
| Operations | Step changes with contracts | AddRoute, CreateVLAN, EnableInterface |
| Events | Events at a time | LinkDown, timer expired, frame received |
| Observations | Evidence read from a source | CLI, telemetry, or SNMP results |

Properties need clear types and units. Physical relationships, L2/L3 connections, and service dependencies should not be merged into one meaningless line. Capabilities should state support parameters and limits, not just the boolean `supported`.

## State and provenance

| Layers | Meaning | Example |
|---|---|---|
| Observations | Original proof with source and time | CLI reads the interface is down at time T |
| Observed state | Interpretation of available evidence | The interface is rated down based on certain observations |
| Desired state | Desired conditions | Administratively enabled interface |
| Simulated state | Model results on a specific branch and virtual time | Frames are forwarded after simulation route changes |

Conflicting observations must remain checkable. Unread or expired data needs to be marked unknown/stale; do not automatically fill in from the simulation results.

Production snapshots can be input to simulation branches. The branch needs to designate the origin snapshot, model version, configuration, and assumptions for the unobserved state. Encapsulating a device's configuration does not guarantee that all of its buffers and internal state are captured.

## Intent, IR, and two executors

```text
Intent + snapshot + capabilities
                |
                v
      Planner / constraint checks
                |
                v
    Operation plan / Infrastructure IR
          |                         |
          v                         v
    Sim executor              Real executor
          |                         |
    Virtual world             Device adapters
          |                         |
    Trace + result            Observe + verify
```

IR is an intermediate representation that preserves the meaning of operations before being translated to the backend. The division of modules and serialization formats is still open.

The `EnsureReachability(A, B)` example needs to be narrowed down: traffic direction, address/prefix, protocol/port, VRF, link state, and assumed return path. Planner generates operations that the backend can understand, for example adding a route. The simulator executor changes the virtual state; The real adapter translates it according to the device's capability.

Operation is the logical unit of planning. Its execution on a device may require multiple commands and may fail partially; it is not automatically an atomic transaction across devices.

The proposed operating contract includes targets, arguments, preconditions, expected effects, postconditions, timeouts, idempotency properties, and possible compensation. Rerunning an operation should have a defined outcome, especially after a partial failure.

## Verify model sharing with simulator

The proposed Verify mode uses the same declarative IR as the simulator. Rule contains `match`, `action`, and evaluation order per device/chain. The simulator runs the rule against a single packet; The verifier calculates the set of packets and issues reachability, isolation, loop/blackhole, and shadowed rule queries in the supported stateless subset.

The counterexample verifier must be able to be taken as concrete input, rerun through the simulator, and then linked to decisions and byte changes. The differential test on the package remains a gate so that the two executors do not deviate. Stateful connection tracking and NAT remain unsupported until the state model has its own contract. Details of the proposal and gate are in [ADR-0012](../adr/0012-shared-verify-ir.md) and [Verify plan](verification.md).

## Adapter and transport

Conceptual driver contracts of vision: `discover`, `observe`, `capabilities`, `plan`, `execute`, and `validate`. The division of responsibilities for general planners and driver-specific planning still needs to be designed.

SSH/CLI, SNMP, NETCONF, RESTCONF, gNMI, HTTP, Modbus, and serial are candidate transports. Output parsing and firmware behavior resides in the respective adapter/domain. Transport support does not mean all vendor capabilities are available.

User-reported inventory is available in [pilot-v1.json](../../fixtures/inventory/pilot-v1.json): Linux VM/server, RouterOS major 7, C320 with per board version, as well as GPON ONU state examples. ONU model/firmware, Linux distro, RouterOS minor version, access, and unknown capabilities remain unknown. Drivers not yet available; the testing stage is in [pilot plan](pilot.md).

## Real change, reconciliation, and AI

The intended flow is plan → validation → simulation according to the model → applicable policy/authorization → execute → observe → verify. The plan needs to state dependencies, impacts, and what happens if some steps fail.

Rollback is a per operation/backend capability. Restoring an old configuration does not necessarily restore active sessions, protocol state, or physical effects. Branch/merge/bisect state infrastructure also requires domain rules for conflicts; the similarity to Git is the direction of the experience.

Reconciliation compares the desired and observed states, then produces a repair plan. Automatic execution and its policies have not yet been decided. AI can help with querying, diagnosis, and planning over the same objects and operations; Model access does not automatically grant rights to execute changes.

See [roadmap](roadmap.md) for proof order and [decisions](decisions.md) for unselected boundaries.
