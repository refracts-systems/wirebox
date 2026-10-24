# What is Wyrlex?

Wyrlex is an **infrastructure studio and runtime built around a shared semantic model**. Engineers can inspect infrastructure objects, examine state and packets, try changes in simulation, and plan operations on real devices through adapters.

*Semantic* means that the model expresses meaning: interfaces, routes, VLANs, services, dependencies, and device capabilities. A vendor CLI or management protocol is one way to read or change those objects.

Updated 24 October 2026. The build foundation, Zig/Rust/GPUI boundary, graph and virtual-time runtime, and [headless packet simulation](packets.md) are available. Ethernet/ARP/IPv4/ICMP, static routing, and byte versions were tested at [G3](../evidence/phase-3/README.md). The initial [Packet Studio](studio.md) integration connects trace/byte data to UI controls; [Phase 4 technical agent acceptance](../evidence/phase-4-acceptance/README.md) passed for the declared fixture and handler-to-render scope, while human usability remains unknown. [Phase 5 local checkpoint/replay](../evidence/phase-5/README.md) now covers active ARP state, fork/restore, and seeded impairment; Linux and real-device adapters remain on the roadmap. Earlier evidence is in [Phase 0/1](../evidence/phase-0-1/README.md) and [Phase 2](../evidence/phase-2/README.md).

Wyrlex does not recreate an operating system. *Kernel* means the simulation engine core; Linux, RouterOS, and device firmware remain their own systems in a real lab.

## Problem

A network appears in many forms: diagrams, vendor CLI output, monitoring, configuration, packet captures, and incident notes. The relationship between a configuration change, network behaviour, and transmitted bytes often has to be reconstructed manually.

Wyrlex keeps that relationship in one model:

```text
Infrastruktur = graph + state + behavior + constraints + time
```

For example, when a route changes, a user can inspect the changed object, the next-hop decision, affected packets, and simulation result before preparing the real change.

## Intended product experience

| Mode | User question | Intended result |
|---|---|---|
| Inspect | What exists, what is its condition, and where did the information come from? | Topology, state, observations, dependencies, and provenance |
| Simulate | What happens if a route, link, or configuration changes? | A simulation branch, decision trace, packet bytes, and a model-scoped result |
| Execute | What operation is needed and did it produce the expected result? | A plan, adapter operations, execution records, and re-verification |

The intended flow is **record → replay → modify → simulate → apply → verify**. A replay can restore only what was recorded; further simulation uses model assumptions that must remain visible.

The discussed inspirations are device interaction like Winbox, topology and simulation like Packet Tracer, protocol inspection like Wireshark, declarative intent like Terraform, and debugger-style execution/state inspection. These are experience analogies, not a list of implemented features.

## One foundation, multiple domains

Routers, switches, firewalls, Linux hosts, servers, OLTs, and ONUs are candidate early or later domains. PLCs, UPS systems, IoT gateways, radios, and storage are extensions in the initial vision, each with its own behaviour model.

An OLT is a rich domain for testing the model: PON, ONU, VLAN, GEM, TCONT, alarms, and optical conditions. Supporting optical fields does not by itself provide optical-physics simulation.

The planned UI includes a topology canvas, object inspector, byte/packet inspector, event timeline, state diff, and operation planner. UI layout stores display positions only; operational connections and dependencies belong to the domain model.

## Why the idea is plausible

The parts are buildable: an object model, event scheduler, packet codecs, protocol state machines, device adapters, and inspection UI. The [external mini-simulator](../packet-tracer/mini-simulator.md) demonstrates a limited network scenario without hardware. [Comparison projects](positioning.md) show that network models, intent, and validation are established approaches.

The main challenge is preserving consistent meaning between the model, simulator, and vendors. Equal names for operations do not create equivalence. Each capability needs a contract and a declared validation scope.

The differentiator to test is a traceable relationship from **intent → state → protocol decision → byte → result** in one workspace. That value remains a product hypothesis until usage demonstrates it.

## Initial scope

The PC1–SW1–R1–PC2 simulation runs headless and is connected to Studio: Ethernet, ARP, IPv4, ICMP, virtual time, and byte-change inspection. The next work is MVP acceptance and measurement, followed by checkpoint/replay, lab validation, and adapters in stages.

Emulation of every chipset, equivalence with every firmware version, support for every vendor, firmware-upgrade simulation, and universal rollback are not product promises. Successful `.pkt` import is not a prerequisite for the standalone engine.

## Reading map

- [Architecture](architecture.md): the shared model and its relationship to the real world.
- [Simulation](simulation.md) and the [byte tracker](byte-tracker.md): how behaviour is built and explained.
- [Technology](technology.md): Zig/Rust and GPUI/GPUix considerations.
- [Roadmap](roadmap.md) and [decisions](decisions.md): proposed work and choices still open.
- [Phase todos](todos.md) and the [pilot plan](pilot.md): checklists, scenarios, and success evidence for the PoC, MVP, Linux/MikroTik core pilot, and ZTE with Huawei/VSOL ONU pilot.

The full vision source is the [initial Wyrlex document](../packet-tracer/sources/01-wirebox-vision.txt). Design additions in thematic documents are marked as proposals so they are not mistaken for user requirements or approved features.
