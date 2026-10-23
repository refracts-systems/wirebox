# Audit awal Packet Tracer dan baseline pembanding

Run lokal **21 Oktober 2026**, Windows x64. Permintaan pengguna: memakai REA dan Ghidra headless terhadap instalasi Packet Tracer 9.0.1 untuk memulai pengujian pembanding Wirebox.

**Hasil: audit statis terarah dan smoke test runtime PT tersedia; kesetaraan S01–S05 belum dievaluasi.** Wirebox lulus acceptance sendiri. Contoh PT yang dijalankan mempunyai topologi/config berbeda, sehingga dua hasil tersebut belum boleh digabung menjadi verdict kesetaraan atau keunggulan.

## Target dan alat

| Item | Identitas / hasil |
|---|---|
| Target | `C:\Program Files\Cisco Packet Tracer 9.0.1\bin\PacketTracer.exe`, 60.005.312 byte |
| SHA-256 | `843579cc806a41d57a4ca524d6805b97ee1f91e0ddd02ac09be8461db04b94a1` |
| REA | CLI `rea-agents@6.5.0`; [artifact evidence](rea-artifact.json), [managed check](rea-managed-check.json) |
| Ghidra | 12.1.4, Java 25; proyek lokal `PTProj`, program `/PacketTracer.exe`; input proyek mempunyai hash identik dengan target terpasang |
| REA native Ghidra | Impor/analisis penuh timeout 330.000 ms; [status](rea-status.json). Tidak menghasilkan dekompilasi yang dipakai sebagai bukti |
| REA native UI | Tidak tersedia di Windows pada provider yang aktif; [error asli](pt-ui-observe.json) |
| Fallback | Ghidra langsung, analisis proyek yang sudah ada, `-noanalysis`; smoke GUI memakai Windows UIAutomation dan pesan mouse ke window PT |

REA tidak ditulis ke folder instalasi vendor dan konfigurasi MCP Codex tidak diubah. REA artifact evidence ber-ID `ev_7aa5c12de345210113e7021e81edfea0313c1ad129e92c2d1770167ab3dfef66`; managed check `ev_c2093ec66b05da007741c5e466daa6b780434f220a822e6cfb08fa051661b053` menyatakan tidak ada CLI data directory. Evidence artifact hanya membuktikan identitas/inventaris PE.

## Pemeriksaan statis terarah

[Log run pertama](ghidra-run.log) mencatat empat fungsi selesai didekompilasi. Interpretasi berikut berasal dari pembacaan pseudocode lokal; alamat berlaku hanya untuk hash target di atas.

| Fungsi | Pengamatan lokal | Batas kesimpulan |
|---|---|---|
| `0x141d1d3e0` | Penetapan vftable `Simulation::CScheduler`; storage melalui `+0x18`; elemen berstride 16 byte, pemilihan anak `2*i+2`/saudaranya dan pergeseran parent, pengurangan end 16 byte; pemanggilan destructor Qt pada `+0x48/+0x50/+0x58` | Konsisten dengan pelepasan binary heap. Ada indirection ke storage; jangan menyebut inline vector pada `+0x18` sebagai layout yang sudah pasti. Fungsi ini sendiri belum membuktikan urutan `(time,id)`, jenis registry RB-tree, atau lifecycle thread |
| `0x141d22cf0` | Lookup helper, error `SimulationTimer`, RTTI `CIpcTimer`, dan jalur error `getIpcTimer implementation error` | Mendukung lookup/validasi timer terpisah. Detail helper lookup belum diaudit pada run ini |
| `0x14094c530` | Jalur dengan diagnostik RIB, unreachable host, dan ICMP time exceeded; konstruksi message type `0xb`, code `0` pada jalur terkait | Mendukung routing/ICMP software. Belum memulihkan seluruh kondisi forwarding atau membuktikan byte/checksum runtime |
| `0x142580fc0` | `QMessageBox`, maksimum event, Clear Event List dan View Previous Events | Batas Event List GUI nyata di kode. Kebijakan GUI berbeda dari struktur heap; run ini tidak mengukur perilaku saat buffer benar-benar penuh |

Jadi klaim lama heap dan panel log mendapat dukungan lokal tambahan. Klaim map-by-ID, komparator waktu/ID, worker thread, distribusi jitter, dan checkpoint lengkap tetap memerlukan fungsi/pengamatan lain. Kehadiran kode routing tidak meluluskan conformance protokol.

Extractor yang dapat diulang: [WireboxPtAudit.java](../../../scripts/ghidra/WireboxPtAudit.java). Ia memeriksa hash program, dekompilasi empat fungsi, serta mencatat marker dan ukuran body. Marker hanya membantu peninjauan, bukan oracle perilaku. Output pseudocode penuh disimpan lokal di `artifacts/pt-audit/` yang diabaikan Git.

[Hasil extractor](ghidra-audit.json) dan [log run read-only](ghidra-audit-run.log) menunjukkan empat dekompilasi selesai dan semua marker yang dipilih ditemukan. Ukuran body berturut-turut 589, 966, 26.901, dan 944 alamat byte. Verifikasi hash dilakukan terhadap metadata program di database Ghidra, selain pencocokan hash file input proyek dan binary terpasang.

Ulang dari root repo dengan proyek yang sudah dianalisis; command ini tidak membuat database baru:

```powershell
$ghidra = 'C:\Users\ricky\Downloads\Compressed\ghidra_12.1.4_PUBLIC_20260921\ghidra_12.1.4_PUBLIC'
$projectDir = 'C:\Users\ricky\AppData\Local\Temp\opencode\ghidra_pt\proj'
$repo = (Get-Location).Path
$sha = (Get-FileHash 'C:\Program Files\Cisco Packet Tracer 9.0.1\bin\PacketTracer.exe' -Algorithm SHA256).Hash
& "$ghidra\support\analyzeHeadless.bat" $projectDir PTProj -process PacketTracer.exe -readOnly -noanalysis -scriptPath "$repo\scripts\ghidra" -postScript WireboxPtAudit.java "$repo\artifacts\pt-audit" $sha
if ($LASTEXITCODE -ne 0) { throw 'Ghidra failed; inspect its log and JSON completeness' }
npx -y rea-agents@6.5.0 inspect-artifact 'C:\Program Files\Cisco Packet Tracer 9.0.1\bin\PacketTracer.exe' --format json
npx -y rea-agents@6.5.0 inspect-managed-artifact 'C:\Program Files\Cisco Packet Tracer 9.0.1\bin\PacketTracer.exe' --format json
./target/debug/wirebox-headless.exe --check-network
```

Prasyarat Java/Ghidra harus sudah tersedia; jangan memakai `-noanalysis` pada database baru sebagai pengganti analisis. Ghidra kadang mengembalikan exit 0 walau script gagal; periksa juga bahwa JSON lengkap, keempat `decompile_completed` true, dan hash benar. Perubahan format pseudocode dapat mengubah marker tanpa mengubah algoritme; marker hilang memerlukan pemeriksaan ulang.

## Runtime yang benar-benar dijalankan

[Observasi PT](pt-runtime-observations.json) diambil dari tabel **User Created PDU**, menggunakan contoh bawaan `HSRP/Hsrp_Simple_Ping_With_Same_Priority.pkt` (hash tercatat dalam artifact):

| Probe PT | Status GUI |
|---|---|
| Simple PDU ICMP PC0 → Router0 | Successful |
| Simple PDU ICMP PC0 → PC1 | Failed |

Penyebab probe kedua belum ditentukan. Nilai tabel `Time(sec)=0.000` tidak dipakai sebagai latency. Tidak dikumpulkan raw PDU, checksum, packet capture, atau trace keputusan untuk dua probe ini. HSRP, dua switch, dan tiga router pada contoh ini berbeda dari fixture empat node Wirebox. Ini smoke test runtime PT, bukan hasil S01/S02. Perubahan sesi tidak disimpan; hash file contoh setelah penutupan sama dengan sebelum pengujian.

[Baseline Wirebox baru](wirebox-network.json): **36 run skenario dan 37 input negatif lulus**; S01 echo reply, S02 no route, S03 TTL expired, S04 ARP timeout, S05 truncated IPv4. [Verifikasi core](wirebox-verify.log) juga lulus. Rincian oracle dan batas model tetap mengikuti [G3](../phase-3/README.md). Tidak ada hasil PT yang dimasukkan sebagai expected data palsu.

## Corpus setara yang masih harus dieksekusi

Gunakan [fixture normatif](../../../fixtures/scenarios/icmp-static-v1.json) dan [runbook pembanding](../../packet-tracer/comparison-tests.md). Tidak ada `.pkt` untuk fixture itu yang dibuat pada run ini. Nilai verdict saat ini **not_evaluated**, bukan pass dan bukan mismatch.

Sebelum menyatakan lebih baik dari PT, diperlukan hasil berpasangan pada input yang setara: kebenaran paket/state, penjelasan byte yang dapat diverifikasi, keberhasilan diagnosis oleh pengguna, dan performa pada mesin serta workload yang sama. Coverage firmware/vendor tetap membutuhkan capture/read-back Linux atau perangkat yang ditetapkan, terpisah dari PT.
