# Hardening Phase 0–3: cara membaca perubahan sebagai backend engineer

23 Oktober 2026. Baca [laporan hardening](../evidence/hardening-phase-0-3/README.md) untuk hasil command. Launcher pribadi `cognitive-coverage/cognitive-coverage-open.html` menghubungkan guide, kuis, dashboard dan manifest jika tersedia lokal; foldernya diabaikan Git dan tidak ikut clone. Ini pendalaman fondasi yang sudah ada; Phase 4 tetap belum dikerjakan.

## Apa yang ditambahkan dan mengapa

| Perubahan | Analogi backend | Yang perlu kamu bisa jelaskan |
|---|---|---|
| Revalidasi route sebelum workload | Deferred constraint pada commit transaction | Mengapa valid saat addRoute belum menjamin valid setelah setPort? |
| Namespace/generation registry runtime dan network | Optimistic-lock version dan stale object reference | Mengapa slot pada generation maksimum harus retired, bukan wrap? |
| Model linear untuk schedule/cancel/step | Reference model bagi priority job queue | Mengapa test yang memakai heap produksi sebagai oracle bisa menyembunyikan bug? |
| Mutation corpus codec dengan kontrol valid | Parsing request untrusted | Mengapa 4.000 input yang semuanya ditolak terlalu dini belum cukup menguji field map? |
| Zig Debug + ReleaseSafe pada verifier | Pengujian lintas konfigurasi compiler | Mengapa cargo --release belum berarti Zig ikut release? |

## Temuan nyata: validasi bukan satu titik untuk selamanya

Di [network.zig](../../kernel/src/network.zig), `addRoute` memastikan gateway ada di subnet port saat route dibuat. Namun konfigurasi masih editable: `setPort` bisa mengubah IP atau prefix sesudahnya. Contohnya gateway `10.0.0.2`, port awal `10.0.0.1/24`, lalu port diubah menjadi `10.0.1.1/24`. Setiap operasi tampak valid secara lokal, tetapi kombinasi akhirnya invalid.

Test baru membuktikan bahwa sebelum patch, `ping` menerima kombinasi itu. `Network.validate` sekarang mengecek ulang route sebelum `ping` atau `inject` mengubah `started=true`. Assertion memeriksa bukan hanya error: `started/failed` tetap false, packet count dan queue tetap nol, trace belum bertambah, lalu perbaikan konfigurasi dapat diterapkan dan ping dimulai kembali.

Ini mirip menjaga business constraint lintas row saat commit. Namun jangan menyebut seluruh engine transactional: `Runtime.step` counter mempersiapkan pekerjaan fallible sebelum commit, sedangkan `beginDomain` mengambil event sebelum network dispatcher berjalan. Jika dispatcher kehabisan resource, state bisa parsial dan `failed=true`; retry langsung bukan rollback.

## Cara memakai komentar kode

Komentar sebaiknya menjelaskan **alasan, invariant, dan konsekuensi**, bukan menerjemahkan baris berikutnya. Pada perubahan ini, komentar di `Network.validate` menjelaskan mengapa constraint diperiksa lagi dan mengapa pemeriksaan harus sebelum `started=true`. Komentar generation menjelaskan kaitan retirement dengan ABA. Komentar test menyebut mengapa state boundary disuntikkan, bukan melakukan satu miliar create/destroy.

Pedoman ketika kamu mengembangkan kode:

- Zig: letakkan alasan pada titik commit, ownership, bound, atau arithmetic yang tidak jelas; jelaskan kapan overflow dibolehkan seperti PRNG versus kapan harus error seperti waktu/ID.
- Rust FFI: `// SAFETY:` harus menyebut fakta yang membuat call sah: handle owned, lock, buffer aligned/initialized, kapasitas, lifetime dan siapa yang melepas memori. Kata “safe” saja bukan pembuktian.
- Test: jelaskan adversarial setup dan oracle independen. Assertion menunjukkan output; komentar menerangkan mengapa input itu penting.
- Docs: simpan penjelasan panjang, tradeoff, contoh kegagalan dan pertanyaan belajar di sini. Hindari paragraf tutorial pada setiap fungsi produksi.

Contoh komentar yang ada di kode: `All fallible work precedes mutation, including the periodic reschedule.` Pertanyaan lanjutnya: apakah itu berlaku pada jalur network juga? Jawabannya harus mengikuti `beginDomain` dan dispatcher, bukan menyalin asumsi dari `step` counter.

## Apa arti generated

Skill [Cognitive Coverage upstream](https://github.com/ryannadel/cognitive-coverage) dipasang pada project di [.agents/skills/cognitive-coverage](../../.agents/skills/cognitive-coverage/SKILL.md), pada commit yang dicatat di [UPSTREAM.md](../../.agents/skills/cognitive-coverage/UPSTREAM.md). MCP optional tidak dipasang karena artifact pembelajaran dapat bekerja sendiri.

Empat output utama di dalam `cognitive-coverage/` disimpan hanya secara lokal dan tidak ditautkan sebagai file GitHub:

1. `learning-guide.html`: sembilan bagian berisi source snippet aktual, mental model, warning, tiga difficulty lenses, tiga depth layers dan 18 soal; tiap kombinasi level menampilkan dua soal.
2. `cognitive-coverage.json`: inventaris source, konsep, alur, source hash, dependency area dan gap pendalaman. Hash merekam baseline saat materi dihasilkan dan perlu diperiksa kembali setelah source atau dokumentasi berubah.
3. `cognitive-coverage.html`: dashboard files/concepts/flows/gap, status manual, hasil kuis dan export/import JSON.
4. `cognitive-coverage-open.html`: pintu masuk portabel untuk ketiga artifact lain.

Semua status pemahaman awal adalah **uncovered**. Agent membaca source dan membuat materi tidak membuktikan kamu sudah memahami source. Jawaban kuis dapat menaikkan status item yang dipetakan; itu indikator pembelajaran, bukan sertifikasi atau jaminan seluruh file dikuasai. Status manual adalah self-assessment. Browser menyimpan progress lokal bila tersedia; export JSON untuk mencadangkan progress secara lokal. HTML tidak menulis file source secara otomatis.

Large Corpus Mode dipakai karena kernel, wrapper, CLI, UI dan arsip docs mempunyai kebutuhan belajar berbeda. Overview memprioritaskan FFI, handle, scheduler, jurnal, codec, config dan ping. Pendalaman GPUI, CLI/oracle, serta seluruh laporan RE belum selesai diajarkan. Source yang diinventarisasi bukan berarti semuanya sudah diaudit mendalam.

## Jalur belajar dan teach-back

Mulai pada intermediate / standard. Untuk setiap bagian, baca snippet source dan jelaskan kembali tanpa melihat jawabannya:

1. Telusuri satu `Network::ping` Rust sampai `Network.ping` Zig. Siapa memegang lock, siapa memiliki bytes, dan apa status jika handle stale?
2. Gambarkan generation 1 → 2 → … → max → retired. Jelaskan mengapa dua bit namespace mengurangi generation menjadi 30 bit.
3. Bedakan queue, timer registry, journal dan snapshot. Data apa yang belum disimpan sehingga snapshot belum checkpoint?
4. Sebutkan byte TTL dan checksum IP sebelum/sesudah router. Mengapa payload ICMP tidak berubah?
5. Jelaskan batas atomicity: konfigurasi ditolak sebelum commit versus dispatch gagal setelah event dikonsumsi.

Setelah itu jawab kuis, Submit ke dashboard, dan export progress. Untuk pembelajaran yang lebih kuat, coba ubah satu assertion di salinan lokal, prediksi test mana yang gagal, lalu jalankan. Jangan menaikkan status hanya karena membaca jawaban.
