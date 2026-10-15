use zenachain::config::{Config, CONFIG_FILE};
use zenachain::datadir;
use zenachain::storage;
use zenachain::utils::now_millis;
use zenachain::wallet::Wallet;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "help" => print_help(),
        "addtoaddress" => cmd_add_to_address(&args),
        "createwallet" => cmd_create_wallet(),
        "showwallet" => cmd_show_wallet(),
        "initdatadir" => cmd_init_datadir(),
        "info" => cmd_info(),
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

fn cmd_create_wallet() {
    if Wallet::exists_default() {
        println!("[main] wallet sudah ada di {}", datadir::as_str(&datadir::wallet_path()));
        println!("[main] hapus file tersebut bila ingin membuat wallet baru.");
        return;
    }
    let w = Wallet::new();
    match w.save_default() {
        Ok(_) => {
            println!("[main] wallet dibuat.");
            println!("[main] address: {}", w.address);
            println!("[main] lokasi: {}", datadir::as_str(&datadir::wallet_path()));
        }
        Err(e) => println!("[main] gagal menyimpan wallet: {}", e),
    }
}

fn cmd_show_wallet() {
    match Wallet::load_default() {
        Ok(w) => {
            println!("address: {}", w.address);
            println!("public_key: {}", w.public_key);
            println!("created_at: {}", w.created_at);
            println!("lokasi: {}", datadir::as_str(&datadir::wallet_path()));
        }
        Err(e) => println!("[main] wallet tidak dapat dimuat: {}", e),
    }
}

fn cmd_init_datadir() {
    match datadir::ensure_dirs() {
        Ok(_) => {
            println!("[main] struktur .chain disiapkan di {}", datadir::as_str(&datadir::datadir()));
            println!("[main] blocks    : {}", datadir::as_str(&datadir::blocks_dir()));
            println!("[main] chainstate: {}", datadir::as_str(&datadir::chainstate_dir()));
            println!("[main] wallet    : {}", datadir::as_str(&datadir::wallet_path()));
            println!("[main] config    : {}", datadir::as_str(&datadir::config_path()));
            println!("[main] peers     : {}", datadir::as_str(&datadir::peers_path()));
            println!("[main] mempool   : {}", datadir::as_str(&datadir::mempool_path()));
            println!("[main] debug log : {}", datadir::as_str(&datadir::debug_log_path()));
        }
        Err(e) => println!("[main] gagal menyiapkan .chain: {}", e),
    }
}

fn cmd_info() {
    let chain = storage::load_chain().unwrap_or_else(|_| zenachain::blockchain::Blockchain::new());
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
    println!("  zenachain createwallet                  buat wallet baru di .chain/wallet.dat");
    println!("  zenachain showwallet                    tampilkan wallet tersimpan");
    println!("  zenachain addtoaddress \"<alamat>\"       set alamat tujuan mining");
    println!("  zenachain info                          ringkasan chain dan datadir");
    println!("  zenachaind                              jalankan node / mining server");
}

#[allow(dead_code)]
fn _unused_config_file_reference() -> &'static str {
    CONFIG_FILE
}
