# Zig, Rust, GPUI, dan GPUix

Status implementasi per 21 Oktober 2026: **Zig untuk engine simulasi, Rust untuk wrapper/host/oracle, GPUI langsung untuk UI**. Phase 3 menambahkan Ethernet/ARP/IPv4/ICMP dan tracker byte headless pada runtime phase 2; [bukti G3](../evidence/phase-3/README.md). Interaksi GPUI masih spike probe phase 1. Integrasi Studio, benchmark, dan release packaging belum tersedia. Keputusan pada [ADR](../adr/README.md); hasil sebelumnya [G0/G1](../evidence/phase-0-1/README.md) dan [G2](../evidence/phase-2/README.md).

## Baseline versi yang ditetapkan

| Komponen | Ketetapan pengguna | Penerapan di repository |
|---|---|---|
| Zig | Minimal **0.17.0** | Build probe dipin dan diverifikasi pada 0.17.0 oleh [build.rs](../../crates/wirebox-kernel/build.rs); versi lebih baru perlu diuji sebelum pin berubah |
| Rust | **1.99.1** | Dipin di [rust-toolchain.toml](../../rust-toolchain.toml), profile minimal dengan rustfmt dan Clippy |

Kode probe sudah dibangun dengan keduanya. Batas minimum produk Zig tidak menjamin rilis berikutnya kompatibel tanpa perubahan API/build system; implementasi build saat ini memakai pin exact 0.17.0. Workflow Windows memverifikasi checksum unduhannya dan Cargo.lock mengunci dependency Rust.

Pemeriksaan sumber resmi pada 19 Oktober 2026: [halaman unduhan Zig](https://ziglang.org/download/) mencantumkan 0.17.0, dirilis 1 Oktober 2026. Tautan Rust `releases/latest` saat diperiksa mengarah ke [pengumuman Rust 1.99.1](https://blog.rust-lang.org/2026/10/15/Rust-1.99.1/), dirilis 15 Oktober 2026. Gunakan nomor versi tetap untuk build proyek, bukan alias `latest` yang bergerak.

## Instalasi dan verifikasi lokal

Pada 19 Oktober 2026, atas permintaan pengguna, kedua compiler dipasang dan diuji pada Windows x86_64:

| Komponen | Hasil verifikasi |
|---|---|
| Zig | `zig version` → `0.17.0` |
| Rust | `rustc --version` → `rustc 1.99.1 (58cf8f9c3 2026-10-13)` |
| Cargo | `cargo --version` → `cargo 1.99.1 (5f94df478 2026-08-27)` |
| rustfmt | `rustfmt --version` → `rustfmt 1.10.0-stable` |
| Clippy | `cargo clippy --version` → `clippy 0.1.99` |

Zig dipasang di `C:\Users\ricky\AppData\Local\Programs\Zig\zig-x86_64-windows-0.17.0` dan direktori tersebut ditambahkan ke PATH pengguna. Paket Windows x86_64 diambil dari manifest unduhan resmi Zig. SHA-256 arsip cocok dengan manifest: `b5663f69581dcf391293fbf16c06cb80d81d806545ce618b4d0bab7f0eb8c428`.

Rust dipasang melalui rustup sebagai `1.99.1-x86_64-pc-windows-msvc`, dengan rustfmt dan Clippy. Pin repository memilih versi ini; default global stable sebelumnya tidak diubah.

Program kecil untuk memeriksa interpretasi dua byte dalam urutan big-endian berhasil dikompilasi dan dijalankan dengan masing-masing compiler. Program Rust juga membuktikan linking menggunakan lingkungan MSVC yang tersedia. Source dan executable uji disimpan di direktori sementara di luar repository. Ini pemeriksaan instalasi compiler, bukan pengujian engine Wirebox atau integrasi Zig–Rust/GPUI.

Terminal/aplikasi yang sudah terbuka mungkin perlu dibuka ulang untuk mengambil PATH Zig yang baru. Periksa dari root repository:

```powershell
zig version
rustc --version
cargo --version
```

## Zig dan Rust untuk keterlihatan byte

Zig menekankan alur kontrol serta alokasi yang eksplisit. Karakter ini cocok dengan preferensi pengguna untuk memahami pengelolaan buffer dan perilaku engine. Dokumentasi bahasa menjelaskan filosofi dan pilihan runtime safety tersebut. [Zig overview](https://ziglang.org/learn/overview/).

Borrowing Rust mengatur penggunaan referensi dan ownership; ia tidak menghilangkan akses ke byte. Buffer byte, slice, codec eksplisit, dan pemetaan field dapat digunakan dalam Rust. Transparansi tracker tetap memerlukan pencatatan perubahan dan sebabnya di kedua bahasa.

Pertimbangan desain proyek:

| Opsi | Alasan memilih | Biaya yang perlu diuji |
|---|---|---|
| Zig sebagai kernel | Kontrol buffer, allocator, dan representasi yang sesuai preferensi pengguna | Ownership, lifetime, dan invariant engine perlu dikelola dengan disiplin |
| Rust sebagai kernel dan aplikasi | Satu bahasa dan kontrak ownership yang diperiksa compiler pada kode Rust yang sesuai | Pemodelan graph, arena, dan mutasi state perlu dirancang agar mudah dibaca |
| Zig kernel + Rust aplikasi | Mempertahankan preferensi Zig dan akses langsung ke ekosistem UI Rust | Dua toolchain, FFI, packaging, dan debugging lintas bahasa |

Tabel ini adalah penilaian desain, bukan pengukuran bahwa satu bahasa selalu lebih cepat atau lebih mudah. Model data berbasis ID/arena dapat dipertimbangkan di kedua bahasa untuk graph dan state berumur panjang.

## Pembagian tanggung jawab yang diusulkan

```text
Rust application + GPUI
  topology / inspector / timeline / application commands
                    |
          Rust wrapper untuk C ABI
                    |
Zig simulation kernel
  scheduler / packet buffers / codecs / protocol state / trace
```

Pembagian planner, persistence, dan driver perangkat antara modul/bahasa belum final. Kernel harus dapat diuji melalui runner headless sebelum bergantung pada UI. PDU dan state protokol tidak perlu memiliki salinan model yang dapat dimutasi secara independen di setiap bahasa.

## Kontrak Zig–Rust

API [runtime](runtime.md) dan [network](packets.md) kini menyediakan handle opaque, konfigurasi sebelum run, eksekusi berbatas, pembacaan trace batch, serta salinan versi paket/field. Command UI/persistence dan perubahan state aktif masih memerlukan kontrak phase berikutnya.

Kontrak harus menjawab:

- Siapa mengalokasikan dan melepas setiap buffer/handle?
- Sampai kapan pointer/view valid, dan apakah engine boleh berjalan selama view digunakan?
- Bagaimana length, kapasitas, alignment, enum, dan status error direpresentasikan?
- Bagaimana invalid handle, cancel, dan penghentian simulasi ditangani?
- Thread mana yang memiliki state dan apakah API mengizinkan callback/reentrancy?
- Bagaimana versi ABI, panic/failure, dan kompatibilitas build ditangani?

Mulai dengan batas sederhana yang mudah diverifikasi, misalnya menyalin batch kecil ke buffer milik aplikasi. Zero-copy adalah optimasi setelah kontrak lifetime jelas. Rust wrapper tidak membuktikan memory safety di dalam implementasi Zig.

## GPUI dibanding GPUix

[GPUI](https://gpui.rs/) adalah framework UI Rust dengan rendering GPU. Source-nya menyediakan elemen [canvas untuk custom painting](https://raw.githubusercontent.com/zed-industries/zed/main/crates/gpui/src/elements/canvas.rs). Ini memberi titik awal untuk menggambar topologi; layout graph, hit testing, pan/zoom, dan optimasi tetap pekerjaan Wirebox.

[GPUix](https://github.com/remorses/gpuix) membungkus GPUI untuk React/Solid dengan aplikasi TypeScript/JavaScript. Dokumentasi yang diperiksa menyediakan adapter Solid resmi, menyatakan status pre-1.0, meminta versi adapter/native dipin bersama, dan menjelaskan perlunya culling oleh aplikasi pada node graph atau timeline. [README GPUix](https://raw.githubusercontent.com/remorses/gpuix/main/README.md).

| Kebutuhan Wirebox | GPUI langsung | GPUix + Solid |
|---|---|---|
| Aplikasi yang sudah memakai Rust | UI berada di bahasa aplikasi | Memerlukan jembatan ke runtime JS |
| Preferensi penulisan UI | API Rust | Komponen TSX dan reaktivitas Solid |
| Integrasi Zig | Melalui wrapper C ABI di Rust | Tetap perlu jalur native/IPC menuju kernel |
| Topologi dan tracker khusus | Akses API drawing GPUI | Perlu memeriksa API yang diekspos binding pada versi terpilih |
| Biaya pembuktian | Build, input, drawing, packaging | Hal yang sama ditambah integrasi binding/runtime |

Rekomendasi GPUI berasal dari kesesuaian dengan arah aplikasi Rust dan kebutuhan inspector/canvas khusus. Ini belum membuktikan GPUI menang dalam benchmark Wirebox. GPUix relevan jika produktivitas Solid/TSX menjadi prioritas yang cukup besar untuk membenarkan lapisan tambahan.

## Catatan koreksi dan uji kelayakan

Diskusi sebelumnya menyebut versi tertentu serta canvas GPUix yang masih direncanakan. Informasi tersebut adalah snapshot historis. README yang diperiksa ulang sekarang juga memuat jalur input pixel untuk drawing; status kemampuan terbaru perlu ditentukan dari API dan demo versi terpilih. Jangan menjadikan klaim lama tentang ketiadaan canvas sebagai satu-satunya alasan memilih framework.

Uji kelayakan dasar Windows sudah dilakukan: window GPUI 0.2.2, node/link fixture yang dapat dipilih, hex inspector dari batch probe Zig, keyboard input, serta close/shutdown. Timeline jaringan dan pengukuran performa masih fase berikutnya; probe tidak memproses paket. Lihat [hasil runtime](../evidence/phase-0-1/README.md) dan [kontrak v1](phase-0-contract.md) untuk target acceptance yang dibekukan.

Sumber web diperiksa pada 19 Oktober 2026; tautan branch `main` dapat berubah. Saat implementasi dimulai, catat revisi yang benar-benar dibangun dan hasilnya pada [keputusan](decisions.md).
