# Hardening Phase 0–3 dan Cognitive Coverage

**Lulus lokal untuk pengujian yang dicatat, 23 Oktober 2026.** Ini satu pass hardening prioritas pada konfigurasi, ABI registry, scheduler dan codec; bukan audit keamanan lengkap atau kelulusan phase baru. Stack tetap Zig 0.17.0, Rust 1.99.1, Windows x64 MSVC dan GPUI 0.2.2.

## Temuan dan perbaikan

| Area / phase | Hasil aktual | Bukti |
|---|---|---|
| Kontrak / 0 | Paragraph pekerjaan berikutnya di todos yang masih menunjuk Phase 3 dikoreksi menjadi P4-01 | [todos](../../wirebox/todos.md); G0–G3 lama tidak ditulis ulang |
| ABI / 1 | Batas generation registry runtime/network diturunkan dari dua namespace bits; uji max−1/max serta foreign discriminator ditambahkan | [sim_abi.zig](../../../kernel/src/sim_abi.zig), [net_abi.zig](../../../kernel/src/net_abi.zig) |
| Scheduler / 2 | 5 seed × 4.000 operasi schedule/cancel/step dibandingkan dengan reference model linear, termasuk timer count dan counter | [runtime_tests.zig](../../../kernel/src/runtime_tests.zig), [scheduler.log](scheduler.log) |
| Config / 3 | **Bug terkonfirmasi:** setPort sesudah addRoute dapat membuat gateway off-link. Revalidasi route ditambahkan sebelum workload dimulai | [network.zig](../../../kernel/src/network.zig), [red test](route-before.log) dan [targeted setelah perbaikan](targeted.log) |
| Codec / 3 | 4.000 input valid/mutasi/truncation/random pada dua layer; semua successful field view harus berada di dalam buffer | [network_tests.zig](../../../kernel/src/network_tests.zig) |
| Verifier / lintas phase | Default Zig tests menjadi Debug + ReleaseSafe; ReleaseFast tersedia sebagai pilihan. Nama workflow diperjelas Phase 0–3 | [verify.ps1](../../../scripts/verify.ps1), [workflow](../../../.github/workflows/verify.yml) |

Test route tidak hanya menguji return code: setelah reject, started/failed tetap false, packet/queue/trace tetap kosong, dan caller dapat memperbaiki port lalu memulai ping. Validasi akhir dipilih agar konfigurasi tetap dapat diedit bertahap. Jalur dispatch yang sudah commit event tetap mempunyai failed-state contract sendiri.

Lima test baru: codec corpus, staged route, interleaved scheduler, runtime registry boundary, network registry boundary. Test registry network membersihkan test-owned allocation juga ketika assertion gagal. Tidak ada tambahan ABI test hook atau dependency runtime.

## Command dan hasil

```powershell
./scripts/verify.ps1 -ZigModes Debug,ReleaseSafe,ReleaseFast
./scripts/verify-clean-export.ps1
node scripts/check-cognitive-coverage.mjs
```

- [workspace-final.log](workspace-final.log): **37 hasil test Zig pada masing-masing tiga mode**, termasuk import aggregator; **16 unit test Rust, 3 compile-fail doctest**, layout C, format, Clippy seluruh target, build GPUI dan acceptance probe/fixture/runtime/network lulus.
- Acceptance network tetap **36 scenario run / 37 input negatif** dari oracle Rust yang sudah ada; tidak dihitung sebagai 37 test Zig yang sama. Tidak ada penambahan protocol scope.
- [clean-export.log](clean-export.log): build workspace dari salinan source baru dan acceptance CLI lulus, dengan cache compiler/dependency dipakai ulang. Ini bukan cold-cache build atau job CI remote.
- [targeted.log](targeted.log) mencatat empat test hardening awal + import aggregator; scheduler ditambahkan sesudahnya dan tercakup pada [scheduler.log](scheduler.log) serta final suite.
- `proc-macro-error2 2.0.1` tetap mengeluarkan future incompatibility warning transitif. Warning itu tidak diperbaiki dengan upgrade dependency pada tugas ini.

Scope verifikasi GPUI adalah build/lint/test. Interaksi window aplikasi native tidak diulang; browser QA di bawah menguji artifact HTML pembelajaran, bukan UI Studio.

## Integrasi Cognitive Coverage

Skill upstream [ryannadel/cognitive-coverage](https://github.com/ryannadel/cognitive-coverage) dipasang pada scope repo, commit `2c85334c389cb2b527dff2cf0a0c9c626290c891`. [Provenance](../../../.agents/skills/cognitive-coverage/UPSTREAM.md), LICENSE MIT dan schema disertakan. Tidak ada konfigurasi MCP atau instalasi global yang diubah.

[Launcher](../../../cognitive-coverage/cognitive-coverage-open.html), [guide](../../../cognitive-coverage/learning-guide.html), [manifest](../../../cognitive-coverage/cognitive-coverage.json), dan [dashboard](../../../cognitive-coverage/cognitive-coverage.html) dihasilkan untuk audience backend engineer. Inventaris mencakup first-party source dan canonical docs; build artifacts, evidence historis dan arsip RE sengaja dikecualikan dari inventory pembelajaran. Tidak semua source telah diaudit/diajarkan mendalam.

Materi berisi 9 konsep dan 3 alur, source snippets aktual, 18 soal dalam sembilan kombinasi difficulty/depth, status manual, transfer kuis dan export/import. Status pemahaman awal **uncovered**: generated content dan test engine lulus tidak sama dengan manusia memahami sistem. Module GPUI, detail CLI oracle, dan sejarah RE tetap gap pendalaman.

- [browser-qa.json](browser-qa.json): Edge headless, 9 kombinasi filter dengan 2 soal masing-masing, skor, transfer fragment/cleanup, focus recalculation ke bawah, manual status, export, input JSON/fragment invalid, storage ditolak dan mobile overflow diuji; nol page error. Jawaban sintetis hanya berada dalam browser context test terisolasi.
- [learning-desktop.png](learning-desktop.png), [learning-mobile.png](learning-mobile.png), [dashboard-desktop.png](dashboard-desktop.png): hasil render lokal diperiksa visual.
- [artifact-integrity.log](artifact-integrity.log): checker source hashes/anchor/quiz mapping dan schema upstream dijalankan. Schema validator JSONSchema 4.26.0 hanya di scratch `artifacts/`; tidak ditambahkan ke runtime atau package manifests.

Dashboard tidak menulis repo secara otomatis. Export menyimpan manifest lokal; sourceHash perlu diperiksa saat source berubah. Kuis berbasis mapping adalah indikator belajar terbatas dan dapat ditimpa self-assessment, bukan sertifikasi penguasaan seluruh file.

## Bahan belajar dan batas lanjutan

[Learning notes](../../learning/hardening_phase_0_3.md) menjelaskan keputusan, analogue backend, aturan komentar why/invariant, contoh `SAFETY` FFI, dan pertanyaan teach-back. AGENTS.md menambahkan panduan komunikasi perubahan agar artifact dan alasannya terus dijelaskan.

Belum dilakukan: long-running fuzzing/sanitizers, fault injection allocator, benchmark release, audit seluruh unsafe/dependency, seluruh interaksi GPUI, checkpoint/replay, PT differential corpus setara, akses Linux/MikroTik atau perubahan vendor. Sample mutation dan reference model tidak membuktikan seluruh RFC atau semua jadwal mungkin. Phase 4 tetap pekerjaan berikutnya.

Source dan artifact run ini diikat oleh [source-manifest.json](source-manifest.json). Repo belum mempunyai HEAD; hash file dipakai sebagai provenance. Manifest gate lama tetap baseline historis.
