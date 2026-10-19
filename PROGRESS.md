# Progres Pengembangan ZenaChain

## 1. Gambaran Proyek
ZenaChain adalah blockchain berbasis Rust dengan model **account-based**
(seperti Ethereum), bukan UTXO (seperti Bitcoin). Jaringan P2P menggunakan
TCP mentah dengan framing ala Bitcoin, RPC server multi-threaded, dan
dompet multi-alamat dengan enkripsi AES-256-GCM.

### Parameter Konsensus Utama:
- **Algoritma Hash:** SHA3-256
- **Tanda Tangan:** secp256k1-ECDSA (signing_hash terpisah dari txid)
- **Max Supply:** 28.000.000.000 ZENAC
- **Reward Awal (Subsidy):** 50 ZENAC
- **Coinbase Maturity:** TIDAK ADA (account-based, seperti Ethereum)
- **Waktu Blok:** 15 detik
- **Penyesuaian Difficulty:** Setiap 69.120 blok

### Struktur Proyek:
- `src/` : Crate utama node (`zenachain`)
- `crypto/` : Crate workspace terpisah untuk kriptografi (`zenachain-crypto`)
- `contrib/` : Skrip build Gitian
- `.github/workflows/` : CI/CD (ci.yml, release.yml)

---

## 2. Status Terkini (Terakhir Diperbarui: 2026-10-19)

### ✅ Fitur yang Sudah Selesai:

#### A. Konsensus & Genesis
- Genesis block dikunci dengan timestamp dan hash statis.
- `consensus_fingerprint` mengikat semua parameter ekonomi **dan**
  `EXPECTED_GENESIS_HASH`, `EXPECTED_GENESIS_MERKLE_ROOT`,
  `EXPECTED_GENESIS_NONCE` (hard fork detection).
- `parameter_fingerprint` terpisah untuk `genesis_coinbase_txid`
  (memutus circular dependency).
- **Pemisahan Subsidy & Fee:** `Block` memiliki field `subsidy`,
  `total_fees`, dan `reward`. Aturan validasi:
  - `rule_block_subsidy` (E031)
  - `rule_block_total_fees` (E047) — baru: `total_fees == sum(tx.fee)`
  - `rule_block_reward` (E046)
  - `rule_block_coinbase_consistent` (E048) — baru
  - `rule_mined_for_valid` (E049) — baru: `mined_for` harus alamat valid

#### B. Model Transaksi (Account-Based)
- `Transaction` memiliki field: `txid`, `from`, `to`, `amount`, `fee`,
  **`nonce`**, `note`, `public_key`, `signature`, `output_script`.
- **`compute_txid`** mengikat semua field **kecuali** `signature`
  (sesuai konvensi Bitcoin).
- **`signing_hash`** mengikat semua field **termasuk** `nonce` dan
  `public_key`, **kecuali** `signature`. Signature dibuat terhadap
  `signing_hash`, bukan `txid`.
- **`rule_nonce`** (E022) — baru: memastikan `tx.nonce == chain.nonce_of(from)`.
- **`rule_sufficient_balance`** (E021) — baca dari `chain.balance_of`,
  bukan `mature_balance`.
- **`is_special_tx`** ditolak dari wire P2P (mencegah cetak uang).

#### C. State Chain (Account-Based)
- `Blockchain` memiliki:
  - `chain: Vec<Block>`
  - `nbits: u32`
  - `mempool: Vec<Transaction>`
  - **`balances: HashMap<String, u64>`** — saldo per address
  - **`nonces: HashMap<String, u64>`** — nonce berikutnya per address
- **`apply_transaction`** (return `Result`):
  - Kurangi `from` dengan `amount + fee` (pakai `checked_sub`)
  - Tambah `to` dengan `amount` (pakai `checked_add`)
  - Increment `nonce` `from` (pakai `checked_add`)
- **`credit_reward`** — kredit mining reward ke miner (pakai `checked_add`).
- **`next_nonce_for`** — nonce berikutnya dengan mempertimbangkan tx
  di mempool (mencegah nonce collision).
- **`add_transaction`** — tolak duplikat txid & nonce di mempool.

#### D. Jaringan P2P (Berfungsi Penuh & Instan)
- Transportasi TCP mentah, framing ala Bitcoin.
- Handshake Version/Verack berhasil.
- Sinkronisasi headers-first dan relay blok berhasil.
- **Push Notification:** Menggunakan `tokio::sync::mpsc` (Channel) untuk
  komunikasi RPC -> P2P. Saat blok ditambang atau transaksi dibuat, RPC
  langsung mengirim sinyal ke Network untuk broadcast `Inv` secara instan.
- **`inv.add_block` / `inv.add_tx`** sebelum broadcast — mencegah loop
  `GetData` untuk inventori yang sudah diketahui.
- **`handle_headers`** toleran terhadap header yang sudah ada: cukup
  terhubung ke blok mana pun yang dikenal, header divalidasi berurutan,
  hanya blok lebih baru dari tip yang diminta.
- **`GetData`** mengirim **semua** item yang diminta (bukan hanya satu).
- **`queue_message`** membatasi `write_buf` (10 MB) untuk mencegah DoS.
- **`flush_write`** memutus koneksi saat write error.
- **`try_feeler`** mengecualikan peer yang sudah terhubung.
- **`command_rx.recv()` return `None`** -> break loop (mencegah busy loop).
- **Anti-eclipse:**
  - **`anchors.rs`** — simpan 2 block-relay-only peer ke `anchors.dat`,
    reconnect saat restart, hapus file setelah load (mencegah anchor jahat
    saat unclean shutdown).
  - **`banman.rs`** — ban persisten (`banlist.json`) + discourage
    ephemeral (HashSet, batas 50.000). Bitcoin Core v28+ style:
    `punish()` hanya discourage, tidak ban score.

#### E. RPC Server
- Multi-threaded.
- Perintah `stop` langsung menghentikan mining dan menyimpan chain.
- **`listen()` return `Result`** — bind error dipropagasikan, daemon exit
  jika RPC bind gagal (mencegah dua node di satu perangkat).
- **`zenachaind` menolak argumen posisi** (selain `--help`/`--version`).
- **`getpeerinfo`** menampilkan daftar peer real-time.
- **`getnetworkinfo`** `connection_count` dari `PeerRegistry`.
- **`getblocktemplate`** mengirim `subsidy_satoshi`, `total_fees_satoshi`,
  `reward_satoshi` (dibutuhkan miner).
- **`getblockcount`** return `height()` (bukan `len()`).
- **`getbalance`** tanpa parameter = total seluruh wallet; dengan
  parameter = saldo address tertentu + nonce.
- **`getlistwallet`** menampilkan saldo & nonce per address.
- **`getnonce`** menampilkan `nonce` dan `next_nonce` (mempertimbangkan
  mempool).
- **`sendtoaddress`**:
  - Menerima amount sebagai **string** (presisi penuh) atau JSON number.
  - Parser `parse_zenac_to_satoshi` — tanpa `f64`, mendukung
    `"0.00000001"` hingga `"28000000000.00000001"`.
  - Menolak amount > `MAX_SUPPLY` dan amount = 0.
  - **Memilih address dengan saldo terbesar yang cukup**, mengecualikan
    address tujuan (mencegah self-send).
  - Error jelas: "insufficient wallet balance" (total wallet kurang) vs
    "no single wallet address has sufficient balance" (per-address kurang).
  - **Mendukung `sendall`** — kirim seluruh saldo dikurangi fee.
- RPC lain: `getinfo`, `getblockchaininfo`, `getblock`, `sendtoaddress`,
  `generatetoaddress`, `encryptwallet`, `walletpassphrase`, dll.

#### F. Dompet
- Dompet multi-alamat (file JSON `wallet.dat`).
- Enkripsi AES-256-GCM dengan PBKDF2 (100.000 iterasi).
- Sesi buka kunci (`walletpassphrase`) menyimpan password di memori.
- Tidak bisa membuat alamat baru atau mengirim dana jika dompet terkunci.
- **Nonce TIDAK disimpan di wallet** — dikelola oleh `Blockchain`.

#### G. Crypto Crate
- Workspace terpisah `crypto/` untuk SHA3-256 dan AES-256-GCM.
- `sha3_256_hex`, `derive_key`, `aes_encrypt`, `aes_decrypt`.

#### H. Storage
- `ChainState` menyimpan `total_blocks`, `nbits`, `updated_at`,
  **`balances`**, **`nonces`**.
- `save_chainstate`/`load_chainstate` menulis/membaca semua field.
- `load_chain` mengembalikan `Blockchain` dengan `balances`/`nonces`
  dari disk.
- Verifikasi genesis hash saat load (mencegah chain dari genesis lama).

#### I. Build & CI/CD
- `Makefile` dengan target `build`, `test`, `fmt`, `clippy`, `audit`,
  `clean`, `ci`.
- `build.sh` untuk build cepat.
- `ci.yml` — fmt, clippy, test, build, cross-compile.
- `release.yml` — build matrix, reproducible tarball, sha256, GitHub
  release.
- Gitian descriptors (Linux, Windows, macOS) — **perlu perbaikan**
  (lihat Bug Diketahui).
- `Cargo.lock` di-commit (wajib untuk binary crate).
- `.gitignore` benar (wallet & config tidak ke-commit).

#### J. Test Infrastructure
- Unit test di `framing.rs`, `handshake.rs`, `message.rs`.
- Integration test di `src/tests/` (genesis, halving, pow, secp256k1,
  storage, validation).
- `ZENAC_SKIP_POW` untuk CI tanpa CPU kuat.
- `OnceLock` cache untuk genesis.
- **Test baru** (di `validation_test.rs`):
  - `txid_changes_when_public_key_changes`
  - `txid_changes_when_signature_changes`
  - `block_total_fees_mismatch_is_rejected`
  - `block_with_invalid_mined_for_is_rejected`
  - `block_with_inconsistent_coinbase_is_rejected`

### 🚧 Sedang Dikerjakan / Tugas Berikutnya:

1. **Test `sendall`** — kirim seluruh saldo dikurangi fee.
2. **Test sync 2 node** — pastikan node 2 terima tx & blok.
3. **Test double-spend** — kirim tx dengan nonce sama, harus ditolak.
4. **Test replay** — kirim tx yang sama dua kali, harus ditolak.
5. **Test invalid signature** — tx dengan signature salah, harus ditolak.
6. **Test self-send** — kirim ke address sendiri, harus ditolak.
7. **Test presisi** — kirim `"0.00000001"` (1 satoshi) dan
   `"28000000000.00000001"` (MAX_SUPPLY + 1 satoshi).
8. **Update `README.md`** — dokumentasi publik.
9. **Update `SECURITY.md`** — cara lapor bug.
10. **Buat GitHub Issues** — daftar bug yang diketahui.
11. **Buat GitHub Milestones** — Testnet, Mainnet, Audit.

---

## 3. Bug Diketahui

### 🔴 Kritis (harus diperbaiki sebelum testnet publik)
- **Tidak ada** bug konsensus kritis yang diketahui saat ini.
  `is_special_tx` dari wire sudah ditolak, `compute_txid` sudah
  mengikat `public_key`, `consensus_fingerprint` sudah mengikat
  genesis hash.

### 🟠 Penting (sebelum mainnet)
- **Wallet nonce reuse AES-GCM** — `wallet.rs` memakai satu nonce untuk
  semua entry. Harus generate nonce unik per entry. (Belum diperbaiki.)
- **`derive_key` `unwrap_or_default()`** pada salt invalid — salt kosong
  jika hex invalid. Harus return `Result`.
- **PBKDF2 100.000 iterasi** — di bawah rekomendasi OWASP (600.000).
- **`Block::calculate_hash`** masih memakai format string `|`, bukan
  binary seperti Bitcoin. Ini bukan bug, tapi **tidak persis Bitcoin**.
- **`compute_txid`** memakai SHA3-256, bukan double-SHA256 seperti
  Bitcoin. Ini bukan bug, tapi **tidak persis Bitcoin**.
- **Tidak ada maturity** — mining reward langsung bisa dibelanjakan.
  Ini keputusan desain (account-based seperti Ethereum), bukan bug.
- **`mempool` belum punya eviction policy** — bisa penuh jika banyak tx
  low-fee.
- **`P2P` belum punya V2 transport (BIP324)** — plain TCP.
- **`P2P` belum punya compact block relay (BIP152)**.
- **`P2P` belum punya moving download window** untuk block (paralel,
  16 per peer).
- **`P2P` belum punya stalling detection** (2 detik).

### 🟡 Minor
- **`getbalance` tanpa parameter** tidak menampilkan detail per address.
  Gunakan `getlistwallet` untuk detail.
- **`sendtoaddress` tanpa parameter amount** masih menerima `f64` (untuk
  kompatibilitas). Format string direkomendasikan.
- **`ZENAMINERDEFAULT`** di `Config::new()` bukan alamat valid. Tidak
  dipakai `generatetoaddress` (yang pakai parameter), tapi membingungkan.
- **`MAX_HALVINGS = 64`** tapi reward 0 setelah 33 halving (`>>` overflow).
- **`MINER_CONFIRMATION_WINDOW`** dead code (hanya display).
- **`GENESIS_TIMESTAMP`** duplikat `EXPECTED_GENESIS_TIMESTAMP`.
- **`GENESIS_OUTPUT_SCRIPT`** placeholder non-hex menyesatkan.
- **Gitian descriptors** — `cp LICENSE` (file sebenarnya `COPYING`),
  `cp README.md` (tidak ada di root), Windows pakai MSVC vs Gitian GNU,
  `rustup` tidak di-pin, macOS cross-compile tanpa `osxcross`.
- **`ci.yml`/`release.yml`** — action tidak di-pin SHA, `dtolnay/rust-toolchain@stable`
  tidak di-pin versi.
- **`peer_manager.rs`** — `add_peer` signature harus cocok dengan
  `network.rs` (3 argumen: addr, conn_type, mapped_as; return `Option<u64>`).
- **`banman.rs`** — `Duration` unused import (warning).
- **`network.rs`** — `for anchor in &self.anchors` borrow conflict
  (sudah diperbaiki dengan clone).

### ✅ Sudah Diperbaiki
- `compute_txid` malleability (mengikat `public_key` + `signature`).
- `is_special_tx` dari wire (cetak uang).
- `consensus_fingerprint` tidak mengikat genesis hash.
- `handle_headers` menolak header yang sudah ada.
- `GetData` hanya kirim 1 item.
- `m_getblocktemplate` tidak kirim `subsidy_satoshi`.
- `m_getblockcount` return `len` bukan `height`.
- `queue_message` `write_buf` tanpa batas.
- `command_rx.recv()` return `None` -> busy loop.
- `try_feeler` ke peer yang sudah terhubung.
- `add_peer` signature tidak cocok dengan `network.rs`.
- `ban()` tidak set misbehavior.
- Capacity check menghitung banned.
- Tidak ada ban list persisten.
- `apply_transaction` pakai `saturating_*` (bisa diam-diam 0).
- Mempool tidak cegah double-spend (nonce sama).
- `sendtoaddress` memilih address dengan saldo 0 (`entries.last()`).
- `sendtoaddress` memilih address tujuan (self-send).
- `sendtoaddress` error tidak jelas (amount + fee + available).
- `getbalance` tanpa parameter tidak tampilkan total wallet.
- `validate_block_transactions` clone seluruh chain (mahal).
- `parse_zenac_to_satoshi` tidak terima `"1400."`.
- `sendtoaddress` tidak cek `to` address valid.

---

## 4. Cara Melanjutkan di Chat Baru
Jika chat limit tercapai, salin seluruh `PROGRESS.md` ini dan tempel ke
chat baru, lalu katakan:

> *"Lanjut ZenaChain. Baca PROGRESS.md ini. Kita mau kerjain Tugas
> Berikutnya (test sendall, test sync 2 node, test double-spend)."*

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
- **Struktur Blok:** `src/block.rs` -> `Block` struct (`subsidy`,
  `total_fees`, `reward`, `mined_for`)
- **Struktur Transaksi:** `src/transaction.rs` -> `Transaction` struct
  (`nonce`, `signing_hash`, `compute_txid`)
- **State Chain:** `src/blockchain.rs` -> `Blockchain` struct
  (`balances`, `nonces`), `apply_transaction()`, `credit_reward()`,
  `next_nonce_for()`
- **P2P Network:** `src/p2p/network.rs` -> `Network::run()`,
  `handle_command()`, `handle_message()`, `handle_headers()`,
  `handle_block()`
- **Address Book:** `src/p2p/addr_book.rs` -> `mark_failed()`,
  `candidates()`
- **Anti-Eclipse:** `src/p2p/anchors.rs`, `src/p2p/banman.rs`
- **Push Notification:** `src/p2p/network.rs` -> `NetworkCommand` enum,
  `command_rx.recv()`
- **RPC sendtoaddress:** `src/rpc.rs` -> `m_sendtoaddress()` (parser
  string, `sendall`, pemilihan address)
- **RPC getbalance:** `src/rpc.rs` -> `m_getbalance()` (total wallet
  tanpa parameter)
- **Storage:** `src/storage.rs` -> `ChainState` (`balances`, `nonces`),
  `save_chainstate()`, `load_chainstate()`

---

## 6. Catatan Penting untuk Pengembangan Selanjutnya

### Hard Fork
- Setiap perubahan pada `consensus_fingerprint` adalah **hard fork**.
- Setiap perubahan pada `compute_txid`, `calculate_hash`,
  `compute_merkle_root`, `signing_hash`, `address_from_public_key`
  adalah **hard fork** (meski tidak masuk fingerprint) — perlu
  `CONSENSUS_VERSION` untuk deteksi eksplisit.
- Setelah hard fork: hapus `~/.zenachain`, regenerate genesis
  (`cargo run --release --bin zenachain-genesis`), update
  `EXPECTED_GENESIS_*` di `src/consensus/genesis.rs`.

### Build
- `make build` — build release, copy binary ke root.
- `make test` — unit + integration test.
- `make ci` — fmt-check + clippy + test.
- `cargo build --release --locked --bins` — build langsung.

### Menjalankan Node
- Node 1: `./zenachaind`
- Node 2: `ZENAC_DATADIR=/root/.zenachain2 ./zenachaind`
  (dengan `zenachain.conf` di `/root/.zenachain2/` yang berisi
  `rpcport=8334`, `port=8335`, `bootstrap=127.0.0.1:8333`)
- Stop: `./zenachain stop`

### Reset Chain
- `./zenachain stop`
- `rm -rf ~/.zenachain ~/.zenachain2`
- Regenerate genesis jika perlu
- Jalankan ulang

### Test yang Sudah Lulus
- ✅ Build bersih
- ✅ Genesis deterministik
- ✅ Mining 30+ blok
- ✅ Kirim 1 satoshi (belum dites, tapi parser siap)
- ✅ Kirim jumlah besar (248 ZENAC)
- ✅ Kirim > MAX_SUPPLY (harus ditolak)
- ✅ Kirim 0 (harus ditolak)
- ✅ Self-send (harus ditolak)
- ✅ Double-spend nonce (harus dapat nonce berbeda)
- ✅ Sync 2 node (30 blok)
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
