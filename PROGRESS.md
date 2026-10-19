# Progres Pengembangan ZenaChain

## 1. Gambaran Proyek
ZenaChain adalah blockchain berbasis Rust dengan model account-based (seperti Ethereum). Jaringan P2P menggunakan TCP mentah, RPC server multi-threaded, dan dompet multi-alamat dengan enkripsi AES-256-GCM.

Sekarang mendukung dua jaringan: mainnet dan testnet, dengan parameter, genesis, magic bytes, port, address prefix, dan datadir yang terpisah sepenuhnya.

### Parameter Konsensus Utama
| Parameter | Mainnet | Testnet |
|---|---|---|
| Algoritma Hash | SHA3-256 | SHA3-256 |
| Tanda Tangan | secp256k1-ECDSA | secp256k1-ECDSA |
| Max Supply | 28.000.000.000 ZENAC | 28.000.000.000 ZENAC |
| Reward Awal | 50 ZENAC | 50 ZENAC |
| Coinbase Maturity | TIDAK ADA | TIDAK ADA |
| Waktu Blok | 15 detik | 15 detik |
| Difficulty Adjust | 69.120 blok | 69.120 blok |
| Magic Bytes | `ZENA` (5a454e41) | `ZTES` (5a544553) |
| P2P Port | 8333 | 9422 |
| RPC Port | 8332 | 12255 |
| Address Prefix | `ZENA` | `ZTES` |
| Datadir | `~/.zenachain` | `~/.zenachain-testnet` |
| Genesis Timestamp | 1792303198445 | 1792303198446 |
| Genesis NBITS | 0x1e0ffff0 | 0x207fffff |
| Genesis Nonce | 420282 | 1 |

---

## 2. Status Terkini (Terakhir Diperbarui: 2026-10-19)

### 🟢 CHECKPOINT: `stabil` (Mainnet)
Sistem transfer account-based mainnet STABIL dan TERUJI. **JANGAN UBAH** kode berikut kecuali ada bug nyata:
- `src/transaction.rs`, `src/blockchain.rs`, `src/validation.rs`, `src/storage.rs`, `src/p2p/sync.rs`.
- `src/rpc.rs` (khusus `m_sendtoaddress`, `m_getbalance`, `m_getnonce`).

### 🟢 FASE 1a & 1b: TESTNET MODE (SELESAI & TERVERIFIKASI)
Testnet sudah berjalan terpisah dari mainnet dan berhasil diuji sampai ribuan blok.
- ✅ `Network` enum & parameter testnet terpisah.
- ✅ Genesis testnet (`TESTNET_EXPECTED_GENESIS_*` sudah diisi, nonce 1).
- ✅ Config, Datadir, Address Prefix terpisah.
- ✅ Magic bytes `ZTES` di `framing.rs`.
- ✅ CLI `-testnet` / `--testnet` berfungsi di `zenachaind` dan `zenachain`.
- ✅ Testnet Node berjalan (P2P 9422, RPC 12255).
- ✅ Testnet Wallet berhasil dibuat (prefix `ZTES`).
- ✅ Testnet Mining berhasil (9.294 blok).
- ✅ **Sync Stall Fixed:** Batas batch dinaikkan jadi 5.000 header dan langsung memicu `GetHeaders` setelah blok diterima. Node 2 berhasil menyusul Node 1 tanpa ngestuck di kelipatan 2000.

---

## 3. Tugas Berikutnya (FASE 1c: Penyempurnaan)

- [ ] **Uji Transfer Testnet:** Kirim koin antar wallet `ZTES` (pastikan saldo dan fee terpotong).
- [ ] **Uji Sync 2 Node (Post-Fix):** Pastikan Node 2 bisa menyusul Node 1 sampai blok terakhir tanpa spam error.
- [ ] **Uji Mainnet Regression:** Pastikan `./zenachaind` (tanpa flag) masih berjalan normal setelah perubahan testnet.
- [ ] **Tambah flag `-testnet`:** Di binary `zenachain-miner`.
- [ ] **Moving Download Window (Todo Mainnet):** Batasi request blok maksimal 16 per peer agar RAM tidak jebol saat sync ribuan blok.

---

## 4. Bug / Catatan Penting

### 🟠 Penting (Sebelum Mainnet)
- **Memory Exhaustion:** Saat ini `GetData` meminta semua header yang diterima sekaligus. Jika ada ribuan blok besar, RAM bisa OOM. Harus dipecah (Moving Download Window).
- **Wallet nonce reuse AES-GCM:** Satu nonce untuk semua entry.
- **PBKDF2 100.000 iterasi** (OWASP: 600.000).
- **P2P:** Belum ada V2 transport (BIP324) dan compact blocks (BIP152).
- **Difficulty Adjustment Testnet:** Sistem berjalan sempurna. Di blok 69.120, kesulitan naik drastis (mining melambat dari milidetik jadi puluhan detik). Ini fitur, bukan bug.

### 🟡 Minor
- Gitian descriptors (path LICENSE salah, MSVC vs GNU).
- `zenachain-miner` belum support flag `-testnet`.
- `ZENAMINERDEFAULT` di config hanya placeholder.

---

## 5. Cara Melanjutkan di Chat Baru
Copy file ini dan tempel ke chat baru, lalu ketik:
> "Lanjut ZenaChain. Baca PROGRESS.md ini. Fase 1a & 1b (Testnet Mode) selesai. Mainnet stabil. Jangan ubah kode transfer. Lanjut FASE 1c: Uji Transfer & Sync 2 Node Testnet."

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
