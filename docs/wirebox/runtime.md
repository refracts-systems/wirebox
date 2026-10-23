# Runtime phase 2 — graph, waktu virtual, dan jurnal

Implementasi: [graph Zig](../../kernel/src/graph.zig), [runtime Zig](../../kernel/src/runtime.zig), [C ABI](../../kernel/include/wirebox_sim.h), dan [wrapper Rust](../../crates/wirebox-kernel/src/simulation.rs). Keputusan normatif: [ADR-0003](../adr/0003-virtual-time-and-trace.md) dan [ADR-0006](../adr/0006-phase-2-runtime.md). Hasil pengujian: [G2](../evidence/phase-2/README.md).

API runtime pada dokumen ini menjalankan simulated state dan event counter. [Layer network phase 3](packets.md) memakai graph/heap/timer yang sama dan menambahkan perilaku host/router/switch melalui API terpisah. Koordinat, label tampilan, konfigurasi IP, observed/desired state, dan adapter tidak dimasukkan ke graph dasar. Nama fixture dipetakan ke ID oleh host.

## Menjalankan

```powershell
cargo run --locked -p wirebox-headless -- --scheduler-demo
cargo run --locked -p wirebox-headless -- --check-runtime
./scripts/verify.ps1
```

Demo membaca relasi graph dari fixture `icmp-static-v1`: 4 device, 6 port, 3 link. Runner menjadwalkan penambahan counter, timer random periodik, timer yang dibatalkan, dan event pada objek sementara yang kemudian dihapus. Output JSON memuat trace dan hasil; ini belum menjalankan S01–S05 atau mengirim ICMP.

Dengan seed 20260117, urutan event adalah `2, 1, 3, 4, 7`, waktunya `5, 10, 10, 10, 15` ns. Urutan tersebut menunjukkan bahwa ID schedule dan ordinal eksekusi berbeda. Counter akhir PC1=1, SW1=104, R1=12, PC2=0. Timer periodik menghasilkan sampel 33 lalu 71; event 7 menunjuk cause 4. Batas run menghentikan eksekusi pada 15 ns. Setelah timer dibatalkan, horizon kosong dapat maju ke 100 ns.

## Graph dan lifecycle

ID entity `u64` mulai 1, monotonik dalam satu World, tidak didaur ulang; nol invalid. ID pada World lain tidak dapat digunakan sebagai identitas global. `kind`: 1 device, 2 port, 3 link. Role device: 1 host, 2 switch, 3 router.

- Device: `owner`, `a`, `b` nol.
- Port: `owner` menunjuk device; `a`, `b`, `role` nol.
- Link: `a` dan `b` menunjuk dua port berbeda; satu port maksimum satu link.
- `counter` adalah state sintetis phase 2, bukan jumlah paket/interface statistics.

Device yang dihapus menghapus port miliknya dan link yang menyentuh port tersebut. Port yang dihapus menghapus link terkait. Link yang dihapus mempertahankan endpoint. Timer untuk setiap entity yang terhapus langsung dibatalkan. Membuat entity baru pada slot kosong tidak menghidupkan ID lama.

## Waktu, event, dan pembatalan

Tick `u64` nanodetik, awal nol. Urutan heap `(time_ns, event_id)`; ID event mulai 1 dan menjadi tie-break sequence. Timestamp masa lalu ditolak. ID maksimum disisihkan untuk mendeteksi exhaustion sebelum increment; nilai waktu maksimum tetap sah untuk one-shot jika aksi tidak overflow.

`step()` menjalankan satu event paling awal. Antrean kosong mengembalikan reason `empty` tanpa memajukan jam. `run_until(horizon, budget)` menjalankan event `<= horizon`; setelah semua due selesai, jam menjadi horizon. Budget nol sah. Jika budget habis sementara event due masih ada, jam tetap pada event terakhir dan reason `budget_exhausted`. Event setelah horizon tetap pending.

Timer memiliki ID tersendiri, registry, dan satu event pending. Interval nol = one-shot. Interval positif = periodik; firing berikutnya pada `previous_time + interval`, event ID baru, cause event sebelumnya. Cancel membuang event pending langsung dari heap. Timer yang selesai/dibatalkan menghasilkan `not_found` bila dibatalkan lagi. Host dapat menyisipkan cancel/remove di antara run bounded; belum ada interrupt worker.

Action 1 `add_counter` menambahkan amount. Action 2 `random_add_counter` menambahkan sampel `[0, amount)`, amount harus positif. SplitMix64-v1 dengan seed/state eksplisit dipin pada ADR-0006; PRNG hanya dikonsumsi saat firing berhasil. State PRNG tidak bergantung waktu nyata, kecepatan GUI, jadwal thread, atau read jurnal.

Overflow pada waktu ulang, sequence/ID, jumlah eksekusi, atau counter menghasilkan `overflow`. Firing gagal tidak mengubah queue/state/jam/PRNG/jurnal. Run yang sebelumnya sudah menyelesaikan event lain tetap mempertahankan prefix tersebut; `RunError` Rust memuat `error` dan `progress`. Event gagal tetap dapat diperiksa melalui konteks jadwal pemanggil lalu dibatalkan, atau objeknya dihapus. Belum ada API inspeksi seluruh queue.

Helper delay serialisasi menggunakan ceil(bytes × 8 × 1e9 / bitrate) dengan intermediate u128 dan output checked u64. Pengiriman frame phase 3 memakai helper ini; batas wire representation dijelaskan pada [model paket](packets.md).

## Queue, jurnal, dan visible list

Heap menyimpan pekerjaan yang belum dieksekusi. Jurnal ring menyimpan eksekusi yang sudah berhasil, dengan `ordinal` mulai 1, event ID/waktu/target/timer/cause/action/amount, serta counter sebelum/sesudah. Cancel, schedule, dan penghapusan dari host tidak menjadi record jurnal pada tahap ini; karena itu jurnal ini belum event-sourced world atau audit semua command.

Retensi 0–256 dapat dipilih saat create. Full ring menimpa record tertua dan menaikkan `dropped`; retensi nol tetap menjalankan event. `trace(after_ordinal, capacity)` menyalin record tertahan ber-ordinal lebih besar dari cursor, tidak mengonsumsi. Cursor nol membaca dari awal yang masih tersedia. Gap diketahui lewat ordinal pertama dan snapshot `dropped`; cause dapat menunjuk record yang sudah hilang.

Visible list merupakan proyeksi host dari salinan jurnal. Demo membaca proyeksi satu row; test membandingkan read setiap step dengan read sekaligus. UI GPUI masih memakai probe phase 1; kontrol runtime masuk phase 4. Trace terbatas dan hasil ulang dari seed sama belum membuktikan checkpoint/restore phase 5.

## Kontrak C ABI runtime v1

API `wb_sim_*` versi 1 independen dari probe `wb_*` v1. Revisi [ADR-0007](../adr/0007-packet-model.md) memakai namespace dua bit atas (probe 00/counter 10/network 11), generation 30 bit, dan index+1 untuk mencegah akses stale/lintas registry; generation exhausted dipensiunkan. Ini menggantikan pembagian satu bit/31 bit pada baseline G2. Registry maksimal 8 World. Setiap World maksimal 128 entity, 256 pending event/timer, 256 record jurnal. Keterbatasan ini terlihat lewat error, bukan drop event diam-diam.

| Fungsi | Perilaku |
|---|---|
| `wb_sim_version` | Query versi 1 |
| `wb_sim_create` | Version, seed, retention; handle nol pada error jika output valid |
| `wb_sim_destroy` | Tutup World dan invalidasi generation |
| `wb_sim_add` | Device/port/link; ID nol pada kegagalan, input invalid tidak mengambil ID |
| `wb_sim_remove` | Hapus entity dengan cascade/cancel |
| `wb_sim_schedule` | Copy spec, kembalikan timer ID; interval nol one-shot |
| `wb_sim_cancel` | Batalkan satu timer aktif |
| `wb_sim_step`, `wb_sim_run_until` | Copy progress termasuk prefix pada error |
| `wb_sim_snapshot` | Copy metadata runtime; bukan checkpoint lengkap |
| `wb_sim_entities` | Copy graph lengkap atau capacity error; Rust mengurutkannya menurut ID |
| `wb_sim_trace` | Copy batch setelah ordinal, non-consuming; capacity nol no-op |

Status `u32`: 0 ok, 1 invalid_argument, 2 abi_mismatch, 3 invalid_handle, 4 capacity, 5 not_found, 6 conflict, 7 past_time, 8 overflow. Progress reason: 0 empty, 1 horizon, 2 budget_exhausted, 3 stepped, 4 error. Error selain run tidak menjanjikan progress. Null pada output wajib invalid; output count/ID diinisialisasi nol jika pointer valid. Snapshot diinisialisasi nol sebelum validasi handle.

Semua pointer harus valid, aligned, cukup panjang, non-overlapping. Null array hanya sah ketika capacity nol. Capacity entity maksimal 128, trace maksimal 256. Output entity terlalu kecil mengembalikan capacity tanpa salinan parsial. Semua input disalin saat call; semua output milik caller. Tidak ada pointer World, allocator, callback, atau container native melewati ABI.

C caller wajib menserialisasi seluruh panggilan ketiga registry process-wide. Wrapper Rust menggunakan mutex yang sama dengan Session probe dan Network, dan World adalah `!Send + !Sync`. Hasil Vec/struct tetap hidup setelah World ditutup; close eksplisit atau Drop menghancurkan World satu kali. Lock meliputi satu call bounded; caller harus memilih budget yang sesuai respons aplikasi.

Layout Windows x64, alignment 8:

| Record | Ukuran byte | Field/offset utama |
|---|---:|---|
| Entity | 48 | id/owner/a/b/counter u64 @0/8/16/24/32; kind/role u32 @40/44 |
| Event | 56 | id/time/target/timer/cause/amount u64 @0..40; action/reserved u32 @48/52 |
| Trace | 80 | Event @0; ordinal/before/after u64 @56/64/72 |
| Schedule | 40 | target/time/interval/amount u64 @0..24; action/reserved u32 @32/36 |
| Progress | 24 | now/processed u64 @0/8; reason/reserved u32 @16/20 |
| Snapshot | 80 | 8 u64 @0..56; 4 count u32 @64..76 |

Reserved harus nol. ABI in-process ini bukan format paket, file persistence, atau janji kompatibilitas lintas endianness/versi. Snapshot menyimpan now, seed, rng_state, next_event_id, next_timer_id, executed, dropped, last_event_id, entity/pending/timer/journal count; tidak memuat semua queue, konfigurasi, atau state untuk restore.
