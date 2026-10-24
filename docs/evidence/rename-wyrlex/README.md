# Evidence renamed Wyrlex

Date: October 24, 2026. The active repository uses the product identity **Wyrlex**.

## Results

- `cargo metadata --format-version 1 --no-deps` reports packages `wyrlex-kernel`, `wyrlex-headless`, and `wyrlex-studio`.
- `cargo check --locked -p wyrlex-headless` passes with Zig 0.17.0 and Rust 1.99.1.
- `cargo test --locked -p wyrlex-headless` passed 4 tests.
- `cargo test --locked -p wyrlex-kernel` passed 12 tests and 3 doctests.
- `cargo test --locked -p wyrlex-studio --bin wyrlex-studio` passed 9 tests.
- Environment build and log UI using `WYRLEX_*`; static library uses `wyrlex_kernel`.
- Active Markdown link checking found no missing targets. The two false positives in the source archive come from simple parser-scanned `aes,` text, not file links.

## Migration limits

ABI C prefix `wb_*`, header file `kernel/include/wirebox*.h`, checkout path, old crate folder name, old RE script audit, and historical evidence are maintained. This limit is described in [ADR-0011](../../adr/0011-wyrlex-product-rename.md) so that product renames do not secretly become ABI v2 or change the hash of old proofs.

The GitHub repository has been changed to [`refracts-systems/wyrlex`](https://github.com/refracts-systems/wyrlex), and the local `origin` has been redirected to that URL. Fixed checkout folder name `wirebox` for workspace continuity.
