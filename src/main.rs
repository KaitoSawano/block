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
        "stop" => rpc::call_and_print("stop", vec![]),
        "getinfo" => rpc::call_and_print("getinfo", vec![]),
        "getblockchaininfo" => rpc::call_and_print("getblockchaininfo", vec![]),
        "getnetworkinfo" => rpc::call_and_print("getnetworkinfo", vec![]),
        "getcheckpoints" => rpc::call_and_print("getcheckpoints", vec![]),
        "getblocktemplate" => rpc::call_and_print("getblocktemplate", vec![]),
        "getblockcount" => rpc::call_and_print("getblockcount", vec![]),
        "getbestblockhash" => rpc::call_and_print("getbestblockhash", vec![]),
        "verifychain" => rpc::call_and_print("verifychain", vec![]),
        "validatemempool" => rpc::call_and_print("validatemempool", vec![]),
        "getgenesishash" => rpc::call_and_print("getgenesishash", vec![]),
        "getgenesis" => rpc::call_and_print("getgenesis", vec![]),
        "getwallet" => rpc::call_and_print("getwallet", vec![]),
        "getprivkey" => {
            if args.len() < 3 {
                println!("wajib memasukkan \"alamat tujuan\"");
                println!("pemakaian: zenachain getprivkey \"<alamat>\"");
                return;
            }
            rpc::call_and_print("getprivkey", vec![json!(args[2])]);
        }
        "getnewwallet" => rpc::call_and_print("getnewwallet", vec![]),
        "getaddtowallet" => rpc::call_and_print("getaddtowallet", vec![]),
        "getaddtoaddress" => {
            if args.len() < 3 {
                println!("pemakaian: zenachain getaddtoaddress \"<alamat>\"");
                return;
            }
            rpc::call_and_print("getaddtoaddress", vec![json!(args[2])]);
        }
        "getblockhash" => {
            if args.len() < 3 {
                println!("pemakaian: zenachain getblockhash <height>");
                return;
            }
            let h: u64 = match args[2].parse() {
                Ok(v) => v,
                Err(_) => {
                    println!("height tidak valid");
                    return;
                }
            };
            rpc::call_and_print("getblockhash", vec![json!(h)]);
        }
        "getblock" => {
            if args.len() < 3 {
                println!("pemakaian: zenachain getblock \"<hash>\"");
                return;
            }
            rpc::call_and_print("getblock", vec![json!(args[2])]);
        }
        "getrawtransaction" => {
            if args.len() < 3 {
                println!("pemakaian: zenachain getrawtransaction \"<txid>\"");
                return;
            }
            rpc::call_and_print("getrawtransaction", vec![json!(args[2])]);
        }
        "listunspent" => {
            let params = if args.len() >= 3 {
                vec![json!(args[2])]
            } else {
                vec![]
            };
            rpc::call_and_print("listunspent", params);
        }
        "getmempoolinfo" => rpc::call_and_print("getmempoolinfo", vec![]),
        "generatetoaddress" => {
            if args.len() < 4 {
                println!("pemakaian: zenachain generatetoaddress <jumlah> \"<alamat>\"");
                return;
            }
            let n: u64 = match args[2].parse() {
                Ok(v) => v,
                Err(_) => {
                    println!("jumlah blok tidak valid");
                    return;
                }
            };
            rpc::call_and_print("generatetoaddress", vec![json!(n), json!(args[3])]);
        }
        "sendtoaddress" => {
            if args.len() < 4 {
                println!("pemakaian: zenachain sendtoaddress <jumlah> \"<alamat>\"");
                return;
            }
            let amount: f64 = match args[2].parse() {
                Ok(v) => v,
                Err(_) => {
                    println!("jumlah tidak valid");
                    return;
                }
            };
            rpc::call_and_print("sendtoaddress", vec![json!(amount), json!(args[3])]);
        }
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
    println!("  zenachain help                                tampilkan bantuan ini");
    println!("  zenachain stop                                simpan chain dan hentikan node");
    println!("  zenachain getinfo                             informasi jaringan");
    println!("  zenachain getblockchaininfo                   informasi blockchain");
    println!("  zenachain getnetworkinfo                      parameter jaringan");
    println!("  zenachain getcheckpoints                      daftar checkpoint aktif");
    println!("  zenachain getblocktemplate                    template blok untuk miner");
    println!("  zenachain getblockcount                       jumlah blok di chain");
    println!("  zenachain getbestblockhash                    hash blok terakhir");
    println!("  zenachain verifychain                         verifikasi integritas seluruh chain");
    println!("  zenachain validatemempool                     validasi seluruh isi mempool");
    println!("  zenachain getgenesishash                      hash genesis");
    println!("  zenachain getgenesis                          detail genesis");
    println!("  zenachain getblockhash <height>               hash blok pada height tertentu");
    println!("  zenachain getblock \"<hash>\"                   detail blok berdasarkan hash");
    println!("  zenachain getrawtransaction \"<txid>\"          detail transaksi berdasarkan txid");
    println!("  zenachain listunspent [address]               daftar UTXO wallet");
    println!("  zenachain getmempoolinfo                      ringkasan mempool");
    println!("  zenachain getwallet                           buka dompet tersimpan");
    println!("  zenachain getnewwallet                        buat dompet baru");
    println!("  zenachain getaddtowallet                      pakai alamat wallet sebagai tujuan mining");
    println!("  zenachain getaddtoaddress \"<alamat>\"          set alamat mining manual");
    println!("  zenachain generatetoaddress <n> \"<alamat>\"    mining n blok langsung ke alamat");
    println!("  zenachain sendtoaddress <jumlah> \"<alamat>\"   kirim koin dari wallet ke alamat tujuan");
    println!("  zenachain getprivkey \"<alamat>\"               tampilkan private key untuk alamat itu");
    println!("  zenachain dumpprivkey \"<privkey>\"             pulihkan dompet dari private key");
    println!("  zenachain getbalance [address]                hitung saldo wallet");
    println!();
    println!("daemon:");
    println!("  zenachaind                                    jalankan node (tidak mining)");
    println!("  zenachain-miner                               jalankan miner terpisah");
}
