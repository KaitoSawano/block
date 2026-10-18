// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Peer management for the ZenaChain peer-to-peer network.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use crate::p2p::peer::{ConnectionType, Peer, PeerState};

#[derive(Debug)]
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

    pub fn outbound_full_relay_count(&self) -> usize {
        self.peers
            .values()
            .filter(|p| p.conn_type == ConnectionType::OutboundFullRelay)
            .count()
    }

    pub fn outbound_block_relay_count(&self) -> usize {
        self.peers
            .values()
            .filter(|p| p.conn_type == ConnectionType::OutboundBlockRelayOnly)
            .count()
    }

    pub fn inbound_count(&self) -> usize {
        self.peers
            .values()
            .filter(|p| p.conn_type == ConnectionType::Inbound)
            .count()
    }

    pub fn can_add_outbound_full_relay(&self) -> bool {
        self.outbound_full_relay_count() < self.max_outbound_full_relay
    }

    pub fn can_add_outbound_block_relay(&self) -> bool {
        self.outbound_block_relay_count() < self.max_outbound_block_relay
    }

    pub fn can_accept_inbound(&self) -> bool {
        self.inbound_count() < self.max_inbound
    }

    pub fn add_peer(&mut self, addr: SocketAddr, conn_type: ConnectionType) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let peer = Peer::new(id, addr, conn_type);
        self.peers.insert(id, peer);
        self.events.push(PeerEvent::Connected(id));
        id
    }

    pub fn get(&self, id: u64) -> Option<&Peer> {
        self.peers.get(&id)
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut Peer> {
        self.peers.get_mut(&id)
    }

    pub fn all_peers(&self) -> Vec<&Peer> {
        self.peers.values().collect()
    }

    pub fn remove(&mut self, id: u64, reason: &str) {
        if self.peers.remove(&id).is_some() {
            self.events
                .push(PeerEvent::Disconnected(id, reason.to_string()));
        }
    }

    pub fn ban(&mut self, id: u64, reason: &str) {
        if let Some(peer) = self.peers.get_mut(&id) {
            peer.state = PeerState::Banned;
        }
        self.events.push(PeerEvent::Banned(id, reason.to_string()));
    }

    pub fn drain_events(&mut self) -> Vec<PeerEvent> {
        std::mem::take(&mut self.events)
    }

    pub fn push_event(&mut self, event: PeerEvent) {
        self.events.push(event);
    }

    pub fn tick(&mut self) {
        let now = Instant::now();
        let mut to_remove = Vec::new();
        let mut to_ping = Vec::new();

        for (id, peer) in self.peers.iter_mut() {
            match peer.state {
                PeerState::Handshaking => {
                    if now.duration_since(peer.last_recv) > self.handshake_timeout {
                        to_remove.push((*id, "handshake timeout".to_string()));
                    }
                }
                PeerState::Connected => {
                    if peer.is_timed_out(self.idle_timeout) {
                        to_remove.push((*id, "idle timeout".to_string()));
                    } else if peer.is_ping_timed_out(self.ping_timeout) {
                        to_remove.push((*id, "ping timeout".to_string()));
                    } else if peer.should_ping(self.ping_interval) {
                        let nonce = crate::p2p::peer::rand_nonce();
                        peer.mark_ping_sent(nonce);
                        to_ping.push((*id, nonce));
                    }
                }
                _ => {}
            }
        }

        for (id, reason) in to_remove {
            self.remove(id, &reason);
        }
        for (id, nonce) in to_ping {
            self.events.push(PeerEvent::PingDue(id, nonce));
        }
    }

    pub fn connected_peers(&self) -> Vec<u64> {
        self.peers
            .iter()
            .filter(|(_, p)| p.is_connected())
            .map(|(id, _)| *id)
            .collect()
    }

    pub fn connected_addrs(&self) -> Vec<SocketAddr> {
        self.peers
            .values()
            .filter(|p| p.is_connected())
            .map(|p| p.addr)
            .collect()
    }
}
