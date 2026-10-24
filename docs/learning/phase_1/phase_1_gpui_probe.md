# Phase 1 — GPUI menampilkan probe, belum simulasi paket

**Yang dikerjakan.** [Studio GPUI](../../../crates/wirebox-studio/src/main.rs) membuka window native, menampilkan node/link fixture dan hex buffer, menerima pilihan serta shortcut, lalu menutup session Zig ketika view dihancurkan. [ADR-0004](../../adr/0004-gpui-windows.md) mencatat keputusan memakai GPUI langsung untuk spike Windows.

**Keputusan kode.** `Studio` memiliki `Session`, event probe terakhir, selection, offset byte terpilih, dan focus handle. `Studio::new` mengirim `PROBE_BYTES` melalui Zig lalu membaca satu record hasilnya. `ReadProbe` mengirim batch baru dan mengganti event yang ditampilkan; `cx.notify()` meminta GPUI menggambar ulang. Node dan link pada kanvas adalah fixture statis. Karena itu memilih R1 atau sebuah byte tidak berarti router virtual sedang memproses paket.

`Drop for Studio` menutup `Session` secara eksplisit dan mencatat hasilnya. Pilihan ini membuat lifecycle terlihat di smoke log dan menjaga cleanup saat window ditutup. Window memiliki keyboard shortcut untuk read, reset selection, dan close. UI memiliki state presentasi sendiri, sedangkan buffer event berasal dari salinan Zig; kedua jenis state tidak dicampur.

**Mengapa fase ini dibuat lebih awal.** Engine yang berjalan headless saja belum membuktikan bahwa Zig, Rust, dan GPUI dapat dikompilasi serta berinteraksi pada Windows target. Spike menutup risiko build, input dasar, rendering, dan shutdown sebelum UI berisi workflow simulator. [Evidence G1](../../evidence/phase-0-1/README.md) merekam interaksi window aktual. Build GPUI juga diperiksa pada G3, tetapi UI belum menerima trace paket phase 3; integrasi itu ada di phase 4.

**Coba sendiri.** Jalankan `cargo run --locked -p wyrlex-studio`. Pilih node, link, dan byte; gunakan Ctrl+R untuk probe baru. Bandingkan ID event/byte dengan output `cargo run --locked -p wyrlex-headless -- --probe`. Kesamaan data membuktikan jalur salinan ABI yang dipakai UI, bukan akurasi jaringan.
