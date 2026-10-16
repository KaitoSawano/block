use zenachain::blockchain::Blockchain;
use zenachain::node::Node;
use zenachain::rpc::RpcServer;

fn main() {
    // Bikin chain sekali saja. Genesis dihitung dan di-assert di sini.
    let node = Node::new();
    let _ = node.mining_address();

    let chain = Blockchain::new();

    let mut server = RpcServer::new(chain);
    server.listen();
}
