# Phase 1 — Menghubungkan build Zig dan Rust

**Yang dikerjakan.** Workspace Cargo menjalankan [build.rs](../../../crates/wirebox-kernel/build.rs). Script itu memanggil Zig untuk membangun static library dari [abi.zig](../../../kernel/src/abi.zig), menaruhnya di `OUT_DIR`, lalu memberi tahu Rust linker lokasi dan nama library. Library tersebut juga membawa modul runtime dan network yang diimpor oleh entry Zig; phase 1 pertama kali membuktikan jalur build-nya.

Alurnya:

```text
cargo build/run
  → build.rs memilih zig.exe dan memeriksa versi
  → zig build-lib kernel/src/abi.zig -target x86_64-windows-msvc
  → wirebox_kernel.lib di Cargo OUT_DIR
  → rustc link library
  → executable Rust memanggil wb_* melalui C ABI
```

**Keputusan kode.** [rust-toolchain.toml](../../../rust-toolchain.toml) memilih Rust 1.99.1; `build.rs` menerima tepat Zig 0.17.0 pada build baseline. Pengguna menetapkan Zig minimal 0.17; exact pin dalam build berarti versi berikutnya harus diuji sebelum syarat itu dilonggarkan. Target eksplisit `x86_64-pc-windows-msvc` mencegah hasil build platform lain disalahartikan sebagai sudah didukung. `WIREBOX_ZIG` memberi cara memilih compiler tanpa mengubah kode.

`cargo:rerun-if-changed` mengamati `kernel/src` dan `kernel/include`; perubahan Zig/header memicu rebuild. `OUT_DIR` dipilih agar static library hasil build tidak bergantung pada file `.lib` di source checkout. `Cargo.lock` mengunci dependency Rust. Zig dan Rust tetap mempunyai compiler, type system, dan allocator masing-masing; yang disepakati hanya ABI kecil pada header C.

**Mengapa ini perlu.** Ketika mesin simulator berkembang, satu command Cargo harus mengompilasi Zig yang sesuai sebelum wrapper dan UI Rust. Pemeriksaan versi dan target membuat kegagalan kompatibilitas terlihat di awal. [Verifikasi G1](../../evidence/phase-0-1/README.md) juga membangun dari salinan source baru dengan cache dependency/compiler; itu menguji bahwa build tidak diam-diam memakai static library yang tertinggal di checkout.

**Coba sendiri.** Dari root repository jalankan `cargo run --locked -p wirebox-headless -- --probe`, kemudian `./scripts/verify.ps1 -CoreOnly`. Perintah probe harus mengembalikan byte yang sama setelah melalui Zig. Pemeriksaan seluruh workspace juga membangun GPUI. Ini build debug Windows; hasil G1 tidak menyatakan release package atau platform lain sudah diuji.
