# Phase 5: corpus perilaku dan pembanding Linux di WSL

Tanggal: 24 Oktober 2026. [Laporan bukti](../../evidence/corpus-linux/README.md) berbahasa Inggris mencatat hasil aktual. [Corpus](../../../corpus/README.md) menyimpan identitas target, kontrak skenario, dan run yang dapat diperiksa ulang.

## Hubungannya dengan backend engineering

Anggap corpus sebagai pipeline ingestion dengan data yang disimpan secara append-only. `target.json` mengidentifikasi lingkungan yang diuji; `scenario.json` adalah kontrak; `record.json` adalah manifest satu run; `comparison.json` menyimpan assessment yang dapat dihitung ulang dari artifact. Hash mengikat isi artifact ke manifest, tetapi tidak membuktikan pengukuran dari kontributor eksternal itu autentik.

Folder versi tidak cukup diisi `RouterOS 7`. Model hardware, komponen software, exact build, dan tools juga memengaruhi interpretasi. Linux pertama memakai versi kernel dan hash identitas lingkungan. Untuk ZTE kelak, board image/bootloader yang berbeda harus direkam sebagai komponen tersendiri dalam `identity.software`. Belum ada record MikroTik, ZTE, Huawei, atau VSOL yang dianggap terbukti.

## File yang menjalankan eksperimen

- `scripts/corpus/run-linux-lab.ps1` membangun binary Windows, mengekspor input/trace simulator, mencatat hash source/binary, lalu memanggil runner Linux melalui WSL. Path Windows memakai slash agar backslash tidak hilang saat parsing command WSL.
- `scripts/corpus/linux_lab.py` membuat network namespace privat untuk PC1, SW1, R1, dan PC2. Veth menghubungkan namespace; bridge Linux menjadi switch dan kernel Linux melakukan forwarding. Capture worker AF_PACKET berjalan di namespace sumber dan router. Cleanup hanya menghapus namespace yang berhasil dibuat oleh instance lab tersebut.
- `scripts/corpus/compare.py` membaca capture dan membandingkannya dengan hasil simulator pada invariant yang dinyatakan. Tidak ada kesimpulan bahwa nanodetik virtual sama dengan latency Linux.
- `scripts/corpus/validate.py` memeriksa schema, hash, coverage skenario, PCAP versus JSONL, cleanup, dan hasil perbandingan yang dihitung ulang. Ia tidak mengeksekusi source Python yang diarsipkan dalam record.
- `scripts/corpus/test_corpus.py` sengaja merusak artifact atau hasil untuk memastikan validator/comparator menolak false success.

Tidak perlu membangun engine Zig/Rust di Linux untuk eksperimen ini: simulator berjalan pada baseline Windows, sedangkan kernel Linux menjadi pembanding independen. Byte request IPv4 yang sama dikirim melalui raw socket `IP_HDRINCL`. Linux kemudian harus menyelesaikan ARP, meneruskan request, menurunkan TTL, atau membuat ICMP error sendiri.

## Mengapa run pertama tetap disimpan sebagai gagal

Classifier pertama mengasumsikan semua paket IPv4 di capture adalah ICMP dengan IHL 5. Bridge Linux juga mengeluarkan IGMP multicast membership report. Paket itu berada di luar kontrak ICMP, tetapi parser menghitungnya sebagai kegagalan. Akibatnya 4 dari 9 perbandingan ditandai gagal walaupun invariant request/reply yang relevan cocok.

Perbaikannya memakai revisi comparator 2 dan skenario baru yang secara eksplisit mengecualikan serta menghitung IGMP latar belakang. Eksperimen diulang dan 9 perbandingan lulus. Record awal, hasil gagal, dan capture tetap utuh. Ini contoh mengapa mismatch belum otomatis berarti bug firmware: alat ukurnya juga bisa salah.

## Invariant yang diuji

Setiap skenario diulang tiga kali pada namespace baru. Kasusnya ping sehat, route R1 hilang, dan TTL habis. Comparator memeriksa byte ingress, TTL dan byte request yang diteruskan, rewrite MAC router, checksum aktual, ICMP type/code, serta payload atau kutipan error. ID IPv4 response yang dibuat kernel tidak dituntut sama dengan simulator; checksum masing-masing tetap harus valid.

Validator membedakan dua hal: record utuh dan hasil eksperimen pass. Record awal bisa lolos pemeriksaan integritas sambil tetap menyatakan comparison fail. `--require-pass` menambah syarat bahwa eksperimennya sendiri juga harus lulus. Test menghapus kasus, memalsukan hasil, merusak PCAP lalu memperbarui hash-nya, dan mengubah TTL sambil menghitung ulang checksum untuk memastikan pengecekan semantik tidak berhenti pada hash/checksum saja.

## Cara menjalankan

```powershell
./scripts/corpus/run-linux-lab.ps1 -Distro Ubuntu-26.04 -Repetitions 3
wsl -d Ubuntu-26.04 -- python3 /mnt/c/Users/ricky/src/wirebox/scripts/corpus/validate.py --repo /mnt/c/Users/ricky/src/wirebox
```

Ini baru pembanding Linux WSL2 pada subset IPv4/ICMP. G5 keseluruhan dan adapter Phase 6 belum selesai. Materi Cognitive Coverage lokal belum mencakup tambahan corpus dan masih stale; pembuatan dokumen atau test lulus tidak memverifikasi pemahaman manusia.

Pertanyaan teach-back: mengapa record dengan hash yang benar dan checksum paket valid masih bisa berisi hasil perbandingan jaringan yang salah?
