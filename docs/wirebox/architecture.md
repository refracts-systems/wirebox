# Model semantik dan runtime

Status: model domain mengikuti [ADR](../adr/README.md). [Runtime phase 2](runtime.md) menyediakan graph device/port/link, counter, scheduler, timer, dan jurnal. [Phase 3](packets.md) menambahkan simulated state ARP/MAC/route, perilaku Ethernet/IPv4/ICMP, serta versi byte. Observed/desired state, planner, dan operation executor perangkat masih mengikuti roadmap. [ABI transport probe](ffi.md) phase 1 tetap terpisah.

## Infrastruktur sebagai objek yang dapat ditanya

Analogi *Infrastructure DOM* berarti objek dan relasinya dapat ditelusuri secara terstruktur. Contoh konseptual: cari semua ONU pada suatu PON yang sinyalnya rendah dan layanan internetnya tidak aktif. Ini tidak mengharuskan pemakaian DOM browser atau JavaScript.

Sepuluh primitive dari visi awal:

| Primitive | Makna | Contoh |
|---|---|---|
| Entity | Sesuatu dengan identitas stabil | Router, interface, ONU, process |
| Capability | Kemampuan dengan cakupan yang diketahui | Routing IPv4, switching, observasi optik |
| Property | Nilai pada suatu objek | Address, admin state, rx power |
| Relation | Hubungan bertipe | `attached_to`, `member_of`, `depends_on` |
| Resource | Sesuatu yang terbatas atau dialokasikan | IP, VLAN ID, bandwidth, GEM port |
| Constraint | Aturan validitas dalam domain | Referensi interface harus valid |
| Intent | Kondisi yang ingin dicapai | Subnet A dapat mengakses layanan B |
| Operation | Langkah perubahan dengan kontrak | AddRoute, CreateVLAN, EnableInterface |
| Event | Peristiwa pada suatu waktu | LinkDown, timer expired, frame received |
| Observation | Bukti yang dibaca dari suatu sumber | Hasil CLI, telemetry, atau SNMP |

Property perlu tipe dan satuan yang jelas. Relasi fisik, koneksi L2/L3, dan dependency layanan tidak boleh dilebur menjadi satu jenis garis tanpa arti. Capability sebaiknya menyatakan parameter dan batas dukungan, bukan hanya boolean `supported`.

## State dan provenance

| Lapisan | Makna | Contoh |
|---|---|---|
| Observation | Bukti asli dengan sumber dan waktu | CLI membaca interface down pada waktu T |
| Observed state | Interpretasi atas bukti yang tersedia | Interface dinilai down berdasarkan observation tertentu |
| Desired state | Kondisi yang dikehendaki | Interface administratively enabled |
| Simulated state | Hasil model pada cabang dan waktu virtual tertentu | Frame diteruskan setelah perubahan route simulasi |

Observation yang bertentangan harus tetap dapat diperiksa. Data yang belum dibaca atau sudah kedaluwarsa perlu ditandai unknown/stale; jangan otomatis diisi dari hasil simulasi.

Snapshot produksi dapat menjadi input cabang simulasi. Cabang perlu menunjuk snapshot asal, versi model, konfigurasi, dan asumsi untuk state yang tidak teramati. Merangkum konfigurasi perangkat tidak menjamin seluruh buffer dan state internalnya ikut tertangkap.

## Intent, IR, dan dua executor

```text
Intent + snapshot + capabilities
                |
                v
      Planner / constraint checks
                |
                v
    Operation plan / Infrastructure IR
          |                         |
          v                         v
    Sim executor              Real executor
          |                         |
    Virtual world             Device adapters
          |                         |
    Trace + result            Observe + verify
```

IR adalah representasi perantara yang mempertahankan makna operasi sebelum diterjemahkan ke backend. Pembagian modul dan format serialisasinya masih terbuka.

Contoh `EnsureReachability(A, B)` perlu dipersempit: arah trafik, alamat/prefix, protokol/port, VRF, keadaan link, dan asumsi return path. Planner menghasilkan operasi yang dapat dipahami backend, misalnya menambahkan route. Executor simulator mengubah state virtual; adapter nyata menerjemahkannya sesuai capability perangkat.

Operation adalah unit logis perencanaan. Eksekusinya pada perangkat dapat memerlukan beberapa perintah dan dapat gagal sebagian; ia tidak otomatis merupakan transaksi atomik lintas perangkat.

Kontrak operasi yang diusulkan mencakup target, argumen, precondition, expected effect, postcondition, timeout, sifat idempotensi, dan kemungkinan kompensasi. Menjalankan ulang operasi harus memiliki hasil yang didefinisikan, terutama setelah kegagalan parsial.

## Adapter dan transport

Kontrak driver konseptual dari visi: `discover`, `observe`, `capabilities`, `plan`, `execute`, dan `validate`. Pembagian tanggung jawab planner umum dan perencanaan khusus driver masih perlu dirancang.

SSH/CLI, SNMP, NETCONF, RESTCONF, gNMI, HTTP, Modbus, dan serial adalah calon transport. Parsing output dan perilaku firmware berada pada adapter/domain terkait. Dukungan transport tidak berarti seluruh kemampuan vendor sudah tersedia.

Inventaris user-reported tersedia pada [pilot-v1.json](../../fixtures/inventory/pilot-v1.json): Linux VM/server, RouterOS major 7, C320 dengan versi per board, serta contoh state GPON ONU. Model/firmware ONU, distro Linux, versi minor RouterOS, akses, dan capability yang belum diketahui tetap unknown. Driver belum tersedia; tahap pengujiannya ada pada [rencana pilot](pilot.md).

## Perubahan nyata, reconciliation, dan AI

Alur yang dituju adalah plan → validasi → simulasi sesuai model → kebijakan/otorisasi yang berlaku → execute → observe → verify. Plan perlu menyatakan dependency, dampak, dan apa yang terjadi jika sebagian langkah gagal.

Rollback adalah kemampuan per operasi/backend. Mengembalikan konfigurasi lama belum tentu memulihkan sesi aktif, state protokol, atau efek fisik. Branch/merge/bisect state infrastruktur juga memerlukan aturan domain untuk konflik; kemiripan dengan Git adalah arah pengalaman.

Reconciliation membandingkan desired dan observed state, lalu menghasilkan repair plan. Eksekusi otomatis beserta kebijakannya belum diputuskan. AI dapat membantu query, diagnosis, dan perencanaan melalui objek serta operasi yang sama; akses model tidak otomatis memberi hak mengeksekusi perubahan.

Lihat [roadmap](roadmap.md) untuk urutan pembuktian dan [keputusan](decisions.md) untuk batas yang belum dipilih.
