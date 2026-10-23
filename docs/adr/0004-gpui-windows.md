# ADR-0004 — GPUI langsung pada Windows x64

- Tanggal: 20 Oktober 2026
- Status: Accepted untuk spike phase 1. Hasil build dan interaksi dicatat terpisah di laporan gate.

## Konteks

Pengguna membandingkan GPUI dengan GPUix/Solid. Arah kernel Zig dan aplikasi Rust membutuhkan topology/byte inspector khusus.

## Keputusan

Gunakan published crate **gpui = 0.2.2**, `default-features = false`, dengan `windows-manifest`, dan simpan Cargo.lock. Baseline platform adalah `x86_64-pc-windows-msvc`, debug build. Crate paket menyebut upstream commit `69e2130295c2649963eb639fc70b4f2ee8ea1624` dengan `dirty: true`; karena itu identitas build memakai **versi + checksum paket di Cargo.lock**, bukan menganggap source identik persis dengan commit tersebut.

Spike menampilkan empat node/link statis yang dapat dipilih, byte inspector, command membaca batch Zig, keyboard shortcut, dan shutdown. Semua byte yang ditampilkan berasal dari transport probe aktual. Tampilan menyatakan bahwa simulasi paket belum tersedia.

Gunakan API pada source paket yang dipin. Dokumentasi branch main dapat mempunyai pemisahan crate/API berbeda. Phase 1 tidak menjamin release packaging, platform lain, skala besar, atau seluruh kebutuhan MVP.

## Alternatif dan konsekuensi

GPUix/Solid tetap alternatif jika pengalaman TSX diperlukan atau GPUI langsung gagal memenuhi uji. Ia menambah batas runtime pada stack saat ini. GPUI langsung dipilih sebagai percobaan terukur, bukan klaim benchmark superior.

Build UI membutuhkan dependensi native lebih banyak daripada kernel; workspace default hanya membangun runner/kernel. `--workspace` atau `-p wirebox-studio` memasukkan UI. Build concurrency dibatasi empat pada mesin referensi 16 GB.

Sumber: [GPUI](https://gpui.rs/), [paket 0.2.2](https://crates.io/crates/gpui/0.2.2), [teknologi proyek](../wirebox/technology.md).
