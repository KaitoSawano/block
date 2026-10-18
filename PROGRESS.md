# Progres Pengembangan ZenaChain

## 1. Gambaran Proyek
ZenaChain adalah blockchain berbasis Rust (konsep diadaptasi dari Dogecoin/Litecoin). Jaringan P2P menggunakan TCP mentah, RPC server multi-threaded, dan dompet multi-alamat dengan enkripsi AES-256-GCM.

### Parameter Konsensus Utama:
- **Algoritma Hash:** SHA3-256
- **Tanda Tangan:** secp256k1-ECDSA
- **Max Supply:** 28.000.000.000 ZENAC
- **Reward Awal (Subsidy):** 50 ZENAC
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

2. **Jaringan P2P (Berfungsi Penuh):**
   - Transportasi TCP mentah, framing ala Bitcoin.
   - Handshake Version/Verack berhasil.
   - Sinkronisasi headers-first dan relay blok berhasil (Node 2 menerima blok dari Node 1).
   - **Address Book Optimization:** Hanya menerima alamat dengan port default (8333), memberikan *backoff* (gagal 3x = hapus), dan mencegah spam "Connection refused".
   - **Self-Connection Detection:** Node mem-ban peer yang mengirim blok duplikat/lama (fitur keamanan).

3. **RPC Server:**
   - Multi-threaded.
   - Perintah `stop` langsung menghentikan mining.
   - RPC: `getinfo`, `getblockchaininfo`, `getblock`, `getbalance`, `sendtoaddress`, `generatetoaddress`, `listunspent`, `getwallet`, `encryptwallet`, `walletpassphrase`, `getpeerinfo`, dll.

4. **Dompet:**
   - Dompet multi-alamat (file JSON `wallet.dat`).
   - Enkripsi AES-256-GCM dengan PBKDF2 (100.000 iterasi).
   - Sesi buka kunci (`walletpassphrase`) menyimpan password di memori.
   - Tidak bisa membuat alamat baru atau mengirim dana jika dompet terkunci.

5. **Mining & Reward (100% Mirip Bitcoin):**
   - `Block` struct memiliki field terpisah: `subsidy`, `total_fees`, dan `reward` (penjumlahan keduanya).
   - `generatetoaddress` menghitung `total_fees` dari mempool dan menambahkannya ke `subsidy`.
   - Aturan validasi `rule_block_subsidy` dan `rule_block_reward` sudah dipindahkan ke `validation.rs`.
   - Penambang sekarang mendapatkan imbalan dari Subsidy + Fee.

6. **Crypto Crate:**
   - Workspace terpisah `crypto/` untuk SHA3-256 dan AES-256-GCM.

### 🚧 Sedang Dikerjakan / Tugas Berikutnya:
1. **Uji Sinkronisasi Transaksi:**
   - Kirim transaksi di Node A, pastikan Node B menerimanya di mempool (broadcast Inv/Tx).
   - Pastikan transaksi yang diterima divalidasi dan disimpan ke mempool Node B.

2. **Update Test:**
   - File test (`src/tests/*.rs`) mungkin perlu disesuaikan karena struktur `Block` berubah (menambahkan `subsidy` dan `total_fees`).
   - `zenachain-miner.rs` sudah diperbaiki agar memanggil `Block::new_for` dengan 7 argumen.

3. **Build Gitian:**
   - `contrib/gitian-descriptors/` perlu diperbarui agar sesuai dengan `Cargo.toml` (versi Rust, dependensi).

---

## 3. Bug Diketahui
- **Saat ini tidak ada bug fatal.** 
- **Catatan Penting:** Jika `Cargo.lock` terhapus karena `rm -rf`, jalankan `cargo generate-lockfile` sebelum `make build` (karena flag `--locked` akan error jika file lock tidak ada).
- **Peer Spam (Solved):** Sebelumnya Node spam "Connection refused" karena menyimpan port acak. Sudah diperbaiki dengan filter port di `network.rs` dan *backoff* di `addr_book.rs`.

---

## 4. Cara Melanjutkan di Chat Baru
Jika chat limit tercapai, salin seluruh `PROGRESS.md` ini dan tempel ke chat baru, lalu katakan:

> *"Lanjut ZenaChain. Baca PROGRESS.md ini. Kita mau kerjain Tugas Berikutnya (Uji Sinkronisasi Transaksi)."*

AI akan memahami konteks dan melanjutkan dari titik terakhir.

---

## 5. Potongan Kode Penting (Untuk Referensi)
- **Consensus Fingerprint:** `src/consensus/mod.rs` -> `consensus_fingerprint()`
- **Genesis:** `src/consensus/genesis.rs` -> `EXPECTED_GENESIS_HASH`, `GENESIS_TIMESTAMP`
- **Enkripsi Dompet:** `crypto/src/encryption.rs` -> `aes_encrypt`, `aes_decrypt`, `derive_key`
- **Mining Logic:** `src/rpc.rs` -> `m_generatetoaddress()`
- **Aturan Validasi:** `src/validation.rs` -> `validate_block()`, `rule_block_subsidy()`, `rule_block_reward()`
- **Struktur Blok:** `src/block.rs` -> `Block` struct (`subsidy`, `total_fees`, `reward`).
- **P2P Network:** `src/p2p/network.rs` -> `Network::run()`, `handle_block()`, `handle_message()`.
- **Address Book:** `src/p2p/addr_book.rs` -> `mark_failed()`, `candidates()`.
