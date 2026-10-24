# ADR-0011: Rename product identity to Wyrlex

- Status: Accepted for the active product surface, 24 October 2026
- Scope: product name, Rust package names, build environment names, UI labels, and current documentation

## Decision

The product is named **Wyrlex**. Active Cargo packages are `wyrlex-kernel`, `wyrlex-headless`, and `wyrlex-studio`; the Zig build variable is `WYRLEX_ZIG`; acceptance and UI logging variables use the `WYRLEX_*` prefix; and the static library is linked as `wyrlex_kernel`.

The checkout directory remains `C:\Users\ricky\src\wirebox` during this migration because the workspace is open there and historical evidence links use that path. The source directory names under `crates/wirebox-*` and the C header filenames remain stable for the same reason. They are path compatibility details, not the product name. A later repository migration may rename those paths in a separate, reviewable change.

The C ABI prefix `wb_*`, types such as `wb_handle`, and `WB_*` constants remain unchanged. They are the versioned ABI surface used by Zig, Rust, C layout tests, and evidence artifacts. Renaming them would be an ABI v2 change and is outside this identity migration.

Historical evidence under `docs/evidence/` and archived Packet Tracer sources are not rewritten. Their old branding records the target and source state at the time of each run. The active documentation points to Wyrlex and this ADR explains the compatibility boundary.

## Consequences

- New commands use `cargo run -p wyrlex-headless` and `cargo run -p wyrlex-studio`.
- Existing local C callers can continue using the `wb_*` ABI while the product identity changes.
- GitHub repository `refracts-systems/wyrlex` is now the canonical remote; the local checkout directory remains `wirebox` for continuity.
- Cognitive Coverage output remains ignored and local-only. Any generated guide that predates this migration is stale until its source hash is refreshed.

## Verification

The rename is verified by Cargo metadata, the core build/test command, an active-source search, and documentation link validation. Evidence hashes from prior phases are preserved rather than regenerated.
