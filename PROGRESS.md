# Progres Pengembangan ZenaChain

## 1. Gambaran Proyek
ZenaChain adalah blockchain berbasis Rust dengan model *account-based* (seperti Ethereum). Jaringan P2P menggunakan TCP mentah, RPC server multi-threaded, dan dompet multi-alamat dengan enkripsi AES-256-GCM.

Sekarang mendukung tiga jaringan: **Mainnet**, **Testnet**, dan **Regtest**, dengan parameter, genesis, magic bytes, port, address prefix, dan datadir yang terpisah sepenuhnya.

**Fondasi penyimpanan *state* telah dimigrasikan ke LevelDB + Merkle Patricia Trie (MPT)**, dengan *snapshot* JSON untuk persistensi antar-*session*.

### Parameter Konsensus Utama
| Parameter | Mainnet | Testnet | Regtest |
|---|---|---|---|
| Algoritma Hash | SHA3-256 | SHA3-256 | SHA3-256 |
| Tanda Tangan | secp256k1-ECDSA | secp256k1-ECDSA | secp256k1-ECDSA |
| Max Supply | 28.000.000.000 ZENAC | 28.000.000.000 ZENAC | 28.000.000.000 ZENAC |
| Reward Awal | 50 ZENAC | 50 ZENAC | 50 ZENAC |
| Waktu Blok | 15 detik | 15 detik | 15 detik |
| Difficulty Adjust | 69.120 blok | 69.120 blok | Tidak ada (instan) |
| Magic Bytes | `ZENA` (5a454e41) | `ZTES` (5a544553) | `ZREG` (5a524547) |
| P2P Port | 8333 | 9422 | 18444 |
| RPC Port | 8332 | 12255 | 18443 |
| Address Prefix | `ZENA` | `ZTES` | `ZREG` |
| Datadir | `~/.zenachain` | `~/.zenachain-testnet` | `~/.zenachain-regtest` |
| Genesis Timestamp | 1792303198445 | 1792303198446 | 1792303198447 |
| Genesis NBITS | 0x1e0ffff0 | 0x1e0ffff0 | 0x207fffff |
| Genesis Nonce | 378161 | 106933 | 1 |

---

## 2. Status Terkini (Terakhir Diperbarui: 2026-10-20)

### 🟢 CHECKPOINT: `hardening-mainnet` (Fase 2 Selesai)
Sistem transfer *account-based* mainnet **STABIL** dan **TERUJI**. Fondasi penyimpanan telah dimigrasikan ke LevelDB + MPT.

**JANGAN UBAH** kode berikut kecuali ada bug nyata:
- `src/transaction.rs`, `src/blockchain.rs`, `src/storage/`, `src/storage_legacy.rs`.
- `src/validation.rs` (kecuali aturan baru untuk *Policy*).
- `src/rpc.rs` (khusus `m_sendtoaddress`, `m_getbalance`, `m_getnonce`, `m_stop`).

### 🟢 FASE 2: HARDENING MAINNET (SELESAI)
Semua pilar keamanan dan ekonomi telah diimplementasikan:

- ✅ **Dynamic Fee (Bitcoin-Style):** Fee market murni dengan `min_relay_fee_per_byte` (1 sat/byte). Tidak ada *base fee* dinamis di state.
- ✅ **Mempool Eviction:** Batas 300 MB, evict transaksi dengan *fee rate* terendah, dan expiry 14 hari.
- ✅ **P2P Rate Limiting:** Batas 50 tx/detik dan 100 inv/detik per peer. Putus koneksi jika melanggar.
- ✅ **RPC Security:** Cookie Authentication (tanpa password manual) + proteksi *bind address* (menolak RPC publik tanpa konfigurasi eksplisit).
- ✅ **Moving Download Window:** Batas 16 blok *in-flight* saat sinkronisasi P2P.
- ✅ **Wallet Nonce Reuse AES-GCM:** Setiap entry memiliki nonce enkripsi unik.
- ✅ **PBKDF2 600.000 Iterasi:** Standar keamanan OWASP.
- ✅ **Replay Protection (Chain ID):** *Signing hash* menyertakan *magic bytes* jaringan, sehingga transaksi Regtest tidak valid di Testnet/Mainnet.
- ✅ **RBF (Replace-By-Fee):** Transaksi dengan fee rendah bisa digantikan oleh transaksi fee tinggi (minimal +10%) dengan nonce yang sama.
- ✅ **Regtest Mode:** Jaringan ketiga untuk *development* dan *testing* instan.

### 🟢 FASE 3: MIGRASI STORAGE (SELESAI)
Penyimpanan *state* telah dimigrasikan dari `HashMap` + JSON ke **LevelDB + Merkle Patricia Trie (MPT)** ala Ethereum.

- ✅ **LevelDB (Pure Rust):** Menggunakan `rusty-leveldb` untuk menghindari drama CMake/C++.
- ✅ **Merkle Patricia Trie (MPT):** Menggunakan `eth_trie` + `rlp` untuk struktur *state* yang efisien dan bisa diverifikasi.
- ✅ **Account Object:** Setiap akun menyimpan `nonce` dan `balance` dalam format RLP.
- ✅ **State Snapshot (JSON):** Saat `stop`, state disimpan ke `state_snapshot.json` untuk persistensi antar-*session*. Saat *start*, *snapshot* dimuat kembali ke LevelDB jika kosong.
- ✅ **Flush on Write:** Setiap penulisan ke LevelDB langsung di-*flush* ke disk untuk menjamin persistensi.

---

## 3. Tugas Berikutnya (FASE 4: Menuju Mainnet)

Sebelum Mainnet diluncurkan, masih ada beberapa hal krusial yang harus diselesaikan:

- [ ] **Coinbase Maturity:** Hadiah *mining* harus menunggu 100 blok sebelum bisa dibelanjakan (mencegah *chain reorg* palsu).
- [ ] **Reorg Logic:** Menangani percabangan blok (jika dua *miner* menemukan blok di ketinggian yang sama). Pilih rantai dengan total *work* terbesar.
- [ ] **Dokumentasi:** Memperbarui `README.md` dengan panduan instalasi, penggunaan, dan arsitektur.
- [ ] **Audit Keamanan:** Mengundang pihak ketiga untuk mengaudit kode (terutama `storage/` dan `validation.rs`).
- [ ] **Testnet Publik:** Meluncurkan Testnet untuk diuji oleh komunitas.
- [ ] **DNS Seeds:** Mendaftarkan *domain* untuk *peer discovery* otomatis.

---

## 4. Catatan Penting
- **Difficulty Adjustment:** Fitur berjalan sempurna. Di Testnet, saat mencapai blok 69.120, kesulitan naik drastis. Ini fitur, bukan bug.
- **Jeda 5.000 Blok:** Batas batch sync P2P sudah dinaikkan jadi 5.000 header agar sinkronisasi lebih cepat.
- **LevelDB Lock:** `rusty-leveldb` mengunci database saat dibuka. CLI tidak boleh membuka LevelDB secara langsung saat `zenachaind` mati. Gunakan *snapshot* JSON sebagai *fallback*.
- **Persistensi:** Data *state* disimpan di LevelDB (saat *running*) dan JSON *snapshot* (saat *stop*). Keduanya sinkron.

---

## 5. Cara Melanjutkan di Chat Baru
Copy file ini dan tempel ke chat baru, lalu ketik:
> "Lanjut ZenaChain. Baca PROGRESS.md ini. Fase 2 dan 3 selesai. Lanjut FASE 4: Coinbase Maturity dan Reorg Logic."

---

## 6. Perintah Penting

### Mainnet
- Start: `./zenachaind`
- RPC: `./zenachain getinfo`
- Reset: `./zenachain stop && rm -rf ~/.zenachain`

### Testnet
- Start Node 1: `./zenachaind -testnet`
- Start Node 2: `ZENAC_DATADIR=/root/.zenachain-testnet2 ./zenachaind -testnet`
- RPC: `./zenachain -testnet getinfo`
- Wallet Baru: `./zenachain -testnet getnewwallet`
- Mining: `./zenachain -testnet generatetoaddress 10 "ZTES..."`
- Reset: `./zenachain -testnet stop && rm -rf ~/.zenachain-testnet ~/.zenachain-testnet2`

### Regtest
- Start: `./zenachaind -regtest`
- RPC: `./zenachain -regtest getinfo`
- Wallet Baru: `./zenachain -regtest getnewwallet`
- Mining: `./zenachain -regtest generatetoaddress 10 "ZREG..."`
- Reset: `./zenachain -regtest stop && rm -rf ~/.zenachain-regtest`
