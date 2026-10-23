# Hasil gate G0 dan G1 — 20 Oktober 2026

## Keputusan gate

| Gate | Status | Scope yang terbukti |
|---|---|---|
| G0 | Lulus untuk development lokal, baseline v1 | Scope/expected result dibekukan, fixture konsisten, inventaris user-reported beserta unknown tersedia, mesin dan protokol pembuktian dicatat |
| G1 | Lulus lokal pada Windows x64 debug | Build Zig/Rust/GPUI, copy batch melalui ABI, ownership/error/lifecycle, pemeriksaan fixture, interaksi desktop, dan source export baru |
| G2–G9 | Belum diuji | Scheduler, protokol, MVP, replay, adapter, intent executor, dan pilot perangkat nyata belum tersedia |

G0 tidak menyatakan akses ke VM/server atau hardware telah diuji. Dependency model/firmware/access yang tersisa tetap tercatat di [inventaris](../../../fixtures/inventory/pilot-v1.json), untuk G6/G8. Dasar scope ini ditetapkan pada [ADR-0005](../../adr/0005-pilot-scope.md) dan [kontrak v1](../../wirebox/phase-0-contract.md).

## Lingkungan dan identitas

[environment.json](environment.json) merekam OS, CPU/RAM/GPU, compiler, dan target aktual. Source belum memiliki commit; [source-manifest.json](source-manifest.json) menyimpan SHA-256 berkas build/kernel/host/fixture/ADR terkait. Cargo.lock menyimpan versi dan checksum dependency. Binary tidak dimasukkan ke repository.

## Verifikasi otomatis

Command aktual: `./scripts/verify.ps1`. Hasil disimpan pada [verification.log](verification.log).

| Pemeriksaan | Hasil |
|---|---|
| Rustfmt workspace dan Zig fmt | Lulus |
| Unit test Zig | 3 lulus: layout/copy, stale handle, kapasitas session dan reuse |
| Compile C11 layout header | Lulus: sizeof, alignof, offset, dan handle width |
| Unit test Rust wrapper | 5 lulus: copy/lifetime, batch/order/capacity, error, ABI/null/stale handle, layout |
| Unit test fixture | 2 lulus: fixture consistency dan deteksi checksum rusak |
| Compile-fail doctest | 1 lulus: Session tidak dapat dipindahkan melalui thread::spawn |
| Clippy workspace/all-targets dengan `-D warnings` | Lulus untuk source workspace |
| Runner `--probe` | 30 byte yang dikirim Rust disalin Zig dan dibaca kembali; session ditutup; result pass |
| Runner `--check-fixtures` | 4 node/5 skenario tervalidasi sebagai fixture; checksum expected header valid; tidak menjalankan simulasi |
| Build desktop debug | Lulus dengan GPUI 0.2.2 pada Windows MSVC |

Total test: 3 Zig + 7 Rust unit + 1 compile-fail doctest. Angka ini bukan jumlah skenario jaringan yang sudah dijalankan.

Cargo melaporkan future-incompatibility pada dependency transitif `proc-macro-error2 2.0.1` (private re-export proc_macro, E0365). Build pada Rust 1.99.1 tetap berhasil. Ini dicatat sebagai dependency follow-up saat upgrade toolchain; tidak disembunyikan dengan perubahan lint source atau patch third-party yang belum diuji.

## Source export baru

Command: `./scripts/verify-clean-export.ps1`. [clean-export.log](clean-export.log) merekam build workspace, ABI probe, serta fixture validation dari direktori source baru di TEMP. Tidak ada .git, target, Zig static library, atau artifact source-side yang disalin.

Dependency serta compiler cache lokal digunakan kembali melalui CARGO_TARGET_DIR. Ini pembuktian fresh-source build **dengan cache**, bukan cold-cache install atau remote CI run. Workflow Windows sudah dibuat pada [.github/workflows/verify.yml](../../../.github/workflows/verify.yml), tetapi belum dijalankan di GitHub.

## Verifikasi runtime GPUI

Window target dipilih melalui Computer Use. Hasil yang diamati:

- Node R1 terpilih dan inspector menunjukkan alamat fixture.
- Link SW1–R1 terpilih dan inspector menjelaskan link fixture.
- Byte offset 0 terpilih dan menunjukkan 0x57 / decimal 87 dari buffer Zig.
- Ctrl+R menaikkan event ID dari 4 ke 5 dan membaca salinan batch baru.
- Escape mengembalikan selection ke PC1.
- Ctrl+Q menutup window; log mencatat `window_closed` dan `session_destroyed`.

[ui-interaction.log](ui-interaction.log) juga mencatat interaksi sebelum langkah automation yang aktornya tidak ditentukan; bukti automation spesifik adalah urutan di atas. Setelah perbaikan tinggi minimum window agar footer terlihat, build akhir diinspeksi ulang dan tombol close native diuji. [ui-final.log](ui-final.log) mencatat sesi akhir tersebut dan penghancuran session. Kedua stderr run kosong; tidak ditemukan crash pada run tersebut.

![Preview native Wirebox phase 1](ui-preview.jpg)

Tampilan adalah topology fixture dan transport probe, bukan animasi simulasi jaringan. Screenshot hanya memuat window Wirebox. Build GUI awal memerlukan pemakaian API judul window yang sesuai paket GPUI yang dipin; perbaikan telah masuk source dan build akhir lulus.

## Batas pembuktian

Belum ada packet scheduler, codec/field mapping protokol, ping simulator, snapshot/replay, login perangkat, mutation jaringan, benchmark p95, sesi 30 menit, release packaging, atau pengujian lintas platform. G1 membuktikan batas stack/data dan input dasar. Berikutnya adalah P2-01 dan implementasi semantics waktu pada [ADR-0003](../../adr/0003-virtual-time-and-trace.md).
