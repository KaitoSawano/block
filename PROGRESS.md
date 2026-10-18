# Progres Pengembangan ZenaChain

## 1. Gambaran Proyek
ZenaChain adalah blockchain berbasis Rust (konsep diadaptasi dari Dogecoin/Litecoin). Jaringan P2P menggunakan TCP mentah, RPC server multi-threaded, dan dompet multi-alamat dengan enkripsi AES-256-GCM.

### Parameter Konsensus Utama:
- **Algoritma Hash:** SHA3-256
- **Tanda Tangan:** secp256k1-ECDSA
- **Max Supply:** 28.000.000.000 ZENAC
- **Reward Awal:** 50 ZENAC
- **Coinbase Maturity:** 100 blok
- **Waktu Blok:** 15 detik
- **Penyesuaian Difficulty:** Setiap 69.120 blok

### Struktur Proyek:
- `src/` : Crate utama node (`zenachain`)
- `crypto/` : Crate workspace terpisah untuk kriptografi (`zenachain-crypto`)
- `contrib/` : Skrip build Gitian

---

## 2. Status Terkini (Terakhir Diperbarui: 2026-10-18)

### ✅ Fitur yang Sudah Selesai:
1. **Konsensus & Genesis:**
   - Genesis block dikunci dengan timestamp dan hash statis.
   - `consensus_fingerprint` mengikat semua parameter ekonomi.
   - Perubahan aturan konsensus memaksa pembuatan genesis baru (Hard Fork).

2. **Jaringan P2P:**
   - Transportasi TCP mentah (tanpa libp2p).
   - Framing ala Bitcoin (magic bytes `0x5a454e41`, checksum double-SHA3-256).
   - Handshake Version/Verack.
   - Sinkronisasi headers-first, relay Inv/GetData.

3. **RPC Server:**
   - Multi-threaded (setiap koneksi di-handle di thread terpisah).
   - Perintah `stop` langsung menghentikan mining.
   - RPC: `getinfo`, `getblockchaininfo`, `getblock`, `getbalance`, `sendtoaddress`, `generatetoaddress`, `listunspent`, `getwallet`, `encryptwallet`, `walletpassphrase`, dll.

4. **Dompet:**
   - Dompet multi-alamat (file JSON `wallet.dat`).
   - Enkripsi AES-256-GCM dengan PBKDF2 (100.000 iterasi).
   - Sesi buka kunci (`walletpassphrase`) menyimpan password di memori.
   - Tidak bisa membuat alamat baru atau mengirim dana jika dompet terkunci.

5. **Mining & Reward:**
   - `generatetoaddress` sekarang menghitung `total_reward = subsidy + total_fees`.
   - Menghapus transaksi palsu "NETWORK" dan "MEMPOOL" dari pembuatan blok.

6. **Crate Crypto:**
   - Workspace terpisah `crypto/` untuk SHA3-256 dan AES-256-GCM.
   - Tidak bergantung pada `src/` untuk primitif kriptografi.

### 🚧 Sedang Dikerjakan / Tugas Berikutnya:
1. **Pindahkan Logika Reward+Fee ke `validation.rs`:**
   - Saat ini, logika `reward + fee` ada di `rpc.rs` (untuk pembuatan blok).
   - Perlu menambahkan field `subsidy` dan `total_fees` ke struct `Block`.
   - Implementasikan `rule_block_reward` di `validation.rs` untuk memastikan `block.reward == block.subsidy + block.total_fees`.
   - **Butuh Hard Fork**: Hapus `~/.zenachain`, regenerate genesis, update `EXPECTED_GENESIS_*`.

2. **Enkripsi Dompet:**
   - Dompet tetap terenkripsi di disk setelah `walletpassphrase`.
   - Jika node restart, dompet terkunci lagi (password hilang dari memori).

3. **Uji Sinkronisasi P2P:**
   - Perlu uji dua node yang saling sinkron blok dan transaksi lewat jaringan.
   - Pastikan `accept_block` memanggil `chain.adjust_difficulty()`.

4. **Build Gitian:**
   - `contrib/gitian-descriptors/` perlu diperbarui agar sesuai dengan `Cargo.toml` (versi Rust, dependensi).

---

## 3. Bug Diketahui
- **Saat ini tidak ada.** (Bug terakhir: `salt` berubah saat membuat alamat ketika dompet terenkripsi; SUDAH DIPERBAIKI dengan menolak `getnewwallet` jika dompet terkunci).

---

## 4. Cara Melanjutkan di Chat Baru
Jika chat limit tercapai, salin seluruh `PROGRESS.md` ini dan tempel ke chat baru, lalu katakan:

> *"Lanjut ZenaChain. Baca PROGRESS.md ini. Kita mau kerjain Tugas Berikutnya nomor 1 (pindahin logika reward ke validation.rs)."*

AI akan memahami konteks dan melanjutkan dari titik terakhir.

---

## 5. Potongan Kode Penting (Untuk Referensi)
- **Consensus Fingerprint:** `src/consensus/mod.rs` -> `consensus_fingerprint()`
- **Genesis:** `src/consensus/genesis.rs` -> `EXPECTED_GENESIS_HASH`, `GENESIS_TIMESTAMP`
- **Enkripsi Dompet:** `crypto/src/encryption.rs` -> `aes_encrypt`, `aes_decrypt`, `derive_key`
- **Logika Mining:** `src/rpc.rs` -> `m_generatetoaddress()`
- **Aturan Validasi:** `src/validation.rs` -> `validate_block()`, `validate_transaction()`
