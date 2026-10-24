# C ABI v1 — transport probe phase 1

Normative contract: [wirebox.h](../../kernel/include/wirebox.h). Implementation: [Zig](../../kernel/src/abi.zig) and [Rust wrapper](../../crates/wirebox-kernel/src/lib.rs). Design rationale: [ADR-0002](../adr/0002-zig-rust-abi.md).

This document is probe specific. The graph/scheduler API is available separately as [runtime ABI v1](runtime.md), protocol via [network ABI v1](packets.md). The phase 3 revision of the ADR-0007 uses two bits above the handle: probe 00, counter 10, network 11, with generation 30 bits. This replaces the one bit/31 bit phase 2 split; handles are not persisted or transferred between processes.

## API and errors

| Function | Contract |
|---|---|
| `wb_abi_version` | Returns 1; query without mutation |
| `wb_create(version, out_handle)` | Create an empty session; out_handle is required to be valid and is set to zero on version/capacity | failure
| `wb_destroy(handle)` | Close session, delete queue, invalidate generation; the second destroy results in invalid_handle |
| `wb_probe_submit(handle, data, length)` | Synchronous copy 1–64 bytes; enqueue one record kind=1; input is not stored as a pointer |
| `wb_probe_read(handle, out, capacity, out_count)` | Copy and consume until min(queue length, capacity), capacity 0–16; zero capacity does not drain the queue |

Status `u32`: 0 ok, 1 invalid_argument, 2 abi_mismatch, 3 invalid_handle, 4 capacity. The count output is set to zero if the count pointer is valid before handle/buffer validation. Submit failure does not consume event ID. A full queue does not overwrite data. Event IDs start at 1 per session, without virtual time.

`wb_probe_event`: id u64 offset 0, kind u32 offset 8, length u32 offset 12, bytes[64] offset 16. Total 80 bytes, alignment 8 on Windows x64 target. The bytes after length are initialized to zero. This is an in-process ABI format, **not** a network wire format or persistence file.

## Ownership, pointers, and threads

The Registry has 16 slots; The queue for each session has 16 records. Handle contains generation and index; slots with generation exhausted are retired. The registry uses bounded static storage without heap allocation or cross-language deallocators at this stage.

The caller must guarantee that the non-null pointer is valid, aligned, long enough, and does not overlap with other output. Null data is invalid; null out is only allowed if capacity is zero. The API does not promise recovery from arbitrary pointer or memory corruption. The header and wrapper must change together if the layout changes.

The C caller is required to serialize all registry calls process-wide and not perform reentrancy. The rust wrapper enforces serialization with mutexes and limits the Session to the thread that created it; other sessions still pass through the same mutex. The kernel doesn't call the host, doesn't unwind to Rust, and doesn't save the caller pointer.

`Session::read_probe` returns Rust's `Vec<ProbeEvent>`; results remain valid after mutation, next read, or close. `Session::close` returns destroy; Drop becomes a fallback without panic. The queue/handle limit is a probe limit, not the design capacity of the phase 2/3 simulator.

## Verification required by phase 1

Layout C/Zig/Rust; payload remains correct after caller changes input; snapshot persists after owner is closed; partial batch preserves order; zero capacity does not drain; invalid input does not change the queue; incorrect version and null output rejected; the old handle does not access the new slot. Compile-fail doctest proves the wrapper cannot be moved via `thread::spawn`.
