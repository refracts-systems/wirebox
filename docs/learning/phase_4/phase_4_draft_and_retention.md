# Phase 4 — Draft, format sesi dan ring trace

24 Oktober 2026. File utama: `crates/wirebox-studio/src/model.rs`, `crates/wirebox-studio/src/main.rs`, `kernel/src/network.zig`.

Draft seperti request perubahan aggregate: pengguna boleh memasukkan beberapa nilai yang belum konsisten secara terpisah. Validasi final serta konstruksi Network baru dilakukan saat Apply. Owner lama diganti setelah sukses. Ini menjamin kegagalan konfigurasi tidak menghilangkan hasil aktif; tidak berarti semua eksekusi runtime memiliki rollback.

Sesi v2 membawa route, daftar port down dan timing per link. Pembaca v1 mempertahankan default lama dan membandingkan report hasil rerun. Memigrasi bentuk Config berbeda dari memulihkan cache ARP, timer dan queue checkpoint.

Ring trace memakai ordinal monoton. Setelah 10.257 row dengan kapasitas 10.000, row pertama yang tersedia adalah 258. Pembaca memulai pada maksimum antara ordinal tertua dan cursor+1. Pengecekan cursor lebih besar/sama dengan total dilakukan sebelum penambahan untuk menghindari overflow u64. Test juga memastikan versi paket tidak tertukar dan scheduler tidak ikut dipangkas.

Pagination membatasi elemen render menjadi 200 per halaman. Ini keputusan awal untuk membatasi pekerjaan render; manfaat latency harus diukur. Menambah kapasitas ring dan lulus test storage belum membuktikan acceptance UI 10.000 row atau sesi 30 menit.

Komentar kode menjelaskan titik publish owner, arti None pada route, serta alasan cursor ring. Test mempertahankan oracle perilaku: echo reply, longest-prefix match, timeout ARP, hasil sumber tetap sama dan file v1 aktual masih dapat dimuat.

Command verifikasi: `./scripts/verify.ps1`; lihat [evidence baru](../../evidence/phase-4-hardening/README.md) untuk hasil aktual. Materi cognitive coverage lokal yang menyimpan hash source lama harus dianggap stale sampai di-refresh; kelulusan test tidak memverifikasi pemahaman manusia.

Teach-back: mengapa mengganti konfigurasi dengan Network baru setelah validasi berbeda dari rollback event yang sudah memutasi state protokol?
