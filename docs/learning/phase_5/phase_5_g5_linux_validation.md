# Phase 5: menutup G5 dengan pembanding Linux

Catatan ini menjelaskan perubahan kode dan bukti pada [laporan G5](../../evidence/phase-5-gate/README.md). G5 lulus pada cakupan yang dinyatakan di laporan itu. Ini belum membuktikan semua firmware atau semua perilaku Linux.

## Yang berubah dan alasannya

| Berkas | Tanggung jawab dan keputusan |
|---|---|
| `crates/wirebox-headless/src/packet_demo.rs` | Menambahkan export `corpus_demo`: S01–S04, pemulihan route R01, checksum IPv4 N01, dan checksum ICMP N02 beserta paket control yang benar |
| `crates/wirebox-headless/src/main.rs` | Menyediakan `--corpus-demo CASE` tanpa menambah FFI atau mengubah API simulator |
| `crates/wirebox-headless/src/phase5.rs` | Memenuhi kontrak lima seed dan tiga pengulangan, termasuk baseline tanpa checkpoint serta home/fork/restore; status CLI dibatasi pada pekerjaan command itu sendiri |
| `scripts/corpus/linux_lab.py` | Menjalankan namespace Linux, mengirim input, merekam batas waktu probe dan perubahan route, lalu membersihkan hanya namespace yang dibuatnya |
| `scripts/corpus/compare.py` | Menambahkan comparator revision 3, dengan invariant per kasus dan pembandingan terhadap capture serta konteks eksperimen |
| `scripts/corpus/validate.py` | Memilih schema yang didukung secara eksplisit, mengecek artifact, lalu menghitung ulang comparison; revision lama tetap bisa direproduksi |
| `scripts/corpus/test_corpus.py` | Menambah negative controls untuk mematahkan klaim lulus yang keliru |
| `corpus/scenarios/linux-static-ipv4-v3.json` dan schema v2 | Membekukan kontrak kasus tambahan tanpa memperlebar schema lama diam-diam |

Tidak ada perubahan perilaku Zig engine, C ABI, atau wrapper Rust. Schema v2 ini hanya untuk eksperimen runtime Linux, bukan implementasi dossier firmware ADR-0014.

## Jangan sampai alat pengirim memperbaiki paket salah

Untuk paket normal, `AF_INET` dengan `IP_HDRINCL` cukup bagi eksperimen sebelumnya. Namun Linux dapat menghitung ulang checksum header IPv4. Kalau kita hendak menguji checksum rusak melalui jalur itu, target mungkin menerima paket yang sudah diperbaiki oleh pengirim.

N01/N02 memakai `AF_PACKET`, membentuk header Ethernet, dan mengirim dari port SW1 menuju R1. Comparator mewajibkan capture pada ingress R1 berisi byte input salah yang sama persis. Baru sesudah itu ketiadaan balasan selama jendela pengamatan punya makna.

Analogi backend: jangan menguji parser menggunakan client yang menormalisasi payload sebelum request sampai ke server. Periksa juga byte yang benar-benar melewati batas server.

## Silence membutuhkan positive control

Tidak ada balasan dapat berarti paket ditolak, jalur mati, capture gagal, atau paket tidak pernah dikirim. Karena itu comparator tidak menerima silence saja.

Urutannya: paket salah terlihat di ingress → amati minimal satu detik tanpa reply/forwarding → kirim versi yang benar melalui jalur yang sama → wajib ada balasan yang sesuai. Statistik capture juga harus lengkap tanpa kernel drop. Uji ini membuktikan perilaku dalam jendela tersebut, bukan janji tidak akan ada kejadian selamanya.

`inject()` sendiri tidak membuat outstanding ping pada Wyrlex. Balasan control terlihat di PC1 sebagai `unsolicited_reply`, sehingga summary bukan `echo_reply`. Exporter memeriksa reception trace dan byte paket; comparator tidak mengarang status ping sukses. Test awal yang mengharapkan summary ping untuk input injection gagal dan diperbaiki pada oracle host, tanpa mengubah engine.

## ARP: hasil yang berbeda tetap terlihat

S04 Wyrlex berakhir sebagai `arp_timeout` tanpa pending work. Linux pada eksperimen mengirim ICMP 3/1 setelah ARP tidak dijawab. Itu dua hasil internal/output yang berbeda. Kontrak v3 menyatakannya sejak sebelum capture: yang dibandingkan adalah kegagalan resolusi, tidak adanya forwarding paket IP ke tujuan, serta bounded completion.

Linux memakai link yang tetap up dengan `arp_ignore=8` agar ARP bisa ditangkap. Wyrlex memakai peer port nonaktif. Karena itu hasilnya tidak membuktikan kesetaraan admin-down, carrier, atau kegagalan perangkat fisik. Mengubah expected result sesudah melihat kegagalan agar terlihat hijau akan merusak pembuktian; di sini batas dan kebijakan berbeda sudah ada dalam kontrak sebelum run.

## Pemulihan route memerlukan bukti urutan

R01 mempertahankan namespace yang sama. Probe pertama gagal dengan route tidak ada. Lalu route ditambahkan, kemudian paket input yang sama dikirim lagi dan mendapat echo reply. `experiment.json` menyimpan batas monotonic probe serta awal/akhir perubahan. Snapshot sebelum/sesudah mencatat route.

Comparator mengecek kegagalan awal, keberhasilan akhir, dan perubahan yang berada di antaranya. Ini mirip menguji transisi state layanan backend, bukan hanya memeriksa response akhir. Belum ada planner atau adapter perangkat nyata dalam pengujian ini.

## Replay dan sejarah yang tidak lengkap

Oracle sekarang mencakup lima seed, tiga pengulangan, enam skenario, empat kapasitas trace, empat konfigurasi impairment, dan lima titik checkpoint. Hasilnya: 1.440 baseline runs, 7.200 checkpoint cases, dan 21.600 continuation yang cocok persis.

Retensi trace yang kecil memang membuang histori. Oracle memeriksa suffix ordinal dan jumlah baris tersisa; tidak menyebutnya histori lengkap. Beberapa run loss tetap memiliki outcome pending karena belum ada timeout echo setelah ARP berhasil. Reproduksi state pending membuktikan determinisme, bukan membuktikan konektivitas.

## Cara memeriksa ulang

```powershell
./scripts/verify.ps1 -CoreOnly
./scripts/corpus/run-linux-lab.ps1 -Repetitions 3
wsl -d Ubuntu-26.04 -- python3 -m unittest discover -s /mnt/c/Users/ricky/src/wirebox/scripts/corpus -p test_corpus.py -v
```

Command tersebut benar-benar dijalankan pada pekerjaan ini. Verifikasi core dan 24 test corpus lulus; pengujian Linux menghasilkan 21 comparison dan 186 invariant checks. Record lama yang comparison-nya gagal tetap disimpan. Manifest source, log, cleanup, dan keterbatasan ada pada laporan G5. Materi Cognitive Coverage pribadi masih stale; test hijau tidak menandakan pemahaman manusia telah diverifikasi.

**Pertanyaan teach-back:** jika tidak ada balasan terhadap paket checksum salah, tetapi versi paket yang benar juga tidak mendapat balasan, mengapa kita belum boleh menyimpulkan bahwa parser target berhasil menolak checksum?
