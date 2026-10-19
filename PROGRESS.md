# Progres Pengembangan ZenaChain

## 1. Gambaran Proyek
ZenaChain adalah blockchain berbasis Rust dengan model **account-based**
(seperti Ethereum), bukan UTXO (seperti Bitcoin). Jaringan P2P menggunakan
TCP mentah dengan framing ala Bitcoin, RPC server multi-threaded, dan
dompet multi-alamat dengan enkripsi AES-256-GCM.

### Parameter Konsensus Utama (Mainnet):
- **Algoritma Hash:** SHA3-256
- **Tanda Tangan:** secp256k1-ECDSA (signing_hash terpisah dari txid)
- **Max Supply:** 28.000.000.000 ZENAC
- **Reward Awal (Subsidy):** 50 ZENAC
- **Coinbase Maturity:** TIDAK ADA (account-based, seperti Ethereum)
- **Waktu Blok:** 15 detik
- **Penyesuaian Difficulty:** Setiap 69.120 blok
- **Magic Bytes:** `ZENA` (0x5a, 0x45, 0x4e, 0x41)
- **P2P Port:** 8333
- **RPC Port:** 8332
- **Address Prefix:** `ZENA`
- **Datadir:** `~/.zenachain`

### Struktur Proyek:
- `src/` : Crate utama node (`zenachain`)
- `crypto/` : Crate workspace terpisah untuk kriptografi (`zenachain-crypto`)
- `contrib/` : Skrip build Gitian
- `.github/workflows/` : CI/CD (ci.yml, release.yml)

---

## 2. Status Terkini (Terakhir Diperbarui: 2026-10-19)

### 🟢 CHECKPOINT: `stabil`

Sistem transfer account-based **STABIL** dan **TERUJI**. Tag Git `stabil`
sudah dibuat (atau akan dibuat). **JANGAN UBAH** kode berikut kecuali ada
bug nyata yang bisa direproduksi:

- `src/transaction.rs` — `nonce`, `signing_hash`, `compute_txid`
- `src/blockchain.rs` — `apply_transaction`, `credit_reward`,
  `next_nonce_for`, `add_transaction`
- `src/validation.rs` — `rule_nonce`, `rule_sufficient_balance`,
  `rule_signature`, `validate_block_transactions`
- `src/rpc.rs` — `m_sendtoaddress`, `m_getbalance`, `m_getnonce`
- `src/storage.rs` — `ChainState` (`balances`, `nonces`)
- `src/p2p/sync.rs` — `accept_block`

**Hard fork** diperlukan jika mengubah:
- `compute_txid`, `signing_hash`, `calculate_hash`, `compute_merkle_root`
- `consensus_fingerprint`, `parameter_fingerprint`
- `rule_*` di `validation.rs`
- `apply_transaction`, `credit_reward` di `blockchain.rs`

### ✅ Fitur yang Sudah Selesai (Stabil):

#### A. Konsensus & Genesis (Mainnet)
- Genesis block dikunci dengan timestamp dan hash statis.
- `consensus_fingerprint` mengikat parameter ekonomi + genesis hash
  (hard fork detection).
- `parameter_fingerprint` terpisah untuk `genesis_coinbase_txid`.
- `rule_block_subsidy` (E031), `rule_block_total_fees` (E047),
  `rule_block_reward` (E046), `rule_block_coinbase_consistent` (E048),
  `rule_mined_for_valid` (E049).

#### B. Model Transaksi (Account-Based)
- `Transaction`: `txid`, `from`, `to`, `amount`, `fee`, **`nonce`**,
  `note`, `public_key`, `signature`, `output_script`.
- `compute_txid` mengikat semua field **kecuali** `signature`.
- `signing_hash` mengikat semua field **termasuk** `nonce` dan
  `public_key`, **kecuali** `signature`.
- `rule_nonce` (E022), `rule_sufficient_balance` (E021),
  `rule_signature` (E010) pakai `signing_hash`.
- `is_special_tx` ditolak dari wire P2P.

#### C. State Chain (Account-Based)
- `Blockchain`: `chain`, `nbits`, `mempool`, **`balances`**, **`nonces`**.
- `apply_transaction` (checked_*), `credit_reward`, `next_nonce_for`,
  `add_transaction` (dedup txid & nonce).

#### D. Jaringan P2P (Stabil & Teruji)
- TCP mentah, framing ala Bitcoin, handshake Version/Verack.
- Headers-first sync, relay blok, Inv/GetData.
- Push notification via `tokio::sync::mpsc`.
- `inv.add_block`/`add_tx` sebelum broadcast (anti-loop).
- `handle_headers` toleran terhadap header yang sudah ada.
- `GetData` multi-item, `write_buf` limit 10 MB, `flush_write` disconnect.
- `try_feeler` mengecualikan peer terhubung.
- `command_rx.recv()` return `None` -> break loop.
- **Anti-eclipse:** `anchors.rs` (2 block-relay-only, persist ke
  `anchors.dat`), `banman.rs` (ban persisten + discourage ephemeral).
- **Tested:** 2 node sync 30+ blok, transfer 248 & 5051 ZENAC,
  restart persistence.

#### E. RPC Server
- Multi-threaded, `stop` menyimpan chain.
- `listen()` return `Result` (bind error dipropagasikan).
- `zenachaind` menolak argumen posisi.
- `getpeerinfo`, `getnetworkinfo`, `getblocktemplate` (subsidy_satoshi,
  total_fees_satoshi, reward_satoshi), `getblockcount` (height),
  `getbalance` (total wallet / per address), `getlistwallet`,
  `getnonce` (nonce + next_nonce).
- **`sendtoaddress`:**
  - Amount sebagai **string** (presisi penuh) atau JSON number.
  - Parser `parse_zenac_to_satoshi` — tanpa `f64`, mendukung
    `"0.00000001"` hingga `"28000000000.00000001"`.
  - Menolak amount > `MAX_SUPPLY` dan amount = 0.
  - Memilih address dengan saldo terbesar yang cukup, **mengecualikan
    address tujuan** (anti self-send).
  - Error jelas: "insufficient wallet balance" vs "no single wallet
    address has sufficient balance".
  - **`sendall`** — kirim seluruh saldo dikurangi fee.

#### F. Dompet
- Multi-alamat (`wallet.dat`), AES-256-GCM, PBKDF2 100.000 iterasi.
- `walletpassphrase`, `encryptwallet`, `backupwallet`.
- **Nonce TIDAK disimpan di wallet** — dikelola `Blockchain`.

#### G. Crypto Crate
- `crypto/` workspace terpisah: `sha3_256_hex`, `derive_key`,
  `aes_encrypt`, `aes_decrypt`.

#### H. Storage
- `ChainState`: `total_blocks`, `nbits`, `updated_at`, **`balances`**,
  **`nonces`**.
- `load_chain` verifikasi genesis hash (mencegah chain dari genesis lama).

#### I. Build & CI/CD
- `Makefile`, `build.sh`, `ci.yml`, `release.yml`, Gitian descriptors
  (perlu perbaikan — lihat Bug Diketahui).
- `Cargo.lock` di-commit. `.gitignore` benar.

#### J. Test Infrastructure
- Unit test (`framing.rs`, `handshake.rs`, `message.rs`).
- Integration test (`src/tests/`): genesis, halving, pow, secp256k1,
  storage, validation.
- Test baru: `txid_changes_when_public_key_changes`,
  `txid_changes_when_signature_changes`, `block_total_fees_mismatch_is_rejected`,
  `block_with_invalid_mined_for_is_rejected`,
  `block_with_inconsistent_coinbase_is_rejected`.

### 🚧 Sedang Dikerjakan / Tugas Berikutnya:

#### FASE 1: Testnet Mode (SEGERA DIMULAI)

**Tujuan:** Memisahkan jaringan testnet dari mainnet **tanpa mengubah
logika mainnet**.

**Parameter Testnet:**
- **Magic bytes:** `ZTES` = `[0x5a, 0x54, 0x45, 0x53]`
- **Genesis timestamp:** `1792303198446` (mainnet + 1)
- **GENESIS_NBITS:** `0x207fffff` (mudah, mining instan)
- **GENESIS_NONCE:** dihitung otomatis saat mining genesis testnet
- **Address prefix:** `ZTES`
- **P2P port:** `9422`
- **RPC port:** `12255`
- **Datadir:** `~/.zenachain-testnet`

**File yang akan diubah (additive):**
- `src/consensus/spec.rs` — tambah `Network` enum + parameter testnet
- `src/consensus/genesis.rs` — tambah genesis testnet
- `src/consensus/mod.rs` — `consensus_fingerprint` per network
- `src/config.rs` — tambah `network: Network`
- `src/datadir.rs` — datadir per network
- `src/blockchain.rs` — `network()` method (hanya akses, tidak ubah logika)
- `src/block.rs` — `genesis()` pakai network
- `src/secp256k1/address.rs` — `address_from_public_key` pakai prefix network
- `src/validation.rs` — `rule_address_format` pakai prefix network
- `src/main.rs` — flag `--testnet`
- `src/bin/zenachaind.rs` — flag `--testnet`
- `src/rpc.rs` — `getinfo` tampilkan `network`

**Prinsip:** **Jangan ubah** logika mainnet. Hanya **tambah** testnet.
Setiap fase, jalankan `cargo test` dan test manual mainnet. Kalau ada
regresi, rollback.

**Fase:**
1. Fase 1a — `Network` enum + parameter testnet (`spec.rs`)
2. Fase 1b — genesis testnet (`genesis.rs`, `mod.rs`)
3. Fase 1c — `Config` + `datadir` per network
4. Fase 1d — address prefix + validation per network
5. Fase 1e — CLI flag `--testnet`
6. Fase 1f — RPC `getinfo` network

**Setelah Fase 1 selesai:**
- Testnet genesis di-mine, `EXPECTED_GENESIS_*` testnet diisi
- Mainnet tetap berfungsi (regression test)
- Commit dengan pesan "feat: testnet mode"

#### FASE 2: Regtest (NANTI)
- Parameter regtest (magic, port, nbits, genesis)
- Mining instan (nbits mudah)
- Datadir `~/.zenachain-regtest`

#### FASE 3: Testnet Publik (NANTI)
- Buka GitHub Issues
- Ajak komunitas test
- Bug bounty kecil

---

## 3. Bug Diketahui

### 🔴 Kritis
- **Tidak ada** bug konsensus kritis yang diketahui saat ini.

### 🟠 Penting (sebelum mainnet)
- **Wallet nonce reuse AES-GCM** — `wallet.rs` memakai satu nonce untuk
  semua entry. Harus generate nonce unik per entry. (Belum diperbaiki.)
- **`derive_key` `unwrap_or_default()`** pada salt invalid.
- **PBKDF2 100.000 iterasi** — di bawah OWASP (600.000).
- **`Block::calculate_hash`** format string `|`, bukan binary Bitcoin.
- **`compute_txid`** SHA3-256, bukan double-SHA256 Bitcoin.
- **Tidak ada maturity** — keputusan desain (account-based).
- **`mempool` belum punya eviction policy.**
- **P2P belum punya V2 transport (BIP324).**
- **P2P belum punya compact block relay (BIP152).**
- **P2P belum punya moving download window.**
- **P2P belum punya stalling detection.**

### 🟡 Minor
- **`ZENAMINERDEFAULT`** di `Config::new()` bukan alamat valid.
- **`MAX_HALVINGS = 64`** tapi reward 0 setelah 33 halving.
- **`MINER_CONFIRMATION_WINDOW`** dead code.
- **`GENESIS_TIMESTAMP`** duplikat `EXPECTED_GENESIS_TIMESTAMP`.
- **`GENESIS_OUTPUT_SCRIPT`** placeholder non-hex.
- **Gitian descriptors** — `cp LICENSE` (file `COPYING`), `cp README.md`
  (tidak ada), Windows MSVC vs Gitian GNU, `rustup` tidak di-pin,
  macOS tanpa `osxcross`.
- **`ci.yml`/`release.yml`** — action tidak di-pin SHA,
  `dtolnay/rust-toolchain@stable` tidak di-pin versi.
- **`peer_manager.rs`** — `add_peer` signature harus cocok.
- **`banman.rs`** — `Duration` unused import.

### ✅ Sudah Diperbaiki
(Lihat bagian 2 — semua fitur stabil di atas adalah hasil perbaikan.)

---

## 4. Cara Melanjutkan di Chat Baru
Jika chat limit tercapai, salin seluruh `PROGRESS.md` ini dan tempel ke
chat baru, lalu katakan:

> *"Lanjut ZenaChain. Baca PROGRESS.md ini. Sistem transfer sudah stabil
> (tag `stabil`). Kita mau mulai FASE 1: Testnet Mode. Jangan ubah kode
> transfer (transaction.rs, blockchain.rs, validation.rs, rpc.rs bagian
> sendtoaddress/getbalance/getnonce, storage.rs, sync.rs) kecuali ada bug
> nyata."*

AI akan memahami konteks dan melanjutkan dari titik terakhir.

---

## 5. Potongan Kode Penting (Untuk Referensi)
- **Consensus Fingerprint:** `src/consensus/mod.rs` ->
  `consensus_fingerprint()`, `parameter_fingerprint()`
- **Genesis:** `src/consensus/genesis.rs` -> `EXPECTED_GENESIS_HASH`,
  `GENESIS_TIMESTAMP`
- **Enkripsi Dompet:** `crypto/src/encryption.rs` -> `aes_encrypt`,
  `aes_decrypt`, `derive_key`
- **Mining Logic:** `src/rpc.rs` -> `m_generatetoaddress()`
- **Aturan Validasi:** `src/validation.rs` -> `validate_block()`,
  `validate_transaction()`, `rule_nonce()`, `rule_sufficient_balance()`
- **Struktur Blok:** `src/block.rs` -> `Block` struct
- **Struktur Transaksi:** `src/transaction.rs` -> `Transaction` struct
  (`nonce`, `signing_hash`, `compute_txid`)
- **State Chain:** `src/blockchain.rs` -> `Blockchain` struct
  (`balances`, `nonces`), `apply_transaction()`, `credit_reward()`,
  `next_nonce_for()`
- **P2P Network:** `src/p2p/network.rs` -> `Network::run()`,
  `handle_command()`, `handle_message()`, `handle_headers()`,
  `handle_block()`
- **Anti-Eclipse:** `src/p2p/anchors.rs`, `src/p2p/banman.rs`
- **RPC sendtoaddress:** `src/rpc.rs` -> `m_sendtoaddress()`
- **RPC getbalance:** `src/rpc.rs` -> `m_getbalance()`
- **Storage:** `src/storage.rs` -> `ChainState` (`balances`, `nonces`)

---

## 6. Catatan Penting

### Hard Fork
- Setiap perubahan pada `consensus_fingerprint` adalah **hard fork**.
- Setiap perubahan pada `compute_txid`, `calculate_hash`,
  `compute_merkle_root`, `signing_hash`, `address_from_public_key`
  adalah **hard fork** (meski tidak masuk fingerprint).
- Setelah hard fork: hapus datadir, regenerate genesis
  (`cargo run --release --bin zenachain-genesis`), update
  `EXPECTED_GENESIS_*`.

### Build
- `make build` — build release, copy binary ke root.
- `make test` — unit + integration test.
- `make ci` — fmt-check + clippy + test.
- `cargo build --release --locked --bins` — build langsung.

### Menjalankan Node (Mainnet)
- Node 1: `./zenachaind`
- Node 2: `ZENAC_DATADIR=/root/.zenachain2 ./zenachaind`
  (dengan `zenachain.conf` di `/root/.zenachain2/` yang berisi
  `rpcport=8334`, `port=8335`, `bootstrap=127.0.0.1:8333`)
- Stop: `./zenachain stop`

### Menjalankan Node (Testnet — setelah Fase 1)
- Node 1: `./zenachaind --testnet`
- Node 2: `ZENAC_DATADIR=/root/.zenachain-testnet2 ./zenachaind --testnet`
  (dengan `zenachain.conf` di `/root/.zenachain-testnet2/` yang berisi
  `rpcport=12256`, `port=9423`, `bootstrap=127.0.0.1:9422`)
- Stop: `./zenachain --testnet stop`

### Reset Chain
- Mainnet: `./zenachain stop && rm -rf ~/.zenachain`
- Testnet: `./zenachain --testnet stop && rm -rf ~/.zenachain-testnet`
- Regenerate genesis jika perlu.

### Test yang Sudah Lulus (Mainnet)
- ✅ Build bersih
- ✅ Genesis deterministik
- ✅ Mining 107+ blok
- ✅ Kirim 248 ZENAC, 5051 ZENAC (presisi penuh)
- ✅ Kirim > MAX_SUPPLY (ditolak)
- ✅ Kirim 0 (ditolak)
- ✅ Self-send (ditolak)
- ✅ Double-spend nonce (dapat nonce berbeda)
- ✅ Sync 2 node (30+ blok)
- ✅ Persistensi (restart node)
- ✅ Verify chain
- ✅ Fee masuk ke miner
- ✅ Balance update
- ✅ Nonce increment

### Test yang Belum Lulus
- ⏳ Test `sendall`
- ⏳ Test invalid signature (E010)
- ⏳ Test replay attack
- ⏳ Test presisi `"0.00000001"` dan `"28000000000.00000001"`
- ⏳ Testnet mode (Fase 1 — segera)
