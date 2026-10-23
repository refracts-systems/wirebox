# S01 — byte yang benar-benar diproses Wirebox

Sumber: [S01.json](S01.json), dihasilkan runner Zig–Rust pada 21 Oktober 2026. Ini simulated state model `ethernet-arp-ipv4-icmp-v1`, bukan capture perangkat. [Kontrak model](../../wirebox/packets.md) menjelaskan batas protokol/link.

## Request diteruskan oleh R1

Pada **34.800 ns**, event **7** (cause **6**) menerima frame versi 4 di R1:eth0. Decapsulation menghasilkan versi **5**, route lookup memilih R1:eth1 (entity 8), next-hop **10.0.1.2**. Row ordinal **29**, `decrement_ttl`, membuat versi **6** dari versi 5. Keduanya mempunyai `packet_id=1`; versi lama tetap dapat dibaca.

```text
versi 5 (IPv4 + ICMP, 28 byte)
45 00 00 1c 00 01 00 00 40 01 65 dd 0a 00 00 02 0a 00 01 02 08 00 e5 ca 12 34 00 01

versi 6 (sesudah forwarding)
45 00 00 1c 00 01 00 00 3f 01 66 dd 0a 00 00 02 0a 00 01 02 08 00 e5 ca 12 34 00 01
```

Offset berikut dihitung dari awal versi IPv4, mulai nol:

| Offset byte | Field | Sebelum | Sesudah | Makna |
|---|---|---|---|---|
| 0, bit 0..3 | Version | 4 | 4 | IPv4 |
| 0, bit 4..7 | IHL | 5 | 5 | Header 20 byte |
| 2..3 | Total length | 0x001c | 0x001c | Datagram 28 byte |
| 8 | TTL | 0x40 (64) | 0x3f (63) | Router mengurangi satu hop |
| 10..11 | Header checksum | 0x65dd | 0x66dd | Checksum baru sesuai perubahan TTL; byte 11 tetap dd |
| 12..15 | Source IP | 10.0.0.2 | 10.0.0.2 | Source tetap PC1 |
| 16..19 | Destination IP | 10.0.1.2 | 10.0.1.2 | Destination tetap PC2 |
| 20..27 | ICMP | 0800e5ca12340001 | 0800e5ca12340001 | Type 8/code 0, ID 0x1234, sequence 1; tidak berubah saat forwarding |

Hanya byte **8** dan **10** berubah. Header 20 byte cocok persis dengan [golden fixture](../../../fixtures/packets/ipv4-forwarding-v1.json) yang dibekukan sebelum implementasi. Rust menghitung ulang checksum header dan ICMP secara terpisah dari codec Zig.

## ARP dan Ethernet hop berikutnya

R1 belum mengenal MAC 10.0.1.2, sehingga versi 6 dibuffer. ARP request versi 7 dikirim pada 34.800 ns; ARP reply versi 8 tiba pada 46.400 ns (event 10). Timer retry dibatalkan dan R1 membentuk frame **versi 9**, masih `packet_id=1`, parent versi 6:

```text
02 00 00 00 01 02  02 00 00 00 01 01  08 00
45 00 00 1c 00 01 00 00 3f 01 66 dd 0a 00 00 02 0a 00 01 02
08 00 e5 ca 12 34 00 01
00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
```

Destination MAC = **02:00:00:00:01:02** (PC2), source MAC = **02:00:00:00:01:01** (R1:eth1). EtherType 0800 menunjukkan IPv4. Padding 18 byte membuat frame 60 byte, tanpa FCS. Pada versi Ethernet, field IPv4 berada 14 byte lebih jauh: TTL di offset 22 dan checksum di offset 24..25.

Transmisi **9** mengirim versi 9 melalui link 13. Waktu start **46.400 ns**, serialisasi `60 × 8 / 100 Mb/s = 4.800 ns`, propagation **1.000 ns**, tiba **52.200 ns** sebagai event **11** dengan cause **10**. Identitas versi byte, transmisi, dan event mempunyai fungsi berbeda.

## Reply dan hasil

PC2 menghasilkan echo reply versi **11**, `packet_id=6`, parent versi **10** (request setelah decapsulation). ICMP berubah menjadi `00 00 ed ca 12 34 00 01`: type 0 dan checksum reply baru, ID/sequence tetap. Reply juga melewati R1 dan mengalami pengurangan TTL sendiri.

PC1 menerima versi **16** pada **69.600 ns (69,6 µs)**, event **14**, trace ordinal **63**. Summary: echo_reply, 12 event scheduler dieksekusi, 16 versi byte, 12 transmisi, 63 row trace, pending event/ARP nol. ID event 2 dan 9 adalah timer ARP yang dibatalkan sehingga tidak dieksekusi. Summary `now_ns=5.000.000.000` berasal dari horizon run; itu bukan waktu perjalanan ping.

## Membaca kegagalan dengan cara yang sama

- [S02](S02.json): route statis alias hilang di R1; ICMP 3/0 kembali ke PC1. Quote memuat 28 byte request asal.
- [S03](S03.json): TTL 1 berhenti di R1; ICMP 11/0 kembali ke PC1. Request tidak diteruskan.
- [S04](S04.json): request ARP R1:eth1 pada 34.800, 1.000.034.800, dan 2.000.034.800 ns. Timeout di R1 pada 3.000.034.800 ns, buffer pending dilepas.
- [S05](S05.json): byte `4500` tetap tersimpan, reason truncated_ipv4; tidak ada event frame atau pass palsu.

Seluruh run menggunakan seed 20260117 tanpa jitter/loss. Capture Linux pembanding dan replay dari checkpoint belum tersedia.
