// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! Command-line interface for ZenaChain.

use serde_json::json;
use zenachain::consensus::spec::{self, Network};
use zenachain::rpc;

fn main() {
    let raw_args: Vec<String> = std::env::args().collect();

    // Parse -testnet / --testnet / -regtest / --regtest before dispatching RPC.
    let mut filtered: Vec<String> = vec![raw_args[0].clone()];
    let mut selected_network = Network::Main;

    for arg in &raw_args[1..] {
        if arg == "-testnet" || arg == "--testnet" {
            selected_network = Network::Test;
        } else if arg == "-regtest" || arg == "--regtest" {
            selected_network = Network::Regtest;
        } else {
            filtered.push(arg.clone());
        }
    }

    if selected_network != Network::Main {
        spec::set_current_network(selected_network);
    }
    let args = filtered;

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
        "getpeerinfo" => rpc::call_and_print("getpeerinfo", vec![]),
        "getcheckpoints" => rpc::call_and_print("getcheckpoints", vec![]),
        "getblocktemplate" => rpc::call_and_print("getblocktemplate", vec![]),
        "getblockcount" => rpc::call_and_print("getblockcount", vec![]),
        "getbestblockhash" => rpc::call_and_print("getbestblockhash", vec![]),
        "verifychain" => rpc::call_and_print("verifychain", vec![]),
        "validatemempool" => rpc::call_and_print("validatemempool", vec![]),
        "getgenesishash" => rpc::call_and_print("getgenesishash", vec![]),
        "getgenesis" => rpc::call_and_print("getgenesis", vec![]),
        "getdifficulty" => rpc::call_and_print("getdifficulty", vec![]),
        "getchaintips" => rpc::call_and_print("getchaintips", vec![]),
        "getlistwallet" => rpc::call_and_print("getlistwallet", vec![]),
        "getaddressinfo" => {
            if args.len() < 3 {
                println!("an address argument is required");
                println!("usage: zenachain getaddressinfo \"<address>\"");
                return;
            }
            rpc::call_and_print("getaddressinfo", vec![json!(args[2])]);
        }
        "setlabel" => {
            if args.len() < 4 {
                println!("usage: zenachain setlabel \"<address>\" \"<label>\"");
                return;
            }
            rpc::call_and_print("setlabel", vec![json!(args[2]), json!(args[3])]);
        }
        "backupwallet" => {
            if args.len() < 3 {
                println!("usage: zenachain backupwallet \"<destination_path>\"");
                return;
            }
            rpc::call_and_print("backupwallet", vec![json!(args[2])]);
        }
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
        "encryptwallet" => {
            if args.len() < 3 {
                println!("usage: zenachain encryptwallet \"<password>\"");
                return;
            }
            rpc::call_and_print("encryptwallet", vec![json!(args[2])]);
        }
        "walletpassphrase" => {
            if args.len() < 3 {
                println!("usage: zenachain walletpassphrase \"<password>\"");
                return;
            }
            rpc::call_and_print("walletpassphrase", vec![json!(args[2])]);
        }
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
        "getblockheader" => {
            if args.len() < 3 {
                println!("usage: zenachain getblockheader \"<hash>\"");
                return;
            }
            rpc::call_and_print("getblockheader", vec![json!(args[2])]);
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
        "decoderawtransaction" => {
            if args.len() < 3 {
                println!("usage: zenachain decoderawtransaction \"<txid>\"");
                return;
            }
            rpc::call_and_print("decoderawtransaction", vec![json!(args[2])]);
        }
        "testmempoolaccept" => {
            if args.len() < 3 {
                println!("usage: zenachain testmempoolaccept \"<txid>\"");
                return;
            }
            rpc::call_and_print("testmempoolaccept", vec![json!(args[2])]);
        }
        "getmempoolinfo" => rpc::call_and_print("getmempoolinfo", vec![]),
        "getnonce" => {
            let params = if args.len() >= 3 {
                vec![json!(args[2])]
            } else {
                vec![]
            };
            rpc::call_and_print("getnonce", params);
        }
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
            let mut params: Vec<serde_json::Value> = vec![json!(args[2]), json!(args[3])];
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
        // --- NEW COMMANDS FOR REORG TESTING ---
        "submitblock" => {
            if args.len() < 3 {
                println!("usage: zenachain submitblock <block_json_file>");
                return;
            }
            let path = &args[2];
            let data = match std::fs::read_to_string(path) {
                Ok(d) => d,
                Err(e) => {
                    println!("failed to read file {}: {}", path, e);
                    return;
                }
            };
            let block: serde_json::Value = match serde_json::from_str(&data) {
                Ok(b) => b,
                Err(e) => {
                    println!("failed to parse block JSON: {}", e);
                    return;
                }
            };
            rpc::call_and_print("submitblock", vec![block]);
        }
        "connect" => {
            if args.len() < 3 {
                println!("usage: zenachain connect <addr>");
                return;
            }
            // This is a P2P command, but for now we can just print it.
            // A full implementation would send a command to the network loop.
            println!("connect command is not yet implemented in CLI. Use P2P bootstrap instead.");
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
    println!("  zenachain [-testnet|-regtest] <command> [args]");
    println!();
    println!("global:");
    println!("  -testnet, --testnet                    use the testnet parameters");
    println!("  -regtest, --regtest                    use the regtest parameters");
    println!();
    println!("commands:");
    println!("  help                                display this help message");
    println!("  stop                                save the chain and stop the node");
    println!("  getinfo                             network information");
    println!("  getblockchaininfo                   blockchain information");
    println!("  getnetworkinfo                      network parameters");
    println!("  getpeerinfo                         list connected peers");
    println!("  getcheckpoints                      list of active checkpoints");
    println!("  getblocktemplate                    block template for miners");
    println!("  getblockcount                       number of blocks in the chain");
    println!("  getbestblockhash                    hash of the latest block");
    println!("  getdifficulty                       current proof-of-work difficulty");
    println!("  getchaintips                        information about chain tips");
    println!("  verifychain                         verify the integrity of the entire chain");
    println!("  validatemempool                     validate every transaction in the mempool");
    println!("  getgenesishash                      genesis block hash");
    println!("  getgenesis                          genesis block details");
    println!("  getblockhash <height>               hash of the block at the given height");
    println!("  getblockheader \"<hash>\"             block header details for the given hash");
    println!("  getblock \"<hash>\"                   block details for the given hash");
    println!("  getrawtransaction \"<txid>\"          transaction details for the given txid");
    println!("  decoderawtransaction \"<txid>\"       decode a transaction in a readable format");
    println!("  testmempoolaccept \"<txid>\"          test whether a transaction would be accepted into the mempool");
    println!("  getmempoolinfo                      mempool summary");
    println!("  getnonce [address]                  next expected nonce for an address");
    println!("  getlistwallet                       list all addresses in the wallet");
    println!("  getaddressinfo \"<address>\"          get detailed information for an address");
    println!("  setlabel \"<address>\" \"<label>\"      set a label for a wallet address");
    println!("  backupwallet \"<destination>\"        backup the wallet.dat file");
    println!("  getnewwallet                        create a new address in the wallet");
    println!("  encryptwallet \"<password>\"          encrypt the wallet with a password");
    println!("  walletpassphrase \"<password>\"       unlock the wallet (for testing)");
    println!("  getaddtowallet                      use the default wallet address as the mining target");
    println!("  getaddtoaddress \"<address>\"         set a manual mining address");
    println!("  generatetoaddress <n> \"<address>\"   mine n blocks directly to the given address");
    println!("  sendtoaddress <amount> \"<address>\" [\"<from>\"] send coins from the wallet to the destination");
    println!("  submitblock <block_json_file>       submit a block from a JSON file");
    println!("  dumpprivkey \"<address>\"             display the private key for the given address");
    println!("  importprivkey \"<privkey>\"           import a private key into the wallet");
    println!("  getbalance [address]                compute the wallet balance");
    println!();
    println!("daemon:");
    println!("  zenachaind [-testnet|-regtest]       run the node (without mining)");
    println!("  zenachain-miner                     run the standalone miner");
}
