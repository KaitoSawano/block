use libp2p::gossipsub::{self, MessageAuthenticity, ValidationMode};
use libp2p::identify;
use libp2p::kad;
use libp2p::ping;
use libp2p::swarm::NetworkBehaviour;
use libp2p::PeerId;
use std::time::Duration;

use crate::p2p::PROTOCOL_VERSION;

#[derive(NetworkBehaviour)]
pub struct ZenaBehaviour {
    pub ping: ping::Behaviour,
    pub identify: identify::Behaviour,
    pub gossipsub: gossipsub::Behaviour,
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
}

impl ZenaBehaviour {
    pub fn new(
        keypair: &libp2p::identity::Keypair,
        peer_id: PeerId,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Ping: interval 15 detik, timeout 20 detik.
        let ping = ping::Behaviour::new(
            ping::Config::new()
                .with_interval(Duration::from_secs(15))
                .with_timeout(Duration::from_secs(20)),
        );

        // Identify: bertukar informasi peer.
        let identify = identify::Behaviour::new(
            identify::Config::new(PROTOCOL_VERSION.to_string(), keypair.public())
                .with_interval(Duration::from_secs(60)),
        );

        // Gossipsub: broadcast blok dan transaksi.
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(1))
            .validation_mode(ValidationMode::Strict)
            .build()
            .map_err(|e| format!("gossipsub config: {}", e))?;

        let mut gossipsub = gossipsub::Behaviour::new(
            MessageAuthenticity::Signed(keypair.clone()),
            gossipsub_config,
        )
        .map_err(|e| format!("gossipsub behaviour: {}", e))?;

        // Subscribe ke topik blok dan transaksi.
        for topic in [crate::p2p::TOPIC_BLOCKS, crate::p2p::TOPIC_TX] {
            let t = gossipsub::IdentTopic::new(topic);
            gossipsub
                .subscribe(&t)
                .map_err(|e| format!("subscribe {}: {}", topic, e))?;
        }

        // Kademlia: peer discovery.
        let store = kad::store::MemoryStore::new(peer_id);
        let mut kademlia = kad::Behaviour::new(peer_id, store);
        kademlia.set_mode(Some(kad::Mode::Server));

        Ok(ZenaBehaviour {
            ping,
            identify,
            gossipsub,
            kademlia,
        })
    }
}
