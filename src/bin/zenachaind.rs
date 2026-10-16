use zenachain::blockchain::{self, Blockchain};
use zenachain::rpc::RpcServer;

fn main() {
    // Paksa bikin genesis dan cek terhadap EXPECTED_GENESIS_*.
    // Kalau tidak cocok, panic dan node keluar.
    let genesis_block = zenachain::block::Block::genesis();
    blockchain::check_genesis_consensus(&genesis_block);

    let node = zenachain::node::Node::new();
    let _ = node.mining_address();

    // Bikin chain. Di dalam Blockchain::new() cek dijalankan ulang.
    let _chain = Blockchain::new();

    let mut server = RpcServer::new();
    server.listen();
}
