# ADR-0001 — Model semantik bersama dan provenance state

- Tanggal: 20 Oktober 2026
- Status: Accepted; arah model, implementasi domain menyusul phase 2/6/7.

## Konteks

Visi pengguna menghubungkan inspeksi, simulasi, diagnosis, dan eksekusi. Menyatukan semuanya sebagai satu nilai “current state” dapat mengubah prediksi menjadi fakta yang tampak teramati.

## Keputusan

Gunakan entity, relation, capability, property, resource, constraint, intent, operation, event, dan observation sebagai vocabulary bersama. Simpan observation asli dengan sumber/waktu; pisahkan observed, desired, dan simulated state. State turunan merujuk evidence dan asumsi. Unknown/stale/conflict tidak diisi diam-diam dari simulasi.

Kernel memiliki state simulasi yang dapat dimutasi. Host/UI mengirim command dan menerima salinan snapshot/trace; koordinat tampilan tidak menjadi koneksi operasional. Operation IR dibagi simulator dan adapter lewat makna serta pre/postcondition, bukan string CLI yang sama.

## Alternatif dan konsekuensi

UI yang langsung mengubah field internal lebih sederhana untuk demo, tetapi menyulitkan headless testing dan replay. Model vendor terpisah sepenuhnya memudahkan integrasi pertama, tetapi menggandakan logika intent. Model bersama dipilih dengan capability extension, sehingga tidak memaksa semua vendor memiliki fitur identik.

Operasi adalah unit logis, tidak menjamin transaksi atomik lintas perangkat. Schema detail belum menjadi API publik; dukungan tipe/relasi ditambahkan bersama fixture dan pengujiannya.

Rujukan: [arsitektur](../wirebox/architecture.md), [visi asli](../packet-tracer/sources/01-wirebox-vision.txt).
