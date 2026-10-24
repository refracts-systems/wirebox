# Implications for Wyrlex

This section contains design suggestions from discussions, not findings about Packet Tracer implementation.

A more complete product discussion and results are available at [Wyrlex documentation](../wirebox/README.md). This note focuses on the relationship of RE results to design proposals.

## Adaptable foundation

- World model: devices, ports, links, configurations, and protocol states.
- Virtual time scheduler with stable event sequence and timer identity.
- Composable protocol behavior components.
- Trace decisions with packet ID, event ID, and traceable causes.
- Separation of execution queue, journal and Event List display.

There is no need to follow memory layout, multiple inheritance, class names, or PT function addresses to implement the pattern.

## Standalone simulation

Neither the physical device nor the vendor's OS image is required to implement the behavior defined by the model itself. Virtual devices still require state and behavioral rules. Testing of specifications, vendor OS, or hardware is required according to the level of equality you wish to claim.

## Byte tracker

Wyrlex's need to display each byte goes further than the prototype which primarily stores semantic fields. Proposed representation:

1. Packet buffer as byte source.
2. View the protocol that maps fields to byte/bit ranges.
3. Package versions or snapshots and change notes: before, after, event, process, reason.

The native struct layout is not automatically the same as the wire format. Endianness, alignment, variable-length headers, and input validation must be handled explicitly.

## Language direction of the discussion

Temporary recommendations are Zig for kernel/codec and Rust for application runtime and GPUI UI. This has not been implemented by this documentation work.

Zig can export a small C ABI; Rust wraps it up with clear ownership, buffer length, error, and thread contracts. Memory safety in the Zig implementation is not automatically proven by Rust's borrow checker.

## Replay and connection with the real world

Deterministic replay requires device/protocol state, pending events/timers, buffers, PRNG state, and external input. GUI logs and seeds alone do not guarantee complete checkpoint and restore.

Observed state, desired state, and simulated state must be distinguished. Simulation results do not automatically become facts about production. The device adapter performs the operation and reads back the results.

## Recommended first stage

Port the PC1–SW1–R1–PC2 scenario to the new kernel. Starting from Ethernet, ARP, IPv4, and ICMP with byte change traces. Add lost route, TTL timeout, and timer cancellation scenarios. Use this example to test the model and UI before expanding the protocol or vendor scope.
