# Architecture Decision Records

ADR mencatat konteks, keputusan, alternatif, konsekuensi, dan status implementasi. `Accepted` berarti keputusan kerja yang diambil dalam tugas yang diotorisasi pengguna; bukan klaim bahwa semua fitur sudah selesai atau setiap rincian dipilih langsung oleh pengguna. Perubahan material dibuat sebagai ADR pengganti dengan referensi ke keputusan sebelumnya.

| ADR | Keputusan | Status |
|---|---|---|
| [0001](0001-semantic-state.md) | Model bersama dan pemisahan sumber state | Accepted, graph simulated state tersedia; observed/desired masih rencana |
| [0002](0002-zig-rust-abi.md) | Kernel Zig, host Rust, C ABI kecil dengan data salinan | Accepted, probe ABI v1 diimplementasikan |
| [0003](0003-virtual-time-and-trace.md) | Waktu virtual deterministik dan jurnal terpisah | Accepted, diimplementasikan phase 2 |
| [0004](0004-gpui-windows.md) | GPUI langsung, baseline Windows x64 | Accepted untuk spike phase 1; bukti runtime pada laporan gate |
| [0005](0005-pilot-scope.md) | Pembuktian bertahap dan scope v1 | Accepted untuk baseline pilot; jaringan nyata belum diuji |
| [0006](0006-phase-2-runtime.md) | Lifecycle graph/timer, budget, PRNG, jurnal, ABI runtime | Accepted, diimplementasikan; [bukti G2](../evidence/phase-2/README.md) |
| [0007](0007-packet-model.md) | Subset protokol, versi byte/transmisi, link/ARP policy, dan ABI network | Accepted, diimplementasikan; [bukti G3](../evidence/phase-3/README.md) |

Tanggal keputusan: 20 Oktober 2026. Hasil aktual ada di [evidence phase 0–1](../evidence/phase-0-1/README.md), scope di [kontrak phase 0](../wirebox/phase-0-contract.md), dan checklist di [todos](../wirebox/todos.md).
