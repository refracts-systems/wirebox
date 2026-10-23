# Catatan reverse engineering Cisco Packet Tracer 9.0.1

## Kesimpulan yang dapat dipakai

Laporan menunjukkan model simulasi berbasis objek perangkat, port, link, PDU, proses protokol, dan scheduler waktu virtual. Mini-simulator lokal memperagakan bahwa sebagian perilaku tersebut dapat diimplementasikan tanpa perangkat fisik dan tanpa image OS vendor.

Temuan arsitektur merupakan acuan desain. Temuan tersebut belum membuktikan kesetaraan implementasi baru dengan semua perilaku Packet Tracer atau perangkat produksi.

## Status pengetahuan

| Kategori | Arti |
|---|---|
| Dilaporkan terkonfirmasi | Laporan menyebut bukti RTTI, resource, dekompilasi, atau pengujian runtime; bukti asal belum seluruhnya diaudit ulang di repository ini. |
| Diverifikasi lokal | Source atau executable mini-simulator diperiksa dalam sesi dokumentasi ini. |
| Inferensi | Penjelasan yang konsisten dengan bukti tetapi belum ditunjukkan langsung. |
| Usulan | Keputusan desain untuk Wirebox, bukan hasil RE Packet Tracer. |
| Unknown | Bukti belum cukup atau implementasi belum berhasil direproduksi. |

## Koreksi yang harus dibaca lebih dahulu

1. Scheduler queue dan Event List GUI adalah komponen terpisah. Jumlah baris trace tidak sama dengan jumlah event scheduler.
2. Hipotesis AES/Rijndael + Mash sebagai enkripsi container .pkt digantikan oleh laporan decoder dokumen Twofish/EAX. Jalur atribut terenkripsi harus dibedakan dari container dokumen.
3. Pencarian passphrase default container .pkt tidak lagi menjadi kesimpulan akhir. Laporan terakhir menyebut key dan nonce konstan untuk keluarga dokumen yang diuji.
4. Reader offline masih bergantung pada keystream hasil capture; reproduksi cipher mandiri belum berhasil. Pembacaan sampel tidak membuktikan writer atau round-trip sudah tersedia.
5. Pasangan konstanta 0xBA/0xBE dilaporkan berkaitan dengan metadata uuid/nets, bukan terbukti sebagai decoder extension.
6. Nama class dan kode flow-chart bukan pemulihan seluruh algoritme. Bukti yang tersedia juga belum merupakan audit semua jalur socket OS.

## Urutan baca

Mulai dari [arsitektur](architecture.md), lalu [evidence ledger](evidence-ledger.md) dan [mini-simulator](mini-simulator.md). Baca [format .pkt](pkt-format.md) untuk konteks interoperabilitas. [Implikasi Wirebox](wirebox-implications.md) memisahkan usulan implementasi dari temuan.

[Audit lokal REA/Ghidra dan smoke runtime PT](../evidence/pt-comparison/README.md) mencatat pemeriksaan ulang 21 Oktober 2026. [Runbook pembanding](comparison-tests.md) membedakan smoke, kesetaraan subset protokol, dan klaim keunggulan.

Laporan historis disimpan apa adanya di [sources](sources/README.md), termasuk klaim lama yang kemudian dikoreksi. Ringkasan tematik di direktori ini menjadi panduan status terkini dari bahan yang tersedia.
