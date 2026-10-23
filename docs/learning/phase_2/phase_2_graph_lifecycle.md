# Phase 2 — Graph device, port, link, dan lifecycle

**Yang dikerjakan.** [graph.zig](../../../kernel/src/graph.zig) menyimpan entity `device`, `port`, dan `link`. `Graph.add` memberi ID `u64` monotonik dalam satu World. Port menunjuk owner device; link menunjuk dua endpoint port. Graph menyimpan relasi operasional saja: posisi atau warna tampilan tidak masuk ke kernel.

**Keputusan kode.** Ada maksimum 128 entity. ID nol invalid, dan ID lama tidak dipakai ulang setelah entity dihapus. Sebuah port hanya boleh mempunyai satu link; owner dan endpoint harus ada dengan kind yang tepat. Validasi terjadi sebelum relasi disimpan. Batas dan konflik menjadi error yang bisa dibaca caller, sehingga model tidak diam-diam membuat topologi ambigu.

`Graph.remove` menghapus dependent secara berantai: menghapus device menghapus port dan link yang menyentuh portnya; menghapus port juga menghapus linknya. [Runtime.remove](../../../kernel/src/runtime.zig) lalu membatalkan timer/event milik entity yang dihapus. Urutan ini mencegah event tertunda mengeksekusi terhadap identitas yang sudah tidak hidup. Menghapus link mempertahankan endpoint.

**Mengapa graph menjadi dasar.** Dalam topologi `PC1 — SW1 — R1 — PC2`, packet engine membutuhkan jawaban pasti: port ini milik device mana, link mana yang menghubungkan dua port, dan apa yang ikut hilang saat sebuah device dihapus. ID entity stabil memungkinkan trace menunjuk objek yang sama tanpa bergantung pada label UI. ID masih lokal ke satu World, jadi ID 7 pada dua World bukan objek global yang sama.

**Cara membaca kode.** Mulai dari `Kind` dan `Entity` di `graph.zig`, lalu `Graph.add`, `Graph.remove`, dan `Runtime.remove`. Cocokkan empat device, enam port, dan tiga link dari [fixture](../../../fixtures/scenarios/icmp-static-v1.json) dengan ID yang dikeluarkan [demo scheduler](../../evidence/phase-2/scheduler-demo.json). [Tes lifecycle](../../../kernel/src/runtime_tests.zig) memeriksa endpoint eksklusif, cascade, dan timer owner yang dihapus. Graph phase 2 sendiri belum menyimpan IP/MAC; konfigurasi itu ditambahkan oleh [Network phase 3](../phase_3/phase_3_alur_paket.md).
