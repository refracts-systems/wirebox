# Mini-simulator: implementasi dan verifikasi

## Asal artifact

Prototipe tersedia pada C:\Users\ricky\AppData\Local\Temp\opencode\pt_minisim. Berkas utama: ptmini.h, main.cpp, build.bat, dan ptmini.exe. Source/executable tidak dipindahkan ke repository ini; dokumentasi dan catatan verifikasinya yang disimpan.

Source dan executable diperiksa pada 19 Oktober 2026. Executable yang tersedia dijalankan langsung; belum dilakukan rebuild yang membuktikan kesesuaian binary dengan source. Hash artifact verifikasi tersedia di [verification/artifact-manifest.json](verification/artifact-manifest.json).

## Skenario

Topologi: PC1 — SW1 — R1 — PC2. PC1 berada di 10.0.0.0/24 dan PC2 di 10.0.1.0/24. Link pada demo disetel 100 Mb/s dengan propagation delay 1 mikrodetik.

1. ICMP echo: ARP, pembelajaran/flooding MAC, forwarding router, dan echo reply.
2. TCP: handshake, buffering Nagle sederhana, satu kehilangan frame, retransmisi, lalu FIN.
3. UDP: pengiriman ke port terdaftar, lalu port tertutup yang menghasilkan ICMP port unreachable.

Data aplikasi seperti GET /index.html hanyalah payload demo, bukan bukti adanya implementasi server HTTP penuh.

## Hasil eksekusi lokal

| Metrik | Jitter ON | Jitter OFF |
|---|---:|---:|
| Kedatangan pertama di SW1, detik | 0.0000244705 | 0.00002436 |
| Waktu event terakhir, detik | 1.00167 | 1.00163 |
| Baris Event List | 461 | 461 |
| Record frame arrival | 66 | 66 |
| Record transisi state TCP | 7 | 7 |
| Record retransmisi TCP | 1 | 1 |
| Record kehilangan frame yang disuntikkan | 1 | 1 |

Output disimpan di [summary.txt](verification/summary.txt) dan [selected-trace.txt](verification/selected-trace.txt). Ini demonstrasi skenario, bukan suite pengujian conformance atau benchmark throughput.

eventCount() mengembalikan ukuran eventList_. Satu callback scheduler bisa menghasilkan beberapa trace, sehingga jumlah baris tidak boleh dilabeli sebagai jumlah event scheduler yang dieksekusi.

## Mekanisme yang diperiksa pada source

Scheduler memakai priority_queue berurutan (time, seq). Port::send menghitung processing delay + transmission time + propagation delay + jitter, lalu menjadwalkan pemanggilan receive() milik port tujuan.

runUntil(limit) memproses event yang jatuh tempo sampai batas, tetapi implementasi ini tidak memajukan now() ke limit jika tidak ada event di sana. Angka waktu di atas adalah waktu callback terakhir, bukan otomatis durasi horizon yang diminta.

Generator random menggunakan seed tetap 20260117. Komentar/output demo yang menyebut jitter ON selalu tidak reproducible terlalu luas: dengan seed, urutan input, build, dan lingkungan yang sama, run dapat diulang. Reproduksibilitas lintas standard library/platform belum diuji.

## Batas model

- TCP adalah model pengajaran: tanpa SACK, congestion control, atau timer TIME_WAIT lengkap; MSS/window dan initial sequence disederhanakan.
- Link belum memodelkan antrean, contention, atau collision.
- IPv6, fragmentasi, NAT, dan aplikasi penuh belum tersedia menurut README prototipe.
- Mode realtime tersedia sebagai enum, tetapi demo menjalankan mode stepped.
- Kesamaan nama kode *_FC_* tidak membuktikan kesamaan semua perilaku dengan PT.

Bug timer retransmisi pada versi awal prototipe diperbaiki dengan meng-arm timer untuk SYN/FIN/data dan memakai runUntil. Hal itu menjelaskan kebutuhan desain prototipe; tidak membuktikan bug identik pada PT.

## Makna untuk simulasi tanpa hardware

Prototipe menunjukkan bahwa perangkat virtual, PDU, state protokol, dan waktu dapat berjalan di dalam program lokal. Model perangkat tetap dibutuhkan. Klaim kesetaraan dengan vendor, kondisi optik, atau hardware produksi memerlukan model tambahan dan pengujian pembanding.

Sumber historis: [README prototipe](sources/ptmini-README.original.md) dan [addendum](sources/REPORT_addendum.original.md).
