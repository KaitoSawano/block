use zenachain::node::Node;

fn main() {
    println!("[zenachaind] node ZenaChain dijalankan.");
    let mut node = Node::new();
    println!("[zenachaind] alamat mining: {}", node.mining_address());
    node.run_forever();
}
