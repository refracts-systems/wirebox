# Keputusan, preferensi, dan pertanyaan terbuka

Catatan sampai 21 Oktober 2026. Percakapan eksplorasi tidak otomatis menjadi keputusan final. Dokumen ini membedakan arahan pengguna, rekomendasi dari pembahasan, serta tambahan rancangan untuk memperjelas langkah berikutnya.

## Rekap pembahasan

| ID | Topik | Status | Hasil dan rujukan |
|---|---|---|---|
| WB-01 | Definisi Wirebox | Visi pengguna | Studio/runtime infrastruktur dengan model bersama untuk inspect, simulate, dan execute; [ringkasan](README.md) |
| WB-02 | Apakah idenya masuk akal? | Penilaian desain | Layak dibangun bertahap; akurasi model dan cakupan vendor merupakan tantangan utama; [pembanding](positioning.md) |
| WB-03 | Bukti reverse engineering PT | Laporan + verifikasi lokal terbatas | Fondasi objek, proses, scheduler, dan trace mendukung arah simulator; [evidence ledger](../packet-tracer/evidence-ledger.md) |
| WB-04 | Simulasi tanpa device | Arah yang dieksplorasi | Bisa tanpa hardware/image vendor untuk perilaku yang dimodelkan; perangkat virtual tetap perlu state dan aturan; [simulasi](simulation.md) |
| WB-05 | Memahami byte per byte | Kebutuhan pengguna | Tracker perlu byte, field mapping, versi, serta alasan perubahan; [rancangan tracker](byte-tracker.md) |
| WB-06 | Bahasa Zig | Keputusan pengguna; runtime/protokol phase 2–3 tersedia | Minimum produk 0.17, pin build 0.17.0; [model paket](packets.md) berjalan headless |
| WB-07 | Zig bersama Rust | Diimplementasikan untuk phase 1–3 | Probe/runtime/network Zig lewat copy C ABI; [ADR-0002](../adr/0002-zig-rust-abi.md), [ADR-0007](../adr/0007-packet-model.md) |
| WB-08 | GPUI atau GPUix + Solid | GPUI dipilih dan diuji untuk spike | GPUI 0.2.2 langsung; node/link/hex/input/close lokal terbukti, benchmark belum; [ADR-0004](../adr/0004-gpui-windows.md) |
| WB-09 | Urutan MVP | Rencana revisi 20 Oktober 2026 | Percobaan stack/FFI/UI lebih awal, PoC paket, MVP Studio, replay/validasi, pilot inti, lalu domain ISP; [roadmap](roadmap.md). TCP/UDP lengkap ditunda kecuali diperlukan oleh scope pilot |
| WB-10 | Repository proyek | Dilakukan atas permintaan pengguna | Workspace `C:\Users\ricky\src\wirebox`, Git diinisialisasi pada branch `main`, docs dibuat |
| WB-11 | Panduan agent dan dokumentasi konsep | Dilakukan atas permintaan pengguna | [AGENTS.md](../../AGENTS.md) dan rangkaian docs Wirebox ditambahkan |
| WB-12 | Versi compiler | Keputusan pengguna | Zig minimal **0.17.0**, Rust **1.99.1**; Rust dipin di [rust-toolchain.toml](../../rust-toolchain.toml); [status verifikasi](technology.md) |
| WB-13 | Plan bertahap sampai future | Permintaan pengguna; rincian merupakan usulan kerja | Phase 0–10 dengan milestone PoC/MVP/pilot, [todos](todos.md), serta [acceptance criteria](pilot.md); belum berarti gate lulus |
| WB-14 | Perangkat pilot | Arahan pengguna, 20 Oktober 2026 | Linux lab, MikroTik RouterOS, OLT ZTE, ONU Huawei dan VSOL; detail model/firmware serta capability masih perlu diinventarisasi |
| WB-15 | Scope dan kontrak pembuktian v1 | Ditetapkan dalam pelaksanaan phase 0 | [Kontrak baseline](phase-0-contract.md) serta [inventaris user-reported](../../fixtures/inventory/pilot-v1.json); unknown akses/perangkat tetap dependency G6/G8 |
| WB-16 | ADR dan pelaksanaan phase 0/1 | Permintaan pengguna, dikerjakan | [ADR](../adr/README.md), G0 local development dan G1 Windows debug lulus; [bukti](../evidence/phase-0-1/README.md) |
| WB-17 | Lanjut phase 2 | Permintaan pengguna, dikerjakan | Graph, scheduler, timer, PRNG, jurnal, serta headless; [ADR-0006](../adr/0006-phase-2-runtime.md), [G2](../evidence/phase-2/README.md) |
| WB-18 | Lanjut phase 3 | Permintaan pengguna, dikerjakan | Ethernet/ARP/IPv4/ICMP, routing statis, salinan versi byte dan field, trace kausal serta oracle; [ADR-0007](../adr/0007-packet-model.md), [G3](../evidence/phase-3/README.md) |

## Koreksi yang memengaruhi desain

- Borrow checker Rust tidak menyembunyikan byte paket. Keterlihatan byte adalah kebutuhan desain tracker, bukan alasan teknis yang meniadakan Rust.
- Mini-sim membuktikan sebuah skenario software dapat berjalan mandiri; ia belum membuktikan seluruh perilaku PT atau vendor.
- Event List adalah tampilan/log. Queue scheduler serta checkpoint replay memiliki fungsi dan state berbeda.
- Hipotesis awal default passphrase AES/Mash untuk container `.pkt` digantikan laporan jalur dokumen lain. Reader yang tercatat masih memiliki batas captured keystream; [status akhir bahan RE](../packet-tracer/pkt-format.md).
- Klaim fitur/versi GPUix dari percakapan lama tidak menjadi fakta permanen. Dokumen teknologi memisahkan konteks lama dan pemeriksaan ulang sumber.

## Hal yang belum diputuskan

| Pertanyaan | Dibutuhkan sebelum |
|---|---|
| Strategi release/distribusi Windows; build debug, pin Zig/Cargo.lock/GPUI, dan workflow CI sudah tersedia | Phase 9; remote CI run masih perlu dilakukan |
| Peningkatan kapasitas dari baseline bounded ADR-0006 | Sebelum skenario MVP melampaui batas phase 2 |
| Format skenario, snapshot, trace, serta versioning | Persistence dan replay |
| Snapshot/command UI; ABI network sudah tersedia | Phase 4 |
| Perluasan ID/byte model di luar ADR-0007, misalnya retransmission TCP | Saat protokol berikutnya ditambahkan |
| Ukuran skenario sasaran dan target latensi/memori | Benchmark dan optimasi |
| Perluasan subset protokol di luar kontrak phase 0 dan ADR-0007 | Klaim dukungan fitur baru |
| Model/firmware, capability, akses lab, serta data pembanding untuk target WB-14; urutan Linux/MikroTik lalu ISP adalah rencana | Adapter nyata |
| Revisi scope setelah baseline v1, jika fitur pilot baru diperlukan | Harus dicatat sebelum acceptance run berikutnya |
| Model otorisasi, reconciliation otomatis, dan kompensasi | Mutasi perangkat nyata |
| Kebutuhan impor `.pkt` dibanding format Wirebox sendiri | Prioritas interoperabilitas |

## Cara memperbarui

Saat pilihan dikonfirmasi atau implementasi berubah, catat alasan, tanggal, artifact, dan hasil pengujian pada dokumen terkait. Ubah status tabel ini dengan tautan ke bukti. Jangan menandai rekomendasi sebagai keputusan pengguna hanya karena dicatat di repository.

Dokumen tematik adalah sintesis, bukan transkrip verbatim. Visi asli dan laporan RE tetap tersedia pada [arsip sumber](../packet-tracer/sources/README.md).
