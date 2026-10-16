//! Peer bootstrap ZenaChain.
//!
//! Daftar IP dan DNS seed yang dipakai saat node baru start untuk
//! menemukan peer pertama.
//!
//! Parameter di sini TIDAK masuk ke `consensus_fingerprint`.
//! Mengubahnya tidak membuat chain tidak valid, tetapi bisa
//! membuat node terisolasi dari jaringan.

/// Peer bootstrap statis dalam format multiaddr.
/// Ini alamat awal yang dicoba saat node belum punya peer.
pub const BOOTSTRAP_PEERS: &[&str] = &[
    // Placeholder. Ganti dengan multiaddr node produksi.
    // Contoh:
    // "/ip4/1.2.3.4/tcp/8333",
    // "/ip4/5.6.7.8/tcp/8333",
];

/// DNS seed. Format: host:port.
/// Dipakai kalau BOOTSTRAP_PEERS kosong atau gagal.
pub const DNS_SEEDS: &[&str] = &[
    // Placeholder.
    // Contoh:
    // "seed1.zenachain.example.com:8333",
];

/// Alamat listen default node.
pub const LISTEN_ADDR: &str = "/ip4/0.0.0.0/tcp/8333";

/// Jumlah maksimum peer yang diizinkan.
pub const MAX_PEERS: usize = 32;

/// Jumlah maksimum outbound connection.
pub const MAX_OUTBOUND: usize = 8;

/// Berapa header yang diminta sekali request.
pub const HEADERS_PER_REQUEST: u64 = 2_000;

/// Tinggi genesis, selalu 0.
pub const GENESIS_HEIGHT: u64 = 0;
