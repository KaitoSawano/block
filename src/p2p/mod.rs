// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Peer-to-peer networking for ZenaChain.
//!
//! The networking layer uses raw TCP with Bitcoin-style framing,
//! Version/Verack handshaking, headers-first synchronization, and
//! Inv/GetData relay. It does not depend on libp2p.
//!
//! Anti-eclipse mechanisms follow Bitcoin Core:
//! - Anchors (`anchors.rs`): up to two block-relay-only connections are
//!   persisted across restarts to resist eclipse attacks.
//! - Ban manager (`banman.rs`): persistent manual bans and ephemeral
//!   discouragement of misbehaving peers.

pub mod addr_book;
pub mod anchors;
pub mod banman;
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
