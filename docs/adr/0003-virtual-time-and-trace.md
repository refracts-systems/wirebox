# ADR-0003 — Waktu virtual dan trace yang dapat direproduksi

- Tanggal: 20 Oktober 2026
- Status: Accepted, diimplementasikan phase 2; rincian budget/PRNG/retensi di ADR-0006 dan bukti pada laporan G2.

## Konteks

Scheduler PT dan log GUI dilaporkan terpisah. Wirebox juga membutuhkan urutan kausal serta restore yang lebih kuat daripada memutar ulang animasi.

## Keputusan

Gunakan tick `u64` dalam nanodetik, mulai dari nol, dengan urutan `(time_ns, sequence)`. Sequence monotonic dan overflow diperiksa. Penjadwalan ke masa lalu ditolak. `step()` memproses satu record scheduler berikutnya; antrean kosong tidak memajukan jam. `runUntil(t)` mengeksekusi record bertimestamp `<= t`, lalu memajukan jam ke `t`; `t < now` error. Batas event/cancel mencegah timer periodik memblokir eksekusi.

Mulai dengan satu pemilik eksekusi state. Event tertunda menggunakan record/ID stabil yang dapat diserialisasi, bukan menyimpan pointer fungsi/closure native dalam checkpoint. Timer memiliki ID dan lifecycle cancellation. Algoritme PRNG dipin bersama implementasi phase 2 sebelum mengklaim determinisme random; seed dan state termasuk checkpoint.

Pisahkan queue scheduler, jurnal, dan visible event list. Retensi GUI tidak mengubah eksekusi. Untuk delay serialisasi gunakan pembulatan integer ke atas dari bit/bitrate dan periksa overflow. Waktu GUI/realtime hanya mengatur pacing.

## Alternatif dan konsekuensi

Floating-point time dan wall clock langsung mudah dipakai tetapi memperumit reproduksibilitas. Callback arbitrary memudahkan prototipe namun menyulitkan checkpoint. Record bertipe memerlukan dispatch eksplisit, sebagai biaya untuk replay yang dapat diperiksa.

Ini konvensi Wirebox, bukan peniruan semantics `runUntil` mini-sim lama. Replay dijamin hanya pada versi model/build/lingkungan yang dinyatakan sampai kompatibilitas lintas versi dibuktikan.

Rujukan: [simulasi](../wirebox/simulation.md), [mini-simulator dan batasnya](../packet-tracer/mini-simulator.md).
