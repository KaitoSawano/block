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
- `src/transaction.rs`, `src/blockchain.rs`, `src/storage.rs`, `src/p2p/sync.rs`.
- `src/validation.rs` (kecuali aturan baru untuk *Policy*).
- `src/rpc.rs` (khusus `m_sendtoaddress`, `m_getbalance`, `m_getnonce`).

### 🟢 FASE 1a, 1b, 1c: TESTNET MODE (SELESAI & STRES TEST)
Testnet sudah berjalan terpisah dari mainnet, sinkronisasi P2P sudah stabil, dan sistem tahan banting terhadap spam transaksi.
- ✅ `Network` enum & parameter testnet terpisah.
- ✅ Genesis testnet (nonce 1).
- ✅ Config, Datadir, Address Prefix, Magic bytes terpisah.
- ✅ CLI `-testnet` berfungsi.
- ✅ Testnet Node (P2P 9422, RPC 12255).
- ✅ **Sync Stall Fixed:** Batas batch dinaikkan jadi 5.000 header. Node 2 bisa menyusul Node 1 tanpa ngestuck.
- ✅ **Graceful Shutdown:** RPC `stop` berhasil menyelamatkan data meski mining berjalan.
- ✅ **Nonce Mempool Fix:** `rule_nonce` di `validation.rs` sekarang memindai mempool, memungkinkan pengiriman ribuan transaksi beruntun tanpa error `E022`.
- ✅ **Stress Test Mempool:** Berhasil menampung ribuan transaksi (2.000+ tx) di mempool tanpa crash, membuktikan manajemen memori Rust dan RPC berjalan efisien.

---

## 3. Tugas Berikutnya (FASE 2: Hardening Mainnet)

Sebelum mainnet diluncurkan, sistem **wajib** dilindungi dari serangan spam (DoS). Kita harus menambahkan "tameng" berikut:

- [ ] **Dynamic Fee:** Fee harus dihitung berdasarkan ukuran transaksi dan tingkat kemacetan mempool (bukan flat 0.001 ZENAC).
- [ ] **Mempool Eviction:** Membatasi ukuran maksimal mempool (misal 300 MB). Jika penuh, buang transaksi dengan fee terendah.
- [ ] **P2P Rate Limiting:** Batasi jumlah transaksi yang boleh dikirim peer per detik. Putus koneksi jika melanggar.
- [ ] **RPC Security:** Pastikan RPC (port 8332) hanya bisa diakses dari `localhost` dan wajib pakai password kuat.
- [ ] **Moving Download Window:** Batasi request blok maksimal 16 per peer agar RAM tidak jebol saat sinkronisasi (khusus untuk mainnet).
- [ ] **Wallet Nonce Reuse AES-GCM:** Perbaiki `wallet.rs` agar setiap entry punya nonce enkripsi sendiri.
- [ ] **PBKDF2 600.000 iterasi:** Standar OWASP untuk keamanan dompet.

---

## 4. Catatan Penting
- **Difficulty Adjustment:** Fitur berjalan sempurna. Di testnet, saat mencapai blok 69.120, kesulitan naik drastis (mining melambat dari milidetik jadi puluhan detik). Ini fitur, bukan bug.
- **Jeda 5.000 Blok:** Batas batch sync P2P sudah dinaikkan jadi 5.000 header (dari sebelumnya 2.000) agar sinkronisasi lebih cepat.
- **Sync Recovery:** Jika koneksi P2P terputus sesaat dan node melewatkan sinyal `Inv`, node baru akan menyusul saat di-restart (karena belum ada Heartbeat Polling).

---

## 5. Cara Melanjutkan di Chat Baru
Copy file ini dan tempel ke chat baru, lalu ketik:
> "Lanjut ZenaChain. Baca PROGRESS.md ini. Testnet dan stress-test selesai. Lanjut FASE 2: Hardening Mainnet (Dynamic Fee, Mempool Eviction, Rate Limiting)."

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
