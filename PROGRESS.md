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
| Genesis Nonce | 849719 | 491730 | 0 |

---

## 2. Status Terkini (Terakhir Diperbarui: 2026-10-22)

### 🟢 CHECKPOINT: `fase-4-dan-dns-seed-selesai`
Sistem transfer *account-based* mainnet **STABIL** dan **TERUJI**. Fondasi penyimpanan telah dimigrasikan ke LevelDB + MPT. **Coinbase Maturity**, **Reorg Logic**, dan **DNS Seed Discovery** telah diimplementasikan dan diuji.

**JANGAN UBAH** kode berikut kecuali ada bug nyata:
- `src/transaction.rs`, `src/blockchain.rs`, `src/storage/`, `src/storage_legacy.rs`.
- `src/validation.rs` (kecuali aturan baru untuk *Policy*).
- `src/rpc.rs` (khusus `m_sendtoaddress`, `m_getbalance`, `m_getnonce`, `m_stop`, `m_generatetoaddress`).
- `src/p2p/dns_seed.rs`.

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
- ✅ **Account Object:** Setiap akun menyimpan `nonce`, `balance`, dan `immature_rewards` dalam format RLP.
- ✅ **State Snapshot (JSON):** Saat `stop`, state disimpan ke `state_snapshot.json` untuk persistensi antar-*session*. Saat *start*, *snapshot* dimuat kembali ke LevelDB jika kosong.
- ✅ **Flush on Write:** Setiap penulisan ke LevelDB langsung di-*flush* ke disk untuk menjamin persistensi.
- ✅ **`state_root` di Block:** Setiap blok sekarang memiliki `state_root` yang menyimpan *root hash* MPT setelah blok diterapkan.

### 🟢 FASE 4: COINBASE MATURITY & REORG LOGIC (SELESAI)
Dua fitur krusial untuk keamanan ekonomi dan konsensus telah diimplementasikan dan diuji.

- ✅ **Coinbase Maturity (100 blok):** Hadiah *mining* masuk ke `immature_rewards` dan dipindahkan ke `balance` (mature) setelah 100 blok. Transaksi hanya bisa membelanjakan dari `balance`.
- ✅ **Reorg Logic (Total Work):** Setiap blok memiliki `chainwork` (akumulasi work). `handle_reorg` membandingkan total work dan melakukan *rollback* state ke *common ancestor* menggunakan `state_root` jika rantai baru memiliki work lebih besar.
- ✅ **StateDB Rollback:** Menggunakan `EthTrie::from(adapter, root)` untuk *rollback* ke *state root* tertentu.
- ✅ **Genesis Baru:** Hash genesis diperbarui karena `chainwork` ditambahkan ke `calculate_hash`.
- ✅ **Pengujian Regtest:** Mining 110 blok, verifikasi `spendable_balance` = 0 sebelum 100 blok, dan menjadi 500 setelah 101 blok. Transaksi berhasil setelah maturity.
- ✅ **Keamanan `skip-pow`:** Menggunakan *feature flag* `skip-pow` yang hanya aktif saat `cargo test --features skip-pow`. *Binary* rilis tidak memiliki kode skip.

### 🟢 DNS SEED DISCOVERY (SELESAI)
Mekanisme *peer discovery* otomatis telah diimplementasikan dan diuji.

- ✅ **Modul `dns_seed` di `blockchain.rs`:** Daftar seed untuk Mainnet, Testnet, dan Regtest.
- ✅ **Resolusi DNS:** Menggunakan `tokio::net::lookup_host` dengan port default jaringan.
- ✅ **Pengujian:** Node Testnet berhasil me-resolve seed Bitcoin dan menemukan 23 peer (sebagai bukti mekanisme bekerja).
- ⚠️ **Catatan:** Daftar seed saat ini kosong (setelah pengujian). Harus diisi dengan domain ZenaChain yang sebenarnya sebelum peluncuran publik.

---

## 3. Tugas Berikutnya (FASE 5: Dokumentasi, Audit, dan Testnet Publik)

Sebelum Mainnet diluncurkan, ada beberapa tugas yang harus diselesaikan:

- [ ] **Dokumentasi:** Memperbarui `README.md` dengan panduan instalasi, penggunaan, dan arsitektur.
- [ ] **Audit Keamanan:** Mengundang pihak ketiga untuk mengaudit kode (terutama `storage/`, `validation.rs`, dan `p2p/sync.rs`).
- [ ] **Testnet Publik:** Meluncurkan Testnet untuk diuji oleh komunitas.
- [ ] **DNS Seeds ZenaChain:** Mendaftarkan *domain* untuk *peer discovery* otomatis.
- [ ] **Checkpoint:** Mengisi `CHECKPOINTS` dengan hash blok yang sudah dikonfirmasi (saat ini masih kosong).
- [ ] **Pruning:** Implementasi *pruning* untuk menghemat disk.
- [ ] **Optimasi P2P:** *Parallel block download* untuk sinkronisasi lebih cepat.

---

## 4. Catatan Penting
- **Difficulty Adjustment:** Fitur berjalan sempurna. Di Testnet, saat mencapai blok 69.120, kesulitan naik drastis. Ini fitur, bukan bug.
- **Jeda 5.000 Blok:** Batas batch sync P2P sudah dinaikkan jadi 5.000 header agar sinkronisasi lebih cepat.
- **LevelDB Lock:** `rusty-leveldb` mengunci database saat dibuka. CLI tidak boleh membuka LevelDB secara langsung saat `zenachaind` mati. Gunakan *snapshot* JSON sebagai *fallback*.
- **Persistensi:** Data *state* disimpan di LevelDB (saat *running*) dan JSON *snapshot* (saat *stop*). Keduanya sinkron.
- **`state_root`:** Setiap blok sekarang memiliki `state_root` yang dihitung setelah transaksi diterapkan. Ini adalah kunci untuk Reorg Logic.
- **`chainwork`:** Setiap blok memiliki `chainwork` (u128) yang dihitung dari `nbits`. Digunakan untuk membandingkan rantai saat reorg.
- **`immature_rewards`:** Setiap akun memiliki daftar reward yang belum matang. Maturity = 100 blok.
- **`ZENAC_SKIP_POW`:** Sudah tidak digunakan. Beralih ke *feature flag* `skip-pow`. Gunakan `cargo test --features skip-pow` untuk tes cepat.
- **DNS Seed:** Mekanisme resolusi DNS sudah berfungsi. Daftar seed saat ini kosong (setelah pengujian). Isi dengan domain ZenaChain sebelum peluncuran.

---

## 5. Cara Melanjutkan di Chat Baru
Copy file ini dan tempel ke chat baru, lalu ketik:
> "Lanjut ZenaChain. Baca PROGRESS.md ini. Fase 2, 3, 4, dan DNS Seed selesai. Lanjut FASE 5: Dokumentasi, Audit, dan Testnet Publik."

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

### Testing
- Test cepat: `cargo test --workspace --all-targets --features skip-pow`
- Build release: `cargo build --release`
