# Progres Pengembangan ZenaChain

## 1. Gambaran Proyek
ZenaChain adalah blockchain berbasis Rust dengan model **account-based**
(seperti Ethereum), bukan UTXO (seperti Bitcoin). Jaringan P2P menggunakan
TCP mentah dengan framing ala Bitcoin, RPC server multi-threaded, dan
dompet multi-alamat dengan enkripsi AES-256-GCM.

Sekarang mendukung **dua jaringan**: mainnet dan testnet, dengan
parameter, genesis, magic bytes, port, address prefix, dan datadir
yang terpisah.

### Parameter Konsensus Utama

| Parameter | Mainnet | Testnet |
|---|---|---|
| **Algoritma Hash** | SHA3-256 | SHA3-256 |
| **Tanda Tangan** | secp256k1-ECDSA | secp256k1-ECDSA |
| **Max Supply** | 28.000.000.000 ZENAC | 28.000.000.000 ZENAC |
| **Reward Awal** | 50 ZENAC | 50 ZENAC |
| **Coinbase Maturity** | TIDAK ADA | TIDAK ADA |
| **Waktu Blok** | 15 detik | 15 detik |
| **Difficulty Adjust** | 69.120 blok | 69.120 blok |
| **Magic Bytes** | `ZENA` (5a454e41) | `ZTES` (5a544553) |
| **P2P Port** | 8333 | 9422 |
| **RPC Port** | 8332 | 12255 |
| **Address Prefix** | `ZENA` | `ZTES` |
| **Datadir** | `~/.zenachain` | `~/.zenachain-testnet` |
| **Genesis Timestamp** | 1792303198445 | 1792303198446 |
| **Genesis NBITS** | 0x1e0ffff0 | 0x207fffff |
| **Genesis Nonce** | 420282 | 1 |

### Struktur Proyek
- `src/` : Crate utama node (`zenachain`)
- `crypto/` : Crate workspace terpisah untuk kriptografi (`zenachain-crypto`)
- `contrib/` : Skrip build Gitian
- `.github/workflows/` : CI/CD (ci.yml, release.yml)

---

## 2. Status Terkini (Terakhir Diperbarui: 2026-10-19)

### 🟢 CHECKPOINT: `stabil` (Mainnet)
Sistem transfer account-based mainnet **STABIL** dan **TERUJI**.
**JANGAN UBAH** kode berikut kecuali ada bug nyata:
- `src/transaction.rs` — `nonce`, `signing_hash`, `compute_txid`
- `src/blockchain.rs` — `apply_transaction`, `credit_reward`,
  `next_nonce_for`, `add_transaction`
- `src/validation.rs` — `rule_nonce`, `rule_sufficient_balance`,
  `rule_signature`, `validate_block_transactions`
- `src/rpc.rs` — `m_sendtoaddress`, `m_getbalance`, `m_getnonce`
- `src/storage.rs` — `ChainState` (`balances`, `nonces`)
- `src/p2p/sync.rs` — `accept_block`

### 🟢 FASE 1a: TESTNET MODE (SELESAI)

Testnet **berfungsi** dengan parameter terpisah dari mainnet.

**Yang sudah selesai:**
- ✅ `Network` enum di `src/consensus/spec.rs` (`Main`, `Test`)
- ✅ Parameter testnet (magic, port, address prefix, datadir, genesis)
- ✅ Genesis testnet (di-mine, `TESTNET_EXPECTED_GENESIS_*` diisi)
- ✅ `consensus_fingerprint_for(network)` dan `parameter_fingerprint_for(network)`
- ✅ `genesis_coinbase_txid_for(network)`
- ✅ `Block::genesis_for(network)`, `genesis_mined_for(network)`
- ✅ `rule_genesis_fingerprint_for(block, network)`
- ✅ `Config.network: Network` (bukan `String`)
- ✅ `datadir()` per network (`~/.zenachain` vs `~/.zenachain-testnet`)
- ✅ `address_from_public_key` pakai `current_network().address_prefix()`
- ✅ `framing.rs` pakai `current_network().magic()` (bukan hardcoded)
- ✅ `peer.rs` hapus `expected_magic()` (dead code)
- ✅ `blockchain::network` module pakai fungsi (`network_name()`,
  `default_port()`, `network_magic_hex()`)
- ✅ CLI `-testnet` / `--testnet` di `zenachaind` dan `zenachain`
- ✅ `zenachain-genesis -testnet`
- ✅ `RpcState.network: Network`, `RpcServer::new(..., network)`
- ✅ `RPC getinfo`/`getblockchaininfo`/`getgenesis` menampilkan `network`
- ✅ `sync.rs` `accept_block` handle `Result` dari `apply_transaction`
  dan `credit_reward` (fix warning + bug)

**Testnet berjalan:**
