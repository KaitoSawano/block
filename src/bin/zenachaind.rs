use zenachain::node::Node;
use zenachain::rpc::RpcServer;

fn main() {
    let node = Node::new();
    let _ = node.mining_address();

    let mut server = RpcServer::new();
    server.listen();
}
