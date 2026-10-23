# ADR-0005 — Scope pilot v1 dan bukti berjenjang

- Tanggal: 20 Oktober 2026
- Status: Accepted sebagai baseline kerja phase 0. Model/firmware dan akses nyata yang belum diketahui tetap unknown.

## Konteks

Rencana perlu membuktikan ide awal tanpa menunggu seluruh vendor/protokol. Pengguna menyediakan Linux VM/server, RouterOS major 7, laporan C320 board GTXK/GTGHG/SMXA, dan contoh ONU GPON yang berstatus working. Output ONU tidak berisi model/firmware; VSOL belum memiliki detail.

## Keputusan

Bekukan scope, fixture, serta expected result pada [kontrak phase 0](../wirebox/phase-0-contract.md). Gunakan empat milestone: PoC G3, MVP G4, pilot inti Linux/MikroTik G7, dan pilot ISP ZTE–Huawei/VSOL G8. Fitur unsupported menghasilkan status eksplisit; tidak dihitung sebagai pass konektivitas.

G0 dapat lulus untuk **local development scope** setelah fixture, mesin referensi, inventaris diketahui/unknown, dan matriks uji tersedia. Kelengkapan akses Linux/RouterOS tetap dependency G6; detail pasangan ISP tetap dependency G8. Ini penetapan scope sebelum acceptance run, bukan menyatakan hardware telah diverifikasi.

Informasi perangkat dari chat disimpan sebagai user-reported fixture yang disamarkan. Prompt dengan suffix SAAS-SIM tidak membuktikan hardware fisik ataupun fidelity simulator. Timestamp internal perangkat tidak dipakai sebagai waktu koleksi.

## Alternatif dan konsekuensi

Mewajibkan semua perangkat nyata siap sebelum build menunda pembuktian kernel yang independen. Membuat semua fitur vendor sekaligus memperlebar asumsi tanpa evidence. Pembuktian berjenjang dipilih, dengan laporan gate yang menyatakan scope, unknown, serta keterbatasan.

Gate implementasi phase 1 hanya membuktikan stack dan kontrak data. Tidak ada simulasi, ping nyata, atau mutation perangkat yang dinyatakan selesai oleh gate tersebut. Target numerik pilot dibekukan sebagai acceptance baseline; revisi selanjutnya harus tercatat sebelum pengujian ulang.
