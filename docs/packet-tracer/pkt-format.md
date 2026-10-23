# Format dokumen .pkt: hasil dan koreksi

## Status terakhir dalam laporan

Jalur file dokumen dilaporkan berhasil dilacak dari loader CAppWindow melalui FUN_141d79e20. Jika input sudah dapat diparse sebagai XML, decoder dapat melakukan passthrough.

| Varian | Pipeline yang dilaporkan |
|---|---|
| B, dokumen lebih baru pada sampel | reverse/XOR → Twofish/EAX dengan material konstan → XOR posisi → qUncompress → XML |
| A, legacy pada sampel | XOR posisi → qUncompress → XML |
| Plain | XML langsung |

qUncompress mencakup framing Qt; parser baru harus memeriksa detail framing sebelum memakai inflater zlib biasa. Implementasi mandiri decoder tidak ditulis dalam pekerjaan dokumentasi ini.

Menurut laporan, varian B menggunakan key 16 byte berisi 0x89 dan nonce 16 byte berisi 0x10. Hook FUN_141d77560 dilaporkan membaca buffer tersebut secara langsung. Tidak ada passphrase per-file pada envelope dokumen yang diuji. Password aktivitas dan atribut terenkripsi adalah lingkup berbeda.

## Bukti pengujian yang dilaporkan

- Satu file 3.486.271 byte menghasilkan XML 7.320.358 byte, byte-identical antara capture output proses dan reader offline.
- Sweep 411 file: 400 didekode, terdiri dari 344 varian B, 50 varian A, dan 6 plain XML; 11 tidak didekode pada sweep itu.
- Kategori dokumen yang disebut: 293/293 .pkt, 88/88 .ptd, dan 6/6 .pka menghasilkan XML well-formed.
- Keystream dari input berbeda dilaporkan memiliki prefix identik, menguatkan penggunaan stream yang sama pada keluarga dokumen tersebut.

Jumlah per kategori ekstensi yang disebut tidak merinci seluruh 400 output. Catatan lanjutan menyebut 13 extension container, berbeda dari 11 pada sweep sebelumnya. Inventaris/manifest corpus diperlukan untuk menyatukan lingkup angka ini.

## Batas reader

Reader masih memakai captured keystream sepanjang 3.486.255 byte. Percobaan mereproduksi Twofish/EAX secara mandiri belum cocok. Oleh karena itu, frasa historis 'fully recovered' harus dibaca bersama batas ini.

Ukuran yang melebihi cakupan stream belum didukung oleh artifact tersebut. Keberhasilan capture/plaintext juga belum membuktikan pemeriksaan authentication tag, pembuatan tag untuk writer, atau round-trip yang diterima PT. Kebutuhan itu tetap terbuka.

Hasil ini tidak berarti Wirebox membutuhkan .pkt untuk berjalan. Importer adalah jalur input tambahan; engine dapat menerima format topologi sendiri.

## Koreksi jalur kriptografi

Temuan AES-CBC, SHA-256/HMAC, dan Mash 2500 iterasi pada DataDecryptor<Rijndael,...> tidak boleh dipakai sebagai penjelasan envelope .pkt. Addendum §7 menggantikan hipotesis §4b/§4c untuk container dokumen.

FUN_141d5c710/FUN_141d5c880 yang semula disebut constructor decryptor dikoreksi sebagai constructor exception. Detail fungsi kriptografi lama tetap disimpan pada arsip untuk konteks, bukan sebagai spesifikasi decoder akhir.

## Metadata dan extension

Pasangan key/nonce 16 × 0xBA dan 16 × 0xBE dilaporkan digunakan pada uuid/nets. Hipotesis bahwa pasangan itu merupakan keluarga extension telah ditarik kembali.

.pts/.pkp/.pta yang diuji tetap belum berhasil didekode. Gagal pada percobaan tertentu dan tidak terlihat pada observasi startup tidak membuktikan semua kemungkinan jalur decoder telah dieliminasi. Entropy tinggi dan tidak adanya magic yang dikenal belum mengidentifikasi cipher atau formatnya.

## Artifact asal

Laporan menyebut pktread.ps1, ks_universal.bin, probe12.js, probe14.js, kswork.ps1, serta output Ghidra dan decoded XML. Artifact executable, key stream, dan corpus XML tidak disalin dalam pekerjaan dokumentasi ini.

Sumber: [ringkasan addendum akhir](sources/04-pkt-addendum.txt) dan [laporan lengkap §7](sources/REPORT_addendum.original.md).
