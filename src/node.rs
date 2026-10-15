use crate::blockchain::Blockchain;
use crate::config::Config;
use crate::datadir;
use crate::storage;
use crate::transaction::Transaction;

pub struct Node {
    pub chain: Blockchain,
    pub config: Config,
}

impl Node {
    pub fn new() -> Self {
        println!("[node] new: menginisialisasi node dari .chain.");
        let _ = datadir::ensure_dirs();
        let config = Config::load_or_default();
        let chain = storage::load_chain().unwrap_or_else(|_| Blockchain::new());
        Node { chain, config }
    }

    pub fn mining_address(&self) -> &str {
        &self.config.mining_address
    }

    pub fn run_once(&mut self) {
        let addr = self.config.mining_address.clone();
        datadir::append_debug_log(&format!("mine cycle start target={}", addr));

        self.chain.add_transaction(Transaction::new(
            "NETWORK",
            &addr,
            crate::blockchain::satoshi() / 100,
            0,
        ));

        match self.chain.mine_pending() {
            Ok(nonce) => {
                let last = self.chain.chain.last().unwrap();
                datadir::append_debug_log(&format!(
                    "block mined height={} nonce={} difficulty={} reward={} hash={}",
                    self.chain.height(),
                    nonce,
                    self.chain.difficulty,
                    last.reward,
                    last.hash
                ));
                println!(
                    "[node] blok height={} nonce={} difficulty={} reward={}",
                    self.chain.height(),
                    nonce,
                    self.chain.difficulty,
                    last.reward
                );
            }
            Err(e) => {
                datadir::append_debug_log(&format!("mine failed: {}", e));
                println!("[node] mining gagal: {}", e);
            }
        }

        self.chain.adjust_difficulty();
    }

    pub fn run_forever(&mut self) {
        println!("[node] run_forever: node aktif, target mining {}.", self.mining_address());
        loop {
            self.run_once();
            let _ = storage::save_chain(&self.chain);
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::new()
    }
}
