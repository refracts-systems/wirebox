# C ABI v1 — transport probe phase 1

Kontrak normatif: [wirebox.h](../../kernel/include/wirebox.h). Implementasi: [Zig](../../kernel/src/abi.zig) dan [wrapper Rust](../../crates/wirebox-kernel/src/lib.rs). Alasan desain: [ADR-0002](../adr/0002-zig-rust-abi.md).

Dokumen ini khusus probe. API graph/scheduler tersedia terpisah sebagai [runtime ABI v1](runtime.md), protokol melalui [network ABI v1](packets.md). Revisi phase 3 pada ADR-0007 memakai dua bit atas handle: probe 00, counter 10, network 11, dengan generation 30 bit. Ini menggantikan pembagian satu bit/31 bit phase 2; handle tidak dipersist atau ditransfer antarproses.

## API dan error

| Fungsi | Kontrak |
|---|---|
| `wb_abi_version` | Mengembalikan 1; query tanpa mutasi |
| `wb_create(version, out_handle)` | Membuat session kosong; out_handle wajib valid dan diset nol pada kegagalan version/capacity |
| `wb_destroy(handle)` | Menutup session, menghapus antrean, menginvalidasi generation; destroy kedua menghasilkan invalid_handle |
| `wb_probe_submit(handle, data, length)` | Copy sinkron 1–64 byte; enqueue satu record kind=1; input tidak disimpan sebagai pointer |
| `wb_probe_read(handle, out, capacity, out_count)` | Copy dan consume sampai min(queue length, capacity), capacity 0–16; capacity nol tidak menguras antrean |

Status `u32`: 0 ok, 1 invalid_argument, 2 abi_mismatch, 3 invalid_handle, 4 capacity. Output count diset nol jika pointer count valid sebelum validasi handle/buffer. Submit gagal tidak mengonsumsi event ID. Antrean penuh tidak menimpa data. ID event mulai 1 per session, tanpa waktu virtual.

`wb_probe_event`: id u64 offset 0, kind u32 offset 8, length u32 offset 12, bytes[64] offset 16. Total 80 byte, alignment 8 pada target Windows x64. Byte setelah length diinisialisasi nol. Ini format ABI in-process, **bukan** wire format jaringan atau file persistence.

## Ownership, pointer, dan thread

Registry memiliki 16 slot; antrean tiap session memiliki 16 record. Handle berisi generation dan index; slot dengan generation exhausted dipensiunkan. Registry memakai penyimpanan statis bounded tanpa heap allocation atau deallocator lintas bahasa pada tahap ini.

Caller harus menjamin pointer non-null valid, aligned, cukup panjang, dan tidak overlap dengan output lain. Null data invalid; null out hanya diperbolehkan jika capacity nol. API tidak menjanjikan recovery dari arbitrary pointer atau memory corruption. Header dan wrapper harus berubah bersama jika layout berubah.

C caller wajib menserialisasi semua panggilan registry secara process-wide dan tidak melakukan reentrancy. Rust wrapper menegakkan serialisasi dengan mutex dan membatasi Session pada thread pembuatnya; session lain tetap melewati mutex yang sama. Kernel tidak memanggil host, tidak unwind ke Rust, dan tidak menyimpan pointer caller.

`Session::read_probe` mengembalikan `Vec<ProbeEvent>` milik Rust; hasil tetap valid setelah mutasi, read berikutnya, atau close. `Session::close` memberikan hasil destroy; Drop menjadi fallback tanpa panic. Batas queue/handle adalah batas probe, bukan desain kapasitas simulator phase 2/3.

## Verifikasi yang diwajibkan phase 1

Layout C/Zig/Rust; payload tetap benar setelah caller mengubah input; snapshot bertahan sesudah owner ditutup; partial batch menjaga urutan; zero capacity tidak menguras; input invalid tidak mengubah queue; versi salah dan null output ditolak; handle lama tidak mengakses slot baru. Compile-fail doctest membuktikan wrapper tidak dapat dipindahkan lewat `thread::spawn`.
