//! Modul P2P ZenaChain.
//!
//! Berbasis TCP mentah dengan framing ala Bitcoin, handshake
//! Version/Verack, sync headers-first, dan relay Inv/GetData.

pub mod addr_book;
pub mod dns_seed;
pub mod framing;
pub mod handshake;
pub mod inv;
pub mod message;
pub mod network;
pub mod peer;
pub mod peer_manager;
pub mod peers;
pub mod sync;

pub use message::{Message, NetAddr, VersionMessage, PROTOCOL_VERSION, USER_AGENT};
pub use network::{Network, NetworkConfig};
pub use peer::{Peer, PeerState};
pub use peer_manager::{PeerEvent, PeerManager};
pub use peers::DEFAULT_LISTEN_ADDR;
pub use sync::SyncState;

pub const TOPIC_BLOCKS: &str = "zenachain/blocks";
pub const TOPIC_TX: &str = "zenachain/transactions";
pub const PROTOCOL_VERSION_STR: &str = "/zenachain/1.0.0";
pub const SYNC_PROTOCOL: &str = "/zenachain/sync/1.0.0";
