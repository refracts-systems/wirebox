# Kelayakan ide dan proyek pembanding

Wirebox masuk akal sebagai rangkaian kemampuan yang dibangun bertahap. Kesulitan terbesar adalah keluasan model dan kesetaraan antara simulasi dengan sistem nyata. Keberadaan graph, UI, atau compiler intent saja belum membuktikan akurasi maupun manfaat operasional.

## Proyek yang dibahas

Ringkasan fungsi berikut diperiksa dari sumber resmi pada 19 Oktober 2026. Kolom relevansi adalah interpretasi untuk Wirebox, bukan klaim bahwa produk tersebut identik atau kekurangan fitur tertentu.

| Proyek | Fokus yang relevan | Pelajaran untuk Wirebox |
|---|---|---|
| [Batfish](https://batfish.org/) | Analisis konfigurasi jaringan | Pisahkan validasi reachability/policy dari simulasi waktu setiap paket |
| [Forward Networks](https://www.forwardnetworks.com/) | Model digital twin jaringan untuk inspeksi dan validasi perubahan | Ada kebutuhan nyata untuk query perilaku dan dampak perubahan berbasis model |
| [NetBox Labs](https://netboxlabs.com/) | Data/model infrastruktur, intended dan operational state, serta drift | Identitas objek dan provenance state perlu fondasi yang kuat |
| [Apstra](https://www.juniper.net/us/en/products/network-automation/apstra-data-center-director.html) | Otomasi berbasis intent dan validasi jaringan data center | Domain yang terdefinisi membantu menjaga kontrak intent dan backend |
| [SmartOLT](https://www.smartolt.com/) | Manajemen OLT/ONU, provisioning, monitoring, dan operasi | Domain akses ISP punya workflow konkret yang bisa dipilih kemudian |
| [OpenConfig](https://www.openconfig.net/) | Model data vendor-neutral, telemetry, dan protokol manajemen | Model yang ada dapat dipetakan; schema saja belum menjadi simulator perilaku |
| [containerlab](https://containerlab.dev/) | Orkestrasi lab jaringan berbasis container dan koneksi virtual | Lab implementasi nyata dapat menjadi pembanding untuk model Wirebox |

Deskripsi vendor menjelaskan posisi produknya, bukan hasil benchmark independen. Belum ada evaluasi langsung Wirebox melawan produk-produk tersebut atau keputusan integrasi dengan salah satunya.

## Ruang pembeda yang ingin dibuktikan

Hipotesis Wirebox adalah pengalaman yang menyambungkan objek infrastruktur, intent, eksperimen, trace kausal, dan inspeksi byte. Pengguna dapat berpindah dari pertanyaan “mengapa gagal?” ke state, keputusan protokol, serta byte yang relevan tanpa kehilangan hubungan asal-usulnya.

Hipotesis ini perlu dibuktikan melalui satu workflow kecil yang lengkap. Contoh awal: ping gagal karena route hilang; pengguna melihat alasan drop, mengubah route pada cabang simulasi, lalu membandingkan hasil dan perubahan byte.

Untuk use case operasional berikutnya, visi awal mengusulkan reachability pelanggan melewati ONU → OLT → VLAN → router/NAT → uplink. Skenario itu memerlukan model lebih luas dan adapter yang belum ada.

## Batas ambisi

Prioritas awal adalah model yang dapat dijelaskan dan diuji pada skenario terbatas. Dukungan banyak vendor, AI, time travel penuh, serta eksekusi produksi menjadi perluasan setelah fondasi terbukti. Tidak ada keputusan harga, model bisnis, target skala, atau janji dukungan seluruh vendor dalam percakapan ini.

[Roadmap](roadmap.md) menerjemahkan arah tersebut menjadi tahapan dengan hasil yang dapat diperiksa.
