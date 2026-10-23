# Bukti G3 — PoC paket dan byte tracker headless

**Lulus lokal, 21 Oktober 2026**, untuk subset `ethernet-arp-ipv4-icmp-v1` pada Windows x64 debug. Simulator Zig memproses byte, switching, ARP, route, TTL/checksum, ICMP, dan timer melalui C ABI ke Rust. S01–S05 memenuhi [kontrak phase 0](../../wirebox/phase-0-contract.md); healthy static alias diuji tambahan. Ini membuktikan PoC mandiri, belum MVP Studio atau pilot perangkat nyata.

## Hasil yang direkam

| Artifact | Outcome / reason | Event scheduler | Versi byte | Transmisi | Row trace |
|---|---|---:|---:|---:|---:|
| [S01](S01.json) | echo_reply | 12 | 16 | 12 | 63 |
| [S02](S02.json) | unreachable / no_route | 8 | 8 | 8 | 40 |
| [S03](S03.json) | time_exceeded / ttl_expired | 8 | 8 | 8 | 40 |
| [S04](S04.json) | arp_timeout | 12 | 9 | 9 | 44 |
| [S05](S05.json) | invalid_packet / truncated_ipv4 | 0 | 1 | 0 | 2 |
| [Static alias sehat](alias.json) | echo_reply | 12 | 16 | 12 | 63 |

Semua pending event/ARP nol pada akhir horizon 5 detik. S01 menerima reply pada 69.600 ns. Header request di R1 cocok dengan golden: TTL 64→63 dan checksum 0x65dd→0x66dd, ICMP tidak berubah. [Walkthrough byte](byte-walkthrough.md) menautkan versi/event/transmisi yang benar-benar ada dalam artifact.

[acceptance.json](acceptance.json) mencatat **36 scenario run**: untuk masing-masing enam kasus, baseline batch, budget satu event, repeat batch, dan retensi 0/1/16 dibandingkan dengan baseline retensi 1024. Byte/field/transmisi/hasil identik; jurnal terbatas adalah suffix dengan dropped count. Ini fresh-run determinism, bukan checkpoint replay.

**37 input negatif** melalui ABI: 28 prefix terpotong, checksum IP/ICMP rusak, fragment, protokol non-ICMP, options, VLAN, source MAC multicast, ICMP type tak didukung, dan MTU berlebih. Tidak menghasilkan event terjadwal atau status sukses. Zig juga menguji batas setiap decoder, flooding multiport berbagi byte, route LPM, payload ganjil, suppression ICMP error untuk error/broadcast, serta failed-state pada resource exhaustion.

Oracle Rust memakai header golden beku dan implementasi checksum terpisah; memeriksa field/bit terhadap raw bytes, delay dan antrean serialisasi, korelasi event/transmisi, ancestry versi, ICMP quote, reply, MAC rewrite, dan waktu ARP. Fixture sintetis bukan capture OS dan oracle ini tidak membuktikan seluruh RFC.

## Command dan verifikasi

```powershell
./scripts/verify.ps1
./scripts/verify-clean-export.ps1
./target/debug/wirebox-headless.exe --check-network
./target/debug/wirebox-headless.exe --packet-demo S01
```

- [verification.log](verification.log): Rust/Zig format, layout C, **24 hasil tes Zig** (termasuk satu import aggregator), **16 unit test Rust** dan **3 compile-fail doctest** lulus; Clippy seluruh target dengan `-D warnings`; probe, fixture, acceptance runtime/network; build GPUI lulus.
- [clean-export.log](clean-export.log): build workspace offline dari salinan source baru dan probe/fixture/runtime/network lulus. Cache dependency/compiler Cargo dipakai ulang; bukan cold-cache build atau remote CI.
- [environment.json](environment.json): Zig 0.17.0, Rust 1.99.1, OS/build, model, seed, serta pemeriksaan yang tidak dilakukan.
- [source-manifest.json](source-manifest.json): SHA-256 source, fixture, docs aktual, dan artifact G3. Repository belum mempunyai HEAD; hasil diikat ke hash file, bukan commit. Manifest G0/G1/G2 dan arsip PT dipertahankan sebagai baseline historis.
- [integrity.log](integrity.log): tautan/fence Markdown, kebersihan whitespace source, hash arsip PT, artifact gate lama, dan manifest baru diperiksa setelah dokumentasi selesai.

Build GPUI tetap memunculkan pemberitahuan future incompatibility dari dependency transitif `proc-macro-error2 2.0.1`. Build, tests, dan Clippy lulus pada compiler yang dipin; warning ini belum diperbaiki dengan upgrade dependency.

## Batas dan kelanjutan

Kontrak lengkap ada pada [model paket/ABI](../../wirebox/packets.md) dan [ADR-0007](../../adr/0007-packet-model.md). Network/config dibekukan saat satu workload dimulai; kapasitas byte/tx/cache bounded dan resource exhaustion menjadi failed, bukan overwrite sejarah atau sukses. Semua versi byte disalin; belum checkpoint/restore atau snapshot cache lengkap.

Tidak ada VLAN, TCP/UDP, IPv6, NAT/firewall, routing dinamis, aging, STP, jitter/loss, physical optics, atau emulasi vendor. Timing mencakup byte MAC hingga padding tanpa FCS/preamble/IFG. Hasil tidak menyatakan kesetaraan Packet Tracer, firmware, ASIC, atau perangkat nyata.

UI interaction phase 1 tidak diulang dan UI masih probe; pekerjaan berikutnya **phase 4, integrasi simulator ke GPUI**. Pembanding Linux capture tetap G5, adapter Linux/MikroTik G6–G7, dan ISP ZTE/ONU G8. Tidak ada login atau perubahan perangkat; remote CI dan distribusi release belum dijalankan.
