# Boundary audit: bounded transport probe handles

21 Oktober 2026, Zig 0.17.0, Windows x64. Target: `kernel/src/abi.zig`. Implementasi awal file ini sudah Kandidat B. Perubahan production hanya mengganti literal maksimum dengan konstanta yang diturunkan dari layout; lookup, create, dan kebijakan rollover B dipertahankan.

## Invariant dan penurunan batas

`handle: u64 = (u64(generation) << 32) | index`. Index menempati bit 31..0 dan valid dalam `1..session_limit`; generation menempati bit 61..32. Bit 63..62 harus nol.

Karena `handle >> 62 = generation >> 30`, syarat namespace mengharuskan `generation < 2^30`. Jadi maksimum **2^30−1 = 1.073.741.823 = 0x3fffffff**. Generation aktif valid `1..max`; generation nol menandai slot retired dan tidak dipilih create. Empty reusable slot mempunyai context null dan generation nonzero dalam domain itu. Active slot mempunyai context nonnull.

Transisi B saat destroy: context menjadi null; `g<max → g+1`, `g=max → 0`. Generation tidak turun selama reusable dan tidak wrap ke 1. Karena index tetap dan generation lama tidak diulang, handle lama tidak bisa alias context baru. Setelah max, create melewati slot retired. Ini argumen untuk seluruh lifecycle reachable; pengujian tidak mengiterasi satu miliar lifetime.

Check `handle & (u64(1)<<63)` redundan apabila `handle >> 62 != 0` sudah dilakukan. Bit 63 aktif berarti hasil shift adalah 2 atau 3; bit 62 saja menghasilkan 1 dan juga ditolak oleh shift. Menghapus check bit 63 tidak memperlonggar namespace.

## Test yang ditambahkan

Delapan test baru di [abi.zig](../../../kernel/src/abi.zig):

- Lima boundary terpisah: 1, `0x3ffffffe`, `0x3fffffff`, `0x40000000`, `0x7fffffff`; setiap kasus menjalankan lookup input, destroy, create berikutnya, lookup output, dan pemeriksaan namespace/state.
- Layout/index/generation nol, seluruh discriminator asing 01/10/11, serta destroy asing tidak merusak handle sah.
- Riwayat 256 create/destroy pada slot yang sama; semua stale handles diperiksa setelah setiap lifetime, termasuk penolakan submit. Lalu generation dilompati maju ke dua lifetime terakhir, retirement diuji dengan 32 create gagal dan output nol, seluruh riwayat tetap invalid.
- Property sampling deterministik 12.000 nilai: rentang generation valid, lingkungan 512 nilai dekat batas (di bawah dan di atas), dan sampel u32 termasuk namespace asing. Memeriksa rollover/reuse, output namespace, lookup dan stale. B menyelesaikan semua sampel; A berhenti pada counterexample pertama. Ini property sampling, bukan exhaustive fuzzing ruang u32.

State disuntikkan hanya dari test di module yang sama; registry disimpan/dipulihkan dengan defer, termasuk pada assertion gagal. Tidak ada API test yang diekspor. Untuk boundary, slot 1 dibuat **aktif** pada generation yang diuji, slot 2 kosong generation 1, slot lain retired. Maka create berikutnya menunjukkan apakah slot 1 reusable atau dilewati. Untuk dua generation asing, lookup/destroy harus menolak input tanpa mengubah slot; create memilih slot 2.

Angka generation asing disuntikkan untuk menguji penolakan handle. Itu bukan state reusable yang dapat dihasilkan B. Membuat slot **kosong** dengan generation >max melalui korupsi/injeksi internal juga akan membuat `wb_create` yang sama pada kedua kandidat mengeluarkan handle asing. Ketahanan terhadap korupsi internal tersebut bukan klaim kelulusan ini; initialization dan semua transisi B menjaga generation dalam 0..max. Tidak ditambahkan desain recovery state baru.

## Hasil boundary yang dieksekusi

`ok=0`, `invalid_handle=3`, `capacity=4`. Semua handle berikut memakai index 1 kecuali suffix `...00000002` yang memakai fallback slot 2.

| Generation input | Destroy A/B | Stored A | Handle berikut A | Lookup A | Stored B | Handle berikut B | Lookup B |
|---|---|---|---|---|---|---|---|
| `1` | ok / ok | `3` | `0x0000000300000001` | terima | `2` | `0x0000000200000001` | terima |
| `0x3ffffffe` | ok / ok | `0` | `0x0000000100000002` | terima, salah slot | `0x3fffffff` | `0x3fffffff00000001` | terima |
| `0x3fffffff` | ok / ok | `0x40000001` | `0x4000000100000001` | **tolak** | `0` | `0x0000000100000002` | terima |
| `0x40000000` | invalid / invalid | tidak berubah | `0x0000000100000002` | terima | tidak berubah | `0x0000000100000002` | terima |
| `0x7fffffff` | invalid / invalid | tidak berubah | `0x0000000100000002` | terima | tidak berubah | `0x0000000100000002` | terima |

Input lookup menerima tiga baris pertama dan menolak dua baris terakhir pada kedua kandidat. Pada baris terakhir, kondisi A terhadap `0x7fffffff` di destroy tidak pernah tercapai dari handle yang diterima lookup.

**A salah:** kedua assignment dieksekusi berurutan. Pada max−1, assignment pertama menghasilkan max lalu assignment kedua meretire slot sebelum lifetime max digunakan (pelanggaran reuse). Pada max, assignment pertama menghasilkan `0x40000000`, assignment kedua menjadi `0x40000001`; create mengembalikan ok namun bit 62 aktif dan lookup menolak hasilnya (pelanggaran namespace dan validity). Perubahan 1→3 sendiri hanya membuang generation; test tidak menganggap increment dua otomatis melanggar invariant yang hanya meminta generation baru.

**B benar untuk invariant lifecycle registry:** increment sekali sampai max, kemudian retired permanen. Tidak ada ABA ditemukan pada riwayat yang diuji, dan argumen monotonic/no-wrap menutup lifetime yang tidak diiterasi. Kegagalan test riwayat A adalah retirement terlalu awal; hasil ini tidak mengklaim menemukan ABA pada A. A sudah gugur oleh reuse/namespace.

## Reproduksi dan evidence

Dari root repository:

```powershell
./scripts/test-handle-candidates.ps1
./scripts/verify.ps1 -CoreOnly
```

[Runner](../../../scripts/test-handle-candidates.ps1) menyalin source ke `artifacts/handle-generation/A` dan `B`, lalu menyisipkan dua assignment dan check bit 63 persis Kandidat A hanya pada salinan A. Fixture/tests/create sama. Source kerja tidak diganti oleh runner. Mode Debug, ReleaseSafe, dan ReleaseFast memakai assertion test yang sama. Exit 1 A adalah kegagalan invariant yang diharapkan, bukan compiler error; runner juga memeriksa empat kegagalan yang diharapkan dan B harus lulus.

[results.json](results.json) mencatat hasil per mode dan hash source. Log setiap kandidat/mode merekam counterexample boundary. Log [core-verify.log](core-verify.log) merekam verifikasi core setelah penambahan test. Manifest gate lama tidak ditulis ulang.

Hasil aktual pada **ketiga mode**: A 6 passed / 4 failed (exit 1); B 10 passed / 0 failed (exit 0). Sepuluh hasil filtered mencakup delapan test baru, satu stale-handle test lama, dan import aggregator. Verifikasi core lulus: 32 hasil test Zig, 16 unit test Rust, 3 doctest, format/layout/Clippy serta acceptance runtime/network.
