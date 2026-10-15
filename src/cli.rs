use crate::blockchain::{self, Blockchain};
use crate::storage;
use crate::transaction::Transaction;
use crate::utils::{satoshi_to_zenac, zenac_to_satoshi};
use crate::wallet::Wallet;
use serde_json::json;

pub fn run(args: &[String]) {
    if args.len() < 2 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "params" => cmd_params(),
        "info" => cmd_info(),
        "genesis" => cmd_genesis(),
        "mine" => cmd_mine(args),
        "send" => cmd_send(args),
        "wallet" => cmd_wallet(args),
        "save" => cmd_save(),
        "load" => cmd_load(),
        "verify" => cmd_verify(),
        "help" => print_help(),
        _ => {
            println!("perintah tidak dikenal: {}", args[1]);
            print_help();
        }
    }
}

fn cmd_params() {
    println!("params: menampilkan seluruh parameter resmi ZenaChain.");
    let out = json!({
        "coin_name": blockchain::coin_name(),
        "symbol": blockchain::symbol(),
        "version": blockchain::version(),
        "protocol_version": blockchain::protocol_version(),
        "hash_algo": blockchain::hash_algo(),
        "decimals": blockchain::decimals(),
        "satoshi": blockchain::satoshi(),
        "max_supply_zenac": blockchain::max_supply_zenac(),
        "max_supply_satoshi": blockchain::max_supply(),
        "initial_reward_zenac": blockchain::initial_reward_zenac(),
        "initial_reward_satoshi": blockchain::initial_reward(),
        "halving_interval_blocks": blockchain::halving_interval_blocks(),
        "max_halvings": blockchain::max_halvings(),
        "target_block_time_secs": blockchain::target_block_time_secs(),
        "difficulty_adjust_interval": blockchain::difficulty_adjust_interval(),
        "initial_difficulty": blockchain::initial_difficulty(),
        "min_difficulty": blockchain::min_difficulty(),
        "max_difficulty": blockchain::max_difficulty(),
        "genesis_message": blockchain::genesis_message(),
        "genesis_secret": blockchain::genesis_secret(),
        "genesis_timestamp": blockchain::genesis_timestamp(),
    });
    println!("{}", serde_json::to_string_pretty(&out).unwrap());
}

fn cmd_info() {
    println!("info: menampilkan status chain saat ini (height, era, supply, difficulty).");
    let chain = load_or_new();
    let out = json!({
        "height": chain.height(),
        "era": chain.current_era(),
        "blocks_until_halving": chain.blocks_until_halving(),
        "current_reward_satoshi": Blockchain::calculate_reward(chain.height() + 1),
        "current_reward_zenac": satoshi_to_zenac(Blockchain::calculate_reward(chain.height() + 1)),
        "circulating_supply_satoshi": chain.circulating_supply(),
        "circulating_supply_zenac": satoshi_to_zenac(chain.circulating_supply()),
        "max_supply_zenac": blockchain::max_supply_zenac(),
        "difficulty": chain.difficulty,
        "mempool_len": chain.mempool.len(),
        "chain_len": chain.chain.len(),
        "valid": chain.is_valid(),
    });
    println!("{}", serde_json::to_string_pretty(&out).unwrap());
}

fn cmd_genesis() {
    println!("genesis: menampilkan isi genesis block, termasuk pesan yang tertanam.");
    let chain = Blockchain::new();
    let g = &chain.chain[0];
    let txs: Vec<_> = g
        .transactions
        .iter()
        .map(|tx| {
            json!({
                "from": tx.from,
                "to": tx.to,
                "amount": tx.amount,
                "fee": tx.fee,
                "note": tx.note,
            })
        })
        .collect();

    let out = json!({
        "index": g.index,
        "timestamp": g.timestamp,
        "previous_hash": g.previous_hash,
        "hash": g.hash,
        "nonce": g.nonce,
        "reward_satoshi": g.reward,
        "transactions": txs,
    });
    println!("{}", serde_json::to_string_pretty(&out).unwrap());
}

fn cmd_mine(args: &[String]) {
    println!("mine <n>: menambang n blok secara berurutan. Default n=1.");
    let n: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    let mut chain = load_or_new();

    for _ in 0..n {
        chain.add_transaction(Transaction::new(
            "ZENA_alice",
            "ZENA_bob",
            10 * blockchain::satoshi(),
            blockchain::satoshi() / 10,
        ));
        match chain.mine_pending() {
            Ok(nonce) => {
                let last = chain.chain.last().unwrap();
                let out = json!({
                    "mined_height": chain.height(),
                    "nonce": nonce,
                    "difficulty": chain.difficulty,
                    "reward_satoshi": last.reward,
                    "hash": last.hash,
                });
                println!("{}", serde_json::to_string(&out).unwrap());
            }
            Err(e) => {
                let out = json!({ "error": e });
                println!("{}", serde_json::to_string(&out).unwrap());
                break;
            }
        }
        chain.adjust_difficulty();
    }

    let _ = storage::save_chain(&chain, storage::CHAIN_FILE);
}

fn cmd_send(args: &[String]) {
    println!("send <from> <to> <amount_zenac>: memasukkan transaksi ke mempool.");
    if args.len() < 5 {
        let out = json!({ "error": "usage: send <from> <to> <amount_zenac>" });
        println!("{}", serde_json::to_string(&out).unwrap());
        return;
    }
    let from = &args[2];
    let to = &args[3];
    let amount_zenac: f64 = match args[4].parse() {
        Ok(v) => v,
        Err(_) => {
            let out = json!({ "error": "invalid amount" });
            println!("{}", serde_json::to_string(&out).unwrap());
            return;
        }
    };
    let amount_sat = zenac_to_satoshi(amount_zenac);
    let mut chain = load_or_new();
    let tx = Transaction::new(from, to, amount_sat, blockchain::satoshi() / 100);
    let ok = chain.add_transaction(tx);

    let out = json!({
        "accepted": ok,
        "from": from,
        "to": to,
        "amount_satoshi": amount_sat,
        "mempool_len": chain.mempool.len(),
    });
    println!("{}", serde_json::to_string_pretty(&out).unwrap());
    let _ = storage::save_chain(&chain, storage::CHAIN_FILE);
}

fn cmd_wallet(args: &[String]) {
    println!("wallet <new|show>: membuat wallet baru atau menampilkan wallet tersimpan.");
    if args.len() < 3 {
        let out = json!({ "error": "usage: wallet <new|show>" });
        println!("{}", serde_json::to_string(&out).unwrap());
        return;
    }
    match args[2].as_str() {
        "new" => {
            let w = Wallet::new();
            let _ = storage::save_wallet(&w, storage::WALLET_FILE);
            let out = json!({
                "address": w.address,
                "public_key": w.public_key,
                "secret": w.secret,
                "created_at": w.created_at,
                "saved_to": storage::WALLET_FILE,
            });
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
        }
        "show" => match storage::load_wallet(storage::WALLET_FILE) {
            Ok(w) => {
                let out = json!({
                    "address": w.address,
                    "public_key": w.public_key,
                    "created_at": w.created_at,
                });
                println!("{}", serde_json::to_string_pretty(&out).unwrap());
            }
            Err(e) => {
                let out = json!({ "error": e.to_string() });
                println!("{}", serde_json::to_string(&out).unwrap());
            }
        },
        _ => {
            let out = json!({ "error": "unknown wallet subcommand" });
            println!("{}", serde_json::to_string(&out).unwrap());
        }
    }
}

fn cmd_save() {
    println!("save: menyimpan chain ke disk (chain.json).");
    let chain = load_or_new();
    match storage::save_chain(&chain, storage::CHAIN_FILE) {
        Ok(_) => {
            let out = json!({ "saved": true, "path": storage::CHAIN_FILE });
            println!("{}", serde_json::to_string(&out).unwrap());
        }
        Err(e) => {
            let out = json!({ "error": e.to_string() });
            println!("{}", serde_json::to_string(&out).unwrap());
        }
    }
}

fn cmd_load() {
    println!("load: memuat chain dari disk (chain.json).");
    match storage::load_chain(storage::CHAIN_FILE) {
        Ok(chain) => {
            let out = json!({
                "loaded": true,
                "height": chain.height(),
                "chain_len": chain.chain.len(),
                "valid": chain.is_valid(),
            });
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
        }
        Err(e) => {
            let out = json!({ "error": e.to_string() });
            println!("{}", serde_json::to_string(&out).unwrap());
        }
    }
}

fn cmd_verify() {
    println!("verify: memeriksa integritas chain (hash, prev_hash, reward, PoW).");
    let chain = load_or_new();
    let out = json!({
        "valid": chain.is_valid(),
        "chain_len": chain.chain.len(),
        "height": chain.height(),
    });
    println!("{}", serde_json::to_string_pretty(&out).unwrap());
}

fn load_or_new() -> Blockchain {
    storage::load_chain(storage::CHAIN_FILE).unwrap_or_else(|_| Blockchain::new())
}

fn print_help() {
    println!("ZenaChain CLI - daftar perintah:");
    println!("  params                              tampilkan parameter resmi");
    println!("  info                                status chain saat ini");
    println!("  genesis                             isi genesis block");
    println!("  mine <n>                            tambang n blok (default 1)");
    println!("  send <from> <to> <amount_zenac>     kirim transaksi ke mempool");
    println!("  wallet <new|show>                   buat / lihat wallet");
    println!("  save                                simpan chain ke chain.json");
    println!("  load                                muat chain dari chain.json");
    println!("  verify                              verifikasi integritas chain");
    println!("  help                                tampilkan bantuan ini");
}
