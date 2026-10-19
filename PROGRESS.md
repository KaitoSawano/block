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
Testnet sudah berjalan terpisah dari mainnet. 
- ✅ `Network` enum & parameter testnet.
- ✅ Genesis testnet (`TESTNET_EXPECTED_GENESIS_*` sudah diisi).
- ✅ `Config.network` & `datadir` per network.
- ✅ `framing.rs` pakai magic `ZTES`.
- ✅ CLI `-testnet` / `--testnet` di `zenachaind` dan `zenachain`.
- ✅ Testnet Node berjalan di port 9422 (P2P) dan 12255 (RPC).
- ✅ Testnet Wallet berhasil dibuat (prefix `ZTES`).
- ✅ Testnet Mining berhasil (`generatetoaddress`).

**Genesis Testnet:**
- HASH: `65077d85315087ce53c9321fc8735e81e3656108c71dc1b60da2cb1f15aff110`
- MERKLE: `66b3fa4e2de637bb17964e16ecefe77e45872b8913f2aedc77b3fe354a2daf2b`
- TIMESTAMP: `1792303198446`
- NONCE: `1`

---

## 3. Tugas Berikutnya (FASE 1c: Penyempurnaan Testnet)
- [ ] Uji coba **Transfer (sendtoaddress) di Testnet** (pastikan fee & saldo berjalan).
- [ ] Uji coba **Sync 2 Node Testnet** (Node 2 di port 12256/9423).
- [ ] Uji coba **Mainnet Regression** (pastikan `./zenachaind` tanpa flag tetap jalan normal).
- [ ] Tambah dukungan flag `-testnet` di `src/bin/zenachain-miner.rs`.
- [ ] (Opsional) Rapikan warning `unused import` jika ada.

---

## 4. Bug Diketahui (Sebelum Mainnet Publik)
- Wallet nonce reuse AES-GCM (satu nonce untuk semua entry).
- PBKDF2 100.000 iterasi (OWASP: 600.000).
- P2P belum V2 transport (BIP324), compact blocks (BIP152), stalling detection.
- `ZENAMINERDEFAULT` bukan alamat valid (hanya placeholder).
- `MAX_HALVINGS = 64` (reward 0 setelah 33 halving).
- Gitian descriptors (masih pakai MSVC vs GNU, path LICENSE salah).

---

## 5. Cara Melanjutkan di Chat Baru
Copy file `PROGRESS.md` ini dan tempel ke chat baru, lalu ketik:
> "Lanjut ZenaChain. Baca PROGRESS.md ini. Fase 1a & 1b (Testnet Mode) selesai. Mainnet stabil. Jangan ubah kode transfer. Lanjut FASE 1c: Uji Transfer & Sync 2 Node Testnet."

---

## 6. Perintah Penting
### Mainnet
- Start: `./zenachaind`
- RPC: `./zenachain getinfo`
- Reset: `./zenachain stop && rm -rf ~/.zenachain`

### Testnet
- Start: `./zenachaind -testnet`
- RPC: `./zenachain -testnet getinfo`
- Wallet Baru: `./zenachain -testnet getnewwallet`
- Reset: `./zenachain -testnet stop && rm -rf ~/.zenachain-testnet`
