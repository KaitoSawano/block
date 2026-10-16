//! Peer bootstrap ZenaChain.

/// Peer bootstrap statis dalam format multiaddr.
/// Untuk testing lokal, isi dengan alamat node pertama.
pub const BOOTSTRAP_PEERS: &[&str] = &[
    // Contoh untuk testing lokal:
    // "/ip4/127.0.0.1/tcp/8333",
];

/// DNS seed. Format: host:port.
pub const DNS_SEEDS: &[&str] = &[];

/// Alamat listen default node.
pub const LISTEN_ADDR: &str = "/ip4/0.0.0.0/tcp/8333";

/// Jumlah maksimum peer.
pub const MAX_PEERS: usize = 32;

/// Jumlah maksimum outbound connection.
pub const MAX_OUTBOUND: usize = 8;

/// Berapa header yang diminta sekali request.
pub const HEADERS_PER_REQUEST: u64 = 2_000;

/// Tinggi genesis.
pub const GENESIS_HEIGHT: u64 = 0;
