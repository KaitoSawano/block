//! Modul P2P ZenaChain.
//!
//! Berbasis libp2p: TCP + Noise + Yamux, dengan ping, identify,
//! gossipsub, dan kademlia.

pub mod behaviour;
pub mod codec;
pub mod events;
pub mod swarm;
pub mod transport;

pub use behaviour::ZenaBehaviour;
pub use codec::{ZenaBlock, ZenaTransaction};
pub use swarm::build_swarm;

pub const TOPIC_BLOCKS: &str = "zenachain/blocks";
pub const TOPIC_TX: &str = "zenachain/transactions";
pub const PROTOCOL_VERSION: &str = "/zenachain/1.0.0";
pub const DEFAULT_LISTEN_ADDR: &str = "/ip4/0.0.0.0/tcp/8333";
