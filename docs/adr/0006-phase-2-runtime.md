# ADR-0006 — Kontrak runtime phase 2

- Tanggal: 20 Oktober 2026
- Status: Accepted; implementasi dan bukti gate ditautkan dari laporan phase 2.

Perluasan 21 Oktober 2026: [ADR-0007](0007-packet-model.md) menambahkan dispatch domain paket dan namespace handle. Kontrak atomic counter firing pada dokumen ini tetap berlaku untuk action counter; kegagalan dispatch network memiliki failed-state tersendiri.

## Keputusan

Runtime Zig menyimpan **simulated state**. Entity device, port, dan link memakai satu namespace ID `u64` monotonic per dunia; nol invalid dan ID tidak dipakai ulang. Role host/switch/router merupakan metadata. Port mempunyai satu owner device. Link point-to-point menghubungkan dua port berbeda yang belum terhubung; koordinat UI tidak masuk graph.

Menghapus device menghapus port miliknya serta link yang menyentuh port tersebut. Menghapus port menghapus link terkait. Semua event/timer milik entity yang dihapus dibatalkan sebelum storage dapat digunakan lagi. Penghapusan link tidak menghapus endpoint.

Scheduler memakai binary min-heap `(time_ns, event_id)`; event ID sekaligus sequence. Registry timer menyimpan satu event pending per timer. Interval nol berarti one-shot; interval positif menjadwalkan firing berikutnya dari waktu firing sebelumnya, dengan ID event baru dan `cause_id` event yang baru dieksekusi. Schedule dari host mempunyai cause nol. Cancel membuang event dari heap secara langsung; tidak menyisakan tombstone yang memakan kapasitas.

Konvensi waktu mengikuti ADR-0003. `runUntil(t, budget)` hanya maju ke t bila semua event due selesai. Jika budget habis dan masih ada event due, hasil `budget_exhausted` dengan waktu event terakhir; budget nol sah. `step` kosong tidak maju. Panggilan bounded dapat diselingi cancel/remove; tidak ada worker atau async interrupt dalam phase ini.

Overflow waktu, ID, counter, dan counter eksekusi diperiksa. Firing periodik yang tidak dapat dijadwalkan ulang ditolak **sebelum** dequeue, perubahan state, PRNG, waktu, atau jurnal. Run dapat telah menyelesaikan event sebelumnya ketika event berikutnya gagal; output progress dan snapshot menunjukkan prefix yang berhasil.

PRNG dipin **SplitMix64-v1**: state awal sama dengan seed; increment wrapping `0x9e3779b97f4a7c15`, mixer `0xbf58476d1ce4e5b9` dan `0x94d049bb133111eb`, shift 30/27/31. Ini sesuai implementasi yang diperiksa pada `lib/std/Random/SplitMix64.zig` Zig 0.17.0 lokal. Sampling `[0, upper)` memakai rejection atas threshold `(0 -% upper) % upper`; upper nol invalid. Seed dan state dapat diinspeksi, tetapi restore/checkpoint lengkap tetap phase 5.

Action phase 2 hanya `add_counter` dan `random_add_counter`: event bertipe untuk membuktikan dispatch/state/PRNG, bukan protokol jaringan. Randomness dikonsumsi saat eksekusi, tidak saat schedule/cancel/read.

Queue, jurnal eksekusi, dan daftar tampilan terpisah. Jurnal menyimpan ID/waktu/target/timer/cause/action/argumen serta counter sebelum/sesudah. Retensi ring dapat 0–256; overwrite menaikkan dropped count tanpa mengubah eksekusi. Read bersifat copy tanpa consume. Event ber-ID besar dapat dieksekusi lebih dahulu karena timestamp lebih kecil; cursor read menggunakan **ordinal eksekusi** `ordinal`. Cause boleh menunjuk event yang sudah terbuang oleh retensi. Jurnal terbatas bukan checkpoint lengkap.

## ABI dan kapasitas awal

Probe ABI v1 tetap tersedia. Runtime memiliki ABI terpisah `wb_sim_*`, versi 1, handle registry terpisah, ownership salinan dan mutex process-wide yang sama pada host Rust. Kontrak field, pointer, error, layout, serta ownership ada pada [runtime phase 2](../wirebox/runtime.md).

Batas eksplisit: 8 dunia, 128 entity per dunia, 256 timer/event pending, 256 record jurnal maksimum. Storage bounded milik Zig, tanpa allocator menyeberangi ABI. `capacity` adalah error yang terlihat, bukan drop diam-diam. Ini batas fondasi phase 2; perlu evaluasi kapasitas sebelum target MVP besar.

## Alternatif dan konsekuensi

Registry linear dan cancellation O(n) cukup untuk batas ini, dengan heap O(log n) pada schedule/pop. Struktur dinamis/index hash dapat ditambahkan setelah kebutuhan skala diukur. Event berupa record memungkinkan evolusi serialization; layout C bukan format persistence. Tidak ada klaim save/load, restore, fidelity vendor, atau simulasi paket dari hasil gate ini.
