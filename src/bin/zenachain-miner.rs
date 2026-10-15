use zenachain::block::Block;
use zenachain::rpc;
use zenachain::transaction::Transaction;

fn main() {
    println!("[miner] zenachain-miner dijalankan.");
    println!("[miner] terhubung ke node di {}.", rpc::RPC_ADDR);

    loop {
        let template = match rpc::call("getblocktemplate", vec![]) {
            Ok(r) if r.ok => r.result,
            Ok(r) => {
                println!("[miner] node menolak: {}", r.error);
                std::thread::sleep(std::time::Duration::from_secs(5));
                continue;
            }
            Err(e) => {
                println!("[miner] tidak dapat hubungi node: {}", e);
                std::thread::sleep(std::time::Duration::from_secs(5));
                continue;
            }
        };

        let height = template["height"].as_u64().unwrap_or(0);
        let previous_hash = template["previous_hash"].as_str().unwrap_or("").to_string();
        let difficulty = template["difficulty"].as_u64().unwrap_or(0) as usize;
        let reward = template["reward"].as_u64().unwrap_or(0);

        let mempool: Vec<Transaction> = serde_json::from_value(template["mempool"].clone())
            .unwrap_or_default();

        println!(
            "[miner] template height={} difficulty={} reward={}",
            height, difficulty, reward
        );

        let mut block = Block::new(height, previous_hash, reward, mempool);
        zenachain::pow::mine(&mut block, difficulty);

        println!(
            "[miner] blok ditambang nonce={} hash={}",
            block.nonce, block.hash
        );

        let block_json = serde_json::to_value(&block).unwrap();
        match rpc::call("submitblock", vec![block_json]) {
            Ok(r) if r.ok => {
                println!(
                    "[miner] blok diterima node: {}",
                    serde_json::to_string(&r.result).unwrap()
                );
            }
            Ok(r) => println!("[miner] blok ditolak: {}", r.error),
            Err(e) => println!("[miner] gagal submit: {}", e),
        }
    }
}
