# ADR-0002 — Zig kernel, Rust host, dan C ABI v1

- Tanggal: 20 Oktober 2026
- Status: Accepted; probe, runtime graph/scheduler phase 2, dan [network ABI phase 3](../wirebox/packets.md) diimplementasikan. Perluasan kontrak network mengikuti ADR-0007.

## Konteks

Pengguna menetapkan Zig minimal 0.17 dan Rust 1.99.1. Dibutuhkan batas yang dapat diuji sebelum graph/protokol/UI membesar.

## Keputusan

Gunakan Zig **0.17.0** sebagai pin build yang diverifikasi dan Rust **1.99.1**. Baseline produk tetap Zig minimal 0.17; rilis lebih baru dievaluasi sebelum pin diganti. Cargo build script membangun static library Zig untuk Windows x64 MSVC. `WIREBOX_ZIG` dapat menunjuk compiler secara eksplisit.

Kontrak v1 berada di [header C](../../kernel/include/wirebox.h). Session direpresentasikan handle `u64` dengan index/generation; angka nol dan generation lama invalid. Registry probe memiliki 16 session, setiap session antrean 16 record dengan payload maksimal 64 byte. Slot habis menghasilkan status capacity. Ini kapasitas **probe**, bukan janji kapasitas engine/packet MVP.

Data masuk disalin ke Zig, data keluar disalin ke buffer pemanggil. Tidak ada borrowed pointer yang disimpan host dan tidak ada allocator yang harus menyeberang bahasa. ABI memakai integer lebar tetap, `size_t`, extern struct dengan layout yang diperiksa, dan status `u32`; tidak memakai bool lintas ABI, Rust container, Zig slice, exception, atau callback host.

Semua operasi registry diserialisasi process-wide oleh pemanggil. Wrapper Rust menggunakan mutex dan session `!Send + !Sync`. Data hasil baca milik Rust dan dapat bertahan setelah session ditutup. Drop tidak melakukan unwind melintasi ABI. Pointer non-null dari pemanggil C tetap harus valid, aligned, dan berukuran sesuai kontrak; API tidak dapat membuktikan validitas sembarang alamat.

## Alternatif dan konsekuensi

Rust-only mengurangi toolchain, tetapi tidak mengikuti preferensi kernel pengguna. Shared native structs/pointer views mengurangi copy, namun memperluas risiko lifetime. Bounded copy probe dipilih untuk membuktikan kontrak dahulu. Registry statis bukan arsitektur scheduler; revisi API simulasi memerlukan review ABI dan tidak boleh menyamakan probe event dengan network event.

Detail: [kontrak FFI](../wirebox/ffi.md). Mekanisme linking mengikuti [Cargo build scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html); sintaks native mengikuti [Zig 0.17](https://ziglang.org/documentation/0.17.0/).
