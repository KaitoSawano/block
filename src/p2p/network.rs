// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Main network loop for the ZenaChain peer-to-peer layer.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};

use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::p2p::addr_book::AddrBook;
use crate::p2p::dns_seed;
use crate::p2p::framing::{read_message, write_message};
use crate::p2p::handshake;
use crate::p2p::inv::InventoryManager;
use crate::p2p::message::{BlockLocator, Inventory, InvType, Message};
use crate::p2p::peer::{PeerState};
use crate::p2p::peer_manager::{PeerEvent, PeerManager};
use crate::p2p::peers;
use crate::storage;
use crate::utils::{format_datetime, now_millis};
use crate::validation;

#[derive(Debug, Clone, serde::Serialize)]
pub struct PeerInfo {
    pub id: u64,
    pub addr: String,
    pub outbound: bool,
    pub state: String,
}

pub type PeerRegistry = Arc<Mutex<Vec<PeerInfo>>>;

#[derive(Debug, Clone)]
pub enum NetworkCommand {
    BroadcastBlock(String),
    BroadcastTransaction(String),
}

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub listen_addr: SocketAddr,
    pub bootstrap: Vec<SocketAddr>,
    pub max_peers: usize,
    pub max_outbound: usize,
    pub handshake_timeout: Duration,
    pub idle_timeout: Duration,
    pub ping_interval: Duration,
    pub ping_timeout: Duration,
    pub connect_interval: Duration,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        NetworkConfig {
            listen_addr: "0.0.0.0:8333".parse().unwrap(),
            bootstrap: Vec::new(),
            max_peers: peers::MAX_PEERS,
            max_outbound: peers::MAX_OUTBOUND,
            handshake_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(120),
            ping_interval: Duration::from_secs(30),
            ping_timeout: Duration::from_secs(60),
            connect_interval: Duration::from_secs(2),
        }
    }
}

struct Connection {
    stream: TcpStream,
    addr: SocketAddr,
    read_buf: Vec<u8>,
    write_buf: Vec<u8>,
}

pub struct Network {
    config: NetworkConfig,
    chain: Arc<RwLock<Blockchain>>,
    peer_manager: PeerManager,
    addr_book: AddrBook,
    inv: InventoryManager,
    connections: HashMap<u64, Connection>,
    listener: Option<TcpListener>,
    local_addr: Option<SocketAddr>,
    next_connect_at: Instant,
    peer_registry: PeerRegistry,
    command_rx: mpsc::Receiver<NetworkCommand>,
}

impl Network {
    pub fn new(
        config: NetworkConfig,
        chain: Arc<RwLock<Blockchain>>,
        peer_registry: PeerRegistry,
        command_rx: mpsc::Receiver<NetworkCommand>,
    ) -> Self {
        let peer_manager = PeerManager::new(
            config.max_peers,
            config.max_outbound,
            config.handshake_timeout,
            config.idle_timeout,
            config.ping_interval,
            config.ping_timeout,
        );
        Network {
            config,
            chain,
            peer_manager,
            addr_book: AddrBook::new(1024),
            inv: InventoryManager::new(),
            connections: HashMap::new(),
            listener: None,
            local_addr: None,
            next_connect_at: Instant::now(),
            peer_registry,
            command_rx,
        }
    }

    pub async fn run(&mut self) -> Result<(), String> {
        let listener = TcpListener::bind(self.config.listen_addr)
            .await
            .map_err(|e| format!("failed to bind {}: {}", self.config.listen_addr, e))?;
        let local = listener
            .local_addr()
            .map_err(|e| format!("failed to get local address: {}", e))?;
        self.local_addr = Some(local);
        self.listener = Some(listener);
        println!("[p2p] listening on {}", local);

        for addr in &self.config.bootstrap {
            self.addr_book.add(*addr);
        }

        if self.addr_book.is_empty() {
            let seeds = dns_seed::resolve_seeds().await;
            let n = self.addr_book.add_many(&seeds);
            if n > 0 {
                println!("[p2p] {} peers discovered from DNS seeds", n);
            }
        }

        let tick_interval = Duration::from_millis(500);
        let mut last_tick = Instant::now();

        loop {
            let now = Instant::now();
            if now.duration_since(last_tick) >= tick_interval {
                self.on_tick().await;
                last_tick = now;
            }

            self.try_connect().await;

            let has_listener = self.listener.is_some();
            tokio::select! {
                accept = async {
                    if let Some(l) = &self.listener {
                        l.accept().await
                    } else {
                        std::future::pending().await
                    }
                }, if has_listener => {
                    match accept {
                        Ok((stream, addr)) => {
                            self.on_inbound(stream, addr).await;
                        }
                        Err(e) => {
                            eprintln!("[p2p] accept failed: {}", e);
                        }
                    }
                }
                cmd = self.command_rx.recv() => {
                    if let Some(cmd) = cmd {
                        self.handle_command(cmd).await;
                    }
                }
                _ = tokio::time::sleep(Duration::from_millis(50)) => {}
            }

            self.process_connections().await;
            self.process_events().await;
            self.update_registry();
        }
    }

    async fn handle_command(&mut self, cmd: NetworkCommand) {
        match cmd {
            NetworkCommand::BroadcastBlock(hash) => {
                println!("[p2p] broadcasting block {}", hash);
                self.broadcast_inv(&[Inventory {
                    inv_type: InvType::Block,
                    hash,
                }]);
            }
            NetworkCommand::BroadcastTransaction(txid) => {
                self.broadcast_inv(&[Inventory {
                    inv_type: InvType::Transaction,
                    hash: txid,
                }]);
            }
        }
    }

    fn update_registry(&self) {
        let mut reg = self.peer_registry.lock().unwrap();
        reg.clear();
        for peer in self.peer_manager.all_peers() {
            // Filter out peers that are not active
            if peer.state == PeerState::Disconnected || peer.state == PeerState::Banned {
                continue;
            }
            reg.push(PeerInfo {
                id: peer.id,
                addr: peer.addr.to_string(),
                outbound: peer.outbound,
                state: format!("{:?}", peer.state),
            });
        }
    }

    async fn on_tick(&mut self) {
        self.peer_manager.tick();
        self.addr_book.prune(Duration::from_secs(3600));
        
        let chain = self.chain.read().await;
        let height = chain.height();
        drop(chain);
        if height > 0 {
            for id in self.peer_manager.connected_peers() {
                self.send_getheaders(id).await;
            }
        }
    }

    async fn try_connect(&mut self) {
        if Instant::now() < self.next_connect_at {
            return;
        }
        if !self.peer_manager.can_add_outbound() {
            return;
        }
        let candidates = self.addr_book.candidates(Duration::from_secs(60), 4);
        if candidates.is_empty() {
            self.next_connect_at = Instant::now() + self.config.connect_interval;
            return;
        }

        for addr in candidates {
            if !self.peer_manager.can_add_outbound() {
                break;
            }

            // Prevent self-connection (robust check)
            if let Some(local) = self.local_addr {
                if local.ip().is_unspecified() {
                    // Listening on 0.0.0.0: block any localhost/unspecified on the same port
                    if addr.port() == local.port() {
                        continue;
                    }
                } else if addr.ip() == local.ip() && addr.port() == local.port() {
                    continue;
                }
            }

            self.addr_book.mark_tried(&addr);
            match TcpStream::connect(addr).await {
                Ok(stream) => {
                    let _ = stream.set_nodelay(true);
                    let id = self.peer_manager.add_peer(addr, true);
                    self.connections.insert(
                        id,
                        Connection {
                            stream,
                            addr,
                            read_buf: Vec::new(),
                            write_buf: Vec::new(),
                        },
                    );
                    println!("[p2p] outbound connection to {}", addr);

                    if let Some(peer) = self.peer_manager.get_mut(id) {
                        peer.state = PeerState::Handshaking;
                    }

                    self.send_version(id).await;
                }
                Err(e) => {
                    eprintln!("[p2p] failed to connect to {}: {}", addr, e);
                    let _ = self.addr_book.mark_failed(&addr);
                }
            }
        }

        self.next_connect_at = Instant::now() + self.config.connect_interval;
    }

    async fn on_inbound(&mut self, stream: TcpStream, addr: SocketAddr) {
        if !self.peer_manager.can_accept_inbound() {
            eprintln!("[p2p] rejecting inbound {}: peer limit reached", addr);
            return;
        }
        let _ = stream.set_nodelay(true);
        let id = self.peer_manager.add_peer(addr, false);
        self.connections.insert(
            id,
            Connection {
                stream,
                addr,
                read_buf: Vec::new(),
                write_buf: Vec::new(),
            },
        );
        println!("[p2p] inbound connection from {}", addr);

        if let Some(peer) = self.peer_manager.get_mut(id) {
            peer.state = PeerState::Handshaking;
        }

        self.send_version(id).await;
    }

    async fn send_version(&mut self, id: u64) {
        let peer = match self.peer_manager.get(id) {
            Some(p) => p.clone(),
            None => return,
        };
        let local = match self.local_addr {
            Some(a) => a,
            None => return,
        };
        let chain = self.chain.read().await;
        let msg = handshake::build_version(&chain, &peer, local);
        drop(chain);
        self.queue_message(id, &msg);
    }

    fn queue_message(&mut self, id: u64, msg: &Message) {
        let payload = msg.serialize();
        let cmd = msg.command();
        let mut buf = Vec::new();
        if write_message(&mut buf, cmd, &payload).is_ok() {
            if let Some(conn) = self.connections.get_mut(&id) {
                conn.write_buf.extend_from_slice(&buf);
            }
        }
    }

    async fn process_connections(&mut self) {
        let ids: Vec<u64> = self.connections.keys().copied().collect();
        for id in ids {
            self.process_one(id).await;
        }
    }

    async fn process_one(&mut self, id: u64) {
        let (read_result, has_write) = {
            let conn = match self.connections.get_mut(&id) {
                Some(c) => c,
                None => return,
            };

            let mut buf = [0u8; 4096];
            let read_fut = conn.stream.read(&mut buf);
            let read_result = tokio::time::timeout(Duration::from_millis(10), read_fut).await;

            match read_result {
                Ok(Ok(0)) => (Err("peer closed the connection".to_string()), false),
                Ok(Ok(n)) => {
                    conn.read_buf.extend_from_slice(&buf[..n]);
                    (Ok(n), !conn.write_buf.is_empty())
                }
                Ok(Err(e)) => (Err(format!("read error: {}", e)), false),
                Err(_) => (Ok(0), !conn.write_buf.is_empty()),
            }
        };

        match read_result {
            Ok(n) if n > 0 => {
                self.drain_read_buffer(id).await;
            }
            Ok(_) => {}
            Err(reason) => {
                self.disconnect(id, &reason).await;
                return;
            }
        }

        if has_write {
            self.flush_write(id).await;
        }
    }

    async fn flush_write(&mut self, id: u64) {
        let conn = match self.connections.get_mut(&id) {
            Some(c) => c,
            None => return,
        };
        if conn.write_buf.is_empty() {
            return;
        }
        let data = std::mem::take(&mut conn.write_buf);
        if let Err(e) = conn.stream.write_all(&data).await {
            eprintln!("[p2p] write failed to {}: {}", conn.addr, e);
        }
    }

    async fn drain_read_buffer(&mut self, id: u64) {
        loop {
            let parsed = {
                let conn = match self.connections.get_mut(&id) {
                    Some(c) => c,
                    None => return,
                };
                if conn.read_buf.is_empty() {
                    return;
                }
                let mut cursor = std::io::Cursor::new(conn.read_buf.clone());
                match read_message(&mut cursor) {
                    Ok((cmd, payload)) => {
                        let consumed = cursor.position() as usize;
                        conn.read_buf.drain(..consumed);
                        Some((cmd, payload))
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return,
                    Err(e) => {
                        eprintln!("[p2p] framing error from {}: {}", conn.addr, e);
                        self.disconnect(id, "framing error").await;
                        return;
                    }
                }
            };

            if let Some((cmd, payload)) = parsed {
                self.handle_message(id, &cmd, &payload).await;
            } else {
                return;
            }
        }
    }

    async fn handle_message(&mut self, id: u64, cmd: &str, payload: &[u8]) {
        let msg = match Message::deserialize(cmd, payload) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("[p2p] invalid message from {}: {}", id, e);
                self.punish(id, 10, "invalid message").await;
                return;
            }
        };

        if let Some(peer) = self.peer_manager.get_mut(id) {
            peer.last_recv = Instant::now();
        }

        match msg {
            Message::Version(v) => {
                let result = {
                    let peer = match self.peer_manager.get_mut(id) {
                        Some(p) => p,
                        None => return,
                    };
                    handshake::handle_version(peer, &v)
                };
                match result {
                    Ok(_) => {
                        self.queue_message(id, &Message::Verack);
                    }
                    Err(e) => {
                        self.disconnect(id, &format!("version rejected: {}", e)).await;
                    }
                }
            }
            Message::Verack => {
                let result = {
                    let peer = match self.peer_manager.get_mut(id) {
                        Some(p) => p,
                        None => return,
                    };
                    handshake::handle_verack(peer)
                };
                match result {
                    Ok(true) => {
                        self.peer_manager
                            .push_event(PeerEvent::HandshakeComplete(id));
                    }
                    Ok(false) => {}
                    Err(e) => {
                        self.disconnect(id, &format!("verack rejected: {}", e)).await;
                    }
                }
            }
            Message::Ping(nonce) => {
                self.queue_message(id, &Message::Pong(nonce));
            }
            Message::Pong(nonce) => {
                if let Some(peer) = self.peer_manager.get_mut(id) {
                    if !peer.mark_pong_received(nonce) {
                        peer.add_misbehavior(1);
                    }
                }
            }
            Message::GetAddr => {
                let addrs = self.peer_manager.connected_addrs();
                let net_addrs: Vec<_> = addrs
                    .into_iter()
                    .map(crate::p2p::message::NetAddr::from_socket)
                    .collect();
                if !net_addrs.is_empty() {
                    self.queue_message(id, &Message::Addr(net_addrs));
                }
            }
            Message::Addr(addrs) => {
                let mut count = 0;
                for na in addrs {
                    let sa = na.to_socket();
                    if sa.port() != crate::blockchain::network::DEFAULT_PORT {
                        continue;
                    }
                    if self.addr_book.add(sa) {
                        count += 1;
                    }
                }
                if count > 0 {
                    println!("[p2p] {} new addresses from {}", count, id);
                }
            }
            Message::Inv(items) => {
                let new_items = self.inv.filter_new(&items);
                if new_items.is_empty() {
                    return;
                }
                let mut to_request = Vec::new();
                let chain = self.chain.read().await;
                for inv in &new_items {
                    if inv.inv_type == InvType::Block && !self.inv.has_block(&inv.hash) {
                        to_request.push(inv.clone());
                    } else if inv.inv_type == InvType::Transaction
                        && !self.inv.has_tx(&inv.hash)
                    {
                        if chain.mempool.iter().any(|t| t.txid == inv.hash) {
                            self.inv.add_tx(&inv.hash);
                        } else {
                            to_request.push(inv.clone());
                        }
                    }
                }
                drop(chain);
                if !to_request.is_empty() {
                    self.queue_message(id, &Message::GetData(to_request));
                }
            }
            Message::GetData(items) => {
                let chain = self.chain.read().await;
                for inv in items {
                    match inv.inv_type {
                        InvType::Block => {
                            if let Some(b) =
                                chain.chain.iter().find(|b| b.hash == inv.hash).cloned()
                            {
                                drop(chain);
                                self.queue_message(id, &Message::Block(b));
                                return;
                            }
                        }
                        InvType::Transaction => {
                            if let Some(tx) =
                                chain.mempool.iter().find(|t| t.txid == inv.hash).cloned()
                            {
                                drop(chain);
                                self.queue_message(id, &Message::Tx(tx));
                                return;
                            }
                        }
                    }
                }
            }
            Message::GetHeaders(loc) => {
                let chain = self.chain.read().await;
                let headers = build_headers_response(&chain, &loc);
                drop(chain);
                self.queue_message(id, &Message::Headers(headers));
            }
            Message::Headers(headers) => {
                self.handle_headers(id, headers).await;
            }
            Message::Block(b) => {
                self.handle_block(id, b).await;
            }
            Message::Tx(tx) => {
                let mut chain = self.chain.write().await;
                if chain.add_transaction(tx.clone()) {
                    self.inv.add_tx(&tx.txid);

                    let now = now_millis();
                    println!(
                        "[mempool] {} Accepted peer transaction {} from {} to {}",
                        format_datetime(now),
                        tx.txid,
                        tx.from,
                        tx.to
                    );

                    drop(chain);
                    self.broadcast_inv(&[Inventory {
                        inv_type: InvType::Transaction,
                        hash: tx.txid.clone(),
                    }]);
                }
            }
            Message::Reject {
                command,
                code,
                reason,
            } => {
                eprintln!(
                    "[p2p] reject from {}: {} code={} reason={}",
                    id, command, code, reason
                );
            }
        }
    }

    async fn handle_headers(&mut self, id: u64, headers: Vec<Block>) {
        if headers.is_empty() {
            return;
        }
        let chain = self.chain.read().await;
        let tip = match chain.chain.last() {
            Some(b) => b.clone(),
            None => return,
        };
        let first = &headers[0];
        if first.index != tip.index + 1 || first.previous_hash != tip.hash {
            drop(chain);
            self.punish(id, 20, "headers do not connect to the tip").await;
            return;
        }
        let mut to_request = Vec::new();
        for h in &headers {
            if validation::rule_block_pow(h).is_err() {
                drop(chain);
                self.punish(id, 50, "invalid header proof of work").await;
                return;
            }
            if validation::rule_block_subsidy(h).is_err() {
                drop(chain);
                self.punish(id, 50, "invalid header subsidy").await;
                return;
            }
            if !self.inv.has_block(&h.hash) {
                to_request.push(Inventory {
                    inv_type: InvType::Block,
                    hash: h.hash.clone(),
                });
            }
        }
        drop(chain);
        if !to_request.is_empty() {
            self.queue_message(id, &Message::GetData(to_request));
        }
    }

    async fn handle_block(&mut self, id: u64, block: Block) {
        let mut chain = self.chain.write().await;
        match crate::p2p::sync::SyncState::accept_block(&mut chain, block.clone()) {
            Ok(_) => {
                self.inv.add_block(&block.hash);
                let _ = storage::save_chain(&chain);
                drop(chain);
                self.broadcast_inv(&[Inventory {
                    inv_type: InvType::Block,
                    hash: block.hash.clone(),
                }]);
                println!("[p2p] block {} accepted from {}", block.index, id);
            }
            Err(e) => {
                drop(chain);
                if e.contains("does not follow tip") {
                    println!("[p2p] block from {} does not follow tip, requesting headers", id);
                    self.send_getheaders(id).await;
                } else {
                    self.punish(id, 50, &format!("block rejected: {}", e)).await;
                }
            }
        }
    }

    fn broadcast_inv(&mut self, items: &[Inventory]) {
        let peers = self.peer_manager.connected_peers();
        if peers.is_empty() {
            return;
        }
        let msg = Message::Inv(items.to_vec());
        for id in peers {
            self.queue_message(id, &msg);
        }
    }

    async fn punish(&mut self, id: u64, score: u32, reason: &str) {
        let ban = {
            let peer = match self.peer_manager.get_mut(id) {
                Some(p) => p,
                None => return,
            };
            peer.add_misbehavior(score)
        };
        if ban {
            self.peer_manager.ban(id, reason);
            self.disconnect(id, reason).await;
        }
    }

    async fn disconnect(&mut self, id: u64, reason: &str) {
        if let Some(mut conn) = self.connections.remove(&id) {
            let _ = conn.stream.shutdown().await;
        }
        self.peer_manager.remove(id, reason);
        println!("[p2p] disconnect {}: {}", id, reason);
    }

    async fn process_events(&mut self) {
        let events = self.peer_manager.drain_events();
        for ev in events {
            match ev {
                PeerEvent::HandshakeComplete(id) => {
                    println!("[p2p] handshake complete with {}", id);
                    self.send_getheaders(id).await;
                    self.queue_message(id, &Message::GetAddr);
                }
                PeerEvent::PingDue(id, nonce) => {
                    self.queue_message(id, &Message::Ping(nonce));
                }
                PeerEvent::Disconnected(id, reason) => {
                    self.connections.remove(&id);
                    println!("[p2p] peer {} disconnected: {}", id, reason);
                }
                PeerEvent::Banned(id, reason) => {
                    println!("[p2p] peer {} banned: {}", id, reason);
                }
                _ => {}
            }
        }
    }

    async fn send_getheaders(&mut self, id: u64) {
        let chain = self.chain.read().await;
        let locator = build_locator(&chain);
        drop(chain);
        self.queue_message(id, &Message::GetHeaders(locator));
    }
}

fn build_locator(chain: &Blockchain) -> BlockLocator {
    let mut hashes = Vec::new();
    let height = chain.height();
    let mut step = 1u64;
    let mut h = height;
    loop {
        if let Some(b) = chain.chain.get(h as usize) {
            hashes.push(b.hash.clone());
        }
        if h == 0 {
            break;
        }
        h = h.saturating_sub(step);
        if hashes.len() >= 10 {
            step = step.saturating_mul(2);
        }
        if hashes.len() >= 32 {
            break;
        }
    }
    BlockLocator {
        version: crate::p2p::message::PROTOCOL_VERSION,
        locator_hashes: hashes,
        hash_stop: "0".repeat(64),
    }
}

fn build_headers_response(chain: &Blockchain, loc: &BlockLocator) -> Vec<Block> {
    let mut start = 0u64;
    for h in &loc.locator_hashes {
        if let Some(b) = chain.chain.iter().find(|b| &b.hash == h) {
            start = b.index + 1;
            break;
        }
    }
    let end = (start + peers::HEADERS_PER_REQUEST).min(chain.chain.len() as u64);
    chain.chain[start as usize..end as usize]
        .iter()
        .map(|b| {
            let mut hb = b.clone();
            hb.transactions.clear();
            hb
        })
        .collect()
}

pub fn spawn(
    config: NetworkConfig,
    chain: Arc<RwLock<Blockchain>>,
    peer_registry: PeerRegistry,
    command_rx: mpsc::Receiver<NetworkCommand>,
) -> tokio::task::JoinHandle<Result<(), String>> {
    tokio::spawn(async move {
        let mut net = Network::new(config, chain, peer_registry, command_rx);
        net.run().await
    })
}
