//! Peer bootstrap ZenaChain.
//!
//! Nilai default di sini dipakai kalau `zenachain.conf` tidak
//! menyediakan nilai. Ubah lewat config, bukan edit file ini.

pub const DEFAULT_LISTEN_ADDR: &str = "/ip4/0.0.0.0/tcp/8333";
pub const DEFAULT_BOOTSTRAP: &[&str] = &[];

pub const MAX_PEERS: usize = 32;
pub const MAX_OUTBOUND: usize = 8;
pub const HEADERS_PER_REQUEST: u64 = 2_000;
pub const GENESIS_HEIGHT: u64 = 0;
