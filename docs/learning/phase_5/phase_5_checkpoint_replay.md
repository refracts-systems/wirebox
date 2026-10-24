# Phase 5: checkpoint, restore, fork, and replay

Catatan ini menjelaskan kode Phase 5 untuk backend engineer. Targetnya bukan menyimpan ringkasan hasil, tetapi menyimpan state yang cukup untuk melanjutkan event yang masih berada di scheduler.

## State apa yang disalin

`kernel/src/network.zig` memiliki `Network` yang seluruh field-nya berupa nilai tetap: graph, queue, timer, ARP/MAC cache, packet versions, transmission records, trace ring, counter, input ping/inject, dan `runtime.rng.state`. Tidak ada pointer yang harus dibetulkan setelah copy. Karena itu `wb_net_checkpoint()` mengalokasikan `Network` di heap lalu memakai `@memcpy` atas `std.mem.asBytes(...)`.

Keputusan `@memcpy` penting di Windows. Assignment `copy.* = source.*` dapat membuat temporary sebesar seluruh packet dan trace array di stack proses. Unit test dengan thread test yang stack-nya lebih besar bisa lulus, tetapi binary headless mengalami stack overflow. Pengujian standalone menemukan bug itu; byte copy memperbaikinya tanpa mengubah model data.

## Restore dan fork

`wb_net_restore(handle, checkpoint)` mengganti nilai network target dengan copy checkpoint sambil mempertahankan handle target. `wb_net_fork(checkpoint)` membuat network handle baru. Ini mirip snapshot database untuk eksperimen, tetapi ownership-nya lebih sederhana: checkpoint adalah salinan immutable yang dimiliki wrapper Rust, sedangkan Zig tetap membebaskan alokasi melalui registry checkpoint.

Rust memakai `Checkpoint` yang tidak `Send` karena mengikuti kontrak thread-bound engine. `Drop` memanggil `wb_net_checkpoint_destroy`; pointer internal Zig tidak pernah disimpan Rust.

## Noise yang dapat diulang

Loss menggunakan probabilitas integer `loss_per_million`. Jitter menggunakan rentang nanodetik. Keduanya mengambil nilai dari PRNG SplitMix64 milik runtime hanya jika fitur diaktifkan. State PRNG dan penghitung draw/drop/delay ikut masuk checkpoint. Run dengan seed, input, dan checkpoint yang sama harus menghasilkan trace yang sama; perubahan seed adalah eksperimen berbeda.

## Cara menjalankan

```powershell
cargo run --locked -p wyrlex-headless -- --check-phase5
```

Perintah ini membuat dua host, memulai ping, checkpoint saat ARP masih pending, menjalankan home dan fork, restore home, lalu membandingkan summary, packet, dan trace. Setelah itu perintah menguji jitter deterministik dan loss 100% sampai ARP timeout.

## Batas klaim

Hasil ini baru membuktikan replay internal Wyrlex pada model dan fixture yang dinyatakan. `lab_comparison` masih `not_run`; pembandingan Linux lab dan MikroTik adalah pekerjaan P5-05/P5-06 berikutnya. Sukses replay tidak berarti replay lintas versi engine, ASIC, firmware, atau Packet Tracer.

Pertanyaan teach-back: mengapa menyimpan `Summary` saja tidak cukup untuk melanjutkan ping yang masih menunggu ARP?
