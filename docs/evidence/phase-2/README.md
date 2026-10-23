# G2 — Graph, state, scheduler, dan trace dasar

Tanggal: **20 Oktober 2026**. Hasil: **lulus lokal Windows x64 debug** untuk task P2-01 sampai P2-06. Model `counter-actions-v1`, runtime ABI 1, SplitMix64-v1, Zig 0.17.0 dan Rust 1.99.1. Repo belum memiliki HEAD; [manifest](source-manifest.json) mengidentifikasi source dan artifact yang diverifikasi. Environment aktual: [environment.json](environment.json).

Gate ini membuktikan fondasi runtime: invariant graph, scheduler deterministik, lifecycle/cancellation, eksekusi berbatas, PRNG eksplisit, serta independensi retensi jurnal. Action hanya counter sintetis. Hasil ini belum membuktikan Ethernet/ARP/IP/ICMP, checkpoint/restore, fidelity perangkat, MVP UI, atau pilot perangkat nyata.

## Hasil yang dapat diulang

| Pemeriksaan | Hasil aktual |
|---|---|
| `scripts/verify.ps1` | Lulus formatter, Zig test, C11 layout, Cargo test, Clippy `-D warnings`, CLI, build GPUI |
| Zig | 16 test result lulus: 15 pengujian perilaku/layout dan 1 blok impor test |
| Rust unit | 13 lulus: 10 wrapper/probe/runtime dan 3 fixture/acceptance |
| Rust compile-fail doctest | 2 lulus: Session/World tidak dapat dikirim ke thread lain |
| Fixture graph | 4 device, 6 port, 3 link; validasi referensi, kind, endpoint exclusivity |
| Heap vs oracle | 200 jadwal pada 30 timestamp; cancel setiap index kelipatan 3; 133 eksekusi cocok dengan pemilihan minimum linear independen |
| Waktu/budget | Same-time tie-break, zero budget, empty step/horizon, past time, future event, timer periodik |
| Lifecycle | Device/port/link cascade menghapus timer entity terkait; ID lama tidak mengakses entity baru; kapasitas direklamasi setelah cancel |
| Overflow | Time/ID/counter/execution overflow; firing gagal tidak mengonsumsi RNG atau event; progress run mempertahankan prefix berhasil |
| Reproduksibilitas | 5 seed × 3 rerun cocok dengan 5 baseline; state, trace, clock, timer, RNG, ID dibandingkan |
| Retensi | 5 seed × kapasitas 0/1/3/256 = 20 run; state identik, trace suffix dan dropped count sesuai |
| Retensi wrap/read | 300 eksekusi dengan retensi 0/1/7/256; read tiap step cocok dengan batch tanpa read per step |
| Ownership | Salinan graph/trace tetap valid setelah mutasi dan close; runtime/probe handle tidak saling diterima |

Output lengkap: [verification.log](verification.log). Demo: [scheduler-demo.json](scheduler-demo.json). Acceptance matrix: [acceptance.json](acceptance.json). Pengujian matriks menjalankan 40 World baru: 5 baseline, 15 rerun, 20 variasi retensi. Ini pengulangan dari initial state, bukan restore checkpoint.

```powershell
./scripts/verify.ps1
cargo run --locked -p wirebox-headless -- --scheduler-demo
cargo run --locked -p wirebox-headless -- --check-runtime
```

## Expected dan actual demo

Expected dinyatakan di [runner acceptance](../../../crates/wirebox-headless/src/runtime_demo.rs) sebelum run: event ID `2,1,3,4,7`, timestamp `5,10,10,10,15`, total 5 firing, PC1=1, R1=12, PC2=0, event 7 cause=4. Topologi berasal dari fixture phase 0; runtime hanya mengambil identity dan relations.

Actual cocok. Untuk seed 20260117, sampel random SW1 adalah 33 dan 71 sehingga counter akhirnya 104. Budget menghentikan run pada 15 ns; setelah cancel timer periodik, empty horizon mencapai 100 ns. Timer yang dibatalkan dan owner sementara yang dihapus tidak pernah dieksekusi. Pending/timer akhir nol.

Oracle heap menggunakan pemilihan minimum linear pada input jadwal yang sama, tanpa memakai implementasi heap untuk menentukan expected. Oracle arithmetic/header phase 0 tetap terpisah; G2 tidak memperlakukannya sebagai paket yang telah diproses.

## Artifact dan batas

- [Kontrak runtime](../../wirebox/runtime.md), [ADR-0006](../../adr/0006-phase-2-runtime.md), [Zig tests](../../../kernel/src/runtime_tests.zig).
- Registry bounded: 8 World, 128 entity, 256 pending event/timer, retensi maksimum 256. Ini batas eksplisit phase 2, belum ukuran MVP 10.000 row.
- Jurnal hanya firing sukses; command host/cancel/remove belum diaudit sebagai event. Queue/graph/seed metadata yang terlihat belum snapshot lengkap untuk restore.
- Source export baru diverifikasi terpisah oleh `scripts/verify-clean-export.ps1`; [clean-export.log](clean-export.log) mencatat hasil dan path. Dependency/compiler cache sengaja digunakan kembali; bukan cold-cache atau remote CI.
- Desktop dibangun/test/lint ulang. Interaksi native GUI tidak diulang karena UI tetap spike probe phase 1; bukti interaksi sebelumnya berada pada [G0/G1](../phase-0-1/README.md).
- Warning dependency transitif `proc-macro-error2 2.0.1` mengenai kompatibilitas Rust masa depan masih muncul. Build pada Rust 1.99.1 yang dipin lulus; tidak mengklaim versi compiler selanjutnya sudah kompatibel.
- Evidence G0/G1 dan arsip reverse engineering dipertahankan sebagai rekaman baseline lama. Hash source lama memang tidak harus sama dengan source phase 2.
- Tidak ada perangkat yang dihubungi atau diubah; remote CI, release packaging, dan benchmark belum dijalankan.
