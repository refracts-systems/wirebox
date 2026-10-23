# Apa itu Wirebox?

Wirebox adalah rancangan **studio dan runtime infrastruktur berbasis model semantik bersama**. Engineer dapat melihat objek infrastruktur, memeriksa state dan paketnya, mencoba perubahan di simulasi, lalu merencanakan operasi pada perangkat nyata melalui adapter.

Kata *semantik* berarti model menyatakan makna: interface, route, VLAN, layanan, dependency, dan kemampuan perangkat. Perintah CLI atau protokol manajemen menjadi cara membaca atau mengubah objek tersebut.

Dokumen ini merangkum pembicaraan sampai 21 Oktober 2026. Fondasi build, probe Zig/Rust, preview GPUI, runtime graph/waktu virtual, serta [simulasi paket headless](packets.md) sudah tersedia. Ethernet/ARP/IPv4/ICMP, routing statis, dan versi byte diuji pada [G3](../evidence/phase-3/README.md). Integrasi Studio dan adapter perangkat nyata masih mengikuti roadmap. Bukti sebelumnya: [phase 0/1](../evidence/phase-0-1/README.md) dan [phase 2](../evidence/phase-2/README.md).

Wirebox tidak membuat ulang OS. Istilah kernel berarti inti engine simulasi; Linux, RouterOS, dan firmware perangkat tetap berjalan sebagai sistem masing-masing pada lab nyata.

## Masalah yang ingin diselesaikan

Satu jaringan muncul dalam banyak bentuk: diagram, CLI vendor, monitoring, konfigurasi, capture paket, dan catatan insiden. Hubungan antara perubahan konfigurasi, perilaku jaringan, dan byte yang lewat sering harus disusun ulang secara manual.

Wirebox ingin mempertahankan hubungan itu dalam satu model:

```text
Infrastruktur = graph + state + behavior + constraints + time
```

Contohnya, ketika route berubah, pengguna dapat melihat objek yang berubah, alasan pemilihan next-hop, paket yang terdampak, dan hasil simulasi sebelum menyiapkan perubahan nyata.

## Pengalaman produk yang dituju

| Mode | Pertanyaan pengguna | Hasil yang dituju |
|---|---|---|
| Inspect | Apa yang ada, bagaimana kondisinya, dan dari mana informasinya? | Topologi, state, observation, dependency, dan provenance |
| Simulate | Apa yang terjadi jika route, link, atau konfigurasi berubah? | Cabang simulasi, trace keputusan, byte paket, dan hasil sesuai cakupan model |
| Execute | Operasi apa yang diperlukan dan apakah hasilnya sesuai? | Plan, operasi adapter, catatan eksekusi, dan verifikasi ulang |

Alur yang dibayangkan: **record → replay → modify → simulate → apply → verify**. Replay dari data produksi hanya dapat memulihkan hal yang tercatat; simulasi lanjutan memakai asumsi model yang harus terlihat.

Inspirasi yang dibahas: interaksi perangkat seperti Winbox, topologi dan simulasi seperti Packet Tracer, inspeksi protokol seperti Wireshark, intent deklaratif seperti Terraform, serta langkah eksekusi dan inspeksi state seperti debugger. Ini analogi pengalaman, bukan daftar fitur yang sudah diimplementasikan.

## Satu fondasi, beberapa domain

Router, switch, firewall, Linux host, server, OLT, dan ONU adalah calon domain awal atau lanjutan. PLC, UPS, gateway IoT, radio, dan storage merupakan perluasan dalam visi awal, dengan model perilaku masing-masing.

OLT adalah satu domain yang kaya untuk menguji model: PON, ONU, VLAN, GEM, TCONT, alarm, dan kondisi optik. Dukungan field optik tidak dengan sendirinya menyediakan simulasi fisika optik.

UI yang dibahas meliputi topology canvas, object inspector, byte/packet inspector, event timeline, state diff, dan operation planner. Layout UI hanya menyimpan posisi tampilan; koneksi dan dependency operasional berada pada model domain.

## Mengapa idenya masuk akal?

Bagian penyusunnya dapat dibuat: model objek, scheduler event, codec paket, state machine protokol, adapter perangkat, dan UI inspeksi. [Mini-simulator eksternal](../packet-tracer/mini-simulator.md) sudah memperagakan skenario jaringan terbatas tanpa hardware. [Proyek pembanding](positioning.md) menunjukkan bahwa model jaringan, intent, dan validasi sudah menjadi pendekatan nyata.

Tantangan utamanya adalah menjaga makna yang konsisten antara model, simulator, dan vendor. Kesetaraan tidak otomatis muncul karena semuanya memakai nama operasi yang sama. Setiap kemampuan perlu kontrak dan cakupan validasi.

Pembeda yang ingin diuji adalah hubungan yang dapat ditelusuri dari **intent → state → keputusan protokol → byte → hasil**, dalam satu ruang kerja. Nilai pembeda itu masih hipotesis produk sampai dibuktikan lewat penggunaan.

## Cakupan awal

Simulasi PC1–SW1–R1–PC2 sudah berjalan headless: Ethernet, ARP, IPv4, ICMP, waktu virtual, dan inspeksi perubahan byte. Langkah berikutnya menghubungkan hasil tersebut dengan Studio, kemudian replay, validasi lab, serta adapter secara bertahap.

Emulasi seluruh chipset, kesetaraan semua firmware, pengelolaan semua vendor, simulasi upgrade firmware, dan rollback universal belum menjadi janji produk. Keberhasilan impor `.pkt` juga bukan prasyarat engine mandiri.

## Peta bacaan

- [Arsitektur](architecture.md): model bersama dan hubungan dengan dunia nyata.
- [Simulasi](simulation.md) serta [byte tracker](byte-tracker.md): cara membangun dan menjelaskan perilaku.
- [Teknologi](technology.md): pertimbangan Zig/Rust dan GPUI/GPUix.
- [Roadmap](roadmap.md) serta [keputusan](decisions.md): pekerjaan yang diusulkan dan hal yang belum dipilih.
- [Todos per phase](todos.md) dan [rencana pilot](pilot.md): checklist, skenario, serta bukti sukses PoC, MVP, pilot inti Linux/MikroTik, dan pilot ISP ZTE dengan ONU Huawei/VSOL.

Sumber visi lengkap: [naskah Wirebox awal](../packet-tracer/sources/01-wirebox-vision.txt). Tambahan desain pada dokumen tematik ditandai sebagai usulan agar tidak dianggap sebagai permintaan atau fitur yang sudah disetujui.
