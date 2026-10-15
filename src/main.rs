use serde_json::json;
use zenachain::rpc;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "help" => print_help(),
        "getinfo" => rpc::call_and_print("getinfo", vec![]),
        "getblocktemplate" => rpc::call_and_print("getblocktemplate", vec![]),
        "getwallet" => rpc::call_and_print("getwallet", vec![]),
        "getprivkey" => rpc::call_and_print("getprivkey", vec![]),
        "getbalance" => {
            let params = if args.len() >= 3 {
                vec![json!(args[2])]
            } else {
                vec![]
            };
            rpc::call_and_print("getbalance", params);
        }
        "dumpprivkey" => {
            if args.len() < 3 {
                println!("pemakaian: zenachain dumpprivkey \"<privkey>\"");
                return;
            }
            rpc::call_and_print("dumpprivkey", vec![json!(args[2])]);
        }
        _ => {
            println!("perintah tidak dikenal: {}", args[1]);
            print_help();
        }
    }
}

fn print_help() {
    println!("ZenaChain CLI");
    println!("pemakaian:");
    println!("  zenachain help                          tampilkan bantuan ini");
    println!("  zenachain getinfo                       informasi jaringan");
    println!("  zenachain getblocktemplate              template blok untuk miner");
    println!("  zenachain getwallet                     buka dompet tersimpan");
    println!("  zenachain getprivkey                    tampilkan private key wallet");
    println!("  zenachain dumpprivkey \"<privkey>\"       pulihkan dompet dari private key");
    println!("  zenachain getbalance [address]          hitung saldo wallet");
    println!();
    println!("daemon:");
    println!("  zenachaind                              jalankan node (tidak mining)");
    println!("  zenachain-miner                         jalankan miner terpisah");
}
