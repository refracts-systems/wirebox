# Phase 2 — Waktu virtual, heap, dan timer yang bisa dibatalkan

**Yang dikerjakan.** [runtime.zig](../../../kernel/src/runtime.zig) memegang jam virtual `u64` nanodetik. Event pending masuk binary min-heap dengan kunci `(time_ns, event_id)`. Timestamp lebih awal menang; pada waktu sama, ID event yang lebih kecil menang. ID saat schedule dan nomor eksekusi tidak selalu sama, karena event yang dijadwalkan dulu dapat memiliki waktu lebih lambat.

Contoh pada [demo G2](../../evidence/phase-2/scheduler-demo.json): urutan event **2, 1, 3, 4, 7** pada waktu **5, 10, 10, 10, 15 ns**. Ini menguji tie-break dan membuktikan trace bukan sekadar daftar sesuai urutan pembuatan.

**Keputusan kode.** Timer mempunyai ID sendiri dan registry untuk cancel. Interval nol berarti one-shot; interval positif menjadwalkan event berikutnya dari waktu firing sebelumnya. Cancel membuang event pending dari heap segera. Saat entity owner dihapus, timer terkait ikut dibatalkan. `schedule` menolak timestamp sebelum `now`; perhitungan waktu dan ID diperiksa dari overflow.

`step()` memproses satu event, sedangkan `runUntil(horizon, budget)` hanya memproses event dengan timestamp ≤ horizon dan berhenti saat budget habis. Jika semua event due selesai, jam maju ke horizon. Jika budget habis, jam tetap pada waktu event terakhir. Event periodik dapat terus menghasilkan pekerjaan; karena itu caller membutuhkan budget dan cancellation, bukan asumsi bahwa queue akan habis.

Action counter 1 menambah amount dan action 2 menambah sampel acak. [RNG SplitMix64](../../../kernel/src/runtime.zig) memiliki seed dan state eksplisit. Sampel dikonsumsi saat event benar-benar firing; schedule, cancel, atau baca jurnal tidak menggeser RNG. Ini membuat hasil fresh run dapat dibandingkan. `serializationDelay` menghitung `ceil(bytes × 8 × 1e9 / bitrate)` dengan intermediate `u128` untuk mencegah overflow perkalian sebelum cek hasil `u64`.

**Mengapa keputusan ini penting.** Waktu virtual membuat urutan independen dari frame rate UI dan kecepatan CPU. Record event membawa ID/argumen, bukan closure native, sehingga alasan jadwal dapat ditelusuri. Heap memilih event berikutnya; registry timer memudahkan cancel by ID. Ini pelajaran yang diambil dari bukti scheduler Packet Tracer, tetapi format dan semantics Wyrlex ditentukan sendiri pada [ADR-0003](../../adr/0003-virtual-time-and-trace.md) dan [ADR-0006](../../adr/0006-phase-2-runtime.md).

**Periksa.** Ikuti `Heap.push`, `Heap.pop`, `Runtime.schedule`, `Runtime.cancel`, `Runtime.step`, dan `Runtime.runUntil`. [Tes G2](../../../kernel/src/runtime_tests.zig) membandingkan heap dengan pemilihan minimum linear independen, menguji same-time, cancel, zero budget, overflow, dan RNG golden. Jalankan `cargo run --locked -p wyrlex-headless -- --check-runtime` untuk matriks fresh run. Itu belum checkpoint restore.
