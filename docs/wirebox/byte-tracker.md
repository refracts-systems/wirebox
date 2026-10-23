# Byte tracker dan replay

Kebutuhan pengguna adalah memahami paket hingga byte per byte: nilainya, arti field, proses yang membacanya, dan alasan sebuah perubahan. Kemampuan ini berasal dari desain representasi dan instrumentasi; Zig maupun Rust dapat menjadi implementasinya.

Status 21 Oktober 2026: [tracker headless phase 3](packets.md) menyediakan salinan byte immutable, protocol field mapping, packet/transmission ID, dan trace keputusan/mutasi. [Walkthrough aktual](../evidence/phase-3/byte-walkthrough.md) memakai hasil Zig. Inspector GPUI masih probe phase 1; integrasi tampilan paket adalah phase 4.

## Tiga representasi yang saling terhubung

| Representasi | Isi | Kegunaan |
|---|---|---|
| Byte buffer | Urutan byte paket pada suatu versi | Hex view, decode, dan perbandingan nilai |
| Protocol view | Field, offset byte/bit, panjang, endianness, serta hasil validasi | Menghubungkan tampilan semantik dengan buffer |
| Mutation/decision record | Event penyebab, proses, alasan, versi sebelum/sesudah | Menjelaskan perubahan dan keputusan tanpa perubahan byte |

Decoder membaca byte dengan batas yang diperiksa. Encoder menghasilkan wire format secara eksplisit. Alignment dan padding struct native tidak menjadi definisi header jaringan.

Phase 3 memakai salinan penuh per versi, tetap dapat dibaca setelah engine melanjutkan eksekusi. Flooding dapat berbagi versi yang sama melalui beberapa transmisi. Copy-on-write atau delta menjadi opsi optimasi berikutnya; sejarah tidak boleh menunjuk memori yang sudah berubah.

## Identitas dan hubungan sebab

Schema phase 3 menetapkan packet_id, version_id/parent_version, transmission ID, entity, event_id/cause_id, dan ordinal trace. Kontrak serta arti tiap ID dijelaskan pada [model paket](packets.md).

Flooding dapat menciptakan beberapa transmisi dari satu paket. Encapsulation, retransmission, dan echo reply juga memerlukan hubungan yang dapat ditelusuri. Jangan menganggap satu ID tanpa hubungan parent/child cukup untuk semua kasus tersebut.

Keputusan seperti route lookup, cache hit, drop karena policy, atau antrean penuh dapat tidak mengubah byte. Tracker tetap harus mencatat state/input yang relevan serta alasan hasilnya.

## Contoh perilaku yang ingin terlihat

Pada skenario forwarding IPv4 sederhana:

1. Pengguna memilih event penerimaan frame di router.
2. Inspector menampilkan byte frame dan pemetaan Ethernet/IPv4/ICMP yang berhasil didekode.
3. Langkah routing menjelaskan route dan interface yang dipilih.
4. Versi paket berikutnya menunjukkan TTL berubah, checksum IPv4 yang menyesuaikan, serta header Ethernet untuk hop berikutnya.
5. Pengguna dapat kembali membaca versi sebelumnya dan event yang menyebabkan perubahan.

Data untuk alur ini sudah tersedia pada runner headless; navigasi visual belum terintegrasi. Byte frame mencakup destination MAC hingga padding, tanpa preamble/SFD/FCS/IFG; [batas representasi](packets.md) berlaku pada semua hasil.

## Tampilan yang diusulkan

Topology canvas menunjukkan lokasi event; timeline memilih waktu/event; inspector memperlihatkan state perangkat dan keputusan; hex view menyorot rentang field. Memilih field menyorot byte terkait, dan memilih byte menunjukkan field atau rentang yang belum dikenali.

UI sebaiknya menerima snapshot atau batch trace dengan ID stabil. Pengiriman satu panggilan FFI untuk setiap byte akan menambah overhead dan mempersulit konsistensi tampilan. Detail kontrak ada pada [pilihan teknologi](technology.md).

## Replay dan time travel

Ada dua kemampuan berbeda yang harus disebut jelas: membaca ulang rekaman dan melanjutkan simulasi dari state historis. Kemampuan kedua memerlukan checkpoint yang cukup lengkap atau rekonstruksi deterministik dari awal.

State yang perlu dipertimbangkan meliputi konfigurasi/topologi, cache ARP/MAC, state protokol, queue dan timer tertunda, buffer paket, counter/ID, PRNG, input eksternal, serta versi engine/model. Seed saja tidak cukup bila urutan konsumsi random atau input berubah.

Usulan awal adalah checkpoint + replay event dengan urutan stabil. Branch dari suatu checkpoint membuat state simulasi terpisah; perubahan cabang tidak mengubah snapshot asal atau perangkat nyata.

Rekaman produksi sering parsial. Wirebox harus membedakan state yang benar-benar teramati, yang direkonstruksi, dan yang diasumsikan. Kebijakan checkpoint, retensi, serta jaminan replay lintas versi masih terbuka.
