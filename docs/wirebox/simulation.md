# Simulasi tanpa perangkat

Simulasi jaringan tanpa perangkat fisik dan image OS vendor memungkinkan untuk perilaku yang kita implementasikan. Perangkat virtual tetap memiliki identitas, port, konfigurasi, state protokol, dan aturan pemrosesan.

Pada 21 Oktober 2026, [phase 3 Wirebox](packets.md) sudah menjalankan skenario Ethernet/ARP/IPv4/ICMP mandiri melalui Zig–Rust. [Bukti G3](../evidence/phase-3/README.md) memisahkan hasil engine Wirebox dari laporan PT dan mini-simulator eksternal. Berikutnya integrasi Studio; validasi perangkat nyata tetap diperlukan untuk klaim kesesuaian vendor.

## Apa yang didukung bukti Packet Tracer?

| Temuan dalam catatan | Implikasi untuk Wirebox | Batas kesimpulan |
|---|---|---|
| Objek device, port, link, PDU, dan process | Dunia jaringan dapat direpresentasikan dalam program | Seluruh perilaku vendor belum dipulihkan |
| Heap timer serta registry by-ID | Eksekusi dapat diurutkan oleh waktu virtual dan timer dapat ditemukan/dibatalkan | Layout dan inheritance PT tidak perlu disalin |
| Rantai proses protokol dan routing software | Perilaku forwarding dapat diimplementasikan sebagai algoritme | Nama class/kode langkah tidak sama dengan spesifikasi lengkap |
| Event List GUI terpisah dari queue | Eksekusi dan tampilan trace perlu komponen berbeda | Tombol Back tidak membuktikan restore seluruh state |
| Demo mini-sim berjalan lokal | Skenario ARP, switching, routing, ICMP, TCP, UDP dapat diperagakan mandiri | Model demo terbatas dan bukan pengujian kesetaraan PT |

Status rinci tersedia pada [evidence ledger](../packet-tracer/evidence-ledger.md). Dokumen RE terutama bersumber dari laporan pengguna; eksekusi ulang mini-simulator merupakan verifikasi lokal terpisah.

## Tiga jenis pengujian yang dibahas

| Pendekatan | Pertanyaan yang cocok | Yang perlu tersedia |
|---|---|---|
| Analisis konfigurasi/reachability | Apakah jalur atau policy memungkinkan trafik tertentu? | Konfigurasi dan model semantik yang mendukung fitur tersebut |
| Simulasi discrete event | Kapan paket dikirim, timer habis, atau state berubah? | Algoritme protokol, scheduler, delay, dan asumsi model |
| Lab OS/vendor | Bagaimana implementasi OS tersebut berperilaku pada skenario ini? | Image/software yang sesuai dan lingkungan eksekusinya |

Ketiganya bisa saling melengkapi. Hasil reachability tidak otomatis menjelaskan retransmisi TCP. Lab OS pun tidak otomatis meniru ASIC, kondisi optik, atau seluruh kondisi perangkat fisik. Lihat [proyek pembanding](positioning.md).

## Kernel yang diusulkan

Alur dasar: aplikasi membuat paket → proses protokol membentuk byte → port menghitung waktu pengiriman → scheduler menjalankan penerimaan pada port tujuan → state protokol berubah → trace mencatat hasil dan sebabnya.

Pilihan awal yang disarankan:

- Urutan stabil `(virtual_time, sequence)` untuk event dengan waktu sama.
- ID timer dan mekanisme cancel yang tidak meninggalkan callback aktif pada objek yang sudah dihapus.
- API langkah terbatas seperti `step` dan `runUntil`; menguras antrean sampai kosong tidak cocok untuk semua timer periodik.
- PRNG eksplisit dengan seed/state yang dapat disimpan untuk jitter dan loss.
- Kernel yang tidak bergantung pada jam GUI; mode realtime hanya mengatur laju pemutaran waktu virtual.
- Satu pemilik state simulasi sebagai titik awal. Parallel execution ditambahkan jika kebutuhan terukur membenarkannya.

Konvensi [ADR-0003](../adr/0003-virtual-time-and-trace.md) sudah diimplementasikan pada [runtime phase 2](runtime.md): tick u64 nanodetik dan `runUntil(t, budget)` memajukan jam ke t setelah semua event due diproses. Jika budget habis, waktu berhenti pada eksekusi terakhir. [Bukti G2](../evidence/phase-2/README.md) memeriksa invariant scheduler dan counter sintetis; [model paket phase 3](packets.md) memakai scheduler tersebut untuk frame arrival dan ARP timer. Mini-sim lama memakai konvensi berbeda, dicatat pada [verifikasi prototipe](../packet-tracer/mini-simulator.md).

Queue eksekusi tidak boleh kehilangan event hanya karena UI membatasi jumlah baris. Jurnal dapat memiliki kebijakan retensi tersendiri; jika dibatasi, kemampuan replay yang tersisa harus jelas.

## Cakupan dan fidelitas

Setiap model perlu menyatakan fitur yang didukung, yang disederhanakan, dan yang belum diketahui. Contoh: routing IPv4 statis tidak menjanjikan BGP, ECMP, NAT, fragmentasi, atau semua perilaku ICMP.

Trace dapat menjelaskan mengapa model memilih route tertentu. Pernyataan bahwa router nyata akan memilih hasil identik memerlukan perbandingan pada konfigurasi, versi, dan skenario yang sesuai. Model error optik, antrean hardware, congestion, atau firmware membutuhkan pekerjaan tersendiri.

Untuk hasil yang bergantung pada fitur di luar model, tampilkan unsupported/unknown beserta penyebabnya. Jangan menyimpulkan pass hanya karena fitur tersebut dilewati.

## Hubungan dengan format Packet Tracer

Engine Wirebox dapat memakai format skenario sendiri yang berversi. Importer `.pkt` adalah pekerjaan terpisah untuk memetakan topologi dan konfigurasi yang dikenali.

Laporan terakhir mengoreksi hipotesis awal enkripsi `.pkt`, tetapi reader yang didokumentasikan masih bergantung pada captured keystream. Importer lengkap, writer, dan round-trip belum dibuktikan. Rujuk [status format `.pkt`](../packet-tracer/pkt-format.md), bukan potongan laporan lama.
