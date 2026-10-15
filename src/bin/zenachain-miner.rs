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
        let nbits = template["nbits"].as_u64().unwrap_or(0) as u32;

        let mut mempool: Vec<Transaction> = serde_json::from_value(template["mempool"].clone())
            .unwrap_or_default();

        let target_address = std::env::var("ZENAC_MINE_FOR")
            .unwrap_or_else(|_| "UNKNOWN".to_string());

        let reward = zenachain::blockchain::Blockchain::calculate_reward(height);

        mempool.push(Transaction::new_with_inputs(
            "MEMPOOL", "CreateNewBlock", 0, 0, Vec::new(),
            format!("height={} target={}", height, target_address),
        ));

        let mut block = Block::new_for(
            height, previous_hash, reward, nbits, mempool, target_address.clone(),
        );

        if !zenachain::pow::mine(&mut block, nbits) {
            continue;
        }

        let block_json = serde_json::to_value(&block).unwrap();
        match rpc::call("submitblock", vec![block_json]) {
            Ok(r) if r.ok => {
                println!("[miner] blok diterima: {}", serde_json::to_string(&r.result).unwrap());
            }
            Ok(r) => println!("[miner] blok ditolak: {}", r.error),
            Err(e) => println!("[miner] gagal submit: {}", e),
        }
    }
}
