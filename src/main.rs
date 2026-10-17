// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Command-line interface for ZenaChain.

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
        "dumpprivkey" => {
            if args.len() < 3 {
                println!("an address argument is required");
                println!("usage: zenachain dumpprivkey \"<address>\"");
                return;
            }
            rpc::call_and_print("dumpprivkey", vec![json!(args[2])]);
        }
        "importprivkey" => {
            if args.len() < 3 {
                println!("a private key argument is required");
                println!("usage: zenachain importprivkey \"<privkey>\"");
                return;
            }
            rpc::call_and_print("importprivkey", vec![json!(args[2])]);
        }
        "getnewwallet" => rpc::call_and_print("getnewwallet", vec![]),
        "getaddtowallet" => rpc::call_and_print("getaddtowallet", vec![]),
        "getaddtoaddress" => {
            if args.len() < 3 {
                println!("usage: zenachain getaddtoaddress \"<address>\"");
                return;
            }
            rpc::call_and_print("getaddtoaddress", vec![json!(args[2])]);
        }
        "getblockhash" => {
            if args.len() < 3 {
                println!("usage: zenachain getblockhash <height>");
                return;
            }
            let h: u64 = match args[2].parse() {
                Ok(v) => v,
                Err(_) => {
                    println!("invalid height");
                    return;
                }
            };
            rpc::call_and_print("getblockhash", vec![json!(h)]);
        }
        "getblock" => {
            if args.len() < 3 {
                println!("usage: zenachain getblock \"<hash>\"");
                return;
            }
            rpc::call_and_print("getblock", vec![json!(args[2])]);
        }
        "getrawtransaction" => {
            if args.len() < 3 {
                println!("usage: zenachain getrawtransaction \"<txid>\"");
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
                println!("usage: zenachain generatetoaddress <count> \"<address>\"");
                return;
            }
            let n: u64 = match args[2].parse() {
                Ok(v) => v,
                Err(_) => {
                    println!("invalid block count");
                    return;
                }
            };
            rpc::call_and_print("generatetoaddress", vec![json!(n), json!(args[3])]);
        }
        "sendtoaddress" => {
            if args.len() < 4 {
                println!("usage: zenachain sendtoaddress <amount> \"<address>\" [\"<from_address>\"]");
                return;
            }
            let amount: f64 = match args[2].parse() {
                Ok(v) => v,
                Err(_) => {
                    println!("invalid amount");
                    return;
                }
            };
            let mut params = vec![json!(amount), json!(args[3])];
            if args.len() >= 5 {
                params.push(json!(args[4]));
            }
            rpc::call_and_print("sendtoaddress", params);
        }
        "getbalance" => {
            let params = if args.len() >= 3 {
                vec![json!(args[2])]
            } else {
                vec![]
            };
            rpc::call_and_print("getbalance", params);
        }
        _ => {
            println!("unknown command: {}", args[1]);
            print_help();
        }
    }
}

fn print_help() {
    println!("ZenaChain CLI");
    println!("usage:");
    println!("  zenachain help                                display this help message");
    println!("  zenachain stop                                save the chain and stop the node");
    println!("  zenachain getinfo                             network information");
    println!("  zenachain getblockchaininfo                   blockchain information");
    println!("  zenachain getnetworkinfo                      network parameters");
    println!("  zenachain getcheckpoints                      list of active checkpoints");
    println!("  zenachain getblocktemplate                    block template for miners");
    println!("  zenachain getblockcount                       number of blocks in the chain");
    println!("  zenachain getbestblockhash                    hash of the latest block");
    println!("  zenachain verifychain                         verify the integrity of the entire chain");
    println!("  zenachain validatemempool                     validate every transaction in the mempool");
    println!("  zenachain getgenesishash                      genesis block hash");
    println!("  zenachain getgenesis                          genesis block details");
    println!("  zenachain getblockhash <height>               hash of the block at the given height");
    println!("  zenachain getblock \"<hash>\"                   block details for the given hash");
    println!("  zenachain getrawtransaction \"<txid>\"          transaction details for the given txid");
    println!("  zenachain listunspent [address]               list wallet UTXOs");
    println!("  zenachain getmempoolinfo                      mempool summary");
    println!("  zenachain getwallet                           list all addresses in the wallet");
    println!("  zenachain getnewwallet                        create a new address in the wallet");
    println!("  zenachain getaddtowallet                      use the default wallet address as the mining target");
    println!("  zenachain getaddtoaddress \"<address>\"         set a manual mining address");
    println!("  zenachain generatetoaddress <n> \"<address>\"   mine n blocks directly to the given address");
    println!("  zenachain sendtoaddress <amount> \"<address>\" [\"<from>\"] send coins from the wallet to the destination");
    println!("  zenachain dumpprivkey \"<address>\"             display the private key for the given address");
    println!("  zenachain importprivkey \"<privkey>\"           import a private key into the wallet");
    println!("  zenachain getbalance [address]                compute the wallet balance");
    println!();
    println!("daemon:");
    println!("  zenachaind                                    run the node (without mining)");
    println!("  zenachain-miner                               run the standalone miner");
}
