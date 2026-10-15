use zenachain::node::Node;
use zenachain::rpc::RpcServer;

fn main() {
    println!("[zenachaind] node ZenaChain dijalankan.");
    let node = Node::new();
    println!("[zenachaind] target mining: {}", node.mining_address());
    println!("[zenachaind] node tidak mining. Mining dijalankan oleh zenachain-miner.");

    let mut server = RpcServer::new();
    server.listen();
}
