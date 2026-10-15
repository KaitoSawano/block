use crate::blockchain::Blockchain;
use crate::config::{Config, CONFIG_FILE};
use crate::storage;
use crate::transaction::Transaction;

pub struct Node {
    pub chain: Blockchain,
    pub config: Config,
}

impl Node {
    pub fn new() -> Self {
        println!("[node] new: menginisialisasi node dengan chain dan config.");
        let config = Config::load_or_default(CONFIG_FILE);
        let chain = storage::load_chain(storage::CHAIN_FILE)
            .unwrap_or_else(|_| Blockchain::new());
        Node { chain, config }
    }

    pub fn mining_address(&self) -> &str {
        &self.config.mining_address
    }

    pub fn run_once(&mut self) {
        println!("[node] run_once: satu siklus mining ke alamat {}.", self.mining_address());
        let addr = self.config.mining_address.clone();

        self.chain.add_transaction(Transaction::new(
            "NETWORK",
            &addr,
            crate::blockchain::satoshi() / 100,
            0,
        ));

        match self.chain.mine_pending() {
            Ok(nonce) => {
                println!(
                    "[node] blok ditambang height={} nonce={} difficulty={} reward={}",
                    self.chain.height(),
                    nonce,
                    self.chain.difficulty,
                    self.chain.chain.last().unwrap().reward
                );
            }
            Err(e) => {
                println!("[node] mining gagal: {}", e);
            }
        }

        self.chain.adjust_difficulty();
    }

    pub fn run_forever(&mut self) {
        println!("[node] run_forever: node berjalan terus, mining ke {}.", self.mining_address());
        loop {
            self.run_once();
            let _ = storage::save_chain(&self.chain, storage::CHAIN_FILE);
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::new()
    }
}
