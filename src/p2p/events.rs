use libp2p::gossipsub::Event as GossipsubEvent;
use libp2p::identify::Event as IdentifyEvent;
use libp2p::kad::Event as KademliaEvent;
use libp2p::ping::Event as PingEvent;
use libp2p::swarm::SwarmEvent;

use crate::p2p::behaviour::ZenaBehaviourEvent;

/// Ringkasan event P2P yang perlu ditangani di loop utama.
#[derive(Debug)]
pub enum P2PEvent {
    Ping(PingEvent),
    Identify(IdentifyEvent),
    Gossipsub(GossipsubEvent),
    Kademlia(KademliaEvent),
    Other,
}

pub fn classify(event: SwarmEvent<ZenaBehaviourEvent>) -> P2PEvent {
    match event {
        SwarmEvent::Behaviour(ZenaBehaviourEvent::Ping(e)) => P2PEvent::Ping(e),
        SwarmEvent::Behaviour(ZenaBehaviourEvent::Identify(e)) => P2PEvent::Identify(e),
        SwarmEvent::Behaviour(ZenaBehaviourEvent::Gossipsub(e)) => P2PEvent::Gossipsub(e),
        SwarmEvent::Behaviour(ZenaBehaviourEvent::Kademlia(e)) => P2PEvent::Kademlia(e),
        _ => P2PEvent::Other,
    }
}
