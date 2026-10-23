# Phase 3 — Mengikuti satu ping melewati switch dan router

**Yang dikerjakan.** [network.zig](../../../kernel/src/network.zig) memuat state port/link, MAC table per switch, ARP cache per port, route, pending ARP, paket, transmisi, dan trace. `ping` membuat ICMP echo dan IPv4. `sendIp` memilih route. `resolve` memakai ARP atau men-buffer datagram. `transmit` menghitung waktu arrival, lalu `dispatch` menjalankan frame arrival dari scheduler phase 2.

Alur S01 dari [artifact aktual](../../evidence/phase-3/S01.json):

```text
PC1: ping 10.0.1.2 → route via 10.0.0.1
PC1: MAC router belum ada → ARP request
SW1: belajar MAC sumber dan flood broadcast
R1: ARP reply → PC1 kirim frame IPv4
SW1: forward frame ke R1
R1: decap, pilih connected route eth1, TTL 64→63
R1: ARP 10.0.1.2 → PC2 membalas
R1: tulis MAC Ethernet baru, kirim ke PC2
PC2: echo reply → route balik melalui R1 → PC1 menerima
```

**Keputusan routing.** `route` mempertimbangkan connected prefix dari port dan static route, memilih longest prefix match. Jika panjang sama, connected menang; static prefix duplikat ditolak saat konfigurasi. Alias 198.51.100.2/32 pada PC2 membantu S02: route statis khusus di R1 dapat dihapus tanpa menghapus connected route ke subnet PC2. Pada S02 R1 membuat ICMP network unreachable 3/0; pada S03 TTL=1 berhenti di R1 dan menghasilkan time exceeded 11/0. Kedua error harus benar-benar tiba di PC1 agar acceptance lulus.

**Keputusan ARP dan switching.** Bila MAC next-hop belum ada, datagram disimpan sebagai version ID dan ARP dimulai. Request muncul pada 0/1/2 detik relatif lookup; timeout pada 3 detik melepas buffer. Bila reply datang, timer dibatalkan dan byte datagram yang dibuffer dipakai untuk membentuk frame. Switch belajar source MAC, flood broadcast/unknown unicast ke port lain, lalu memakai unicast forwarding setelah mengenal destination. Flooding dapat memakai satu versi frame di beberapa transmisi berbeda.

**Keputusan waktu link.** `transmit` memakai antrean serialisasi tiap egress port: start = maksimum(now, next_free), arrival = start + pembulatan ke atas durasi byte frame + propagation. Link full duplex, sehingga dua arah mempunyai egress berbeda. Pada fixture 100 Mb/s, frame 60 byte butuh 4.800 ns untuk serialisasi dan 1.000 ns propagation. [Walkthrough S01](../../evidence/phase-3/byte-walkthrough.md) memperlihatkan transmisi 9 dari R1 tiba 5.800 ns setelah start.

**Batas perilaku.** Satu Network dikonfigurasi sebelum satu workload `ping` atau `inject`; skenario mutasi memakai Network baru. Tidak ada aging, STP, VLAN, loss/jitter, TCP/UDP, atau emulasi OS/perangkat. MAC learning, ARP retry 1 detik, dan timing link di sini adalah aturan model Wirebox v1, bukan klaim default semua vendor. [Kontrak lengkap](../../wirebox/packets.md) menyatakan byte frame dan kapasitas.

**Periksa.** Ikuti `Network.ping`, `sendIp`, `route`, `resolve`, `arpAttempt`, `switchFrame`, `receiveIp`, `transmit`, `dispatch` secara berurutan. Jalankan `cargo run --locked -p wirebox-headless -- --packet-demo S01`, lalu ulangi S02–S05. Bandingkan trace `route_selected`, `arp_lookup`, `decrement_ttl`, `frame_tx`, `frame_rx`, dan `delivered` dengan versi byte yang ditunjuk row itu.
