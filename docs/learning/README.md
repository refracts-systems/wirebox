# Belajar membangun Wyrlex, phase 1–4

Dokumen ini membaca **kode yang sekarang ada**, tahap demi tahap. Setiap catatan menjawab: bagian mana yang bekerja, keputusan apa yang diambil, mengapa dipilih, dan apa akibatnya saat kode dikembangkan. [ADR](../adr/README.md) adalah keputusan arsitektur; [evidence](../evidence/phase-3/README.md) merekam hasil uji. Catatan belajar ini menghubungkan keduanya dengan fungsi dan data konkret.

Tambahan 23 Oktober 2026: [hardening Phase 0–3 untuk backend engineer](hardening_phase_0_3.md) dan [laporan pengujian](../evidence/hardening-phase-0-3/README.md). Materi interaktif Cognitive Coverage bersifat pribadi dan disimpan dalam folder lokal `cognitive-coverage/` yang diabaikan Git; folder itu tidak ikut clone. Jika tersedia lokal, buka `cognitive-coverage/cognitive-coverage-open.html` dari root project melalui browser. Guide berlevel memprioritaskan FFI, lifecycle handle, scheduler, jurnal, codec, dan jalur ping; progress manusia dimulai dari uncovered.

Urutan yang disarankan:

| Phase | Catatan | Pertanyaan utama |
|---|---|---|
| 1 | [Build Zig dari Cargo](phase_1/phase_1_build_zig_rust.md) | Bagaimana satu build memuat kode Zig dan Rust? |
| 1 | [C ABI dan ownership](phase_1/phase_1_ffi_ownership.md) | Bagaimana byte menyeberangi bahasa tanpa pointer menggantung? |
| 1 | [GPUI dan probe](phase_1/phase_1_gpui_probe.md) | Apa yang sebenarnya ditampilkan UI pertama? |
| 2 | [Graph dan lifecycle](phase_2/phase_2_graph_lifecycle.md) | Bagaimana device, port, link, dan ID dikelola? |
| 2 | [Scheduler dan waktu virtual](phase_2/phase_2_scheduler_timer.md) | Mengapa event selalu dieksekusi dalam urutan yang sama? |
| 2 | [State, jurnal, dan verifikasi](phase_2/phase_2_state_trace.md) | Apa beda queue, state, trace, dan replay? |
| 3 | [Codec dan field byte](phase_3/phase_3_codec_byte.md) | Bagaimana header diubah dan diinspeksi byte demi byte? |
| 3 | [Alur paket jaringan](phase_3/phase_3_alur_paket.md) | Mengapa ping melewati ARP, switch, router, lalu kembali? |
| 3 | [Versi, ABI, dan pembuktian](phase_3/phase_3_tracker_pembuktian.md) | Bagaimana hasil simulasi ditelusuri dan diuji? |
| 4 | [Controller, snapshot dan inspector Studio](phase_4/phase_4_controller_snapshot.md) | Bagaimana UI membaca hasil konsisten, membatalkan Run, serta memverifikasi sesi yang dibuka? |
| 5 | [Checkpoint, restore, fork dan replay](phase_5/phase_5_checkpoint_replay.md) | Mengapa state scheduler dan PRNG harus ikut disalin untuk replay? |

Contoh phase 3 memakai fixture sintetis `PC1 — SW1 — R1 — PC2`. Nama kelas dan alamat Packet Tracer dari laporan reverse engineering memberi petunjuk desain, bukan source engine Wyrlex atau bukti kesamaan perilaku vendor. Baca [batas model paket](../wirebox/packets.md) sebelum memakai hasil simulasi untuk keputusan operasional.

Lanjutan: [draft, sesi v2 dan retensi trace](phase_4/phase_4_draft_and_retention.md).

[Acceptance agent, histori 10.000 row dan pengukuran](phase_4/phase_4_acceptance_measurement.md) menjelaskan metode dan batas benchmark Phase 4.

Catatan Phase 5 menjelaskan implementasi checkpoint dan controlled impairment yang sekarang sudah diuji melalui runner headless.

[Hardening Phase 5](phase_5/phase_5_hardening.md) membahas dua regression test yang gagal sebelum perbaikan, titik commit PRNG, lifecycle checkpoint, dan matriks replay.

[Corpus dan pembanding Linux di WSL](phase_5/phase_5_corpus_linux.md) menjelaskan manifest append-only, identitas versi, capture kernel Linux, revisi classifier, dan test yang menolak false success.

[Penutupan G5 dengan pembanding Linux](phase_5/phase_5_g5_linux_validation.md) membahas paket checksum salah, positive control, urutan pemulihan route, perbedaan ARP, dan lima seed/tiga pengulangan replay.
