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

## 2. Prinsip Pengembangan
**"If it ain't broke, don't fix it."**

Proyek ini menganut prinsip konservatif dalam pengembangan:
- **JANGAN menyentuh kode yang sudah stabil** kecuali ada bug nyata atau fitur baru yang memerlukan modifikasi.
- **Fokus pada penambahan fitur**, bukan perubahan pada kode yang sudah teruji.
- **Setiap perubahan harus memiliki alasan yang jelas** dan diuji secara menyeluruh.

### Daftar Kode Stabil (JANGAN DISENTUH)
Kode berikut sudah **stabil dan teruji**. Jangan mengubahnya tanpa alasan yang sangat kuat:

1.  `src/transaction.rs` — Model transaksi, tanda tangan, nonce.
2.  `src/blockchain.rs` — State chain, reward, difficulty, mempool.
3.  `src/storage/` — LevelDB, MPT, `Account`, `StateDB`.
4.  `src/storage_legacy.rs` — Snapshot JSON.
5.  `src/validation.rs` — Aturan konsensus.
6.  `src/rpc.rs` — `m_sendtoaddress`, `m_getbalance`, `m_getnonce`, `m_stop`, `m_generatetoaddress`, `m_submitblock`.
7.  `src/p2p/dns_seed.rs` — Resolusi DNS.
8.  `src/storage/account.rs` — Struktur `Account` dengan `immature_rewards`.
9.  `src/storage/state.rs` — `credit_reward`, `apply_transaction`, `rollback_to`.
10. `src/p2p/sync.rs` — `handle_reorg`, `accept_block`.

Tapi.. sebenarnya boleh saja mengubah memperbaiki. atau menambahkan kode. selagi ada bug atau keanehan pada kode. jika stabil dilarang keras menyentuh/mengubah kode!.

---

## 3. Status Terkini (Terakhir Diperbarui: 2026-10-22)

### 🟢 CHECKPOINT: `fase-4-lengkap`
Sistem transfer *account-based* mainnet **STABIL** dan **TERUJI**. Fondasi penyimpanan telah dimigrasikan ke LevelDB + MPT. **Coinbase Maturity**, **Reorg Logic**, **DNS Seed Discovery**, dan **Uji Beban** telah diimplementasikan dan **diuji**.

### 🟢 FASE 2: HARDENING MAINNET (SELESAI)
Semua pilar keamanan dan ekonomi telah diimplementasikan:

- ✅ **Dynamic Fee (Bitcoin-Style):** Fee market murni dengan `min_relay_fee_per_byte` (1 sat/byte).
- ✅ **Mempool Eviction:** Batas 300 MB, evict transaksi dengan *fee rate* terendah, dan expiry 14 hari.
- ✅ **P2P Rate Limiting:** Batas 50 tx/detik dan 100 inv/detik per peer.
- ✅ **RPC Security:** Cookie Authentication + proteksi *bind address*.
- ✅ **Moving Download Window:** Batas 16 blok *in-flight* saat sinkronisasi P2P.
- ✅ **Wallet Nonce Reuse AES-GCM:** Setiap entry memiliki nonce enkripsi unik.
- ✅ **PBKDF2 600.000 Iterasi:** Standar keamanan OWASP.
- ✅ **Replay Protection (Chain ID):** *Signing hash* menyertakan *magic bytes* jaringan.
- ✅ **RBF (Replace-By-Fee):** Transaksi dengan fee rendah bisa digantikan oleh transaksi fee tinggi (minimal +10%).
- ✅ **Regtest Mode:** Jaringan ketiga untuk *development* dan *testing* instan.

### 🟢 FASE 3: MIGRASI STORAGE (SELESAI)
Penyimpanan *state* telah dimigrasikan dari `HashMap` + JSON ke **LevelDB + Merkle Patricia Trie (MPT)** ala Ethereum.

- ✅ **LevelDB (Pure Rust):** Menggunakan `rusty-leveldb`.
- ✅ **Merkle Patricia Trie (MPT):** Menggunakan `eth_trie` + `rlp`.
- ✅ **Account Object:** Setiap akun menyimpan `nonce`, `balance`, dan `immature_rewards` dalam format RLP.
- ✅ **State Snapshot (JSON):** Saat `stop`, state disimpan ke `state_snapshot.json`.
- ✅ **Flush on Write:** Setiap penulisan ke LevelDB langsung di-*flush* ke disk.
- ✅ **`state_root` di Block:** Setiap blok memiliki `state_root` yang menyimpan *root hash* MPT.

### 🟢 FASE 4: COINBASE MATURITY (SELESAI & TERUJI)
- ✅ **Coinbase Maturity (100 blok):** Hadiah *mining* masuk ke `immature_rewards` dan dipindahkan ke `balance` setelah 100 blok.
- ✅ **Pengujian Regtest:** Mining 110 blok, verifikasi `spendable_balance` = 0 sebelum 100 blok, dan menjadi 500 setelah 101 blok.
- ✅ **Keamanan `skip-pow`:** Menggunakan *feature flag* `skip-pow` yang hanya aktif saat `cargo test --features skip-pow`.

### 🟢 FASE 4: REORG LOGIC (SELESAI & TERUJI)
- ✅ **Reorg Logic (Total Work):** Setiap blok memiliki `chainwork`. `handle_reorg` membandingkan total work dan melakukan *rollback* state ke *common ancestor*.
- ✅ **StateDB Rollback:** Menggunakan `EthTrie::from(adapter, root)` untuk *rollback* ke *state root* tertentu.
- ✅ **Pengujian End-to-End:** 
    - Node A mining 5 blok.
    - Node B mining 10 blok.
    - Blok 1-10 dari Node B dikirim ke Node A via `submitblock`.
    - Node A berhasil melakukan reorg, best block hash sama dengan Node B.
    - Saldo alamat Node A menjadi 0 (rollback), saldo alamat Node B menjadi 500 (state baru).
- ✅ **RPC `submitblock` Diperbaiki:** Mendeteksi reorg dan memanggil `handle_reorg`.

### 🟢 DNS SEED DISCOVERY (SELESAI & TERUJI)
- ✅ **Modul `dns_seed` di `blockchain.rs`:** Daftar seed untuk Mainnet, Testnet, dan Regtest.
- ✅ **Resolusi DNS:** Menggunakan `tokio::net::lookup_host` dengan port default jaringan.
- ✅ **Pengujian:** Node Testnet berhasil me-resolve seed Bitcoin dan menemukan 23 peer.
- ⚠️ **Catatan:** Daftar seed saat ini kosong (setelah pengujian). Harus diisi dengan domain ZenaChain sebelum peluncuran.

### 🟢 UJI BEBAN (SELESAI & TERUJI)
- ✅ **Pengujian:** Dilakukan menggunakan gabungan Testnet dan Regtest.
- ✅ **Hasil:** Sistem stabil, tidak ada crash, mampu memproses jutaan transaksi.

---

## 4. Tugas Berikutnya (FASE 5: Dokumentasi, Audit, Testnet Publik)

### Prioritas Tinggi (Wajib sebelum Testnet Publik)
- [ ] **Uji P2P Sync Multi-Node:** Jalankan 3-5 node Testnet dan uji sinkronisasi.
- [ ] **Isi Checkpoint:** Tambahkan hash blok yang sudah dikonfirmasi ke `CHECKPOINTS`.
- [ ] **DNS Seed ZenaChain:** Daftarkan domain ZenaChain untuk *peer discovery*.

### Prioritas Sedang (Wajib sebelum Mainnet)
- [ ] **Uji Difficulty Adjustment:** Di Testnet, uji dengan parameter dipercepat (misalnya `DIFFICULTY_ADJUST_INTERVAL = 100`).
- [ ] **Audit Keamanan:** Undang pihak ketiga untuk mengaudit kode.
- [ ] **Uji Wallet Recovery:** Uji skenario *restore* wallet dari *backup*.
- [ ] **Uji Serangan Umum:** Uji *eclipse*, *sybil*, *double-spend*, *selfish mining*.

### Prioritas Rendah (Setelah Mainnet)
- [ ] **Pruning:** Implementasi *pruning* untuk menghemat disk.
- [ ] **Optimasi P2P:** *Parallel block download*.

---

## 5. Catatan Penting
- **Difficulty Adjustment:** Fitur berjalan sempurna. Di Testnet, saat mencapai blok 69.120, kesulitan naik drastis.
- **Jeda 5.000 Blok:** Batas batch sync P2P sudah dinaikkan jadi 5.000 header.
- **LevelDB Lock:** `rusty-leveldb` mengunci database saat dibuka. CLI tidak boleh membuka LevelDB secara langsung saat `zenachaind` mati. Gunakan *snapshot* JSON sebagai *fallback*.
- **Persistensi:** Data *state* disimpan di LevelDB (saat *running*) dan JSON *snapshot* (saat *stop*).
- **`state_root`:** Setiap blok memiliki `state_root` yang dihitung setelah transaksi diterapkan.
- **`chainwork`:** Setiap blok memiliki `chainwork` (u128) yang dihitung dari `nbits`.
- **`immature_rewards`:** Setiap akun memiliki daftar reward yang belum matang. Maturity = 100 blok.
- **`ZENAC_SKIP_POW`:** Sudah tidak digunakan. Beralih ke *feature flag* `skip-pow`.
- **DNS Seed:** Mekanisme resolusi DNS sudah berfungsi. Daftar seed saat ini kosong. Isi dengan domain ZenaChain sebelum peluncuran.
- **RPC `getblock`:** Field `index`, `nbits`, `reward`, `subsidy`, `total_fees` dikembalikan sebagai angka.
- **RPC `submitblock`:** Mendeteksi reorg dan memanggil `handle_reorg`.
- **Uji Beban:** Sistem sudah diuji dengan jutaan transaksi tanpa crash.

---

## 6. Cara Melanjutkan di Chat Baru
Copy file ini dan tempel ke chat baru, lalu ketik:
> "Lanjut ZenaChain. Baca PROGRESS.md ini. Fase 2, 3, 4 (Coinbase Maturity, Reorg Logic), DNS Seed, dan Uji Beban selesai. Lanjut FASE 5: Dokumentasi, Audit, dan Testnet Publik. Patuhi prinsip 'If it ain't broke, don't fix it'."

---

## 7. Perintah Penting

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
