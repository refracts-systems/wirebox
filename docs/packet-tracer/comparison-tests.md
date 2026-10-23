# Runbook uji pembanding Wirebox–Packet Tracer

Baseline disusun 21 Oktober 2026, sebelum run differential. [Audit awal](../evidence/pt-comparison/README.md) menyediakan identitas binary, audit Ghidra, smoke PT, dan acceptance Wirebox. **Belum ada verdict differential.**

## Kontrak perbandingan

Bekukan versi/hash engine dan PT, file topologi/hash, device model, konfigurasi interface/route, initial cache, TTL, payload, serta mode delay/jitter. Mulai setiap kasus dari salinan kondisi awal; jangan membiarkan ARP/MAC hangat mengubah expected sequence kasus cold-start.

Bandingkan keputusan dan invariant yang sama: tujuan lookup, egress, next-hop, delivery/drop, TTL request sebelum/sesudah router, checksum valid, ICMP type/code dan quote. Map identitas MAC/device secara eksplisit bila PT tidak membolehkan MAC yang sama. Checksum numerik hanya harus identik bila seluruh header yang dicakup juga identik; bila tidak, validasi checksum terhadap byte masing-masing serta perubahan yang disebabkan TTL. Jangan menghapus TTL, next-hop, payload, atau error code sebagai normalisasi.

Timestamp, initial reply TTL, ARP retry policy, padding dan overhead link harus dibandingkan terpisah dari hasil semantik. Model v1 Wirebox hanya menghitung frame MAC tanpa FCS/preamble/IFG. Jumlah row Event List tidak dibandingkan dengan jumlah scheduler pop Wirebox.

## Kasus minimum dan akses input

| ID | Setup dan observasi wajib | Catatan akses PT |
|---|---|---|
| S01 | PC1–SW1–R1–PC2, dua subnet; echo reply; request TTL turun satu, L2 rewrite, checksum benar, payload/ICMP request tetap | Atur initial TTL 64 melalui fasilitas yang benar-benar tersedia; jika tidak dapat diatur, bandingkan delta TTL dan catat input berbeda, tanpa mengklaim byte-identical |
| S02 | Destinasi alias 198.51.100.2; hapus route /32 di R1; ICMP type 3/code 0 dan tanpa forward sembarang; restore route sebagai kontrol sehat | PC-PT belum dibuktikan mendukung alias local. Gunakan endpoint yang mendukung loopback dengan pemetaan peran yang tertulis, atau tandai kontrol alias belum representable. Jangan mengganti penghapusan route statis dengan menghapus connected route |
| S03 | Request TTL=1, tidak diteruskan R1, ICMP type 11/code 0 dengan quote benar | Harus ada bukti TTL input, bukan hanya pesan timeout pada aplikasi |
| S04 | Interface egress R1 tetap up; next-hop yang diminta tidak menjawab; rekam ARP retry, interval, timeout dan pelepasan buffer | Mematikan NIC PC2 bisa menjatuhkan link dan connected route, sehingga bukan uji ARP timeout yang setara. Dapat gunakan next-hop kosong di subnet hidup untuk subkasus ARP, dengan revisi fixture terdokumentasi. Kebijakan tiga retry Wirebox bukan default PT yang sudah diketahui |
| S05 | Injeksi IPv4 terpotong `4500`, checksum salah, fragment/options/VLAN unsupported; tidak crash atau sukses palsu | Simple/Complex PDU GUI belum terbukti mengizinkan raw malformed bytes. Jika metode input tidak tersedia, catat `not_testable_via_gui`; jangan menganggap paket valid yang dibentuk GUI sebagai uji negatif |

Untuk tiap kasus simpan konfigurasi sebelum run, screenshot/ekspor PDU ingress/egress, raw bytes jika tersedia, Event List yang relevan, route/ARP/MAC state, hasil aplikasi, dan tiga pengulangan kondisi awal yang sama. Catat juga field yang tidak dapat diamati. Label protokol di GUI tanpa byte tidak cukup untuk lulus uji byte.

## Verdict dan cakupan klaim

Setiap assertion mendapat `match`, `mismatch`, `not_observed`, atau `not_testable`. Status kasus hanya `match` bila semua assertion wajib teramati dan cocok. Assertion yang belum teramati menahan verdict; ia tidak dianggap match. Jika PT dan Wirebox berbeda, periksa spesifikasi protokol dan pembanding Linux sebelum menetapkan pihak yang salah. Perilaku PT bukan satu-satunya oracle kebenaran.

Pisahkan gate berikut:

1. **Kebenaran subset:** fixture/oracle independen, differential PT, lalu Linux capture untuk field serta keputusan yang dimodelkan.
2. **Ketahanan:** fuzz decoder/ABI, input malformed, batas allocator/buffer, timer cancellation, cache penuh, horizon run dan determinisme. Jangan mengirim input malformed ke perangkat produksi sebagai bagian uji ini.
3. **Keunggulan tracker:** pengguna dapat menunjuk byte TTL/checksum/L2 yang berubah beserta penyebabnya; hasil dapat diekspor dan diulang. Bandingkan tugas berpasangan dan kebenaran jawabannya, bukan jumlah panel UI.
4. **Keunggulan performa/workflow:** workload setara, cold/warm cache terpisah, median/p95 respons, waktu diagnosis, bantuan, memory dan crash/hang. Bekukan workload sebelum pengukuran; jangan memakai waktu animasi GUI sebagai kecepatan engine.
5. **Kesetaraan vendor:** model/firmware tertentu, state read-back dan capture lab. PT maupun hasil statis Ghidra tidak menggantikan pengujian Linux/RouterOS/ZTE/ONU.

Satu ping berhasil meluluskan smoke test. Klaim yang lebih luas mengikuti cakupan assertion yang benar-benar lulus.
