# Simulasi paket dan byte tracker headless — phase 3

Model `ethernet-arp-ipv4-icmp-v1` menjalankan `PC1 — SW1 — R1 — PC2` seluruhnya di Zig. Rust memuat fixture, memanggil C ABI, memeriksa hasil, dan mengekspor JSON. Tidak diperlukan perangkat fisik, image OS, atau file `.pkt`. Kontrak: [ADR-0007](../adr/0007-packet-model.md); hasil dan batas pembuktian: [G3](../evidence/phase-3/README.md).

## Menjalankan dan membaca hasil

```powershell
cargo run --locked -p wirebox-headless -- --packet-demo S01
cargo run --locked -p wirebox-headless -- --packet-demo S02
cargo run --locked -p wirebox-headless -- --check-network
```

Demo menerima `S01`–`S05` dan `alias`; tanpa nama memakai S01. Output berisi nama→ID entity, summary, trace keputusan, semua versi byte beserta field, dan transmisi. Contoh lengkap tersedia pada [artifact S01](../evidence/phase-3/S01.json) dan [walkthrough byte](../evidence/phase-3/byte-walkthrough.md). GPUI saat ini masih spike probe phase 1; integrasi timeline/field/hex dan kontrol simulasi adalah phase 4.

| Kasus | Perilaku yang diperiksa |
|---|---|
| S01 | Echo reply dari PC2; request/reply melewati switch dan router |
| S02 | Route statis alias di R1 dihapus; ICMP 3/0 diterima PC1 |
| S03 | TTL request 1; ICMP 11/0 diterima PC1, request tidak diteruskan |
| S04 | Port PC2 disabled; tiga ARP request, timeout, pending buffer dibebaskan |
| S05 | Input IPv4 `4500`; invalid_packet dengan alasan truncated_ipv4 |
| alias | Route statis sehat ke 198.51.100.2/32 dipilih dan echo reply diterima |

Summary `now_ns` adalah horizon yang diminta (5 detik pada demo), bukan latency ping. Waktu penerimaan aktual ada pada trace `delivered`. Jumlah row trace berbeda dari jumlah event scheduler; satu event dapat menghasilkan beberapa keputusan dan versi paket.

## Engine dan cakupan model

[network.zig](../../kernel/src/network.zig) memakai graph, heap `(time_ns, event_id)`, serta registry timer [runtime](runtime.md). Event 100 membawa ID transmisi untuk penerimaan frame; event 101 membawa ID pending ARP untuk retry/timeout. Pembuatan workload awal sinkron pada waktu 0 dengan `event_id=0`; event turunan menunjuk ID event yang menjadwalkannya sebagai cause.

Satu Network dikonfigurasi sebelum satu `ping` atau `inject`. Setelah workload dimulai, graph/address/route/link dibekukan; skenario perubahan dibuat sebagai Network baru. `run_until(horizon, budget)` bisa dipanggil berulang, termasuk budget 1 untuk stepping. Pembacaan output tidak mengubah eksekusi. Belum ada checkpoint/restore, mutation konfigurasi saat run, atau worker thread.

| Bagian | Implementasi dan batas |
|---|---|
| Ethernet II | Decode/encode eksplisit; MAC learning per switch; flood broadcast/unknown destination, forward unicast yang sudah dikenal. Tanpa VLAN, STP, aging, atau collision |
| Link | Full duplex, antrean serialisasi per egress port; arrival = start + ceil(bytes × 8 × 1e9 / bitrate) + propagation. Start menunggu egress tersedia. Tanpa jitter, loss terkontrol, congestion/queue drop |
| ARP | Ethernet/IPv4 request/reply; cache per port; paket menunggu resolution. Request pada 0/1/2 detik relatif lookup, timeout pada 3 detik; timer dibatalkan jika resolved. Tanpa aging/gratuitous ARP |
| IPv4 | IHL 5, MTU 1500, connected/static route longest-prefix match. Equal prefix memilih connected route lebih dulu; duplicate static prefix ditolak, tanpa ECMP. Forwarding menurunkan TTL dan menghitung ulang checksum |
| ICMP | Echo request/reply code 0, identifier/sequence/data dipertahankan; error network unreachable 3/0 dan time exceeded 11/0 dengan quote header + 8 byte awal. Suppression error untuk error/broadcast/invalid source |
| Address | IPv4 unicast biasa; satu alamat per port ditambah alias local host. Model ini tidak memvalidasi seluruh aturan konfigurasi subnet OS. Multicast/broadcast IP, loopback dan source tak didukung diberi alasan eksplisit |

Frame hasil encoder minimal 60 byte, termasuk zero padding. Byte dimulai di destination MAC, tidak termasuk preamble, SFD, FCS, atau IFG; delay dihitung atas byte yang direpresentasikan. Decoder memeriksa batas buffer dan field protokol, bukan seluruh aturan penerima Ethernet fisik/runt. Sebuah switch hanya memproses header Ethernet; validasi IP/ICMP dilakukan oleh endpoint L3.

[codec.zig](../../kernel/src/codec.zig) membedakan truncated/invalid/checksum error dari unsupported options, fragment, VLAN, MTU, protocol, dan ICMP type/code. Tidak ada TCP/UDP, NAT, firewall, routing dinamis, IPv6, atau perilaku vendor. Capture Linux sebagai pembanding belum dijalankan; itu G5.

## Byte, versi, dan hubungan sebab

- `packet_id`: identitas datagram/frame logis. Encapsulation, decapsulation, dan perubahan TTL mempertahankannya. ARP, echo reply, dan ICMP error memperoleh identitas baru.
- `version_id`: salinan byte immutable. `parent_version` menunjuk versi pemicu; reply/error juga menghubungkan paket baru dengan request asal.
- `transmission.id`: satu pengiriman pada satu link. Flooding dapat memakai versi byte yang sama dalam beberapa transmisi dengan ID berbeda.
- `event_id`/`cause_id`: urutan eksekusi dan sebab penjadwalan. `ordinal` adalah posisi row dalam jurnal domain, bukan ID event.
- `prior`/`version`: versi sebelum/sesudah keputusan atau perubahan, jika relevan. Route/cache/drop tetap dicatat meskipun tidak membuat byte baru.

Field view menyimpan `byte_offset`, `bit_offset_msb`, `bit_width`, dan nilai numerik big-endian. Bit 0 berarti bit paling signifikan pada byte. Payload/quote/padding lebih dari 64 bit memakai `value: null` di JSON; byte lengkap tetap tersedia melalui `hex`. Field <=64 bit diperiksa kembali terhadap bit asli oleh oracle Rust. Paket invalid/unsupported tetap menyimpan raw bytes; API field mengembalikan error tanpa partial view (JSON demo memakai array field kosong).

Trace mencatat route terpilih, next-hop, cache lookup/hit/learn, ARP request/reply, buffering, switch learning/flood/forward, transmit/receive, decapsulation, TTL, ICMP, delivery, drop, dan timeout. `detail_a/detail_b` ditentukan oleh kind: route = egress/next-hop, MAC learn = MAC/port, TTL = before/after, ICMP = type/code, timeout = next-hop/attempts. Kode engine menjadi definisi lengkap kind/reason; wrapper menyediakan nama yang terbaca.

## Ownership dan C ABI network v1

[wirebox_net.h](../../kernel/include/wirebox_net.h) dan [net_abi.zig](../../kernel/src/net_abi.zig) mendefinisikan `wb_net_*`. [Wrapper Rust](../../crates/wirebox-kernel/src/network.rs) memiliki `Network`, menyalin semua input/output dan menghancurkan alokasi Zig melalui Zig. `close`/Drop hanya menutup sekali; `Network` adalah `!Send + !Sync`. Mutex host yang sama menserialisasi probe, counter runtime, dan network. C caller wajib menegakkan serialisasi process-wide serta validitas/alignment/panjang/non-overlap pointer sendiri.

Handle opaque mempunyai namespace dua bit atas: probe `00`, counter `10`, network `11`; 30 bit generation dan 32 bit index+1. Slot dipensiunkan ketika generation habis. Ini menggantikan pembagian bit internal phase 2; handle hanya valid selama proses/owner hidup, tidak boleh disimpan sebagai format persistence.

| API | Kontrak |
|---|---|
| create/destroy | Version 1, seed, retention; maksimum 4 Network bersamaan |
| add/port/link/alias/route | Konfigurasi sebelum workload; IP/MAC berupa angka, bukan native wire struct |
| ping/inject | Satu workload; inject layer 1=IPv4 atau 2=Ethernet, copy sampai 1600 byte |
| run | Horizon/budget, copy progress; queue setelah horizon tetap pending |
| summary/packet/tx | Salinan metadata/byte/transmisi. Packet/tx dicari dengan ID |
| trace | Baca non-consuming setelah ordinal, batch <=256; retention 0..1024 |
| fields | <=40 field, seluruh view atau error capacity/invalid_packet/unsupported |

Status 0..8 mengikuti [runtime ABI](runtime.md). Tambahan: 9 state, 10 allocation, 11 invalid_packet, 12 unsupported. Protocol rejection pada `inject` bukan ABI error: periksa summary/trace. `failed=1` berarti kegagalan engine/resource dan mengalahkan nilai outcome apa pun. Outcome: 0 pending, 1 echo_reply, 2 unreachable, 3 time_exceeded, 4 arp_timeout, 5 invalid_packet, 6 unsupported, 7 dropped. Unreachable dapat berasal dari kegagalan route lokal; untuk membuktikan ICMP diterima, periksa `delivered` dan byte seperti acceptance S02/S03.

Kegagalan dispatch dapat terjadi setelah event dikeluarkan dari heap dan sebagian state berubah. Network kemudian failed dan menolak kelanjutan; output masih dapat dibaca. `RunError.progress.processed` menghitung dispatch sukses pada call itu; `summary.executed` juga mencakup event yang dispatch-nya sudah dimulai lalu gagal. Ini berbeda dari atomic counter firing pada runtime phase 2. Input konfigurasi invalid ditolak sebelum workload; `past_time` tidak mematikan Network.

Layout Windows x64 alignment 8: Packet 1640 byte (payload pada offset 40), Trace 96, Transmission 72, Summary 80, Field 24, Progress 24. C/Zig/Rust menguji layout. Struct tersebut adalah transport in-process, bukan format paket atau format file yang dijanjikan stabil.

Kapasitas tiap Network: 128 entity, 32 static route, 16 alias, 64 entry ARP dan 64 MAC, 16 pending ARP dengan 8 paket masing-masing, 128 versi byte, 256 transmisi, 256 pending event/timer. Storage packet tidak overwrite sejarah. Penuh menghasilkan error terminal ketika workload sudah dimulai. Hanya jurnal yang memakai ring; `trace_dropped` dan gap ordinal menunjukkan retensi. Retensi jurnal 0/1/16/1024 terbukti tidak mengubah byte, transmisi, clock, atau hasil.

## Batas pembuktian

[Acceptance runner](../../crates/wirebox-headless/src/packet_demo.rs) membaca fixture beku dan memeriksa header golden sebelum/sesudah, checksum Rust terpisah, field/bit, ICMP quote/correlation, delay, serialization, ancestry, serta ARP timing. Tiap kasus dibandingkan antara batch, budget satu event, repeat, dan beberapa retensi. Zig menambah uji flooding multiport, route LPM, payload ganjil, suppression ICMP, dan batas ABI/resource.

Hasil ini membuktikan PoC mandiri dalam subset di atas. State cache belum diekspor sebagai snapshot lengkap; trace keputusan bukan audit semua state. Persistence/replay, integrasi Studio, perbandingan OS nyata, dan pilot lintas vendor tetap phase selanjutnya.
