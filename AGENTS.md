# Panduan kerja repository Wirebox

## Tujuan proyek

Wirebox adalah rancangan studio dan runtime infrastruktur berbasis model semantik bersama. Tujuannya menghubungkan inspeksi sistem nyata, simulasi, penelusuran paket dan state, perencanaan perubahan, serta eksekusi melalui adapter perangkat.

Baca [indeks dokumentasi](docs/README.md) dan [apa itu Wirebox](docs/wirebox/README.md) sebelum mengubah arah arsitektur. Untuk pekerjaan spesifik, baca dokumen terkait saja; tidak perlu memuat seluruh arsip RE.

## Status dan sumber kebenaran

Pada 21 Oktober 2026, phase 0–3 tersedia: kontrak/fixture, probe Zig–Rust, spike GPUI, runtime waktu virtual, dan simulator paket Ethernet/ARP/IPv4/ICMP dengan routing statis serta tracker byte headless. Baca [kontrak runtime](docs/wirebox/runtime.md), [model paket/ABI](docs/wirebox/packets.md), dan [bukti G3](docs/evidence/phase-3/README.md). GPUI masih menampilkan probe; integrasi paket adalah phase 4. Driver/perbandingan perangkat nyata belum tersedia. Mini-simulator C++ adalah referensi eksternal; jangan menyamakannya dengan engine Wirebox.

- [Keputusan dan pertanyaan terbuka](docs/wirebox/decisions.md) membedakan preferensi pengguna, rekomendasi, dan hal yang belum dipilih.
- [ADR](docs/adr/README.md) mencatat keputusan arsitektur yang diambil; [evidence G0/G1](docs/evidence/phase-0-1/README.md) membatasi klaim hasil aktual. Command build/verify tersedia pada [README](README.md).
- [Roadmap](docs/wirebox/roadmap.md) berisi tahapan yang diusulkan, bukan daftar fitur selesai atau jadwal yang dijanjikan.
- [Todos](docs/wirebox/todos.md) mencatat checklist per phase; [rencana pilot](docs/wirebox/pilot.md) menetapkan skenario dan kriteria bukti. Saat implementasi dimulai, kaitkan task dengan hasil verifikasi dan jangan meluluskan gate hanya dari checklist kode.
- [Catatan Packet Tracer](docs/packet-tracer/README.md) merangkum laporan RE beserta koreksinya.
- Isi `docs/packet-tracer/sources/` adalah arsip historis. Klaim di dalamnya dapat sudah digantikan oleh koreksi pada catatan tematik.

Instruksi pengguna dalam sesi aktif mengendalikan pekerjaan. Jangan menganggap rekomendasi dalam docs sebagai keputusan pengguna yang tidak boleh diubah. Perbarui status dokumentasi saat implementasi atau keputusan berubah.

## Cara bekerja dan berkomunikasi

- Gunakan bahasa Indonesia yang jelas untuk diskusi dan penjelasan dokumentasi. Pertahankan nama API, identifier, istilah protokol, serta kutipan sumber sebagaimana diperlukan.
- Periksa status Git, file yang relevan, dan instruksi lokal sebelum mengedit. Pertahankan pekerjaan pengguna yang sudah ada.
- Kerjakan tugas yang sudah diotorisasi sampai selesai. Pilihan rutin dan perubahan lokal yang dapat dibalik tidak memerlukan konfirmasi tambahan.
- Jangan membuat commit, push, atau publikasi tanpa permintaan atau otorisasi pengguna yang sesuai.
- Jangan menambah dependensi, scaffold, atau engine hanya karena dokumen menyebut rencana implementasinya.
- Laporkan hasil, cara verifikasi, dan keterbatasan yang benar-benar memengaruhi hasil. Jangan mengklaim command atau pengujian telah dijalankan jika belum.

## Arah desain yang perlu dipertahankan

- Utamakan entity, relation, state, behavior, constraint, dan time sebagai model domain; sintaks vendor ditangani adapter.
- Bedakan observation mentah, observed state, desired state, dan simulated state. Setiap hasil turunan perlu asal data dan batas cakupan model.
- Hasil simulasi hanya berlaku pada perilaku yang dimodelkan. Jangan mengklaim kesetaraan firmware, ASIC, hardware, atau semua versi Packet Tracer tanpa bukti pembanding.
- Kernel simulasi harus bisa dijalankan tanpa UI dan perangkat fisik. UI mengonsumsi state/trace dan mengirim command melalui batas yang jelas.
- Pisahkan antrean scheduler, jurnal eksekusi, dan daftar event yang terlihat di GUI.
- Untuk replay, catat urutan event, konfigurasi, input eksternal, versi model, serta state PRNG dan timer. Animasi mundur bukan bukti restore state lengkap.
- Untuk tracker, pertahankan byte paket, pemetaan field ke byte/bit, versi paket, dan sebab perubahan. Jangan menggunakan layout struct native sebagai wire format.

Rincian: [model dan runtime](docs/wirebox/architecture.md), [simulasi](docs/wirebox/simulation.md), dan [byte tracker](docs/wirebox/byte-tracker.md).

## Bahasa dan batas antarkomponen

Implementasi phase 1–3 memakai Zig untuk engine, Rust untuk host/wrapper/oracle, dan GPUI langsung untuk UI. Kontrak serta alasan ada di ADR-0002/0004/0007. GPUix + Solid tetap alternatif historis, bukan stack aktif. Hasil smoke check tidak dianggap benchmark atau bukti bahwa semua kebutuhan MVP sudah terpenuhi.

- Baca [pilihan teknologi](docs/wirebox/technology.md) sebelum memilih atau mengganti stack.
- Baseline versi yang ditetapkan pengguna: Zig minimal **0.17.0**, Rust **1.99.1**. Rust dipin pada `rust-toolchain.toml`; untuk prototipe Zig gunakan 0.17.0 sebagai baseline dan catat versi persis yang diuji. Jangan menurunkan target agar cocok dengan toolchain lokal yang lebih lama. Versi Zig lebih baru memerlukan pengujian kompatibilitas, bukan diasumsikan kompatibel.
- Jika memakai Zig–Rust FFI, tulis kontrak ownership, ukuran buffer, lifetime, error, versi ABI, dan thread sebelum memperluas API.
- Jangan melewatkan container internal Zig/Rust sebagai C ABI. Gunakan representasi yang ABI-nya jelas dan handle opaque bila sesuai.
- Memori harus dilepas oleh allocator/pemilik yang sesuai. Borrow checker Rust tidak memverifikasi implementasi Zig.
- Hindari panic/unwind melintasi FFI. UI tidak boleh menyimpan pointer engine yang dapat invalid setelah langkah simulasi berikutnya.
- Pilih dan pin versi toolchain/dependensi setelah memeriksa dokumentasi serta build pada platform target. Jangan mengarang versi atau command build yang belum ada.

## Disiplin bukti dan dokumentasi

- Bedakan **dilaporkan**, **diverifikasi lokal**, **inferensi**, **usulan**, dan **unknown**. Gunakan definisi pada [catatan PT](docs/packet-tracer/README.md).
- Untuk temuan baru, catat target/versi/hash bila tersedia, metode, input, hasil, tanggal, dan batas pengamatan.
- Alamat fungsi dan offset PT berlaku pada binary laporan asal. Nama RTTI dan string flow-chart tidak memulihkan seluruh algoritme.
- Jangan menghidupkan kembali hipotesis lama AES/Mash sebagai penjelasan akhir container `.pkt`; baca [koreksi format](docs/packet-tracer/pkt-format.md).
- Pertahankan salinan sumber yang sudah diarsipkan beserta hash-nya. Tambahkan revisi sumber sebagai artifact baru dan perbarui manifest, alih-alih mengedit arsip historis diam-diam.
- Evidence/manifest gate lama merekam baseline pada waktu run. Jangan menulis ulang hash G0/G1 agar seolah memvalidasi kode phase berikutnya; buat evidence baru.
- Simpan penjelasan terbaru pada dokumen tematik dan tautkan ke sumber. Hindari duplikasi panjang yang bisa saling bertentangan.
- Pakai tautan relatif antarfile di Markdown repository. Beri tanggal pada pemeriksaan teknologi eksternal yang dapat berubah.

## Pembelajaran dan cognitive coverage

- Skill project: [.agents/skills/cognitive-coverage/SKILL.md](.agents/skills/cognitive-coverage/SKILL.md); provenance/pin pada [UPSTREAM.md](.agents/skills/cognitive-coverage/UPSTREAM.md). Output awal: [launcher](cognitive-coverage/cognitive-coverage-open.html).
- Untuk perubahan kode yang berarti, jelaskan file yang dihasilkan/diubah, invariant dan alasan keputusan, command yang benar-benar dijalankan, serta satu pertanyaan teach-back. Hubungkan dengan konsep backend bila membantu, tanpa menyamakan semua error dengan rollback transaksi.
- Komentar kode menjelaskan why, ownership, bounds dan titik commit; `SAFETY` Rust menyatakan precondition konkret FFI. Tutorial panjang berada di [learning docs](docs/learning/README.md).
- Jangan menandai pemahaman manusia verified karena agent membuat materi atau test lulus. Gap UI/CLI/vendor tetap terlihat. Setelah source berubah, periksa sourceHash pada manifest; refresh artifact yang terdampak atau nyatakan materi stale.
- Checker artifact: `node scripts/check-cognitive-coverage.mjs`. Pemeriksaan ini terpisah dari test engine dan tidak menjadi ukuran pemahaman manusia.

## Verifikasi

Untuk perubahan docs saja, periksa tautan lokal, konsistensi status, dan integritas arsip jika tersentuh. Tidak perlu membuat test suite untuk perubahan prosa.

Build baseline Windows: `./scripts/verify.ps1 -CoreOnly` untuk kernel/runner, atau `./scripts/verify.ps1` untuk seluruh workspace. UI interaction tetap pengujian terpisah. Jangan menandai workflow CI remote lulus hanya karena file YAML tersedia.

Ketika kode tersedia, gunakan command dari manifest/build system yang benar-benar ada. Pilih pengujian sesuai perubahan: fixture byte untuk codec, urutan dan cancellation untuk scheduler, skenario jaringan untuk perilaku protokol, serta ownership/error path untuk FFI. Verifikasi UI dan build platform dilakukan ketika komponen itu ditambahkan.

Jangan memakai jumlah baris trace sebagai jumlah event scheduler, keberhasilan demo sebagai bukti conformance, atau benchmark proyek lain sebagai benchmark Wirebox.
