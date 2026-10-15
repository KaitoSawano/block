use zenachain::blockchain::*;
use zenachain::transaction::Transaction;

fn main() {
    let mut chain = Blockchain::new();

    println!("=== {} ({}) ===\n", COIN_NAME, SYMBOL);

    let genesis = &chain.chain[0];
    println!("Genesis Hash : {}", genesis.hash);
    for tx in &genesis.transactions {
        if tx.from == "GENESIS" {
            println!("Genesis Note : \"{}\"", tx.note);
        }
    }
    println!("\n{}\n", chain.info());

    for i in 1..=3 {
        chain.add_transaction(Transaction::new(
            "ZENA_alice",
            "ZENA_bob",
            10 * SATOSHI,
            1 * SATOSHI,
        ));
        println!("Mining blok #{}...", i);
        chain.mine_pending().unwrap();
        println!("{}\n", chain.info());
    }

    println!("Chain valid? {}", chain.is_valid());
}
