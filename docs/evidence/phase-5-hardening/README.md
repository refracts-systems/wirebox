# Phase 5 hardening — local checkpoint and impairment

Date: 24 October 2026. Target: Windows x64 MSVC; Zig 0.17.0 and Rust 1.99.1. Starting committed baseline: `d363c2d`; this run hardens the existing, uncommitted Phase 5 work. Source and fixture hashes are recorded in [source-manifest.json](source-manifest.json). Earlier [Phase 5 evidence](../phase-5/README.md) remains the initial baseline.

## Proven regressions and fixes

The two regression tests in [regressions-before.log](regressions-before.log) failed before the fixes:

| Failure | Reproduction | Before | Corrected behavior |
|---|---|---|---|
| Lost frames bypass egress occupancy | Two lost 60-byte frames at 100 Mbps | `next_free = 0` | `next_free = 9600 ns`; a subsequent surviving frame starts at 9600 ns |
| Failed transmit consumes random state | Seed 42, jitter enabled, clock near `u64::MAX` | PRNG became `11400714819323198527` despite overflow | PRNG remains 42, noise counters and transmission queue remain unchanged |

The latter test also fills the scheduler and checks that rejected reservation leaves PRNG, counters, IDs, and egress state unchanged. This guarantee is scoped to transmit reservation; a domain dispatch may have committed earlier work before failing. The ABI preserves its failed-network behavior.

`wb_net_checkpoint_info` now zeroes its output on invalid handles instead of returning a misleading default model version of 1. The original stale-checkpoint test previously tested handle 0 after destroy; it now retains the actual stale value, reuses the slot, and proves the old handle is rejected.

## Additional coverage

- Recursive compile-time ownership guard: nested pointer/slice fields cannot silently enter the memcpy checkpoint representation.
- Checkpoint namespace `01`, invalid/foreign handles, null outputs, four-checkpoint capacity, four-network fork capacity, generation retirement, and 128 stale lifetimes.
- Checkpoints outlive their source; forked networks outlive the checkpoint. Configured branches diverge independently, and restore before input makes the restored configuration editable again.
- Failed networks remain failed when checkpointed/forked; restore from a healthy checkpoint recovers an otherwise failed target.
- Loss probability endpoints, invalid probability/jitter, disabled impairment consuming no PRNG draws, and egress serialization after loss.
- A compile-fail doctest checks that Rust `Checkpoint` cannot be sent to another thread.
- Comparator mutation controls individually alter summary, packet bytes, field maps, transmission arrival, noise counters, PRNG state, and trace cause. Each mutation is detected.

## Replay matrix

The headless comparator uses an uninterrupted baseline with no checkpoint, then tests home continuation, fork, and restore at each cut. It also verifies that executing a branch does not mutate the home state or checkpoint metadata. It compares summary, packet bytes/ancestry, field maps, transmissions, noise, PRNG state, and retained trace records. A separate Rust calculation checks link serialization, propagation, and jitter bounds.

| Dimension | Values |
|---|---|
| Scenarios | S01–S05 and the static-route alias fixture |
| Seeds | 0, 1, 42, `u64::MAX` |
| Trace retention | 0, 1, 16, 1024 |
| Impairment `(loss ppm, max jitter ns)` | `(0,0)`, `(0,10000)`, `(250000,10000)`, `(1000000,0)` |
| Scheduler event budgets before checkpoint | 0, 1, 2, 6, 12 |
| Horizon | 5,000,000,000 ns |

Result: **1,920 checkpoint cases and 5,760 continuations matched exactly**. Clean/jitter-only outcomes also matched the fixture outcomes. Different jitter seeds 0 and 1 produced different transmission timing. The collector checked contiguous retained ordinals and retained counts. Of 384 baseline runs, 252 had intentionally incomplete trace histories due to retention; this was never labeled complete history.

Of those baseline runs, 52 retained outcome 0 under loss. The current model lacks an ICMP echo timeout after successful ARP resolution, so a lost IP request/reply can finish with no queued work and an unresolved outcome. Deterministic replay of that state is verified; reachability or failure conformance is not inferred from it.

## Commands and results

`./scripts/verify.ps1` passed. [Full log](verification.log): 46 Zig tests in each of Debug and ReleaseSafe, 16 Rust kernel tests, 6 headless tests, 9 Studio controller tests, 4 compile-fail doctests, C ABI layout compilation, format checks, workspace Clippy, all headless acceptance commands, and Studio build. The existing `proc-macro-error2 2.0.1` future-incompatibility warning remains. Native UI interaction and remote CI were not rerun.

`cargo run --locked -p wyrlex-headless -- --check-phase5` reproduces the matrix. Captured [JSON result](check-phase5.json) explicitly reports G5 incomplete and Linux comparison not run. All comparisons in this report concern the simulator's own modeled behavior.

## Remaining work and limits

P5-05, the lab-comparison portion of P5-06, and the final G5 report in P5-07 remain open. Checkpoints are bounded in-memory process-local copies, not durable files or cross-version replay. Checkpoint controls are not integrated into Studio, and running topology edits remain unsupported. Out-of-memory injection and physical memory corruption recovery were not tested.

The optional `node scripts/check-cognitive-coverage.mjs` still reports stale personal learning artifacts, first at `.github/workflows/verify.yml`; this is separate from the passing engine/workspace checks. No human understanding status was promoted. New [learning notes](../../learning/phase_5/phase_5_hardening.md) explain the fixes in Indonesian.
