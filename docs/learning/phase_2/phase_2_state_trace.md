# Phase 2 — State, queue, jurnal, dan arti “replay”

**Yang dikerjakan.** Runtime mempunyai tiga konsep yang harus dibaca terpisah: graph/counter adalah **state sekarang**; heap adalah **pekerjaan nanti**; jurnal adalah **rekaman firing yang sudah berhasil**. UI kelak membuat visible event list dari salinan jurnal. [Runtime phase 2](../../wirebox/runtime.md) menjelaskan kontrak setiap bagian.

**Keputusan kode.** `Runtime.append` menaruh trace dalam ring berkapasitas 0–256. Ketika penuh, row tertua ditimpa dan `dropped` bertambah; event dalam heap tidak ikut dibuang. `Runtime.read(after_ordinal, out)` mengembalikan salinan tanpa consume. `ordinal` naik menurut urutan eksekusi, sedangkan `event_id` ditetapkan saat schedule. `cause_id` dapat menunjuk event yang row-nya sudah keluar dari ring; gap harus terlihat dari ordinal/dropped.

Firing counter memeriksa risiko overflow sebelum dequeue dan mutasi, termasuk timer periodik yang akan perlu dijadwalkan lagi. Jika satu event gagal, event itu tetap dapat ditangani caller; firing sebelumnya dalam satu `runUntil` tetap menjadi prefix hasil yang sah. `RunError` Rust membawa progress. Ini menghindari klaim salah bahwa satu call multi-event selalu atomic.

**Mengapa retention tidak boleh mengubah hasil.** Jurnal hanya catatan, bukan input bagi scheduler. Jika `read` atau batas tampilan ikut mengubah queue/RNG, dua pengguna dengan pengaturan UI berbeda akan memperoleh simulasi berbeda. [Acceptance G2](../../evidence/phase-2/README.md) membandingkan lima seed, rerun, dan retensi 0/1/3/256: state, clock, ID, timer, RNG, serta hasil cocok; yang berubah hanya row yang masih disimpan.

**Batas replay.** Membaca row lama atau memutar animasi mundur tidak mengembalikan graph, counter, queue, timer, dan RNG ke masa lampau. `Snapshot` phase 2 hanya metadata; checkpoint/restore lengkap adalah pekerjaan phase 5. Pertanyaan pemeriksaan yang baik: “kalau aplikasi ditutup pada event 7, cukupkah row trace untuk melanjutkan dari event 7?” Saat ini jawabannya belum, karena state dan pekerjaan tertunda tidak disimpan lengkap.

**Periksa.** Baca `Runtime.append`, `Runtime.read`, `Runtime.snapshot`, lalu [wrapper simulation.rs](../../../crates/wirebox-kernel/src/simulation.rs). Jalankan `--scheduler-demo` dan bandingkan `executed=5` dengan jumlah row serta `event_id` berbeda dari `ordinal`. Pada phase 3, [jurnal domain paket](../phase_3/phase_3_tracker_pembuktian.md) menambah decision row tetapi tetap bukan queue atau checkpoint.
