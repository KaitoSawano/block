use zenachain::config::{Config, CONFIG_FILE};
use zenachain::utils::now_millis;

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
    };

    match cfg.save(CONFIG_FILE) {
        Ok(_) => {
            println!("[main] alamat mining disimpan: {}", address);
            println!("[main] tersimpan di: {}", CONFIG_FILE);
        }
        Err(e) => {
            println!("[main] gagal menyimpan config: {}", e);
        }
    }
}

fn print_help() {
    println!("ZenaChain CLI");
    println!("pemakaian:");
    println!("  zenachain help                          tampilkan bantuan ini");
    println!("  zenachain addtoaddress \"<alamat>\"       set alamat tujuan mining");
    println!("  zenachaind                              jalankan node / mining server");
}
}
