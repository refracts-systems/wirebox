# Hardening Phase 0–3 and Cognitive Coverage

**Local pass for logged testing, October 23, 2026.** This is one pass of priority hardening on configuration, ABI registry, scheduler and codecs; not a complete security audit or passing a new phase. Fixed stack Zig 0.17.0, Rust 1.99.1, Windows x64 MSVC and GPUI 0.2.2.

## Findings and improvements

| Area / phase | Actual results | Evidence |
|---|---|---|
| Contracts / 0 | The next work paragraph in todos which still designates Phase 3 is corrected to P4-01 | [todos](../../wirebox/todos.md); Old G0–G3 not rewritten |
| ABI / 1 | The runtime/network registry generation limit is derived from two namespace bits; max−1/max test and foreign discriminator added | [sim_abi.zig](../../../kernel/src/sim_abi.zig), [net_abi.zig](../../../kernel/src/net_abi.zig) |
| Scheduler / 2 | 5 seeds × 4,000 schedule/cancel/step operations compared with reference linear model, including timer count and counter | [runtime_tests.zig](../../../kernel/src/runtime_tests.zig), [scheduler.log](scheduler.log) |
| Config / 3 | **Confirmed bug:** setPort after addRoute can create an off-link gateway. Route revalidation is added before workload starts | [network.zig](../../../kernel/src/network.zig), [red test](route-before.log) and [targeted after fix](targeted.log) |
| Codecs / 3 | 4,000 valid/mutation/truncation/random inputs on two layers; all successful field views must be in the buffer | [network_tests.zig](../../../kernel/src/network_tests.zig) |
| Verifier / cross phase | Default Zig tests to Debug + ReleaseSafe; ReleaseFast is available as an option. Workflow name clarified Phase 0–3 | [verify.ps1](../../../scripts/verify.ps1), [workflow](../../../.github/workflows/verify.yml) |

The test route doesn't just test the return code: after reject, started/failed remains false, packet/queue/trace remains empty, and the caller can fix the port and then initiate a ping. Final validation is selected so that the configuration remains subject to incremental editing. Dispatch lines that have committed events still have their own failed-state contracts.

Five new tests: codec corpus, staged route, interleaved scheduler, runtime registry boundary, network registry boundary. The network registry test clears the test-owned allocation as well when the assertion fails. There are no additional ABI test hooks or runtime dependencies.

## Command and results

```powershell
./scripts/verify.ps1 -ZigModes Debug,ReleaseSafe,ReleaseFast
./scripts/verify-clean-export.ps1
node scripts/check-cognitive-coverage.mjs
```

- [workspace-final.log](workspace-final.log): **37 Zig test results in each of three modes**, including import aggregator; **16 Rust unit tests, 3 compile-fail doctests**, C layout, format, Clippy all targets, GPUI build and acceptance probe/fixture/runtime/network passed.
- Fixed acceptance network **36 scenario runs / 37 negative inputs** from existing Rust oracle; does not count as the same 37 Zig tests. There is no additional protocol scope.
- [clean-export.log](clean-export.log): build workspace from new source copy and CLI acceptance pass, with compiler/dependency cache reused. This is not a cold-cache build or a remote CI job.
- [targeted.log](targeted.log) logs four initial hardening tests + import aggregator; The scheduler was added later and is included in [scheduler.log](scheduler.log) as well as the final suite.
- `proc-macro-error2 2.0.1` still issues a transitive future incompatibility warning. The warning was not fixed by upgrading the dependency on this task.

The GPUI verification scope is build/lint/test. Native application window interactions are not repeated; The QA browser below tests learning HTML artifacts, not UI Studio.

## Cognitive Coverage Integration

Upstream skill [ryannadel/cognitive-coverage](https://github.com/ryannadel/cognitive-coverage) installed in scope repo, commit `2c85334c389cb2b527dff2cf0a0c9c626290c891`. [Provenance](../../../.agents/skills/cognitive-coverage/UPSTREAM.md), MIT LICENSE and schema included. No MCP configuration or global installation is changed.

The `cognitive-coverage-open.html` launcher, `learning-guide.html` guide, `cognitive-coverage.json` manifest, and `cognitive-coverage.html` dashboard were generated for the backend engineer audience. The four files are now stored only in the local `cognitive-coverage/` folder that Git ignores; not available via GitHub or new clone. The QA results below record historical runs when the artifact was produced, not a guarantee that local material always matches the latest source. The inventory includes first-party sources and canonical docs; build artifacts, historical evidence and RE archives are deliberately excluded from the learning inventory. Not all sources have been audited/taught in depth.

The material contains 9 concepts and 3 paths, actual source snippets, 18 questions in nine combinations of difficulty/depth, manual status, quiz transfer and export/import. Initial understanding status **uncovered**: generated content and test engine passing is not the same as humans understanding the system. GPUI modules, oracle CLI details, and RE history remain deep gaps.

- [browser-qa.json](browser-qa.json): Edge headless, 9 filter combinations with 2 questions each, score, transfer fragment/cleanup, focus recalculation down, manual status, export, input JSON/fragment invalid, storage rejected and mobile overflow tested; zero page errors. Synthetic answers only reside in an isolated browser test context.
- [learning-desktop.png](learning-desktop.png), [learning-mobile.png](learning-mobile.png), [dashboard-desktop.png](dashboard-desktop.png): local rendering results are checked visually.
- [artifact-integrity.log](artifact-integrity.log): upstream source hashes/anchor/quiz mapping and schema checker executed. Schema validator JSONSchema 4.26.0 scratch only `artifacts/`; not added to runtime or package manifests.

Dashboard does not write repos automatically. Export saves the local manifest; sourceHash needs to be checked when source changes. Mapping-based quizzes are indicators of limited learning and can be overridden by self-assessment, not certification of mastery of the entire file.

## Advanced study materials and limits

[Learning notes](../../learning/hardening_phase_0_3.md) explains decisions, backend analogues, why/invariant comment rules, `SAFETY` FFI examples, and teach-back questions. AGENTS.md added a change communication guide so that artifacts and their reasons are continuously explained.

Not yet done: long-running fuzzing/sanitizers, fault injection allocator, benchmark release, audit of all unsafe/dependencies, all GPUI interactions, checkpoint/replay, equivalent PT differential corpus, Linux/MikroTik access or change of vendor. Sample mutation and reference models do not prove all RFCs or all possible schedules. Phase 4 remains the next job.

The source and artifact run are bound by [source-manifest.json](source-manifest.json). The repo doesn't have a HEAD yet; File hash is used as provenance. The old gate manifest remains the historical baseline.
