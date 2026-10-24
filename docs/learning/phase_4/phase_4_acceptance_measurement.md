# Phase 4 — Membuktikan UI dan mengukur batasnya

24 Oktober 2026. Implementasi ada pada `crates/wirebox-studio/src/main.rs` dan `model.rs`; ringkasan metrik dihitung oleh `scripts/summarize-studio-acceptance.mjs`. [Evidence](../../evidence/phase-4-acceptance/README.md) dan [ADR-0010](../../adr/0010-agent-acceptance-and-studio-measurement.md) membatasi kesimpulan.

## Dari demo ke acceptance

Test controller memastikan route benar, branch asal tidak berubah dan persistence memverifikasi hasil. Klik native memastikan jalur UI memanggil operasi itu dengan benar. Script soak memastikan operasi berulang tetap berjalan selama durasi tertentu. Ketiganya menilai hal berbeda; lulus unit test tidak membuktikan UI responsif, dan screenshot ping sukses tidak membuktikan replay penuh.

Pengguna meminta agent melakukan seluruh acceptance teknis. Itu memungkinkan agent mengisi hasil U1–U5 melalui UI, tetapi tidak mengubah pemahaman manusia menjadi verified. Studi kegunaan manusia tetap memiliki pertanyaan lain: apakah orang baru menemukan informasi tanpa bantuan, dan apakah pekerjaan lebih cepat dibanding workflow manual?

## Histori dengan referensi yang benar

Satu ping engine menghasilkan 63 row. Untuk menguji proyeksi UI 10.000 row tanpa mengarang paket, harness menjalankan kernel 160 kali. Setiap run diberi rentang ID tersendiri untuk event, versi paket dan transmisi. Nol tetap berarti tidak ada referensi. Entity topologi memakai ID fixture yang identik; waktu diberi offset per run. Byte/field disalin tanpa perubahan.

Dari 10.080 row, 80 tertua dibuang dengan penghitung eksplisit. Paket/transmisi yang dirujuk tetap disimpan. Test memeriksa seluruh row→version→packet dan row→transmission; ini mirip foreign key pada read model backend. Mengulang row dengan ID yang sama dapat menghasilkan alias dan bukan pengujian histori yang benar.

Ini proyeksi beberapa run. Kapasitas 128 versi paket/256 transmisi engine per run tetap berlaku; harness tidak membuktikan workload satu engine tanpa batas.

## Ukur lalu perbaiki

Run awal dengan 200 row/page melewati target 100 ms. Renderer kemudian dibatasi menjadi 50 row/page sambil mempertahankan 10.000 row tersimpan. Perubahan ini membatasi pekerjaan render, bukan menghapus histori atau memotong queue scheduler. Semua raw sample, termasuk outlier, tetap dilaporkan.

Timestamp mulai di handler atau dispatch scripted. Script memproses engine/FFI/snapshot sebelum publish ke UI. Callback `Window::on_next_frame` GPUI 0.2.2, menurut source lokal dependensi, dijalankan setelah current frame dirender. Selisihnya adalah handler-to-render; antrean input OS serta presentasi compositor tidak termasuk. Nama metric harus mempertahankan batas ini.

Scripted pulse memilih row lintas halaman, memanggil pause, melangkah engine dan menguji save/load berulang. Sampel klik/Space native dicatat terpisah, sehingga jumlah kecilnya tidak disamarkan sebagai ribuan input fisik. P95 memakai nearest rank, bukan rata-rata, dan setiap jenis harus memiliki setidaknya 100 sampel.

Monitor memori merekam working set, private bytes dan Responding tiap detik. Peak proses dicatat terpisah bila tersedia karena sampling dapat melewatkan alokasi sangat singkat. Bukti stabilitas memerlukan durasi aktual minimal 30 menit serta progress/render terakhir, bukan hanya proses masih ada.

## Cara mengulang

1. Bangun Studio dengan toolchain yang dipin dan tutup instance lama jika Windows mengunci executable.
2. Tentukan path `WYRLEX_UI_LOG`, aktifkan `WYRLEX_ACCEPTANCE=1`, lalu jalankan Studio dalam window terlihat. Judul acceptance membedakannya dari sesi biasa.
3. Catat PID, hash binary/source, lingkungan dan waktu mulai. Monitor dengan `measure-studio-memory.ps1` selama 1.830 detik.
4. Biarkan 1.800 pulse selesai. Reset/preset dapat keluar dari proyeksi acceptance; jangan menggunakannya sebagai interaksi selama benchmark. Klik row/Space boleh dipakai untuk sampel native.
5. Simpan log sebagai `final-ui.log` dan memori sebagai `final-memory.json` dalam folder run tersendiri, lalu jalankan `node scripts/summarize-studio-acceptance.mjs <folder> --require-pass`.

Komentar kode menjelaskan mengapa histori read-only, mengapa ID direbase dan mengapa Network tetap pada thread pemilik. Dokumentasi panjang berada di sini. Materi cognitive coverage pribadi dengan hash source lama tetap stale sampai di-refresh; tidak ada pemahaman manusia yang otomatis dinyatakan verified.

Teach-back: mengapa p95 handler-to-render dari 1.800 pulse scripted tidak dapat langsung diklaim sebagai p95 input-to-photon dari 1.800 klik fisik?
