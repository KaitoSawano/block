use zenachain::config::Config;
use zenachain::datadir;
use zenachain::storage;
use zenachain::utils::{now_millis, satoshi_to_zenac};
use zenachain::wallet::Wallet;

fn main() {
    println!("[main] zenachain CLI.");
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "help" => print_help(),
        "addtoaddress" => cmd_add_to_address(&args),
        "getwallet" => cmd_get_wallet(),
        "createwallet" => cmd_get_wallet(),
        "showwallet" => cmd_get_wallet(),
        "getwalletinfo" => cmd_get_wallet(),
        "initdatadir" => cmd_init_datadir(),
        "info" => cmd_info(),
        "getinfo" => cmd_get_info(),
        "getbalance" => cmd_get_balance(),
        "dumpprivkey" => cmd_dump_privkey(&args),
        "getprivkey" => cmd_get_privkey(),
        _ => {
            println!("perintah tidak dikenal: {}", args[1]);
            print_help();
        }
    }
}

fn cmd_add_to_address(args: &[String]) {
    if args.len() < 3 {
        println!("addtoaddress: butuh argumen alamat.");
        println!("pemakaian: zenachain addtoaddress \"<alamat>\"");
        return;
    }
    let address = args[2].clone();
    let cfg = Config {
        mining_address: address.clone(),
        created_at: now_millis(),
        protocol_version: zenachain::blockchain::protocol_version(),
        network: "main".to_string(),
    };
    match cfg.save_default() {
        Ok(_) => {
            println!("[main] alamat mining disimpan: {}", address);
            println!("[main] lokasi: {}", datadir::as_str(&datadir::config_path()));
        }
        Err(e) => println!("[main] gagal menyimpan config: {}", e),
    }
}

fn cmd_get_wallet() {
    println!("getwallet: membuka dompet. Jika belum ada, dompet baru dibuat.");
    if Wallet::exists_default() {
        match Wallet::load_default() {
            Ok(w) => print_wallet(&w),
            Err(e) => println!("[main] wallet tidak dapat dimuat: {}", e),
        }
        return;
    }

    let w = Wallet::new();
    match w.save_default() {
        Ok(_) => {
            println!("[main] dompet baru dibuat.");
            print_wallet(&w);
        }
        Err(e) => println!("[main] gagal menyimpan wallet: {}", e),
    }
}

fn print_wallet(w: &Wallet) {
    println!("address    : {}", w.address);
    println!("public_key : {}", w.public_key);
    println!("secret     : {}", w.secret);
    println!("created_at : {}", w.created_at);
    println!("lokasi     : {}", datadir::as_str(&datadir::wallet_path()));
}

fn cmd_get_info() {
    println!("getinfo: menampilkan informasi jaringan ZenaChain.");
    let chain = storage::load_chain()
        .unwrap_or_else(|_| zenachain::blockchain::Blockchain::new());
    println!("coin_name        : {}", zenachain::blockchain::coin_name());
    println!("symbol           : {}", zenachain::blockchain::symbol());
    println!("protocol_version : {}", zenachain::blockchain::protocol_version());
    println!("hash_algo        : {}", zenachain::blockchain::hash_algo());
    println!("network          : main");
    println!("height           : {}", chain.height());
    println!("chain_len        : {}", chain.chain.len());
    println!("difficulty       : {}", chain.difficulty);
    println!("circulating      : {}", chain.circulating_supply());
    println!("max_supply       : {}", zenachain::blockchain::max_supply());
    println!("era              : {}", chain.current_era());
    println!("until_halving    : {}", chain.blocks_until_halving());
    println!("mempool_len      : {}", chain.mempool.len());
    println!("valid            : {}", chain.is_valid());
    println!("datadir          : {}", datadir::as_str(&datadir::datadir()));
}

fn cmd_get_balance() {
    println!("getbalance: menghitung saldo wallet dari chain.");
    let wallet = match Wallet::load_default() {
        Ok(w) => w,
        Err(e) => {
            println!("[main] wallet tidak dapat dimuat: {}", e);
            return;
        }
    };
    let chain = storage::load_chain()
        .unwrap_or_else(|_| zenachain::blockchain::Blockchain::new());

    let mut received: u64 = 0;
    let mut sent: u64 = 0;

    for block in &chain.chain {
        for tx in &block.transactions {
            if tx.to == wallet.address {
                received += tx.amount;
            }
            if tx.from == wallet.address {
                sent += tx.amount + tx.fee;
            }
        }
    }

    for tx in &chain.mempool {
        if tx.to == wallet.address {
            received += tx.amount;
        }
        if tx.from == wallet.address {
            sent += tx.amount + tx.fee;
        }
    }

    let balance = received.saturating_sub(sent);
    println!("address   : {}", wallet.address);
    println!("received  : {} sat", received);
    println!("sent      : {} sat", sent);
    println!("balance   : {} sat", balance);
    println!("balance   : {} ZENAC", satoshi_to_zenac(balance));
}

fn cmd_dump_privkey(args: &[String]) {
    println!("dumpprivkey: login dompet memakai private key yang diberikan.");
    if args.len() < 3 {
        println!("pemakaian: zenachain dumpprivkey \"<private_key_hex>\"");
        return;
    }
    let secret = args[2].clone();
    let w = Wallet::from_secret(&secret);
    match w.save_default() {
        Ok(_) => {
            println!("[main] wallet dari private key disimpan.");
            print_wallet(&w);
        }
        Err(e) => println!("[main] gagal menyimpan wallet: {}", e),
    }
}

fn cmd_get_privkey() {
    println!("getprivkey: menampilkan private key wallet tersimpan.");
    match Wallet::load_default() {
        Ok(w) => {
            println!("address    : {}", w.address);
            println!("privkey    : {}", w.secret);
            println!("lokasi     : {}", datadir::as_str(&datadir::wallet_path()));
        }
        Err(e) => println!("[main] wallet tidak dapat dimuat: {}", e),
    }
}

fn cmd_init_datadir() {
    match datadir::ensure_dirs() {
        Ok(_) => {
            println!(
                "[main] struktur .chain disiapkan di {}",
                datadir::as_str(&datadir::datadir())
            );
            println!("[main] blocks    : {}", datadir::as_str(&datadir::blocks_dir()));
            println!(
                "[main] chainstate: {}",
                datadir::as_str(&datadir::chainstate_dir())
            );
            println!(
                "[main] wallet    : {}",
                datadir::as_str(&datadir::wallet_path())
            );
            println!(
                "[main] config    : {}",
                datadir::as_str(&datadir::config_path())
            );
            println!(
                "[main] peers     : {}",
                datadir::as_str(&datadir::peers_path())
            );
            println!(
                "[main] mempool   : {}",
                datadir::as_str(&datadir::mempool_path())
            );
            println!(
                "[main] debug log : {}",
                datadir::as_str(&datadir::debug_log_path())
            );
        }
        Err(e) => println!("[main] gagal menyiapkan .chain: {}", e),
    }
}

fn cmd_info() {
    let chain =
        storage::load_chain().unwrap_or_else(|_| zenachain::blockchain::Blockchain::new());
    let w_exists = Wallet::exists_default();
    println!("datadir      : {}", datadir::as_str(&datadir::datadir()));
    println!("height       : {}", chain.height());
    println!("chain_len    : {}", chain.chain.len());
    println!("difficulty   : {}", chain.difficulty);
    println!("supply       : {}", chain.circulating_supply());
    println!("valid        : {}", chain.is_valid());
    println!("wallet_exists: {}", w_exists);
    println!("config_file  : {}", datadir::as_str(&datadir::config_path()));
    println!("chainstate   : {}", datadir::as_str(&datadir::chainstate_file()));
    println!("blocks_dir   : {}", datadir::as_str(&datadir::blocks_dir()));
}

fn print_help() {
    println!("ZenaChain CLI");
    println!("pemakaian:");
    println!("  zenachain help                          tampilkan bantuan ini");
    println!("  zenachain initdatadir                   siapkan struktur .chain");
    println!("  zenachain getwallet                     buka atau buat dompet baru");
    println!("  zenachain getprivkey                    tampilkan private key wallet");
    println!("  zenachain dumpprivkey \"<privkey>\"       login dompet memakai private key");
    println!("  zenachain getbalance                    hitung saldo wallet dari chain");
    println!("  zenachain getinfo                       informasi jaringan ZenaChain");
    println!("  zenachain addtoaddress \"<alamat>\"       set alamat tujuan mining");
    println!("  zenachain info                          ringkasan chain dan datadir");
    println!("  zenachaind                              jalankan node / mining server");
}
