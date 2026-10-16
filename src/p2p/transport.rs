use libp2p::core::muxing::StreamMuxerBox;
use libp2p::core::transport::Boxed;
use libp2p::core::upgrade;
use libp2p::{identity, noise, tcp, yamux, PeerId, Transport};

/// Bangun transport TCP + Noise + Yamux.
pub fn build_transport(keypair: &identity::Keypair) -> Boxed<(PeerId, StreamMuxerBox)> {
    tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
        .upgrade(upgrade::Version::V1Lazy)
        .authenticate(noise::Config::new(keypair).expect("noise config tidak boleh gagal"))
        .multiplex(yamux::Config::default())
        .boxed()
}
