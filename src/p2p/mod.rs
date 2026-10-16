//! Modul P2P ZenaChain.
//!
//! Berbasis libp2p: TCP + Noise + Yamux, dengan ping, identify,
//! gossipsub, kademlia, dan request-response untuk sinkronisasi.

pub mod behaviour;
pub mod codec;
pub mod events;
pub mod peers;
pub mod swarm;
pub mod sync;
pub mod transport;

pub use behaviour::ZenaBehaviour;
pub use codec::{BlockHeader, SyncMessage};
pub use peers::LISTEN_ADDR;
pub use swarm::build_swarm;
pub use sync::SyncState;

pub const TOPIC_BLOCKS: &str = "zenachain/blocks";
pub const TOPIC_TX: &str = "zenachain/transactions";
pub const PROTOCOL_VERSION: &str = "/zenachain/1.0.0";
pub const SYNC_PROTOCOL: &str = "/zenachain/sync/1.0.0";
pub const DEFAULT_LISTEN_ADDR: &str = "/ip4/0.0.0.0/tcp/8333";
