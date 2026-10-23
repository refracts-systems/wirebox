# Todos Wirebox per phase

Diperbarui 21 Oktober 2026. Urutan dan milestone: [roadmap](roadmap.md). Skenario serta scorecard: [pilot](pilot.md). Target pada [kontrak v1](phase-0-contract.md) tetap berlaku; hasil aktual harus ditautkan per gate.

`[x]` berarti artifact/hasil tersedia; `[ ]` berarti belum dibuktikan selesai. G0/G1 lulus lokal pada 20 Oktober 2026; [bukti dan batas scope](../evidence/phase-0-1/README.md). G2 lulus untuk graph/scheduler/counter model; [bukti G2](../evidence/phase-2/README.md). G3 lulus lokal Windows x64 debug pada 21 Oktober 2026 untuk subset paket headless; [bukti G3](../evidence/phase-3/README.md). G4–G9 belum diuji, G10 mempunyai gate per epic. Inventaris akses nyata tidak dianggap terverifikasi oleh G0.

## Phase 0 — Batas pembuktian dan kesiapan

- [x] P0-01 — Simpan visi, sumber RE, koreksi, dan batas bukti pada [docs](../README.md).
- [x] P0-02 — Pasang dan periksa Zig 0.17.0, Rust 1.99.1, serta compiler smoke checks; catat di [teknologi](technology.md).
- [x] P0-03 — Catat target pilot pengguna: Linux lab, MikroTik RouterOS, OLT ZTE, ONU Huawei dan VSOL.
- [x] P0-04 — Catat model/firmware/PON mode serta unknown pada [inventaris](../../fixtures/inventory/pilot-v1.json) dan [excerpt tersanitasi](../../fixtures/inventory/zte-c320-user-report.txt). Scope [ADR-0005](../adr/0005-pilot-scope.md): akses/model yang belum tersedia harus dilengkapi sebelum G6/G8, bukan dianggap teruji.
- [x] P0-05 — Bekukan scope MVP, subset intent IPv4 statis, serta unsupported pada [kontrak v1](phase-0-contract.md).
- [x] P0-06 — Bekukan fixture empat node/lima kasus, oracle byte/checksum independen, metode perbandingan, serta scorecard di [kontrak v1](phase-0-contract.md).
- [x] P0-07 — Simpan [mesin referensi](../evidence/phase-0-1/environment.json), protokol artifact, source manifest, dan keputusan [G0](../evidence/phase-0-1/README.md).

Gate G0: scope, inventaris minimum lab awal, skenario, serta expected result dapat ditinjau sebelum implementasi diperluas. Inventaris ISP dapat dilanjutkan untuk G8 tanpa menahan kernel.

## Phase 1 — Build dan pembuktian stack

- [x] P1-01 — Struktur kernel Zig, runner headless, Rust host, dan [batas ownership](ffi.md) tersedia; state queue probe hanya dimutasi Zig.
- [x] P1-02 — [Cargo build script](../../crates/wirebox-kernel/build.rs) membangun Zig 0.17.0 dengan Rust 1.99.1 dan Windows MSVC; dependency dikunci Cargo.lock.
- [x] P1-03 — Create/destroy serta copy batch terbukti; [kontrak ABI v1](ffi.md) mendefinisikan ownership/lifetime/error/thread/layout.
- [x] P1-04 — GPUI 0.2.2 menampilkan window, node/link terpilih, hex, input, dan shutdown; [hasil runtime dan screenshot](../evidence/phase-0-1/README.md).
- [x] P1-05 — Zig/Rust tests memeriksa stale/null/error/capacity, copy/lifetime, serta thread-bound session; close UI menghancurkan session; [hasil](../evidence/phase-0-1/verification.log).
- [x] P1-06 — [Build guide](../../README.md), [verifier lokal](../../scripts/verify.ps1), source export baru dengan cache, serta workflow CI tersedia. [Local run lulus](../evidence/phase-0-1/README.md); remote CI belum dijalankan.

Gate G1: build serta FFI/GPUI smoke check dapat diulang. Memasang compiler saja belum memenuhi gate ini.

## Phase 2 — Graph, state, scheduler, dan trace dasar

- [x] P2-01 — [Graph Zig](../../kernel/src/graph.zig): ID monotonik device/port/link, validasi relasi, cascade delete; tanpa koordinat UI. [Uji](../../kernel/src/runtime_tests.zig).
- [x] P2-02 — [ADR-0006](../adr/0006-phase-2-runtime.md) melengkapi tick ns, overflow, tie-break, empty step/horizon, serta progress budget/error; implementasi [runtime](runtime.md).
- [x] P2-03 — Binary heap, registry timer, eager cancel, run bounded, lifecycle entity/timer di [runtime Zig](../../kernel/src/runtime.zig); [bukti](../evidence/phase-2/README.md).
- [x] P2-04 — SplitMix64-v1 dengan seed/state eksplisit dan golden tests; [acceptance 5 seed](../evidence/phase-2/acceptance.json). Jam tidak membaca wall clock/GUI.
- [x] P2-05 — Queue, jurnal ring, dan proyeksi tampilan terpisah; event/cause ID serta ordinal eksekusi; retensi 0/1/3/256 tidak mengubah state. [Kontrak](runtime.md).
- [x] P2-06 — Same-time, cancel, stale/deleted owner, empty horizon, timer periodik berbatas, overflow, serta ownership ABI diuji; [hasil lengkap](../evidence/phase-2/verification.log).

Gate G2: invariant scheduler lolos dan urutan dapat diulang; kapasitas/retensi trace tidak mengubah hasil simulasi.

## Phase 3 — PoC paket dan byte tracker headless

- [x] P3-01 — [Codec Zig](../../kernel/src/codec.zig), salinan versi immutable, protocol view byte/bit, bounds/checksum, serta copy ABI; [kontrak](packets.md).
- [x] P3-02 — Frame arrival memakai heap phase 2, serialisasi per egress + propagation checked; [batas link](../adr/0007-packet-model.md) dan oracle timing terpisah.
- [x] P3-03 — [Network Zig](../../kernel/src/network.zig): Ethernet, MAC learning/flood/unicast, ARP cache/lookup/reply/buffer/retry; multiport flooding diuji.
- [x] P3-04 — IPv4, connected/static LPM, TTL/checksum, echo dan ICMP 3/0 serta 11/0; [acceptance](../evidence/phase-3/acceptance.json).
- [x] P3-05 — Trace route/cache/drop, versi byte, packet/transmission/event IDs; [walkthrough actual bytes](../evidence/phase-3/byte-walkthrough.md).
- [x] P3-06 — S01–S05 dan healthy static alias lulus; 36 run untuk batch/step/repeat/retention, golden header serta oracle checksum/field/quote/timing Rust terpisah. [Bukti G3](../evidence/phase-3/README.md); capture Linux tetap G5.
- [x] P3-07 — Truncation, checksum, unsupported options/fragment/protocol/ICMP/VLAN/MTU, invalid MAC, suppression error ICMP, serta failure resource/ABI; [hasil verifikasi](../evidence/phase-3/verification.log).

Gate G3 / PoC: alur sukses serta gagal dapat dijelaskan hingga byte melalui runner headless; tidak memerlukan UI, hardware, atau `.pkt`.

## Phase 4 — MVP Studio yang dapat dipakai

- [ ] P4-01 — Hubungkan Rust/GPUI dengan kernel melalui command dan batch snapshot/trace konsisten; hindari FFI per byte.
- [ ] P4-02 — Tampilkan topology canvas dengan pan/zoom/selection, inspector perangkat, timeline, protocol tree, dan hex view yang saling terhubung.
- [ ] P4-03 — Tambahkan run/pause/step/reset dengan batas eksekusi dan pembatalan; UI tetap merespons.
- [ ] P4-04 — Sediakan form address/route/link state dan branch konfigurasi awal tanpa mengubah skenario asal.
- [ ] P4-05 — Tambahkan save/load skenario dan hasil dengan versi schema; tolak versi tidak didukung dengan pesan jelas.
- [ ] P4-06 — Jalankan lima tugas pengguna pada [pilot](pilot.md) dari aplikasi tanpa mengedit source atau mengandalkan log tersembunyi.
- [ ] P4-07 — Ukur respons input, jumlah row trace, dan stabilitas pada mesin referensi; simpan demo serta batas MVP.

Gate G4 / MVP: workflow diagnosis–perubahan–perbandingan dapat diselesaikan melalui UI, termasuk membuka kembali hasil tersimpan.

## Phase 5 — Replay, gangguan, dan pembanding

- [ ] P5-01 — Buat checkpoint state protokol/topologi, queue/timer, paket, ID, dan PRNG; catat input eksternal serta versi engine/model.
- [ ] P5-02 — Implementasikan restore, lanjutkan eksekusi, dan fork checkpoint; state asal dan cabang tetap terpisah.
- [ ] P5-03 — Tambahkan loss/jitter terkontrol dan catat input gangguan; ulangi dengan seed sama dan berbeda sesuai skenario.
- [ ] P5-04 — Uji checkpoint ketika ARP/timer/buffer masih aktif; bandingkan state dan kelanjutan dengan run tanpa restore.
- [ ] P5-05 — Buat fixture lab Linux dan capture/probe pembanding; gunakan containerlab bila berguna, tanpa menjadikannya prasyarat kernel.
- [ ] P5-06 — Bandingkan invariant forwarding/byte yang didukung; tetapkan perbedaan yang diizinkan dan laporkan mismatch.
- [ ] P5-07 — Tetapkan retensi/checkpoint dan perilaku log tidak lengkap; simpan laporan G5.

Gate G5: replay identik pada lingkungan yang dinyatakan dan perilaku sesuai pembanding dalam scope. Ini belum menjamin replay lintas semua versi engine.

## Phase 6 — Observe Linux dan MikroTik

- [ ] P6-01 — Lengkapi fixture/version matrix Linux serta RouterOS dari perangkat lab yang tersedia.
- [ ] P6-02 — Buat kontrak adapter discover/observe/capabilities dengan identitas dan provenance stabil.
- [ ] P6-03 — Implementasikan pembacaan topology/interface/address/route yang dibutuhkan pada Linux dan MikroTik; simpan observation terpilih sebagai fixture tersanitasi.
- [ ] P6-04 — Tangani timeout, izin tidak cukup, stale data, conflicting observation, dan capability yang tidak tersedia.
- [ ] P6-05 — Bentuk snapshot observed state, pisahkan desired/simulated state, lalu fork ke simulator beserta asumsi state yang tidak tertangkap.
- [ ] P6-06 — Bandingkan prediksi dengan probe/capture lab; tampilkan sumber serta waktu observation pada inspector.

Gate G6: snapshot kedua keluarga target dapat diperiksa asal-usulnya dan dipakai untuk skenario yang dinyatakan; adapter observasi tidak menulis konfigurasi.

## Phase 7 — Pilot inti: intent → simulate → execute → verify

- [ ] P7-01 — Tetapkan intent IPv4 statis, endpoint uji, return path, capability, dan kontrak operation; gunakan route lab yang tidak memutus manajemen.
- [ ] P7-02 — Implementasikan planner ke operation IR dan pre/postcondition; tampilkan plan serta state diff yang dapat diperiksa.
- [ ] P7-03 — Terapkan operation yang sama secara semantik di simulator, adapter Linux, dan RouterOS; simpan mapping lowering/backend.
- [ ] P7-04 — Tambahkan drift check sebelum apply, identitas run/operation, deadline, status parsial/indeterminate, dan pemeriksaan state sebelum retry.
- [ ] P7-05 — Implementasikan read-back serta probe hasil; definisikan kompensasi operasi pilot beserta batasnya.
- [ ] P7-06 — Uji baseline sehat, route hilang lalu dipulihkan, intent berulang tanpa duplikasi, stale plan, dan kegagalan komunikasi pada setiap backend.
- [ ] P7-07 — Bandingkan workflow manual dan Wirebox pada tugas yang sama; isi scorecard tanpa menganggap simulasi sebagai bukti eksekusi sukses.
- [ ] P7-08 — Tulis hasil G7 dengan plan, artifact, mismatch, dan scope yang benar-benar terbukti.

Gate G7 / pilot inti: kontrak semantik dan workflow lengkap terbukti pada Linux serta MikroTik lab. Jika baru satu backend lulus, catat pilot parsial; pembuktian lintas backend belum lengkap.

## Phase 8 — Pilot ISP: ZTE, ONU Huawei, dan VSOL

- [ ] P8-01 — Inventaris model/firmware OLT ZTE, PON mode, model/firmware ONU Huawei/VSOL, dan pasangan interoperabilitas yang benar-benar teruji.
- [ ] P8-02 — Baca PON/ONU identity, state, VLAN/service mapping, uplink, alarm, dan optik yang tersedia; tandai field unsupported/unknown per model.
- [ ] P8-03 — Petakan ONU → PON → service/VLAN → uplink → MikroTik → endpoint uji, beserta sumber dependency serta observation.
- [ ] P8-04 — Pilih satu layanan dan gap model yang wajib ditutup: VLAN/tagging, service mapping, atau fitur lain yang dipakai. Tambahkan fixture/model sebelum mengklaim simulasi jalur tersebut.
- [ ] P8-05 — Pilih satu perubahan layanan yang dapat diuji beserta precondition, lowering, read-back, serta pemulihannya; verifikasi kemampuan ONU melalui jalur yang tersedia.
- [ ] P8-06 — Uji workflow sehat, gangguan layanan terkontrol, perubahan cabang simulasi, apply pada layanan uji, serta konektivitas dan kondisi layanan lain.
- [ ] P8-07 — Ulangi pada Huawei dan VSOL secara terpisah; tulis batas model/firmware dan hasil pasangan yang tidak kompatibel.
- [ ] P8-08 — Pisahkan observasi LOS/daya optik dari prediksi model; simpan laporan pilot dan keputusan G8.

Gate G8 / pilot ISP: satu workflow layanan terbukti pada matriks yang diuji. Jika satu keluarga ONU belum lolos, dukungan keluarga itu tetap belum terbukti.

## Phase 9 — Beta dan penggunaan berulang

- [ ] P9-01 — Buat paket Windows, panduan install/upgrade/uninstall, serta verifikasi dependensi dan lisensi distribusi.
- [ ] P9-02 — Terapkan credential storage sesuai platform, sanitasi export/log, dan pemisahan hak observe/execute yang dibutuhkan pilot.
- [ ] P9-03 — Uji recovery setelah crash/disconnect, version migration, retensi, backup, dan ekspor bundle diagnosis.
- [ ] P9-04 — Ukur kapasitas topologi/event serta memori; tambahkan culling/virtualization/batching berdasarkan profiler.
- [ ] P9-05 — Jalankan tugas bersama engineer lain memakai runbook; catat kegagalan, kebingungan, waktu, serta langkah manual.
- [ ] P9-06 — Dokumentasikan compatibility matrix, batas dukungan, jalur laporan bug, serta hasil sesi pilot terpisah.
- [ ] P9-07 — Tentukan scope beta inti atau beta ISP berdasarkan gate yang lulus dan hasil penggunaan.

Gate G9: aplikasi dapat dipasang, dipakai, dipulihkan, serta diuji ulang oleh orang selain pembuat pada scope yang didokumentasikan.

## Phase 10 — Future, dipilih menurut kebutuhan

Setiap task berikut adalah calon epic. Pecah menjadi kontrak, checklist, fixture, dan gate sebelum mulai; tidak semuanya wajib dikerjakan.

- [ ] P10-01 — Tambah UDP/TCP bertahap: demultiplexing, port unreachable, handshake, data, loss/retransmisi, close, lalu fidelitas tambahan sesuai kebutuhan.
- [ ] P10-02 — Perluas model jaringan: VLAN lanjutan, NAT, firewall, PPPoE, IPv6, dynamic routing; uji tiap fitur serta interaksinya.
- [ ] P10-03 — Perluas vendor/firmware dan layanan ISP; terbitkan capability serta hasil pembanding per adapter.
- [ ] P10-04 — Tambah reconciliation dan repair planning dengan kebijakan eksekusi eksplisit; ukur hasil terhadap drift nyata.
- [ ] P10-05 — Tambah versioned infrastructure state: diff, branch/merge dengan konflik domain, incident bisect, dan replay compatibility yang dinyatakan.
- [ ] P10-06 — Eksplorasi query/diagnosis/planning AI melalui API semantik yang sama; evaluasi pada insiden berlabel dan pertahankan kontrak operation pada jalur mutasi.
- [ ] P10-07 — Bangun kolaborasi, remote runner, integrasi CI, serta Linux/macOS jika kebutuhan pengguna membenarkannya.
- [ ] P10-08 — Evaluasi importer `.pkt`, OpenConfig, dan format/model lain; pisahkan interoperabilitas dari fidelity serta batas reader yang diketahui.
- [ ] P10-09 — Optimalkan skala/parallelism/penyimpanan berdasarkan bottleneck terukur sambil mempertahankan kontrak determinisme.
- [ ] P10-10 — Evaluasi domain server/PLC/UPS/IoT melalui primitive bersama dengan model dan pembuktian domain tersendiri.

Gate G10: ditetapkan per epic; satu demo tidak meluluskan seluruh kemampuan future. Jika fitur di daftar ini dibutuhkan pilot terpilih, pindahkan subset yang diperlukan beserta pengujiannya ke phase terkait secara eksplisit.

## Pekerjaan pertama

Pekerjaan berikutnya: **P4-01**, integrasi kernel paket ke Rust/GPUI, lalu workflow inspector/run/branch/save-load. P0–P3 sudah memiliki bukti lokal. [Hardening 23 Oktober 2026](../evidence/hardening-phase-0-3/README.md) menambah validasi dan pengujian tanpa meluluskan G4 atau pembanding vendor G5. Lengkapi data akses/firmware secara terpisah sebelum P6/P8.

Untuk task selesai, tambahkan tautan perubahan/artifact dan hasil verifikasi. Untuk gate, gunakan record hasil pada [pilot](pilot.md). Rencana ini belum merupakan otorisasi mengubah perangkat di luar lab/layanan yang nantinya ditetapkan.
