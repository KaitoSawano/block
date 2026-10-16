use libp2p::identity;
use libp2p::swarm::Swarm;
use libp2p::SwarmBuilder;

use crate::p2p::behaviour::ZenaBehaviour;

/// Bangun Swarm dari identity, transport, dan behaviour.
pub async fn build_swarm(
    keypair: identity::Keypair,
) -> Result<Swarm<ZenaBehaviour>, Box<dyn std::error::Error>> {
    let swarm = SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_other_transport(|key| Ok(crate::p2p::transport::build_transport(key)))?
        .with_behaviour(|key| {
            let peer_id = key.public().to_peer_id();
            ZenaBehaviour::new(key, peer_id).map_err(|e| {
                Box::new(std::io::Error::other(e.to_string()))
                    as Box<dyn std::error::Error + Send + Sync>
            })
        })?
        .with_swarm_config(|cfg| {
            cfg.with_idle_connection_timeout(std::time::Duration::from_secs(60))
        })
        .build();

    Ok(swarm)
}

/// Muat atau generate keypair untuk peer identity.
pub fn load_or_generate_keypair(path: &str) -> identity::Keypair {
    if let Ok(bytes) = std::fs::read(path) {
        if let Ok(kp) = identity::Keypair::from_protobuf_encoding(&bytes) {
            return kp;
        }
    }

    let kp = identity::Keypair::generate_ed25519();
    if let Ok(bytes) = kp.to_protobuf_encoding() {
        let _ = std::fs::write(path, bytes);
    }
    kp
}
