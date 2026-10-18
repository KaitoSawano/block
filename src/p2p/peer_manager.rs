// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Peer management for the ZenaChain peer-to-peer network.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use crate::p2p::peer::{rand_nonce, ConnectionType, Peer, PeerState};

#[derive(Debug, Clone)]
pub enum PeerEvent {
    Connected(u64),
    HandshakeComplete(u64),
    Disconnected(u64, String),
    Banned(u64, String),
    PingDue(u64, u64),
}

pub struct PeerManager {
    peers: HashMap<u64, Peer>,
    next_id: u64,
    max_outbound_full_relay: usize,
    max_outbound_block_relay: usize,
    max_inbound: usize,
    handshake_timeout: Duration,
    idle_timeout: Duration,
    ping_interval: Duration,
    ping_timeout: Duration,
    events: Vec<PeerEvent>,
}

impl PeerManager {
    pub fn new(
        max_outbound_full_relay: usize,
        max_outbound_block_relay: usize,
        max_inbound: usize,
        handshake_timeout: Duration,
        idle_timeout: Duration,
        ping_interval: Duration,
        ping_timeout: Duration,
    ) -> Self {
        PeerManager {
            peers: HashMap::new(),
            next_id: 1,
            max_outbound_full_relay,
            max_outbound_block_relay,
            max_inbound,
            handshake_timeout,
            idle_timeout,
            ping_interval,
            ping_timeout,
            events: Vec::new(),
        }
    }

    // ------------------------------------------------------------------
    // Peer lifecycle
    // ------------------------------------------------------------------

    pub fn add_peer(
        &mut self,
        addr: SocketAddr,
        conn_type: ConnectionType,
        mapped_as: u32,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let peer = Peer::new(id, addr, conn_type, mapped_as);
        self.peers.insert(id, peer);
        self.events.push(PeerEvent::Connected(id));
        id
    }

    pub fn remove(&mut self, id: u64, reason: &str) {
        if let Some(peer) = self.peers.remove(&id) {
            self.events
                .push(PeerEvent::Disconnected(peer.id, reason.to_string()));
        }
    }

    pub fn ban(&mut self, id: u64, reason: &str) {
        if let Some(peer) = self.peers.get_mut(&id) {
            peer.state = PeerState::Banned;
            peer.add_misbehavior(crate::p2p::peer::MAX_MISBEHAVIOR);
            self.events
                .push(PeerEvent::Banned(peer.id, reason.to_string()));
        }
    }

    // ------------------------------------------------------------------
    // Accessors
    // ------------------------------------------------------------------

    pub fn get(&self, id: u64) -> Option<&Peer> {
        self.peers.get(&id)
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut Peer> {
        self.peers.get_mut(&id)
    }

    pub fn all_peers(&self) -> Vec<&Peer> {
        self.peers.values().collect()
    }

    pub fn connected_peers(&self) -> Vec<u64> {
        self.peers
            .iter()
            .filter(|(_, p)| p.state == PeerState::Connected)
            .map(|(id, _)| *id)
            .collect()
    }

    pub fn connected_addrs(&self) -> Vec<SocketAddr> {
        self.peers
            .values()
            .filter(|p| p.state == PeerState::Connected)
            .map(|p| p.addr)
            .collect()
    }

    pub fn count(&self) -> usize {
        self.peers.len()
    }

    // ------------------------------------------------------------------
    // Capacity checks
    // ------------------------------------------------------------------

    fn count_by_type<F>(&self, pred: F) -> usize
    where
        F: Fn(&Peer) -> bool,
    {
        self.peers.values().filter(|p| pred(p)).count()
    }

    pub fn can_add_outbound_full_relay(&self) -> bool {
        self.count_by_type(|p| {
            p.state != PeerState::Disconnected
                && p.state != PeerState::Banned
                && p.conn_type == ConnectionType::OutboundFullRelay
        }) < self.max_outbound_full_relay
    }

    pub fn can_add_outbound_block_relay(&self) -> bool {
        self.count_by_type(|p| {
            p.state != PeerState::Disconnected
                && p.state != PeerState::Banned
                && p.conn_type == ConnectionType::OutboundBlockRelayOnly
        }) < self.max_outbound_block_relay
    }

    pub fn can_accept_inbound(&self) -> bool {
        self.count_by_type(|p| {
            p.state != PeerState::Disconnected
                && p.state != PeerState::Banned
                && p.conn_type == ConnectionType::Inbound
        }) < self.max_inbound
    }

    // ------------------------------------------------------------------
    // Periodic maintenance
    // ------------------------------------------------------------------

    /// Called periodically by the network loop. Detects timeouts
    /// (handshake, idle, ping) and schedules ping messages.
    pub fn tick(&mut self) {
        let now = Instant::now();
        let mut to_remove: Vec<(u64, String)> = Vec::new();

        for (id, peer) in self.peers.iter_mut() {
            // Handshake timeout
            if peer.state == PeerState::Handshaking
                && now.duration_since(peer.last_recv) > self.handshake_timeout
            {
                to_remove.push((*id, "handshake timeout".to_string()));
                continue;
            }

            // Idle timeout (no message from peer)
            if peer.is_connected()
                && now.duration_since(peer.last_recv) > self.idle_timeout
            {
                to_remove.push((*id, "idle timeout".to_string()));
                continue;
            }

            // Ping timeout (pong not received)
            if peer.is_connected() && peer.is_ping_timed_out(self.ping_timeout) {
                to_remove.push((*id, "ping timeout".to_string()));
                continue;
            }

            // Schedule ping
            if peer.is_connected() && peer.should_ping(self.ping_interval) {
                let nonce = rand_nonce();
                peer.mark_ping_sent(nonce);
                self.events.push(PeerEvent::PingDue(*id, nonce));
            }
        }

        for (id, reason) in to_remove {
            self.remove(id, &reason);
        }
    }

    // ------------------------------------------------------------------
    // Event queue
    // ------------------------------------------------------------------

    pub fn push_event(&mut self, event: PeerEvent) {
        self.events.push(event);
    }

    pub fn drain_events(&mut self) -> Vec<PeerEvent> {
        std::mem::take(&mut self.events)
    }

    // ------------------------------------------------------------------
    // Registry view for RPC (getpeerinfo)
    // ------------------------------------------------------------------

    pub fn snapshot(&self) -> Vec<PeerSnapshot> {
        self.peers
            .values()
            .map(|p| PeerSnapshot {
                id: p.id,
                addr: p.addr.to_string(),
                outbound: p.outbound,
                conn_type: format!("{:?}", p.conn_type),
                mapped_as: p.mapped_as,
                state: format!("{:?}", p.state),
                user_agent: p.user_agent.clone(),
                start_height: p.start_height,
                misbehavior: p.misbehavior,
            })
            .collect()
    }
}

impl Default for PeerManager {
    fn default() -> Self {
        Self::new(8, 2, 115, Duration::from_secs(30), Duration::from_secs(120), Duration::from_secs(30), Duration::from_secs(60))
    }
}

/// Read-only view of a peer, used by the RPC layer.
#[derive(Debug, Clone)]
pub struct PeerSnapshot {
    pub id: u64,
    pub addr: String,
    pub outbound: bool,
    pub conn_type: String,
    pub mapped_as: u32,
    pub state: String,
    pub user_agent: String,
    pub start_height: u64,
    pub misbehavior: u32,
}
