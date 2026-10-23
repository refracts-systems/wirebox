# Wirebox

Studio infrastruktur untuk inspeksi, simulasi, penelusuran paket, dan perubahan berbasis model bersama. [Konsep lengkap](docs/wirebox/README.md), [learning docs phase 1–3](docs/learning/README.md), [ADR](docs/adr/README.md), [roadmap](docs/wirebox/roadmap.md), dan [todos](docs/wirebox/todos.md).

## Status implementasi

Phase 0–3 tersedia untuk scope lokal: kontrak/fixture, probe Zig–Rust dan GPUI, runtime waktu virtual, serta **simulator Ethernet/ARP/IPv4/ICMP dengan routing statis dan tracker byte headless**. [Model paket](docs/wirebox/packets.md) memproses ping sukses, route hilang, TTL habis, ARP timeout, dan input invalid. [Walkthrough byte](docs/evidence/phase-3/byte-walkthrough.md) menunjukkan byte aktual sebelum/sesudah forwarding. Wirebox membangun model perilaku jaringan; OS dan image vendor tidak diperlukan.

Target build: Windows x64, Zig **0.17.0** sebagai baseline yang diverifikasi (minimum produk 0.17), Rust **1.99.1**, Visual Studio C++ Build Tools dan Windows SDK. Dependency GPUI dipin pada **0.2.2** dengan Cargo.lock. Rust dipilih otomatis oleh rustup melalui rust-toolchain.toml.

## Menjalankan

PowerShell dari root repository:

```powershell
# Jika zig.exe belum tersedia di PATH, tunjuk lokasi instalasinya:
$env:WIREBOX_ZIG = 'C:\Users\ricky\AppData\Local\Programs\Zig\zig-x86_64-windows-0.17.0\zig.exe'

cargo run --locked -p wirebox-headless -- --probe
cargo run --locked -p wirebox-headless -- --check-fixtures
cargo run --locked -p wirebox-headless -- --scheduler-demo
cargo run --locked -p wirebox-headless -- --check-runtime
cargo run --locked -p wirebox-headless -- --packet-demo S01
cargo run --locked -p wirebox-headless -- --check-network
cargo run --locked -p wirebox-studio
```

Sesuaikan path Zig untuk komputer lain. Jika memakai lokasi instalasi lokal standar, build script juga dapat menemukannya melalui LOCALAPPDATA. Kernel dibangun oleh Cargo build script ke OUT_DIR; tidak perlu membangun static library manual.

Pada window: pilih node/link atau byte untuk inspector, tekan **Ctrl+R** untuk batch probe baru, **Esc** untuk reset selection, dan **Ctrl+Q** atau tombol close window untuk keluar. Topologi adalah fixture statis. Byte/event probe berasal dari Zig; tombol ini belum mengirim paket jaringan.

## Verifikasi

Hardening Phase 0–3 dan satu perbaikan validasi route dijelaskan di [learning notes](docs/learning/hardening_phase_0_3.md), dengan [hasil pengujian](docs/evidence/hardening-phase-0-3/README.md).

Materi pribadi Cognitive Coverage (guide HTML, kuis, dashboard, dan progress belajar) disimpan **hanya secara lokal** dalam `cognitive-coverage/`, yang diabaikan Git. Folder ini tidak tersedia melalui GitHub atau clone baru. Jika materi sudah dihasilkan di komputermu, buka `cognitive-coverage/cognitive-coverage-open.html` melalui browser. Skill project dan catatan belajar Markdown tetap tersedia di repository; materi ini tidak menambah dependency runtime engine.

```powershell
# Kernel, C layout, wrapper, fixture, formatter, dan Clippy
./scripts/verify.ps1 -CoreOnly

# Default Zig tests: Debug + ReleaseSafe. Tambahkan ReleaseFast bila diperlukan.
./scripts/verify.ps1 -CoreOnly -ZigModes Debug,ReleaseSafe,ReleaseFast

# Termasuk build/test/lint desktop (tidak membuka window)
./scripts/verify.ps1

# Build dari salinan source baru, memakai dependency/compiler cache yang sudah tersedia
./scripts/verify-clean-export.ps1

```

Opsional, hanya jika folder pembelajaran lokal tersedia: jalankan `node scripts/check-cognitive-coverage.mjs` untuk memeriksa source hash, anchor, dan quiz mapping. Ini bukan prasyarat build atau CI. Perubahan source atau dokumentasi dapat membuat manifest lokal stale; refresh materi terkait sebelum menganggapnya sesuai versi terbaru.

Build pertama GPUI mengunduh dan mengompilasi banyak dependency. Workspace default hanya memuat kernel/headless; `--workspace` memasukkan desktop. Script memeriksa status setiap native command. Workflow Windows pada `.github/workflows/verify.yml` disiapkan; keberadaan workflow tidak berarti job remote telah dijalankan.

Pemeriksaan visual, input, dan close window merupakan verifikasi runtime terpisah. Hasil aktual ada pada [evidence phase 0–1](docs/evidence/phase-0-1/README.md), [phase 2](docs/evidence/phase-2/README.md), dan [phase 3](docs/evidence/phase-3/README.md). Uji build saat ini debug. Integrasi simulator dengan Studio adalah phase 4; validasi Linux lab G5 dan distribusi release belum dijalankan.

## Struktur

- `kernel/`: ABI C, probe, graph, scheduler, PRNG, codec paket, protokol, serta jurnal Zig.
- `crates/wirebox-kernel/`: wrapper ownership/error/thread serta pemanggilan build Zig.
- `crates/wirebox-headless/`: CLI probe, fixture, demo scheduler/paket, dan oracle acceptance terpisah.
- `crates/wirebox-studio/`: spike GPUI dengan node/link/hex inspector.
- `fixtures/`: skenario sintetis, expected bytes, dan inventaris user-reported yang disamarkan.
- `docs/`: desain, keputusan, phase, evidence, serta sumber RE historis.

Panduan perubahan: [AGENTS.md](AGENTS.md). Jangan memasukkan credential, capture pelanggan tanpa sanitasi, atau binary vendor ke repository.
