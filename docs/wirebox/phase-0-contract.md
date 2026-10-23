# Kontrak pembuktian phase 0 — baseline v1

- Ditetapkan: 20 Oktober 2026, dalam tugas pengguna untuk melaksanakan phase 0 dan 1.
- Status: baseline engineering v1; hasil G0/G1 dicatat pada [evidence](../evidence/phase-0-1/README.md).
- Perubahan scope/angka sesudah ini harus memiliki catatan revisi sebelum acceptance run berikutnya.

## Skenario dan batas MVP

Fixture normatif: [icmp-static-v1.json](../../fixtures/scenarios/icmp-static-v1.json). Empat node PC1–SW1–R1–PC2, dua subnet IPv4, tiga link 100 Mb/s dengan propagation 1 µs. PC1=10.0.0.2/24, router=10.0.0.1/24 dan 10.0.1.1/24, PC2=10.0.1.2/24. Host memakai default route ke router; router juga mempunyai route statis ke alias local PC2 198.51.100.2/32 melalui 10.0.1.2.

S01 memakai tujuan 10.0.1.2. S02 memakai alias 198.51.100.2 agar menghapus route **statis** pada R1 menghasilkan no_route tanpa berpura-pura menghapus connected route yang akan dipasang ulang OS. Alias adalah alamat lokal tambahan host, bukan node tambahan.

MVP mendukung Ethernet tanpa VLAN, MAC learning/flooding, ARP IPv4, local address, IPv4 tanpa options/fragmentasi, routing statis longest-prefix match tanpa ECMP, serta ICMP echo/type 3 code 0 dan type 11 code 0 yang diperlukan fixture. Mengabaikan fitur di luar subset tidak diperbolehkan; hasilnya unsupported/unknown atau invalid_packet sesuai sebabnya.

Belum termasuk TCP/UDP, NAT/firewall, PPPoE, routing dinamis, IPv6, VLAN, physical optics, atau image firmware vendor. Kebutuhan subset lain pada pilot ISP memerlukan perluasan scope dan pengujian tersendiri.

Byte frame dimulai pada destination MAC sampai payload/padding yang diperlukan; preamble/SFD/FCS dan encoding fisik belum dimodelkan. MTU IPv4 1500; lebih besar menghasilkan unsupported fragmentation. Header/checksum dihasilkan eksplisit, bukan reinterpretasi struct native. Initial TTL fixture 64.

## Expected result dan oracle

| Kasus | Input/mutasi | Hasil wajib ketika phase 3 tersedia |
|---|---|---|
| S01 | Topologi sehat, tujuan PC2 | Echo reply; R1 meneruskan request TTL 64→63 dengan checksum benar |
| S02 | Hapus route statis alias PC2 di R1 | no_route di R1; ICMP network unreachable type 3/code 0; tidak forwarding ke interface sembarang |
| S03 | TTL request=1 | R1 tidak meneruskan request; ICMP time exceeded type 11/code 0 |
| S04 | PC2 eth0 tidak merespons | ARP tiga attempt pada 0/1/2 detik dari lookup, timeout pada 3 detik; buffering dilepas dan alasan arp_timeout terlihat |
| S05 | Header IPv4 terpotong `4500` | invalid_packet/truncated_ipv4; tidak crash atau membaca di luar buffer |

S04 harus selesai dalam horizon virtual 5 detik. Ketentuan ARP adalah kebijakan model v1, bukan klaim nilai default semua vendor. Aturan ICMP lengkap tetap dibatasi subset; error untuk error/broadcast harus dicegah saat implementasi protokol dibuat.

Oracle awal: [header IPv4 sebelum/sesudah](../../fixtures/packets/ipv4-forwarding-v1.json), disusun independen sebelum codec engine tersedia. Perubahan TTL dan checksum diperiksa dengan one's-complement checksum terpisah. Fixture bukan hasil capture atau hasil run simulator.

Sumber spesifikasi: [IPv4 RFC 791](https://www.rfc-editor.org/rfc/rfc791), [ICMP RFC 792](https://www.rfc-editor.org/rfc/rfc792), dan [ARP RFC 826](https://www.rfc-editor.org/rfc/rfc826). Kutipan scope di sini merupakan subset model; tidak menyatakan conformance seluruh RFC. Pembanding runtime Linux/capture tetap wajib pada G5, terpisah dari fixture manual.

## Intent pilot inti yang dibekukan

`EnsureReachability` v1: sumber dan tujuan uji IPv4 eksplisit, ICMP echo, arah pergi/balik, routing table main, serta return path yang diketahui. Postcondition pilot: tiga probe berurutan berhasil dengan timeout satu detik per probe pada lab tanpa loss yang disengaja, disertai read-back state route. Probe tidak menggantikan pengecekan capability.

`EnsureStaticRoute` adalah calon operation v1: target, prefix, next-hop, interface/egress, dan table main. Menambahkan route yang belum ada; route identik menghasilkan no-op, konflik menghasilkan plan conflict. Tidak mengganti/menghapus route lain secara tersembunyi. Kompensasi hanya menghapus route yang dibuat run tersebut setelah memeriksa bahwa state belum berubah. Bentuk schema runtime dirancang pada phase 7, mengikuti kontrak semantik ini.

Probe ABI phase 1 tidak mengeksekusi intent tersebut. Per 21 Oktober 2026, [phase 3](packets.md) menyediakan routing simulated state dan salinan byte/trace. Executor intent, snapshot observed state, serta mutation perangkat belum tersedia; kontrak pilot nyata di atas belum diuji.

## Angka acceptance yang dibekukan v1

Target pada [scorecard pilot](pilot.md) diadopsi sebagai baseline v1: U1–U5 mandiri dengan runbook; S01–S05 benar; S06/S07 diulang tiga kali pada masing-masing lima seed; kasus G7 tiga pengulangan per backend; nol status sukses palsu; manfaat manual pada minimal empat dari lima tugas; respons input p95 <100 ms pada empat node/10.000 row; sesi stabil 30 menit.

Kelima tugas pembanding manual adalah diagnosis route hilang, diagnosis TTL habis, diagnosis next-hop/ARP gagal, perbaikan route statis, serta peninjauan/ulang plan setelah drift. Catat waktu dari mulai tugas sampai hasil terverifikasi, jumlah command/form submission di luar navigasi biasa, bantuan, dan kebenaran. Freeze variasi kasus dan urutan sebelum eksperimen G7 untuk mengurangi efek belajar.

Budget retensi awal: visible list 10.000 row dan total working set aplikasi ≤1 GiB pada fixture acceptance empat node. Data lama tidak boleh hilang tanpa indikasi batas riwayat; queue scheduler tidak dipangkas karena batas GUI. Budget ini target engineering, bukan hasil pengukuran. Jika baseline implementasi menunjukkan kebutuhan berbeda, revisi kontrak secara eksplisit sebelum acceptance run.

Mesin referensi phase 0/1: Windows 11 x64, build 26300, Ryzen 5 5600H, 16 GB RAM, GPU AMD terintegrasi dan RTX 3050 Laptop; rincian aktual ada pada [environment.json](../evidence/phase-0-1/environment.json). Simulator Linux pembanding belum diakses; distro/kernel tetap unknown.

## Inventaris, akses, dan dependency yang tersisa

[pilot-v1.json](../../fixtures/inventory/pilot-v1.json) mencatat informasi pengguna dan unknown. [Excerpt tersanitasi](../../fixtures/inventory/zte-c320-user-report.txt) mempertahankan versi board, state GPON, serta batas provenance. Tidak ada endpoint/kredensial disimpan.

Scope G0 saat ini adalah development lokal dan fixture sintetis. Linux VM/server dan RouterOS 7 disebut pengguna; distro, minor version, model, serta akses uji dibutuhkan sebelum G6. C320/ONU berasal dari laporan pengguna, tidak dibuktikan sebagai hardware fisik; detail model/firmware ONU serta pasangan PON dibutuhkan sebelum G8.

Inventaris dianggap **tercatat dengan unknown**, bukan seluruh akses terverifikasi. Semua kegiatan phase 0/1 lokal; tidak ada scan jaringan, login perangkat, atau perubahan konfigurasi perangkat.
