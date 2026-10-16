use futures::StreamExt;
use zenachain::blockchain::Blockchain;
use zenachain::p2p;
use zenachain::rpc::RpcServer;

#[tokio::main]
async fn main() {
    // Bikin chain. Genesis dihitung dan di-assert di sini.
    let chain = Blockchain::new();

    // Bikin peer identity (Ed25519). Terpisah dari wallet.
    let keypair = p2p::swarm::load_or_generate_keypair(".chain/peer_id");

    // Bikin swarm.
    let mut swarm = match p2p::build_swarm(keypair).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[p2p] gagal bikin swarm: {}", e);
            return;
        }
    };

    // Listen di alamat default.
    if let Err(e) = swarm.listen_on(p2p::peers::LISTEN_ADDR.parse().unwrap()) {
        eprintln!("[p2p] gagal listen {}: {}", p2p::peers::LISTEN_ADDR, e);
    }

    // Konek ke peer bootstrap.
    for addr in p2p::peers::BOOTSTRAP_PEERS {
        if let Ok(ma) = addr.parse() {
            if let Err(e) = swarm.dial(ma) {
                eprintln!("[p2p] gagal dial {}: {}", addr, e);
            }
        }
    }

    // RPC server di task terpisah.
    let chain_for_rpc = chain.clone();
    tokio::spawn(async move {
        let mut server = RpcServer::new(chain_for_rpc);
        server.listen();
    });

    // Loop event P2P.
    loop {
        match swarm.next().await {
            Some(event) => {
                let _ = p2p::events::classify(event);
            }
            None => break,
        }
    }
}
