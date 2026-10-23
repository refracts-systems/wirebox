# ADR-0007 — Model paket phase 3

- Tanggal: 20 Oktober 2026
- Status: Accepted; implementasi dan [bukti G3](../evidence/phase-3/README.md) tersedia pada 21 Oktober 2026.

## Kontrak sebelum acceptance

Gunakan graph dan heap/timer phase 2 dengan dua event domain one-shot: frame arrival (100), ARP retry/timeout (101). Scheduler menyimpan token/ID, tidak callback/pointer. Dispatch domain berada di Zig. API counter sebelumnya tetap membatasi action 1/2.

Network dibuat/configured terlebih dahulu; topology/configuration dibekukan saat satu workload ping atau injection dimulai. Perubahan skenario dilakukan pada Network baru. Semua byte dihasilkan dan diproses Zig; Rust memuat fixture, menjalankan, memeriksa dengan oracle terpisah, dan mengekspor artifact.

Versi paket adalah salinan immutable maksimal 1600 byte. `packet_id` mengidentifikasi datagram/frame logis; `version_id` menunjuk snapshot, `parent_version` menghubungkan encapsulation/decapsulation/mutation. Reply/error mendapatkan packet ID baru dan menunjuk versi pemicunya. Transmisi memiliki ID tersendiri: flooding berbagi versi byte tetapi menghasilkan transmisi berbeda. Field view memakai offset byte, bit dari MSB, width, dan nilai big-endian; struct native bukan wire format.

Subset mengikuti kontrak phase 0: Ethernet II, ARP Ethernet/IPv4, IPv4 IHL=5/MTU=1500 tanpa fragmentasi, route LPM, ICMP echo 8/0 dan 0/0 serta error 3/0 dan 11/0. Options, VLAN, fragmentasi, protokol lain, dan multicast IP ditandai unsupported. Paket rusak menjadi invalid_packet. Error ICMP tidak dibuat untuk error ICMP, invalid IP, atau paket broadcast/multicast.

Link full-duplex memakai antrean serialisasi per egress port, `ceil(frame_bytes*8*1e9/bitrate) + propagation`. Frame minimum 60 byte termasuk zero padding, tanpa FCS/preamble/SFD/IFG, collision, congestion/drop queue, jitter/loss, STP, atau model fisik. MAC learning per switch, ARP cache per port; belum aging. Cache kosong saat create. ARP men-buffer datagram, request pada 0/1/2 detik relatif lookup dan timeout pada 3 detik. Pending timer dibatalkan setelah resolve.

Jurnal domain ring terpisah dari scheduler queue dan packet storage; retensi tidak memengaruhi eksekusi. Overflow kapasitas packet/tx/cache/pending menjadi error terminal yang terlihat. Dispatch domain dapat telah membuat sebagian state sebelum kegagalan resource; Network dilatch failed dan tidak dapat dilanjutkan. Ini tidak mengubah kontrak atomic counter firing phase 2. Failed tidak boleh dilaporkan sebagai ping sukses.

API network `wb_net_*` v1 menggunakan copy C ABI, mutex host yang sama, serta handle namespace tersendiri. Dua bit atas membedakan probe (`00`), counter runtime (`10`), network (`11`); generation 30 bit, index+1, slot dipensiunkan saat generation habis. Empat Network disimpan oleh allocator Zig dan dibebaskan Zig. Batas model awal: 128 entity, 32 static route, 16 alias, 64 MAC/ARP entry, 16 pending resolution (8 packet masing-masing), 128 versi byte, 256 transmisi/pending scheduler, 1024 row domain maksimum. Tidak overwrite byte historis.

## Pembuktian

S01–S05 tetap memakai fixture beku. Oracle header IPv4 phase 0 dibandingkan langsung dengan byte request sebelum/sesudah forwarding. Rust memverifikasi checksum secara terpisah, ICMP quote/payload, relasi ID, waktu retry/delay, dan outcome dari byte yang benar-benar diterima. Static route diuji juga pada kasus sehat alias, bukan hanya setelah dihapus. Negative tests mencakup truncation, checksum, options/fragment/VLAN/protocol, dan suppression ICMP error.

Rujukan primer diperiksa 20 Oktober 2026: [RFC 791](https://www.rfc-editor.org/rfc/rfc791), [RFC 792](https://www.rfc-editor.org/rfc/rfc792), [RFC 826](https://www.rfc-editor.org/rfc/rfc826), [RFC 1812 §4.3.2.7](https://www.rfc-editor.org/rfc/rfc1812#section-4.3.2.7). Ini subset engineering, bukan klaim seluruh RFC atau firmware vendor. Capture Linux pembanding tetap gate G5.
