# Phase 4 — Menghubungkan controller, snapshot dan inspector

23 Oktober 2026. Baca [ADR-0008](../../adr/0008-studio-network-controller.md), [panduan Studio](../../wirebox/studio.md) dan [evidence](../../evidence/phase-4/README.md).

## Jalur kode dan alasan keputusan

[model.rs](../../../crates/wirebox-studio/src/model.rs) memisahkan controller dari window GPUI. `Config` adalah konfigurasi efektif fixture; `Model` memiliki `Network`; `Frame` hanya memuat salinan Rust. [main.rs](../../../crates/wirebox-studio/src/main.rs) mengirim command dan membaca Frame untuk render.

Bagi backend engineer, Frame mirip read model: dibuat setelah command selesai lalu dipakai berulang untuk query tampilan. Karena Network `!Send + !Sync` dan dimutasi pada satu thread, refresh summary→packet/field→transmission→trace tidak berlomba dengan command lain. Snapshot ini tidak boleh dianggap atomik jika kelak engine dimutasi worker thread; arsitektur itu membutuhkan kontrak baru.

`refresh` menyiapkan Frame berikutnya terlebih dahulu. Paket/transmisi baru disalin berdasarkan ID monotonik, trace dibaca per batch setelah ordinal terakhir, lalu Frame dipublikasikan. Render tidak memanggil FFI per byte. Jika refresh gagal, controller melaporkan error dan menghentikan Run; snapshot lama bukan diklaim hasil mutasi terbaru.

Run memakai kelanjutan async yang hanya menunggu timer lalu kembali memanggil controller pada thread pemilik. Fast memakai budget empat tiap 40 ms; Slow satu tiap detik untuk inspeksi, tanpa mengubah clock virtual. Epoch berubah saat pause/reset/branch atau pergantian pace. Kelanjutan dengan epoch lama berhenti sebelum menyentuh owner baru. Ini serupa generation token untuk membatalkan pekerjaan UI yang hasilnya sudah tidak relevan. Ia tidak membatalkan event engine yang sudah dieksekusi.

## Byte/bit tetap berasal dari engine

UI menggunakan `Packet.bytes` dan `Field.byte_offset/bit_offset/bit_width`; tidak mendekode struct native sebagai Ethernet atau IPv4. `covers` menghitung perpotongan rentang bit field dengan satu byte. Dua field empat bit bisa sama-sama disorot ketika satu byte dipilih. Nilai lebih dari 64 bit ditampilkan melalui raw bytes, bukan angka yang dipotong.

Trace dengan `version=0` tidak meminjam paket dari row sebelumnya. Before mengacu ke `prior`, After ke `version`. Diff offset hanya dipakai jika kedua versi berada pada layer yang sama: perbandingan Ethernet dengan IPv4 mentah memiliki posisi header berbeda.

## Branch dan persistence

`branch` membangun Network baru sebelum UI mengganti owner. Gagal membangun konfigurasi tidak merusak sesi sumber. Ini terpisah dari kegagalan dispatch engine setelah event dikonsumsi; jangan menganggap seluruh runtime transactional.

`save` menulis schema/model/fixture, Config, command log dan report salinan, tanpa handle FFI. `load` membuat engine baru dan mereproduksi report sebelum menerima file. Mengubah outcome menjadi sukses dalam JSON tidak cukup: perbandingan seluruh report gagal. Report baseline juga diverifikasi. Semantik ini rerun, bukan checkpoint cache/timer. Skema dan batas file sengaja ditulis eksplisit agar data yang tidak didukung ditolak.

Test model mencakup S01–S05 batch/step, snapshot owned, branch perbaikan route yang tidak mengubah sumber, save/load parsial dan selesai, schema/hasil/budget/batas invalid, serta hubungan TTL/checksum dengan raw bytes. Registry hanya mempunyai empat slot: test yang memiliki beberapa Network diserialkan, bukan menaikkan kapasitas produksi demi test.

Komentar kode menjelaskan ownership, titik publikasi Frame, pembatalan epoch dan alasan load memeriksa hasil. Tutorial ini menyimpan penjelasan panjang. Materi Cognitive Coverage pribadi yang dibuat sebelum Phase 4 **stale** untuk UI/controller baru dan perlu diperbarui secara lokal; artifact itu tidak ikut Git.

Pertanyaan teach-back: mengapa membatalkan task Run dengan epoch tidak sama dengan membatalkan event scheduler, dan mengapa file hasil yang bisa dibuka kembali belum membuktikan restore checkpoint?
