# Phase 5: hardening checkpoint dan impairment

Tanggal: 24 Oktober 2026. Baca [catatan checkpoint](phase_5_checkpoint_replay.md) lebih dahulu. Bukti dan batas run ada di [laporan hardening](../../evidence/phase-5-hardening/README.md).

## Dua bug yang dibuktikan melalui test gagal

`kernel/src/network.zig` sebelumnya mengembalikan hasil loss sebelum memperbarui `next_free`. Dua frame 60 byte pada 100 Mbps seharusnya memakai waktu 9.600 ns. Test menghasilkan 0 ns. Perbaikannya menetapkan loss sebagai kejadian setelah serialisasi: frame hilang tetap memakai bandwidth, tetapi tidak membuat arrival event. Ini menjaga urutan antrean egress, termasuk ketika frame berikutnya berhasil dikirim.

Bug kedua terjadi saat jitter diambil sebelum pengecekan overflow waktu atau reservasi scheduler. Seed 42 berubah menjadi 11400714819323198527 walau transmit gagal. Sekarang PRNG dan noise counters dihitung pada salinan lokal, lalu dipublikasikan setelah reservasi arrival berhasil. Untuk loss, draw dan reservasi egress dipublikasikan bersama. Dalam istilah backend, ini seperti menunda publikasi state sampai resource yang diperlukan tersedia. Namun seluruh dispatch jaringan tetap dapat gagal setelah sebagian perubahan; jangan menganggap semua error melakukan rollback otomatis.

## Ownership yang dijaga compiler

`kernel/src/net_abi.zig` menambah fungsi comptime `valueOwned`. Fungsi ini memeriksa field struct dan array secara rekursif. Pointer atau slice baru menyebabkan build gagal sebelum copy checkpoint bisa membuat alias memori tanpa sengaja. Saat engine nanti memakai alokasi dinamis, salinannya harus diganti dengan deep copy yang jelas ownership-nya.

Checkpoint memakai namespace handle `01`, network memakai `11`. Keduanya memiliki generation 30 bit; slot retired saat generation habis. Test menyimpan 128 handle lama, memakai ulang slot, lalu melompat ke dua generation terakhir untuk memastikan handle stale tetap ditolak. Test kapasitas membuktikan empat checkpoint dan empat network memiliki lifecycle independen. Checkpoint tetap bisa di-fork setelah network asal ditutup.

## Mengapa comparator diperluas

Summary yang sama bisa menyembunyikan transmission time atau PRNG state yang salah. `crates/wirebox-headless/src/phase5.rs` sekarang membandingkan summary, byte dan ancestry paket, field map, transmission, noise counters, PRNG, serta trace yang masih disimpan. Test sengaja mengubah satu kategori pada satu waktu agar terbukti comparator mendeteksinya. Timing link juga diperiksa dengan rumus Rust yang terpisah dari implementasi Zig.

Matriks memakai enam skenario dari fixture Phase 3, seed `0`, `1`, `42`, dan `u64::MAX`, retensi `0/1/16/1024`, empat konfigurasi loss/jitter, dan lima budget pemotongan. Untuk setiap titik, run normal, fork, dan restore dibandingkan dengan baseline yang dimulai dari awal tanpa checkpoint. Saat fork berjalan, home dan metadata checkpoint harus tetap sama. Ini menghasilkan 1.920 titik checkpoint dan 5.760 kelanjutan yang dibandingkan.

Trace terpotong tidak berarti queue terpotong. Collector memeriksa bahwa trace merupakan suffix ordinal yang kontigu dan jumlahnya cocok dengan `trace_total - trace_dropped`. Retensi nol tetap boleh dipakai untuk replay state, tetapi tidak membuktikan seluruh histori trace tersedia. Restore mengembalikan ID dan ordinal lama; host harus membuang cursor/proyeksi dari timeline yang diganti.

## Perintah dan batas bukti

```powershell
./scripts/verify.ps1
cargo run --locked -p wyrlex-headless -- --check-phase5
```

Checkpoint saat ini hanya berlaku dalam proses yang sama. Tidak ada format file checkpoint atau kontrol checkpoint di Studio. Pengujian replay tidak menggantikan perbandingan Linux/vendor. Model juga belum memiliki ICMP echo timeout; loss setelah ARP berhasil bisa menyisakan outcome 0 tanpa pending event. Itu tetap hasil belum terselesaikan, bukan ping sukses.

Materi Cognitive Coverage lokal masih stale setelah perubahan source. Test lulus dan catatan dihasilkan tidak menandai pemahaman manusia sebagai verified.

Pertanyaan teach-back: jika PRNG sudah berubah tetapi reservasi arrival gagal, mengapa retry dari state itu tidak lagi setara dengan replay dari checkpoint sebelum percobaan?
