# Evidence ledger

ID PT/MINI di bawah adalah indeks dokumentasi Wirebox, bukan Evidence ID yang dihasilkan tool REA. [Audit lokal 21 Oktober 2026](../evidence/pt-comparison/README.md) menambahkan hash target, Evidence REA, pemeriksaan empat fungsi Ghidra, serta smoke GUI. Audit itu belum meluluskan differential protokol.

| ID | Temuan | Jenis bukti | Status dan batas |
|---|---|---|---|
| PT-01 | Perangkat/port/link adalah objek C++ | RTTI MSVC, vftable, resource tipe perangkat | Dilaporkan terkonfirmasi; tidak memulihkan seluruh field |
| PT-02 | PDU berlapis dan bisa diinspeksi | CPdu, kelas header, PDUDefinition.json | Dilaporkan terkonfirmasi; schema display bukan codec lengkap |
| PT-03 | Proses protokol tersusun melalui higher/lower | String error, method protokol, dekompilasi init | Struktur kuat; urutan semua jalur belum diketahui |
| PT-04 | Heap timer dan registry by-ID terpisah | FUN_141d1d3e0, FUN_141d22cf0, komparator CIpcTimer | Dilaporkan terkonfirmasi pada versi target |
| PT-05 | Event List merupakan log GUI | CBaseSimulationPanel, string buffer penuh | Dilaporkan terkonfirmasi; bukan queue scheduler |
| PT-06 | Routing merupakan algoritme software | FUN_14094c530, kode langkah routing | Dilaporkan terkonfirmasi sebagian; tidak membuktikan semua fitur |
| PT-07 | Dua mode waktu dan timer protokol | String GUI, helper waktu, instansiasi CTimer | Dilaporkan terkonfirmasi; mapping lengkap pacing belum diaudit |
| PT-08 | Hubungan Capture/Forward, scheduler, animasi | Nama tombol dan slot | Sebagian inferensi; kesetaraan satu klik/satu event belum pasti |
| PT-09 | Container dokumen berhasil dibaca offline | Dekompilasi loader, hook runtime, sweep corpus | Menurut laporan: 293/293 pkt, 88/88 ptd, 6/6 pka |
| PT-10 | Reader menggunakan captured keystream | Addendum §7.5, percobaan reproduksi gagal | Batas ukuran dan provenance masih material |
| PT-11 | Extension container belum terpecahkan | Percobaan decode negatif dan observasi startup | Hanya berlaku pada percobaan/sampel tersebut; cipher unknown |
| MINI-01 | Demo lokal dapat berjalan mandiri | Eksekusi ptmini.exe dan inspeksi source | Diverifikasi lokal; bukan pengujian kesetaraan PT |
| MINI-02 | Pengiriman antarport memakai callback terjadwal | ptmini.h, Port::send | Diverifikasi dari source; source-binary equivalence belum diuji lewat rebuild |
| MINI-03 | Skenario menghasilkan 461 record dan 66 arrivals | Summary executable, eventCount() pada source | 461 adalah baris trace, bukan jumlah scheduler pops |

## Hal yang tidak boleh dianggap sudah terbukti

- Seluruh jalur simulasi PT bebas pemanggilan socket OS. Bukti objek PDU mendukung jalur in-memory, tetapi belum mencakup seluruh call graph.
- Keberadaan semua kode flow-chart berarti semua algoritme telah dipulihkan.
- Replay GUI berarti time-travel state yang lengkap.
- QWaitCondition/QSemaphore sendiri membuktikan jumlah dan urutan kerja thread.
- Timer registry membuktikan PT memiliki bug retransmisi yang sama dengan prototipe.
- Entropy tinggi saja membuktikan cipher tertentu.
- Pembacaan XML membuktikan writer, round-trip, atau interoperabilitas seluruh versi.

## Kelanjutan verifikasi

Pemeriksaan baru mendukung pola heap pada PT-04, dialog batas log pada PT-05, serta jalur RIB/ICMP pada PT-06. Registry RB-tree, urutan komparator, dan lifecycle worker masih berstatus laporan asal untuk bagian yang tidak diaudit ulang. [Runbook pembanding](comparison-tests.md) menetapkan assertion serta batas akses input GUI sebelum eksekusi corpus setara.

Untuk klaim baru, simpan identitas artifact, jenis pengamatan (static/runtime), input skenario, output, dan batas pengujian. Cocokkan perilaku protokol dengan spesifikasi serta implementasi pembanding. Pisahkan pengujian logic simulator dari pengujian hardware atau firmware vendor.
