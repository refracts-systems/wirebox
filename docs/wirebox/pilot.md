# Pembuktian ide dan rencana pilot Wirebox

Disusun 20 Oktober 2026. Dokumen ini menentukan apa yang perlu dibuktikan, bukan menyatakan pilot sudah berhasil. Pelaksanaan mengikuti [roadmap](roadmap.md) dan [todos](todos.md).

Target perangkat berasal dari pengguna: Linux lab, MikroTik RouterOS, OLT ZTE, ONU Huawei, dan ONU VSOL. Angka keberhasilan di bawah dibekukan pada [kontrak phase 0 v1](phase-0-contract.md) sebagai acceptance criteria; belum merupakan hasil pengukuran atau janji performa. [Hasil G0/G1](../evidence/phase-0-1/README.md) hanya mencakup development lokal dan stack probe.

## Hipotesis yang diuji

| Hipotesis | Bukti yang dicari | Milestone |
|---|---|---|
| H1 — Simulator dapat berjalan mandiri dan menjelaskan paket | Skenario sukses/gagal, byte, keputusan, dan state yang sesuai pembanding | G3 / PoC |
| H2 — Penjelasan itu berguna dalam workflow pengguna | Pengguna menyelesaikan diagnosis, perubahan, dan perbandingan melalui UI | G4 / MVP |
| H3 — Model semantik dapat menghubungkan simulasi dengan sistem nyata | Intent/operation yang sama secara semantik berjalan pada simulator dan Linux/MikroTik lalu diverifikasi | G7 / pilot inti |
| H4 — Model dapat menangani layanan pelanggan ISP | Workflow teruji dari ZTE/ONU sampai router/endpoint pada matriks perangkat nyata | G8 / pilot ISP |

Demo visual mendukung presentasi hasil, tetapi keputusan berhasil memakai artifact serta expected result yang dapat diperiksa. H1/H2 tidak otomatis membuktikan H3/H4.

Status 21 Oktober 2026: H1/G3 lulus untuk subset headless dengan fixture manual beku dan oracle Rust terpisah; [bukti](../evidence/phase-3/README.md). Capture Linux pembanding G5 belum tersedia. H2/G4 dan pilot H3/H4 belum diuji; keberhasilan simulator belum meluluskan pilot perangkat.

## Tiga lingkungan pembuktian

| Lingkungan | Cakupan | Dependency yang harus dicatat |
|---|---|---|
| A — Simulator mandiri | PC1–SW1–R1–PC2; Ethernet/ARP/IPv4/ICMP dan routing statis | Revisi engine/model, fixture, seed, clock semantics, batas byte/link |
| B — Linux/MikroTik lab | Endpoint terkontrol, jalur IPv4 statis, dan satu operasi route | Linux distro/kernel, RouterOS versi/model, interface, return path, akses manajemen, capability |
| C — ISP lab | ONU pelanggan uji → ZTE → uplink → MikroTik → endpoint uji | Model/firmware OLT dan kedua keluarga ONU, PON mode, VLAN/service mapping, operasi yang tersedia |

Lingkungan B dapat memakai namespace/container/VM/perangkat lab sesuai akses yang ada; containerlab adalah opsi orkestrasi, bukan syarat keberhasilan Wirebox. Lingkungan C dimulai dengan endpoint terkontrol sebelum pengujian tujuan internet yang memiliki dependency tambahan.

Inventaris menyimpan sumber observation, timestamp, serta identitas target tanpa credential. Keluarga vendor yang disebut pengguna tidak dianggap sudah kompatibel satu sama lain. Uji Huawei dan VSOL sebagai dua baris matriks tersendiri.

## Lima tugas pengguna untuk MVP

| ID | Tugas | Bukti selesai |
|---|---|---|
| U1 | Buka skenario sehat dan jalankan ping | Jalur, state, dan hasil dapat dilihat tanpa mengedit source |
| U2 | Diagnosis kasus route hilang | Pengguna menunjukkan perangkat, lookup, dan alasan drop yang benar |
| U3 | Jelaskan perubahan paket pada router | Pengguna menemukan TTL, checksum, header hop berikutnya, dan rentang byte terkait |
| U4 | Ubah route pada salinan skenario dan bandingkan | Hasil baru berubah sesuai expected result; skenario asal tetap utuh |
| U5 | Simpan, tutup, buka kembali hasil | Konfigurasi, hasil yang didukung format, trace/byte yang dibutuhkan, serta versi dapat diperiksa ulang |

Pengguna boleh memakai runbook. Kebutuhan bantuan dicatat; keberhasilan yang memerlukan perubahan source atau penjelasan lisan pembuat untuk menemukan hasil belum memenuhi target kemandirian U1–U5.

## Cerita demo pilot inti

1. Observe lab Linux/MikroTik dan simpan snapshot dengan provenance serta asumsi state yang tidak teramati.
2. Tunjukkan kondisi gagal akibat route lab yang hilang menggunakan probe serta observation independen.
3. Buat intent reachability dengan sumber, tujuan, arah, protokol, return path, dan capability yang eksplisit.
4. Planner menghasilkan operation IR; UI memperlihatkan target, perubahan, precondition, dan expected effect.
5. Fork snapshot dan jalankan operation pada simulator. Periksa hasil, trace kausal, serta perubahan paket.
6. Periksa drift, lalu jalankan plan pada lab yang ditetapkan melalui adapter yang sesuai.
7. Baca kembali route dan jalankan probe dari endpoint. Catat sukses hanya ketika postcondition terpenuhi.
8. Jalankan intent yang sama lagi: jangan menghasilkan duplikasi route. Tunjukkan juga stale plan dan kegagalan komunikasi yang terdeteksi.
9. Bandingkan hasil manual dan Wirebox, lalu simpan satu bundle pembuktian.

Ulangi kontrak operasi pada Linux dan RouterOS. Backend dapat memakai transport/sintaks berbeda, tetapi precondition serta makna hasilnya harus sama. Paket real lab dapat dibandingkan dengan prediksi hanya pada field/invariant yang memang dimodelkan.

## Matriks skenario minimum

| ID | Kasus | Expected result | Gate |
|---|---|---|---|
| S01 | Topologi sehat | Echo reply dan jalur sesuai fixture; byte/field tervalidasi | G3, G4 |
| S02 | Route hilang | Alasan gagal benar; perubahan route pada branch memulihkan hasil | G3, G4, G7 |
| S03 | TTL habis | Drop/error sesuai subset ICMP; perubahan TTL/checksum dapat diperiksa | G3 |
| S04 | Next-hop tidak menjawab | ARP/timer berakhir menurut kontrak; run berbatas selesai dan alasan terlihat | G3 |
| S05 | Paket terpotong atau fitur unsupported | Input ditolak atau hasil unsupported/unknown; tidak crash atau pass palsu | G3, G4 |
| S06 | Checkpoint saat buffer/timer aktif | Restore + lanjut cocok dengan state dan event run tanpa restore | G5 |
| S07 | Seed/input sama, termasuk gangguan | Trace kanonis dan hasil dapat diulang pada build/lingkungan sama | G5 |
| S08 | Observation stale, timeout, atau bertentangan | Provenance terlihat; state tidak diam-diam dianggap mutakhir atau pasti | G6 |
| S09 | Intent route diulang | Postcondition tetap benar tanpa duplikasi; hasil no-op/changed dapat dibedakan | G7 |
| S10 | State berubah setelah plan | Drift/precondition mismatch terdeteksi sebelum operation yang mengandalkannya | G7 |
| S11 | Apply gagal atau koneksi putus setelah pengiriman | Status failed/partial/indeterminate sesuai bukti; read-back sebelum retry; tidak mengarang sukses | G7 |
| S12 | Service/VLAN pelanggan uji bermasalah | Diagnosis sesuai capability; plan, apply, dan probe memulihkan jalur yang dimodelkan | G8 |
| S13 | ONU offline/LOS atau telemetry optik tidak tersedia | Evidence dan dependency tampil; tidak memprediksi pulih melalui operasi yang tidak menyelesaikan sebabnya | G8 |

Kasus S12 dipilih setelah service mapping dan operasi diketahui. Jika memakai PPPoE/NAT atau fitur lain di luar model, tutup gap dan tambah uji sebelum menyatakan simulasi end-to-end lulus. Menyatakan unsupported adalah perilaku yang benar, tetapi tidak dihitung sebagai keberhasilan reachability untuk layanan itu.

## Scorecard keberhasilan baseline v1

| Dimensi | Target awal | Cara mengukur |
|---|---|---|
| Kebenaran PoC | S01–S05 sesuai expected result; tidak ada mismatch kritis terbuka | Fixture byte/codec, probe/capture pembanding, dan pemeriksaan alasan keputusan |
| Kegunaan MVP | U1–U5 selesai dengan runbook tanpa source edit | Catat hasil, waktu, bantuan, dan titik kebingungan setiap tugas |
| Reproduksibilitas | Skenario S06/S07: tiga pengulangan untuk masing-masing lima seed terpilih, tanpa perbedaan di dalam seed yang sama | Bandingkan state dan trace kanonis; keluarkan wall-clock metadata, bukan event virtual atau byte penting |
| Konsistensi pilot inti | Seluruh kasus yang berlaku untuk G7 benar pada Linux dan RouterOS, masing-masing tiga pengulangan | Plan/operation ID, state diff, read-back dan probe independen; mismatch punya penjelasan/issue |
| Kejujuran hasil | Nol status sukses palsu pada kasus gagal/unsupported yang diuji | Bandingkan status aplikasi dengan evidence dan postcondition |
| Manfaat workflow | Pada minimal empat dari lima tugas diagnosis/perubahan yang sebanding, waktu lebih pendek atau interaksi manual berkurang tanpa menurunkan kebenaran | Tetapkan lima tugas dan cara menghitung interaksi pada P0-06; catat baseline manual, hasil Wirebox, serta bantuan |
| Respons MVP | Pada fixture empat node dan 10.000 row trace tersimpan, respons selection/pause p95 di bawah 100 ms pada mesin referensi | Ukur dari input sampai tampilan/state yang relevan terbarui; sertakan biaya engine/FFI/render, bukan update UI saja |
| Stabilitas awal | Sesi scripted 30 menit tanpa crash/hang; memori tetap di dalam budget retensi yang ditetapkan | Catat penggunaan memori dan ukuran journal/buffer; budget dibekukan setelah baseline awal, sebelum acceptance run |
| Cakupan ISP | S12/S13 serta pemeriksaan isolasi layanan lolos pada pasangan ZTE–Huawei dan ZTE–VSOL yang dicantumkan | Matriks model/firmware, observation layanan uji dan layanan pembanding, serta probe hasil |

Angka di atas adalah kriteria engineering/pilot kecil, bukan estimasi reliabilitas seluruh produksi atau kesetaraan semua vendor. Jika mesin atau fixture berubah, catat ulang konteks dan jangan membandingkan angka tanpa penyesuaian.

Untuk perbandingan manual, gunakan tugas berpasangan dengan tingkat kesulitan setara dan catat urutan pengerjaan agar efek belajar tidak disalahartikan sebagai manfaat aplikasi. Pengguna awal boleh pemilik proyek; validasi engineer lain menjadi gate beta G9.

## Paket bukti setiap gate

Hasil lokal G0/G1 tersedia pada [phase-0-1](../evidence/phase-0-1/README.md); invariant graph/scheduler serta reproduksibilitas counter model pada [G2](../evidence/phase-2/README.md). Pengujian G2 belum meluluskan S01–S13 atau scorecard pilot jaringan. Run berikutnya dapat memakai direktori seperti `docs/evidence/<gate>/<run-id>/`. Simpan ringkasan dan artifact yang diperlukan, bukan dump credential atau binary vendor ke Git.

Record minimum:

| Field | Isi |
|---|---|
| Gate/task | G0–G9 atau gate epic G10, serta ID task terkait |
| Lingkungan | Revisi source/build, versi model/schema, toolchain/OS, mesin uji |
| Target | Model/firmware, capability, topology/fixture hash, dan endpoint |
| Input | Skenario, intent/plan/operation ID, seed, serta gangguan |
| Expected | Hasil dan asal pembanding yang ditetapkan sebelum run |
| Actual | State, trace/byte/capture relevan, read-back/probe, serta metrik pengguna/performa |
| Result | Belum diuji / lulus / gagal / tertahan; mismatch dan batas yang tersisa |

Hash fixture/trace digunakan sesuai format kanonis yang ditetapkan. Jumlah row GUI tidak dianggap jumlah callback scheduler. Capture lab dibandingkan pada invariant yang didukung: nilai field, keputusan forwarding, checksum, dan hasil. Jangan menuntut kesamaan timestamp, MAC, initial TTL, atau ID yang berbeda tanpa terlebih dahulu menetapkan mapping dan normalisasi yang sah.

## Aturan keputusan lanjut

- **Lulus** ketika seluruh kasus wajib gate benar, bukti dapat diperiksa, dan manfaat pengguna untuk milestone terkait terukur. Hasil teknis lulus dengan manfaat pengguna belum tercapai dicatat terpisah; pilot penuh belum lulus.
- **Gagal** ketika ada mismatch kritis, sukses palsu, crash yang merusak workflow, atau state/byte yang tidak konsisten. Perbaiki sebelum memperluas klaim.
- **Tertahan** ketika akses, kompatibilitas, firmware, atau capability belum tersedia. Kerjakan fase independen yang masih berguna; jangan menandai bukti yang belum diperoleh sebagai lulus.
- **Persempit scope** bila diperlukan, dengan revisi acceptance criteria yang tercatat sebelum acceptance run berikutnya. Jangan menghapus kasus gagal diam-diam agar pilot tampak berhasil.

Jika MVP berguna tetapi bridge ke sistem nyata belum lulus, Wirebox baru terbukti sebagai studio simulasi. Jika G7 lulus, ide inti terbukti untuk kontrak dan perangkat pilot tersebut. G8 memperluas pembuktian ke workflow ISP milik pengguna.
