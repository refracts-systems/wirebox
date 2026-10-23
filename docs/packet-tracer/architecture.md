# Arsitektur simulasi yang dilaporkan

## Metode dan batas

Target laporan adalah PacketTracer.exe 9.0.1 x64. Tahap awal memakai pemeriksaan PE, string, RTTI MSVC, dan ekstraksi resource. Tahap berikutnya memakai proyek Ghidra 12.1.4 headless. Nama fungsi FUN_* adalah label analisis pada binary tersebut; nama method tertentu direkonstruksi melalui RTTI dan string anchoring, tanpa PDB.

REA dilaporkan tidak menyediakan provider decompiler PE native pada sesi investigasi asal. Dekompilasi dikerjakan melalui Ghidra; Frida digunakan pada investigasi runtime format dokumen. Ini bukan klaim bahwa REA sendiri mendekompilasi target.

## Dunia simulasi

| Objek | Peran | Bukti dalam laporan |
|---|---|---|
| CNetwork / CNetworkFile | Topologi dan serialisasi dokumen | RTTI, serializer, parser |
| CWorkspaceList | Menambah/menghapus perangkat dan link | Nama operasi dan referensi terkait |
| Device::CDevice | Perangkat polymorphic | RTTI/vftable, descriptor, factory |
| Port::CPort | Titik kirim/terima | Kelas port dan label send/receive |
| Link::CLink / CCable | Koneksi dan pengiriman tertunda | RTTI, source-path string, timer kabel |
| CProcess | Pemrosesan protokol | processSend/processReceive/processNoConnection, higher/lower vectors |
| Traffic::CPdu | Representasi paket dalam memori | RTTI, kelas header, getDecapsulationInfo |
| CFrameInstance | Representasi instance untuk tampilan/animasi | RTTI dan slot animasi |

CNetwork dilaporkan memakai vector CDevice* pada +0xF0/+0xF8 dan ordered map koneksi pada +0x170. Offset ini adalah hasil analisis versi target, bukan kontrak layout untuk Wirebox.

## Scheduler

Addendum melaporkan CScheduler::destructor FUN_141d1d3e0 dengan loop sift-down serta operasi pop_heap/pop_back.

| Lokasi relatif | Isi yang dilaporkan | Interpretasi |
|---|---|---|
| +0x18 | std::vector<CNTTimerPointer>, elemen 16 byte | Penyimpanan binary heap untuk timer berikutnya |
| +0x48 / +0x50 | QWaitCondition / QSemaphore | Primitif sinkronisasi; detail lifecycle worker masih perlu pemeriksaan langsung |
| +0x58 | QTimer | Berkaitan dengan pacing realtime menurut laporan |
| +0x70 | shared_ptr ke registry ordered map | Lookup timer berdasarkan ID |

Komparator CIpcTimer dilaporkan mengurutkan waktu primer + sekunder lalu ID. Heap menjawab timer berikutnya; registry menjawab pencarian/penghapusan berdasarkan ID. Bentuk registry tidak mengharuskan antrean event menggunakan multimap.

getIpcTimer, FUN_141d22cf0, dilaporkan melakukan lookup dan dynamic_cast ke CIpcTimer, dengan error SimulationTimer atau getIpcTimer implementation error jika gagal.

Laporan juga menyebut multiple inheritance CIpcTimer, vftable sekunder +0x50, serta CSimulation sebagai QObject dan Ipc::CParser. Detail inheritance ini tidak diperlukan untuk menyalin pola scheduler ke engine baru.

## Rantai pemrosesan paket

Model umum yang didukung laporan:

1. Proses pengirim membentuk PDU.
2. Proses enkapsulasi menyiapkan header link.
3. Port dan link menjadwalkan pengiriman.
4. Port tujuan menyerahkan PDU ke proses penerima.
5. Proses melakukan dekapsulasi dan dispatch ke protokol berikutnya.

init() dan addEntryToMapTable() menghubungkan higher/lower processes. String error pada CHdlcProcess::init dan method protokol lain menguatkan struktur ini. Urutan setiap jalur dan seluruh kondisi khusus belum dipulihkan.

## Routing dan instrumentasi

FUN_14094c530, yang diidentifikasi sebagai CRoutingProcess, dilaporkan berisi lookup RIB, penerimaan lokal, TTL expiry, kegagalan enkapsulasi, dan pembuatan ICMP.

Kode CROUTINGPROCESS_FC_*, CARPPROCESS_FC_*, CMACSWITCHER_FC_*, dan CTcpConnection_FC_* membantu memetakan keputusan. Laporan tahap Ghidra menyebut 1.404 kode langkah secara keseluruhan. Keberadaan nama langkah tidak membuktikan seluruh transisi atau perilaku protokol sudah diimplementasikan ulang.

PDUDefinition.json sekitar 183.638 byte berisi sekitar 170 layout tampilan; catatan resource menyebut lebih dari 250 implementasi getDecapsulationInfo. Resource tampilan tidak sama dengan implementasi parser wire yang lengkap.

## Waktu dan GUI

Waktu simulasi dipisahkan dari wall clock. GUI memiliki mode paused/stepped dan realtime. Pengaturan delay/jitter memberikan bukti adanya parameter model timing, tetapi belum mengungkap seluruh formula atau distribusi random PT.

Event List milik CBaseSimulationPanel merupakan log GUI berbatas. FUN_142580fc0/FUN_142582f00 dikaitkan dengan buffer penuh, kontrol list, dan pengaturan delay. Scheduler queue harus dapat terus dikelola terpisah dari kebijakan penyimpanan log.

Tombol Back dan slot animationDone tidak cukup untuk membuktikan restore seluruh state engine. Hubungan satu Capture/Forward dengan satu pop scheduler tetap sebagian inferensi pada laporan asal.

Sumber: [laporan Ghidra](sources/03-ghidra-architecture.txt), [addendum lengkap](sources/REPORT_addendum.original.md), dan [evidence ledger](evidence-ledger.md).
