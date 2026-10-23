# Roadmap Wirebox: pembuktian, MVP, pilot, dan pengembangan

Diperbarui 21 Oktober 2026. Ini rencana kerja bertahap atas permintaan pengguna. Checklist eksekusi ada pada [todos](todos.md); skenario dan ukuran keberhasilan ada pada [rencana pilot](pilot.md). Roadmap menjelaskan urutan, todos mencatat pekerjaan, dan pilot menentukan bukti keberhasilan.

## Hasil yang ingin dibuktikan

Wirebox harus menunjukkan hubungan yang dapat diperiksa dari **objek infrastruktur → intent → plan → perubahan state → keputusan protokol/byte → verifikasi hasil**.

| Milestone | Selesai setelah | Klaim yang boleh dibuat |
|---|---|---|
| PoC teknis | Phase 3 | Kernel mandiri memproses skenario dan menjelaskan paket hingga byte |
| MVP Studio | Phase 4 | Pengguna membuka skenario, menjalankan, mendiagnosis, mengubah, dan membandingkan hasil melalui aplikasi |
| Pilot inti Wirebox | Phase 7 | Satu model operasi diuji di simulator, dijalankan pada Linux/MikroTik lab, dan diverifikasi kembali |
| Pilot domain ISP | Phase 8 | Satu workflow pelanggan terbukti pada ZTE dengan ONU Huawei/VSOL yang telah diuji |

Satu ping sukses memenuhi sebagian PoC. Pembuktian gagasan awal yang menghubungkan simulasi dengan realitas memerlukan pilot inti; workflow ISP memerlukan pilot domain ISP. Setiap klaim memiliki cakupan yang dinyatakan.

## Posisi sekarang

G0/G1 lulus untuk development lokal Windows x64 debug: [laporan gate](../evidence/phase-0-1/README.md). [G2](../evidence/phase-2/README.md) membuktikan graph/scheduler/timer/PRNG. [G3](../evidence/phase-3/README.md) membuktikan PoC Ethernet/ARP/IPv4/ICMP dan tracker byte headless dalam subset yang dinyatakan. Pekerjaan berikutnya phase 4: menghubungkan simulator dengan GPUI dan workflow MVP. Perbandingan Linux, driver, serta pilot nyata belum dijalankan.

Target pilot pengguna: **Linux VM/server, MikroTik RouterOS 7, OLT ZTE C320, ONU Huawei, dan ONU VSOL**. Versi board C320 serta contoh state GPON sudah tercatat sebagai user-reported pada [inventaris](../../fixtures/inventory/pilot-v1.json). Model/firmware ONU, versi minor RouterOS, distro/kernel Linux, akses, serta pasangan kompatibel yang belum diketahui tetap menjadi dependency G6/G8.

## Urutan phase

| Phase | Fokus dan hasil | Dependensi | Syarat keluar / gate |
|---|---|---|---|
| 0 | Kontrak keberhasilan, fixture, inventaris lab, dan batas MVP | Fondasi docs/toolchain tersedia sebagian | G0: skenario utama, matriks uji, dan batas dukungan tertulis |
| 1 | Build Zig/Rust, percobaan FFI, dan window GPUI Windows | G0 | G1: checkout baru dapat dibangun; Rust memakai hasil Zig; window dan shutdown bekerja |
| 2 | Graph, state, waktu virtual, timer, dan jurnal | G1 | G2: urutan stabil, cancellation, dan batas eksekusi lolos uji |
| 3 | Ethernet/ARP/IPv4/ICMP, routing statis, dan trace byte | G2 | G3: PoC headless lulus kasus sukses serta kegagalan yang didefinisikan |
| 4 | Studio: topology, timeline, inspector, save/load | G3 dan percobaan UI G1 | G4: MVP menyelesaikan alur diagnosis–perubahan–perbandingan dari UI |
| 5 | Checkpoint/replay, gangguan terkontrol, validasi terhadap lab | G4 | G5: restore/replay sesuai state dan hasil pembanding dalam scope |
| 6 | Observasi Linux/MikroTik dan snapshot ke simulator | G5 | G6: provenance, unknown/stale, dan pemetaan snapshot terbukti |
| 7 | Intent, dua executor, apply dan read-back Linux/MikroTik | G6 | G7: pilot inti lulus scorecard dan skenario kegagalan |
| 8 | Domain ISP: ZTE → ONU Huawei/VSOL → layanan pelanggan | G7 dan inventaris perangkat | G8: pilot ISP lulus pada kombinasi model/firmware yang diuji |
| 9 | Beta yang dapat diulang dan dioperasikan pengguna lain | G8 untuk beta ISP; G7 cukup untuk beta inti | G9: instalasi, recovery, dokumentasi, dan penggunaan berulang terbukti |
| 10 | Pengembangan berdasarkan bukti penggunaan | G7/G8/G9 sesuai jalur | Gate per kemampuan; pilih fitur berdasarkan kebutuhan terukur |

Inventaris perangkat dan contoh observation phase 6/8 boleh dikumpulkan sejak phase 0. Percobaan GPUI/FFI dilakukan lebih awal agar risiko stack diketahui sebelum engine membesar. Implementasi fitur mengikuti dependensinya.

## Phase 0–3: buktikan fondasi

Mulai dari `PC1 — SW1 — R1 — PC2` dengan format skenario Wirebox sendiri. Tetapkan semantics waktu, ID, ownership buffer, operation, dan batas trace. Pilih referensi pembanding yang independen dari implementasi baru.

PoC menjelaskan ARP, MAC learning/flooding, route lookup, TTL/checksum, dan drop yang relevan. Byte asli, field mapping, versi paket, serta hubungan sebab menjadi bagian implementasi awal. Menempelkan nama langkah PT pada log belum memenuhi gate ini.

Zig minimal 0.17.0 dan Rust 1.99.1 mengikuti keputusan pengguna. GPUI langsung adalah kandidat utama percobaan UI sesuai [arah teknologi](technology.md). Jika percobaan tidak memenuhi kebutuhan, catat kendalanya dan evaluasi alternatif berdasarkan bukti.

## Phase 4: MVP Studio

MVP adalah aplikasi Windows untuk satu workflow lengkap: buka skenario → kirim ping → temukan penyebab gagal → periksa byte dan state → ubah route pada salinan skenario → jalankan ulang → bandingkan → simpan hasil.

Wajib: empat node awal, Ethernet/ARP/IPv4/ICMP, routing statis, kontrol eksekusi berbatas, trace kausal, inspector field/byte, save/load berversi, dan penanda unsupported. Form perubahan sederhana cukup; editor topologi lengkap bukan prasyarat.

MVP belum memerlukan driver nyata, impor `.pkt`, TCP lengkap, AI, emulasi firmware, atau time travel dari checkpoint arbitrer. Mengubah skenario lalu menjalankan ulang dari awal cukup pada MVP; restore state aktif diuji pada phase 5.

## Phase 5–7: buktikan gagasan inti

Pastikan engine dapat direproduksi dan dibandingkan dengan lab pada perilaku yang didukung. Masukkan observation Linux/MikroTik ke model bersama, fork snapshot menjadi simulasi, dan nyatakan asumsi state yang tidak tersedia.

Intent awal dibatasi pada reachability IPv4 statis: sumber, tujuan uji, arah, dan protokol dinyatakan. Plan menghasilkan operation seperti `EnsureStaticRoute` dengan pre/postcondition yang sama secara semantik. Simulator dan adapter nyata menerapkannya melalui backend masing-masing. Nama/schema operation masih rancangan.

Pilot memperagakan kegagalan awal, diagnosis, perubahan simulasi, plan yang dapat ditinjau, eksekusi di lab, lalu observation dan probe ulang. Uji stale plan, operasi berulang, koneksi manajemen terputus, serta kegagalan parsial. Hasil indeterminate memerlukan pemeriksaan state sebelum retry.

**G7 adalah pembuktian awal ide Wirebox dalam cakupan terbatas.** Ping pada perangkat saja belum membuktikan hubungan semantik antara plan simulasi dan operasi nyata. Rincian ada pada [rencana pilot](pilot.md).

## Phase 8: buktikan domain ISP pengguna

Mulai dengan observasi ZTE, PON/ONU, status layanan, VLAN, dan sinyal yang tersedia dari perangkat. Uji ONU Huawei dan VSOL secara terpisah; kesamaan fungsi tidak menjamin field dan operasi identik.

Workflow awal: satu layanan pelanggan uji pada VLAN dan jalur IP terkontrol menuju endpoint di sisi uplink. Pilih operasi setelah capability diketahui. Jangan menjadikan PPPoE, NAT, GEM/TCONT, atau pengubahan WAN ONU sebagai asumsi tersembunyi; modelkan dan uji yang benar-benar diperlukan oleh layanan terpilih.

Target “internet pelanggan dapat diakses” dari visi tetap dipertahankan. Pengujian dimulai pada endpoint terkontrol agar dependency dapat diamati, lalu diperluas ke tujuan eksternal dengan dependency tambahan yang dicatat. State optik awal adalah observation; simulasi fisika PON memerlukan model serta validasi tersendiri.

## Phase 9–10: beta dan pengembangan berikutnya

Beta membuktikan workflow dapat diulang pada lingkungan terdokumentasi, termasuk install, credential handling, retensi, recovery, dan diagnosis kegagalan. Tetapkan kapasitas berdasarkan pengukuran dan terbitkan batas dukungan model/firmware yang diuji.

Jalur pengembangan: fidelitas protokol, skala/performa, domain/vendor, otomasi intent, kolaborasi, dan interoperabilitas. Tiap jalur memerlukan kebutuhan pengguna, kontrak, fixture pembanding, serta gate sendiri. Calon epic tersedia pada [todos](todos.md).

## Cara menjalankan rencana

Gunakan satu phase utama sebagai pekerjaan aktif. Tandai task selesai setelah artifact/bukti tersedia; selesainya implementasi tidak otomatis meluluskan gate. Catat hasil gate sebagai belum diuji, lulus, gagal, atau tertahan beserta sebabnya.

Prioritas **Now**: phase 0–4. **Next**: phase 5–7. **Later**: phase 8–9. **Future**: phase 10. Ini urutan fokus, bukan janji tanggal. Estimasi kalender dibuat setelah kecepatan kerja dan scope pilot diketahui.

Jika gate gagal, perbaiki atau persempit klaim/cakupan secara eksplisit sebelum lanjut. Temuan PT tetap mengacu pada [evidence ledger](../packet-tracer/evidence-ledger.md), bukan pengganti uji Wirebox.
