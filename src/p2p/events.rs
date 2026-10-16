use libp2p::gossipsub::Event as GossipsubEvent;
use libp2p::identify::Event as IdentifyEvent;
use libp2p::kad::Event as KademliaEvent;
use libp2p::ping::Event as PingEvent;
use libp2p::request_response::Event as RequestResponseEvent;
use libp2p::swarm::SwarmEvent;

use crate::p2p::behaviour::ZenaBehaviourEvent;

#[derive(Debug)]
pub enum P2PEvent {
    Ping(PingEvent),
    Identify(Box<IdentifyEvent>),
    Gossipsub(Box<GossipsubEvent>),
    Kademlia(Box<KademliaEvent>),
    Sync(RequestResponseEvent<Vec<u8>, Vec<u8>>),
    Other,
}

pub fn classify(event: SwarmEvent<ZenaBehaviourEvent>) -> P2PEvent {
    match event {
        SwarmEvent::Behaviour(ZenaBehaviourEvent::Ping(e)) => P2PEvent::Ping(e),
        SwarmEvent::Behaviour(ZenaBehaviourEvent::Identify(e)) => {
            P2PEvent::Identify(Box::new(e))
        }
        SwarmEvent::Behaviour(ZenaBehaviourEvent::Gossipsub(e)) => {
            P2PEvent::Gossipsub(Box::new(e))
        }
        SwarmEvent::Behaviour(ZenaBehaviourEvent::Kademlia(e)) => {
            P2PEvent::Kademlia(Box::new(e))
        }
        SwarmEvent::Behaviour(ZenaBehaviourEvent::Sync(e)) => P2PEvent::Sync(e),
        _ => P2PEvent::Other,
    }
}
