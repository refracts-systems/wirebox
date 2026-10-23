# Implikasi untuk Wirebox

Bagian ini berisi usulan desain dari diskusi, bukan temuan tentang implementasi Packet Tracer.

Pembahasan produk dan keputusan yang lebih lengkap tersedia pada [dokumentasi Wirebox](../wirebox/README.md). Catatan ini berfokus pada hubungan hasil RE dengan usulan desain.

## Fondasi yang dapat diadaptasi

- Model dunia: perangkat, port, link, konfigurasi, dan state protokol.
- Scheduler waktu virtual dengan urutan event yang stabil dan identitas timer.
- Komponen perilaku protokol yang dapat disusun.
- Trace keputusan dengan packet ID, event ID, dan sebab yang dapat ditelusuri.
- Pemisahan queue eksekusi, jurnal, serta tampilan Event List.

Tidak perlu mengikuti layout memori, multiple inheritance, nama class, atau alamat fungsi PT untuk menerapkan pola tersebut.

## Simulasi mandiri

Perangkat fisik maupun image OS vendor tidak diperlukan untuk mengimplementasikan perilaku yang didefinisikan oleh model sendiri. Perangkat virtual tetap memerlukan state dan aturan perilaku. Pengujian terhadap spesifikasi, OS vendor, atau hardware diperlukan sesuai tingkat kesetaraan yang ingin diklaim.

## Byte tracker

Kebutuhan Wirebox untuk menampilkan setiap byte lebih jauh daripada prototipe yang terutama menyimpan field semantik. Usulan representasinya:

1. Buffer paket sebagai sumber byte.
2. View protokol yang memetakan field ke rentang byte/bit.
3. Versi atau snapshot paket dan catatan perubahan: before, after, event, proses, alasan.

Layout native struct tidak otomatis sama dengan wire format. Endianness, alignment, variable-length header, dan input validation harus ditangani eksplisit.

## Arah bahasa dari diskusi

Rekomendasi sementara adalah Zig untuk kernel/codec dan Rust untuk runtime aplikasi serta UI GPUI. Ini belum diimplementasikan oleh pekerjaan dokumentasi ini.

Zig dapat mengekspor C ABI kecil; Rust membungkusnya dengan kontrak ownership, panjang buffer, error, dan thread yang jelas. Keamanan memory pada implementasi Zig tidak otomatis dibuktikan oleh borrow checker Rust.

## Replay dan hubungan dengan dunia nyata

Replay deterministik memerlukan state perangkat/protokol, event/timer tertunda, buffer, state PRNG, dan input eksternal. Log GUI serta seed saja belum menjamin checkpoint dan restore lengkap.

Observed state, desired state, dan simulated state harus dibedakan. Hasil simulasi tidak otomatis menjadi fakta tentang produksi. Adapter perangkat menjalankan operasi dan membaca kembali hasilnya.

## Tahap pertama yang disarankan

Port skenario PC1–SW1–R1–PC2 ke kernel baru. Mulai dari Ethernet, ARP, IPv4, dan ICMP dengan trace perubahan byte. Tambahkan skenario route hilang, TTL habis, dan pembatalan timer. Gunakan contoh ini untuk menguji model dan UI sebelum memperluas cakupan protokol atau vendor.
