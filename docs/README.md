# Dokumentasi Wirebox

Catatan konsep, arah desain, keputusan, dan bukti pengembangan Wirebox. Phase 0–3 tersedia, termasuk simulator paket dan tracker byte headless. Studio terintegrasi dan adapter nyata mengikuti roadmap.

## Wirebox

- [Menjalankan build dan preview](../README.md)
- [Architecture Decision Records](adr/README.md)
- [Kontrak phase 0 dan fixture baseline](wirebox/phase-0-contract.md)
- [Kontrak FFI v1](wirebox/ffi.md)
- [Uji boundary generation handle: Kandidat A dan B](evidence/handle-generation/README.md)
- [Hasil verifikasi gate G0/G1](evidence/phase-0-1/README.md)
- [Runtime graph/scheduler dan ABI phase 2](wirebox/runtime.md)
- [Hasil verifikasi gate G2](evidence/phase-2/README.md)
- [Simulator paket, byte tracker, dan ABI phase 3](wirebox/packets.md)
- [Hasil verifikasi gate G3](evidence/phase-3/README.md)
- [Walkthrough perubahan byte aktual](evidence/phase-3/byte-walkthrough.md)
- [Learning docs phase 1–3: keputusan kode dan alasannya](learning/README.md)
- [Hardening Phase 0–3: hasil dan batas pengujian](evidence/hardening-phase-0-3/README.md)
- [Panduan hardening untuk backend engineer](learning/hardening_phase_0_3.md)
- [Catatan Cognitive Coverage](learning/hardening_phase_0_3.md#apa-arti-generated): guide, kuis dan dashboard pribadi tersedia hanya dalam folder lokal yang diabaikan Git.
- [Apa itu Wirebox, tujuan, dan batas cakupan](wirebox/README.md)
- [Model semantik, state, planner, dan runtime](wirebox/architecture.md)
- [Simulasi tanpa perangkat dan hubungan dengan bukti PT](wirebox/simulation.md)
- [Byte tracker, penjelasan perubahan, dan replay](wirebox/byte-tracker.md)
- [Zig, Rust, GPUI, dan GPUix + Solid](wirebox/technology.md)
- [Kelayakan ide dan proyek pembanding](wirebox/positioning.md)
- [Roadmap phase: PoC, MVP, pilot, beta, dan future](wirebox/roadmap.md)
- [Todos per phase dan status pekerjaan](wirebox/todos.md)
- [Rencana pilot, skenario uji, dan ukuran keberhasilan](wirebox/pilot.md)
- [Keputusan, preferensi, serta pertanyaan terbuka](wirebox/decisions.md)

Panduan untuk agent dan kontributor: [AGENTS.md](../AGENTS.md).

## Reverse engineering Packet Tracer

- [Ringkasan dan cara membaca](packet-tracer/README.md)
- [Arsitektur simulasi](packet-tracer/architecture.md)
- [Daftar bukti dan tingkat kepastian](packet-tracer/evidence-ledger.md)
- [Audit REA/Ghidra dan smoke test PT lokal](evidence/pt-comparison/README.md)
- [Runbook pengujian pembanding PT dan Wirebox](packet-tracer/comparison-tests.md)
- [Mini-simulator dan verifikasi lokal](packet-tracer/mini-simulator.md)
- [Format dokumen .pkt dan koreksi temuan](packet-tracer/pkt-format.md)
- [Implikasi untuk Wirebox](packet-tracer/wirebox-implications.md)
- [Arsip laporan asli dan provenance](packet-tracer/sources/README.md)

Catatan disusun pada 19 Oktober 2026 dari visi awal pengguna, percakapan proyek, laporan RE, dan pemeriksaan mini-simulator lokal. Dokumen ini tidak menyatakan bahwa seluruh binary Packet Tracer telah diaudit ulang. Sumber teknologi eksternal yang diperiksa ulang ditautkan pada dokumen terkait.

Rencana phase dan target pilot Linux/MikroTik/ZTE/ONU diperbarui pada 20 Oktober 2026. Gate serta checklist membedakan kesiapan yang sudah tersedia dari implementasi yang belum diuji.
