#G2 — Basic graph, state, scheduler, and trace

Date: **October 20, 2026**. Result: **passed local Windows x64 debug** for tasks P2-01 through P2-06. Model `counter-actions-v1`, runtime ABI 1, SplitMix64-v1, Zig 0.17.0 and Rust 1.99.1. The repo doesn't have a HEAD yet; [manifest](source-manifest.json) identifies the verified source and artifact. Actual environment: [environment.json](environment.json).

This gate proves the foundation of the runtime: invariant graph, deterministic scheduler, lifecycle/cancellation, bounded execution, explicit PRNG, and journal retention independence. Action is only a synthetic counter. These results do not prove Ethernet/ARP/IP/ICMP, checkpoint/restore, device fidelity, UI MVP, or real device pilot.

## Repeatable results

| Examination | Actual results |
|---|---|
| `scripts/verify.ps1` | Passed formatter, Zig test, C11 layout, Cargo test, Clippy `-D warnings`, CLI, build GPUI |
| Zig | 16 test results passed: 15 behavior/layout tests and 1 block import test |
| Rust units | 13 passes: 10 wrapper/probe/runtime and 3 fixture/acceptance |
| Rust compile-fail doctest | 2 pass: Session/World cannot be sent to another thread |
| Fixture graph | 4 devices, 6 ports, 3 links; reference validation, kind, endpoint exclusivity |
| Heap vs oracle | 200 schedules at 30 timestamps; cancel every index multiple of 3; 133 executions matched with independent linear minimum selection |
| Time/budget | Same-time tie-break, zero budget, empty step/horizon, past time, future event, periodic timer |
| Lifecycle | Device/port/link cascade deletes the associated timer entity; The old ID does not access the new entity; capacity reclaimed after cancel |
| Overflow | Time/ID/counter/execution overflow; failed firing does not consume RNG or events; progress run retain prefix success |
| Reproducibility | 5 seeds × 3 reruns matched to 5 baselines; state, trace, clock, timer, RNG, ID compared |
| Retention | 5 seeds × capacity 0/1/3/256 = 20 runs; state is identical, trace suffix and dropped count match |
| Wrap/read retention | 300 executions with retention 0/1/7/256; read per step matches batch without read per step |
| Ownership | The graph/trace copy remains valid after mutation and close; runtime/probe handles are not mutually accepted |

Complete output: [verification.log](verification.log). Demo: [scheduler-demo.json](scheduler-demo.json). Acceptance matrix: [acceptance.json](acceptance.json). Matrix testing runs 40 new Worlds: 5 baselines, 15 reruns, 20 retention variations. This is a repetition of the initial state, not a restore checkpoint.

```powershell
./scripts/verify.ps1
cargo run --locked -p wirebox-headless -- --scheduler-demo
cargo run --locked -p wirebox-headless -- --check-runtime
```

## Expected and actual demo

Expected is declared in [runner acceptance](../../../crates/wirebox-headless/src/runtime_demo.rs) before run: event ID `2,1,3,4,7`, timestamp `5,10,10,10,15`, total 5 firings, PC1=1, R1=12, PC2=0, event 7 cause=4. The topology comes from fixture phase 0; The runtime only takes identity and relations.

Actual match. For seed 20260117, SW1's random samples are 33 and 71 so the final counter is 104. The budget stops the run at 15 ns; after canceling the periodic timer, the empty horizon reaches 100 ns. Canceled timers and deleted temporary owners are never executed. Pending/end timer zero.

Oracle heap uses linear minimum selection on the same schedule input, without using the heap implementation to determine expected. Oracle arithmetic/header phase 0 remains separate; G2 does not treat it as a processed packet.

## Artifacts and boundaries

- [Runtime contracts](../../wirebox/runtime.md), [ADR-0006](../../adr/0006-phase-2-runtime.md), [Zig tests](../../../kernel/src/runtime_tests.zig).
- Registry bounded: 8 World, 128 entities, 256 pending events/timers, maximum retention 256. This is an explicit limit for phase 2, not yet the MVP size is 10,000 rows.
- Journal only successful firings; command host/cancel/remove has not been audited as an event. The visible queue/graph/seed metadata is not a complete snapshot to restore.
- New export source verified separately by `scripts/verify-clean-export.ps1`; [clean-export.log](clean-export.log) logs results and path. Dependency/compiler cache is intentionally reused; not cold-cache or remote CI.
- Desktop build/test/re-lint. Native GUI interactions are not repeated because the UI remains spike probe phase 1; evidence of the previous interaction is in [G0/G1](../phase-0-1/README.md).
- Transitive dependency warning `proc-macro-error2 2.0.1` regarding future Rust compatibility still appears. Build on rust 1.99.1 pinned passed; does not claim that later versions of the compiler are compatible.
- Evidence G0/G1 and reverse engineering archives are maintained as old baseline records. The old source hash does not have to be the same as the phase 2 source.
- No device is contacted or changed; remote CI, release packaging, and benchmarks have not been run.
