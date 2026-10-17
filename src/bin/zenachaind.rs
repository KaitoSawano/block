use futures::StreamExt;
use libp2p::request_response::{Event as RrEvent, Message as RrMessage};
use libp2p::{gossipsub, identify, Multiaddr, PeerId};
use std::collections::HashSet;
use tokio::signal::unix::{signal, SignalKind};
use zenachain::block::Block;
use zenachain::blockchain::Blockchain;
use zenachain::config::Config;
use zenachain::p2p::{self, SyncMessage, SyncState};
use zenachain::rpc::RpcServer;

#[tokio::main]
async fn main() {
    let mut chain = Blockchain::new();

    println!("ZenaChain server starting");
    println!("Gunakan `./zenachain stop` untuk menghentikan node.");

    // Baca config dari .chain/zenachain.conf atau ~/.zenachain/zenachain.conf.
    let config = Config::load_or_default();
    println!("[config] network      : {}", config.network);
    println!("[config] rpc_addr     : {}", config.rpc_addr);
    println!("[config] listen_addr  : {}", config.listen_addr);
    println!("[config] bootstrap    : {:?}", config.bootstrap);
    println!("[config] mining_addr  : {}", config.mining_address);
    println!("[config] lokasi       : {}", Config::default_path());

    // Simpan config default kalau file belum ada.
    if !std::path::Path::new(&Config::default_path()).exists() {
        let _ = config.save_default();
        println!("[config] file config dibuat di {}", Config::default_path());
    }

    // Abaikan SIGINT dan SIGHUP.
    let mut sigint = signal(SignalKind::interrupt()).expect("gagal daftar SIGINT");
    let mut sighup = signal(SignalKind::hangup()).expect("gagal daftar SIGHUP");

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = sigint.recv() => {
                    println!("[signal] SIGINT diabaikan. Gunakan `./zenachain stop` untuk berhenti.");
                }
                _ = sighup.recv() => {
                    println!("[signal] SIGHUP diabaikan. Node tetap jalan.");
                }
            }
        }
    });

    let keypair = p2p::swarm::load_or_generate_keypair(".chain/peer_id");

    let mut swarm = match p2p::build_swarm(keypair).await {
        Ok(s) => s,
        Err(e) => {
            println!("[p2p] gagal bikin swarm: {}", e);
            return;
        }
    };

    let listen_addr: Multiaddr = match config.listen_addr.parse() {
        Ok(a) => a,
        Err(e) => {
            println!("[p2p] alamat listen tidak valid: {}", e);
            return;
        }
    };

    if let Err(e) = swarm.listen_on(listen_addr) {
        println!("[p2p] gagal listen {}: {}", config.listen_addr, e);
    }

    // Dial bootstrap dari config.
    for addr in &config.bootstrap {
        match addr.parse::<Multiaddr>() {
            Ok(ma) => {
                if let Err(e) = swarm.dial(ma) {
                    println!("[p2p] gagal dial {}: {}", addr, e);
                }
            }
            Err(e) => println!("[p2p] alamat bootstrap tidak valid {}: {}", addr, e),
        }
    }

    if let Err(e) = swarm.behaviour_mut().kademlia.bootstrap() {
        println!("[p2p] kademlia bootstrap gagal: {}", e);
    }

    let mut sync_state = SyncState::new();
    let mut requested_headers: HashSet<PeerId> = HashSet::new();

    // RPC di task terpisah, pakai rpc_addr dari config.
    let chain_for_rpc = chain.clone();
    let cfg_for_rpc = config.clone();
    tokio::spawn(async move {
        let mut server = RpcServer::new(chain_for_rpc, &cfg_for_rpc);
        server.listen();
    });

    let blocks_topic = gossipsub::IdentTopic::new(p2p::TOPIC_BLOCKS);

    while let Some(event) = swarm.next().await {
        match event {
            libp2p::swarm::SwarmEvent::Behaviour(
                p2p::behaviour::ZenaBehaviourEvent::Identify(identify::Event::Received {
                    peer_id,
                    ..
                }),
            ) => {
                if !requested_headers.contains(&peer_id) {
                    requested_headers.insert(peer_id);

                    let msg = sync_state.make_get_headers(&chain);
                    let request_id =
                        swarm.behaviour_mut().sync.send_request(&peer_id, msg.to_bytes());

                    println!(
                        "[p2p] minta header dari {} (request {:?})",
                        peer_id, request_id
                    );
                }
            }

            libp2p::swarm::SwarmEvent::Behaviour(p2p::behaviour::ZenaBehaviourEvent::Sync(
                RrEvent::Message { peer, message, .. },
            )) => match message {
                RrMessage::Response { response, .. } => {
                    if let Some(msg) = SyncMessage::from_bytes(&response) {
                        handle_sync_message(&mut swarm, &mut chain, &mut sync_state, peer, msg);
                    }
                }
                RrMessage::Request { request, channel, .. } => {
                    if let Some(msg) = SyncMessage::from_bytes(&request) {
                        let reply = handle_sync_request(&chain, msg);
                        let _ = swarm
                            .behaviour_mut()
                            .sync
                            .send_response(channel, reply.to_bytes());
                    }
                }
            },

            libp2p::swarm::SwarmEvent::Behaviour(p2p::behaviour::ZenaBehaviourEvent::Gossipsub(
                gossipsub::Event::Message { message, .. },
            )) if message.topic == blocks_topic.hash() => {
                if let Ok(block) = serde_json::from_slice::<Block>(&message.data) {
                    match SyncState::accept_block(&mut chain, block) {
                        Ok(_) => println!("[p2p] blok diterima dari gossip"),
                        Err(e) => println!("[p2p] blok gossip ditolak: {}", e),
                    }
                }
            }

            _ => {}
        }
    }
}

fn handle_sync_message(
    swarm: &mut libp2p::Swarm<p2p::behaviour::ZenaBehaviour>,
    chain: &mut Blockchain,
    sync_state: &mut SyncState,
    peer: PeerId,
    msg: SyncMessage,
) {
    match msg {
        SyncMessage::Headers { headers } => {
            if let Err(e) = sync_state.apply_headers(chain, &headers) {
                println!("[p2p] header dari {} ditolak: {}", peer, e);
                return;
            }

            println!("[p2p] {} header diterima dari {}", headers.len(), peer);

            let hashes: Vec<String> = headers.iter().map(|h| h.hash.clone()).collect();
            if !hashes.is_empty() {
                let msg = SyncMessage::GetData { hashes };
                swarm
                    .behaviour_mut()
                    .sync
                    .send_request(&peer, msg.to_bytes());
            }
        }
        SyncMessage::Block { raw, .. } => {
            let block: Block = match serde_json::from_slice(&raw) {
                Ok(b) => b,
                Err(e) => {
                    println!("[p2p] blok dari {} tidak bisa diparse: {}", peer, e);
                    return;
                }
            };
            match SyncState::accept_block(chain, block) {
                Ok(_) => println!("[p2p] blok dari {} diterima", peer),
                Err(e) => println!("[p2p] blok dari {} ditolak: {}", peer, e),
            }
        }
        _ => {}
    }
}

fn handle_sync_request(chain: &Blockchain, msg: SyncMessage) -> SyncMessage {
    match msg {
        SyncMessage::GetHeaders { from_height, max } => {
            let mut headers = Vec::new();
            let start = from_height as usize;
            let end = (start + max as usize).min(chain.chain.len());

            for block in &chain.chain[start..end] {
                headers.push(p2p::BlockHeader {
                    nversion: block.nversion,
                    index: block.index,
                    timestamp: block.timestamp,
                    previous_hash: block.previous_hash.clone(),
                    merkle_root: block.merkle_root.clone(),
                    nonce: block.nonce,
                    nbits: block.nbits,
                    hash: block.hash.clone(),
                    reward: block.reward,
                });
            }

            SyncMessage::Headers { headers }
        }
        SyncMessage::GetData { hashes } => {
            for hash in hashes {
                if let Some(block) = chain.chain.iter().find(|b| b.hash == hash) {
                    let raw = serde_json::to_vec(block).unwrap_or_default();
                    return SyncMessage::Block {
                        height: block.index,
                        hash: block.hash.clone(),
                        raw,
                    };
                }
            }
            SyncMessage::Pong
        }
        _ => SyncMessage::Pong,
    }
}
