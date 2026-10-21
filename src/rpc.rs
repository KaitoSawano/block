// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! RPC server and client for ZenaChain.

use crate::blockchain::{self, Blockchain};
use crate::config::Config;
use crate::consensus;
use crate::consensus::spec::Network;
use crate::datadir;
use crate::p2p::network::{NetworkCommand, PeerRegistry};
use crate::pow;
use crate::storage;
use crate::transaction::Transaction;
use crate::utils::{format_datetime, now_millis};
use crate::validation;
use crate::wallet::Wallet;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};

pub const DEFAULT_RPC_ADDR: &str = "127.0.0.1:8332";
pub const DEFAULT_FEE_SATOSHI: u64 = 100_000;

const SATOSHI_PER_ZENAC: u64 = 100_000_000;

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcRequest {
    pub method: String,
    pub params: Vec<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcResponse {
    pub ok: bool,
    pub result: Value,
    pub error: String,
}

impl RpcResponse {
    pub fn ok(result: Value) -> Self {
        RpcResponse {
            ok: true,
            result,
            error: String::new(),
        }
    }

    pub fn err(msg: &str) -> Self {
        RpcResponse {
            ok: false,
            result: Value::Null,
            error: msg.to_string(),
        }
    }
}

fn get_or_create_cookie() -> std::io::Result<String> {
    let path = datadir::cookie_path();
    if let Ok(data) = std::fs::read_to_string(&path) {
        let trimmed = data.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(trimmed);
        }
    }

    let random = crate::utils::random_hex(32);
    let cookie = format!("__cookie__:{}", random);

    datadir::ensure_dirs()?;
    std::fs::write(&path, &cookie)?;
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&path)?.permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(&path, perms)?;
    }

    Ok(cookie)
}

fn read_cookie_for_client() -> std::io::Result<String> {
    let path = datadir::cookie_path();
    let data = std::fs::read_to_string(&path)?;
    Ok(data.trim().to_string())
}

pub fn call(method: &str, params: Vec<Value>) -> std::io::Result<RpcResponse> {
    let cfg = Config::load_or_default();
    call_at(&cfg, method, params)
}

pub fn call_at(cfg: &Config, method: &str, params: Vec<Value>) -> std::io::Result<RpcResponse> {
    let mut stream = TcpStream::connect(&cfg.rpc_addr)?;

    let auth = match read_cookie_for_client() {
        Ok(c) => c,
        Err(_) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Cookie file not found. Is the node running?",
            ))
        }
    };

    let req = RpcRequest {
        method: method.to_string(),
        params,
    };
    let body = serde_json::to_string(&req).unwrap();

    let request = format!(
        "POST / HTTP/1.1\r\nHost: {}\r\nAuthorization: {}\r\nContent-Length: {}\r\n\r\n{}",
        cfg.rpc_addr,
        auth,
        body.len(),
        body
    );

    stream.write_all(request.as_bytes())?;
    stream.flush()?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    loop {
        line.clear();
        reader.read_line(&mut line)?;
        if line == "\r\n" || line.is_empty() {
            break;
        }
    }

    let mut buf = String::new();
    reader.read_line(&mut buf)?;

    let resp: RpcResponse = serde_json::from_str(buf.trim()).map_err(std::io::Error::other)?;
    Ok(resp)
}

fn to_coin(satoshi: u64) -> String {
    let whole = satoshi / SATOSHI_PER_ZENAC;
    let frac = satoshi % SATOSHI_PER_ZENAC;
    format!("{}.{:08}", whole, frac)
}

fn parse_zenac_to_satoshi(s: &str) -> Result<u64, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("empty amount".into());
    }
    if s.starts_with('-') {
        return Err("amount must not be negative".into());
    }

    let (whole_str, frac_str) = match s.split_once('.') {
        Some((w, f)) => (w, f),
        None => (s, ""),
    };

    if whole_str.is_empty() && frac_str.is_empty() {
        return Err("empty amount".into());
    }

    let whole: u64 = if whole_str.is_empty() {
        0
    } else {
        whole_str
            .parse()
            .map_err(|_| format!("invalid whole part: {}", whole_str))?
    };

    if frac_str.len() > 8 {
        return Err(format!(
            "too many fractional digits: {} (maximum is 8)",
            frac_str.len()
        ));
    }

    let mut frac_padded = frac_str.to_string();
    while frac_padded.len() < 8 {
        frac_padded.push('0');
    }
    let frac: u64 = if frac_padded.is_empty() {
        0
    } else {
        frac_padded
            .parse()
            .map_err(|_| format!("invalid fractional part: {}", frac_str))?
    };

    let whole_sat = whole
        .checked_mul(SATOSHI_PER_ZENAC)
        .ok_or_else(|| "amount overflows u64".to_string())?;
    let total = whole_sat
        .checked_add(frac)
        .ok_or_else(|| "amount overflows u64".to_string())?;

    Ok(total)
}

struct RpcState {
    chain: Arc<RwLock<Blockchain>>,
    network: Network,
    shutdown: Arc<AtomicBool>,
    interrupt: Arc<AtomicBool>,
    cookie: String,
    wallet_password: Arc<RwLock<Option<String>>>,
    peer_registry: PeerRegistry,
    network_cmd_tx: mpsc::Sender<NetworkCommand>,
}

impl RpcState {
    fn fingerprint(&self) -> String {
        consensus::consensus_fingerprint_for(self.network)
    }
}

pub struct RpcServer {
    state: Arc<RpcState>,
    rpc_addr: String,
}

impl RpcServer {
    pub fn new(
        chain: Arc<RwLock<Blockchain>>,
        cfg: &Config,
        peer_registry: PeerRegistry,
        network_cmd_tx: mpsc::Sender<NetworkCommand>,
        network: Network,
    ) -> Self {
        let is_local = cfg.rpc_addr.starts_with("127.0.0.1") || cfg.rpc_addr.starts_with("localhost");
        if !is_local {
            panic!(
                "SECURITY ERROR: RPC is bound to {} but cookie authentication is only safe for local connections. Use an SSH tunnel or a reverse proxy with TLS for remote access.",
                cfg.rpc_addr
            );
        }

        let cookie = match get_or_create_cookie() {
            Ok(c) => c,
            Err(e) => panic!("Failed to create RPC cookie: {}", e),
        };

        let state = Arc::new(RpcState {
            chain,
            network,
            shutdown: Arc::new(AtomicBool::new(false)),
            interrupt: Arc::new(AtomicBool::new(false)),
            cookie,
            wallet_password: Arc::new(RwLock::new(None)),
            peer_registry,
            network_cmd_tx,
        });
        RpcServer {
            state,
            rpc_addr: cfg.rpc_addr.clone(),
        }
    }

    pub fn shutdown_flag(&self) -> Arc<AtomicBool> {
        self.state.shutdown.clone()
    }

    pub fn listen(&mut self) -> std::io::Result<()> {
        let listener = TcpListener::bind(&self.rpc_addr)?;
        let _ = listener.set_nonblocking(true);
        println!("[rpc] listening on {}", self.rpc_addr);
        println!("[rpc] authentication: cookie file at {}", datadir::as_str(&datadir::cookie_path()));

        loop {
            if self.state.shutdown.load(Ordering::SeqCst) {
                break;
            }

            match listener.accept() {
                Ok((stream, _)) => {
                    let state = self.state.clone();
                    std::thread::spawn(move || {
                        let mut handler = RpcHandler { state };
                        if let Err(e) = handler.handle(stream) {
                            eprintln!("[rpc] connection error: {}", e);
                        }
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(e) => eprintln!("[rpc] accept failed: {}", e),
            }
        }
        Ok(())
    }
}

struct RpcHandler {
    state: Arc<RpcState>,
}

impl RpcHandler {
    fn handle(&mut self, stream: TcpStream) -> std::io::Result<()> {
        let mut reader = BufReader::new(stream.try_clone()?);
        let mut line = String::new();

        let mut auth_header = String::new();
        let mut content_length = 0;

        loop {
            line.clear();
            reader.read_line(&mut line)?;
            if line == "\r\n" || line.is_empty() {
                break;
            }
            if line.starts_with("Authorization:") {
                auth_header = line.trim_start_matches("Authorization:").trim().to_string();
            }
            if line.starts_with("Content-Length:") {
                content_length = line
                    .trim_start_matches("Content-Length:")
                    .trim()
                    .parse()
                    .unwrap_or(0);
            }
        }

        if auth_header != self.state.cookie {
            let resp = RpcResponse::err("RPC authentication failed");
            write_http_response(stream, 401, &resp)?;
            return Ok(());
        }

        let mut buf = vec![0; content_length];
        reader.read_exact(&mut buf)?;
        let body = String::from_utf8_lossy(&buf);

        let req: RpcRequest = match serde_json::from_str(&body) {
            Ok(r) => r,
            Err(_) => {
                let resp = RpcResponse::err("invalid request");
                write_http_response(stream, 400, &resp)?;
                return Ok(());
            }
        };

        let resp = self.dispatch(&req);
        write_http_response(stream, 200, &resp)
    }

    fn dispatch(&mut self, req: &RpcRequest) -> RpcResponse {
        match req.method.as_str() {
            "getinfo" => self.m_getinfo(),
            "getblockchaininfo" => self.m_getblockchaininfo(),
            "getnetworkinfo" => self.m_getnetworkinfo(),
            "getpeerinfo" => self.m_getpeerinfo(),
            "getcheckpoints" => m_getcheckpoints(),
            "getblocktemplate" => self.m_getblocktemplate(),
            "submitblock" => self.m_submitblock(req),
            "getbalance" => self.m_getbalance(req),
            "getblockhash" => self.m_getblockhash(req),
            "getblock" => self.m_getblock(req),
            "getblockheader" => self.m_getblockheader(req),
            "getblockcount" => self.m_getblockcount(),
            "getbestblockhash" => self.m_getbestblockhash(),
            "getdifficulty" => self.m_getdifficulty(),
            "getchaintips" => self.m_getchaintips(),
            "verifychain" => self.m_verifychain(),
            "getgenesishash" => self.m_getgenesishash(),
            "getgenesis" => self.m_getgenesis(),
            "getlistwallet" => m_getlistwallet(self.state.chain.clone()),
            "dumpprivkey" => m_dumpprivkey(req),
            "importprivkey" => m_importprivkey(req),
            "getnewwallet" => self.m_getnewwallet(),
            "encryptwallet" => m_encryptwallet(req),
            "walletpassphrase" => self.m_walletpassphrase(req),
            "getaddressinfo" => m_getaddressinfo(req),
            "setlabel" => m_setlabel(req),
            "backupwallet" => m_backupwallet(req),
            "getaddtowallet" => m_getaddtowallet(),
            "getaddtoaddress" => m_getaddtoaddress(req),
            "generatetoaddress" => self.m_generatetoaddress(req),
            "sendtoaddress" => self.m_sendtoaddress(req),
            "stop" => self.m_stop(),
            "getrawtransaction" => self.m_getrawtransaction(req),
            "decoderawtransaction" => self.m_decoderawtransaction(req),
            "testmempoolaccept" => self.m_testmempoolaccept(req),
            "getmempoolinfo" => self.m_getmempoolinfo(),
            "validatemempool" => self.m_validatemempool(),
            "getnonce" => self.m_getnonce(req),
            _ => RpcResponse::err("unknown method"),
        }
    }

    fn m_getnetworkinfo(&self) -> RpcResponse {
        let reg = self.state.peer_registry.lock().unwrap();
        let connected = reg.iter().filter(|p| p.state == "Connected").count();
        RpcResponse::ok(json!({
            "network": self.state.network.name(),
            "magic": self.state.network.magic_hex(),
            "default_port": self.state.network.default_port(),
            "prune_after_height": blockchain::network::PRUNE_AFTER_HEIGHT,
            "assumed_blockchain_size_mb": blockchain::network::ASSUMED_BLOCKCHAIN_SIZE,
            "assumed_chain_state_size_mb": blockchain::network::ASSUMED_CHAIN_STATE_SIZE,
            "p2p_enabled": true,
            "connection_count": connected,
            "min_relay_fee_per_byte": blockchain::MIN_RELAY_FEE_PER_BYTE,
        }))
    }

    fn m_getpeerinfo(&self) -> RpcResponse {
        let reg = self.state.peer_registry.lock().unwrap();
        let peers: Vec<Value> = reg.iter().map(|p| json!({
            "id": p.id,
            "addr": p.addr,
            "outbound": p.outbound,
            "state": p.state,
        })).collect();
        RpcResponse::ok(json!(peers))
    }

    fn m_stop(&mut self) -> RpcResponse {
        println!();
        println!("[node] Stopping the running node. Please wait...");
        println!("[node] Saving blockchain data...");

        self.state.interrupt.store(true, Ordering::SeqCst);
        self.state.shutdown.store(true, Ordering::SeqCst);

        let chain = self.state.chain.blocking_read();
        match storage::save_chain(&chain) {
            Ok(_) => {
                println!("[node]   blocks     : {}", chain.chain.len());
                println!(
                    "[node]   chainstate : height={} difficulty/nbits={:#010x}",
                    chain.height(),
                    chain.nbits
                );
                println!("[node]   mempool    : {} transactions", chain.mempool.len());
                println!("[node]   location   : {}", datadir::as_str(&datadir::datadir()));
                println!("[node] Blockchain data saved successfully.");
            }
            Err(e) => {
                println!("[node] FAILED to save blockchain: {}", e);
                return RpcResponse::err(&format!("failed to save chain: {}", e));
            }
        }
        drop(chain);

        println!("[node] Closing P2P and RPC connections...");
        println!("[node] Shutdown successful.");
        println!();

        RpcResponse::ok(json!({
            "stopping": true,
            "message": "Stopping the running node. Please wait...",
            "saved": true,
            "status": "Shutdown successful",
        }))
    }

    fn m_getinfo(&self) -> RpcResponse {
        let c = self.state.chain.blocking_read();
        RpcResponse::ok(json!({
            "coin_name": consensus::coin_name(),
            "symbol": consensus::symbol(),
            "version": consensus::version(),
            "protocol_version": consensus::protocol_version(),
            "network": self.state.network.name(),
            "hash_algo": consensus::hash_algo(),
            "signature_algo": consensus::signature_algo(),
            "height": c.height(),
            "chain_len": c.chain.len(),
            "nbits": format!("{:#010x}", c.nbits),
            "difficulty": pow::nbits_to_difficulty(c.nbits),
            "pow_limit": consensus::pow_limit(),
            "circulating": to_coin(c.circulating_supply()),
            "max_supply": to_coin(consensus::max_supply()),
            "era": c.current_era(),
            "subsidy_halving_interval": consensus::subsidy_halving_interval(),
            "blocks_until_halving": c.blocks_until_halving(),
            "pow_target_spacing_secs": consensus::pow_target_spacing_secs(),
            "pow_target_timespan_secs": consensus::pow_target_timespan_secs(),
            "miner_confirmation_window": consensus::miner_confirmation_window(),
            "difficulty_adjust_interval": consensus::difficulty_adjust_interval(),
            "mempool_len": c.mempool.len(),
            "mempool_size_bytes": c.mempool_size_bytes(),
            "min_relay_fee_per_byte": blockchain::MIN_RELAY_FEE_PER_BYTE,
            "valid": c.is_valid(),
            "checkpoints_count": blockchain::checkpoint::CHECKPOINTS.len(),
            "checkpoints_highest": blockchain::checkpoint::highest(),
            "consensus_fingerprint": self.state.fingerprint(),
            "datadir": datadir::as_str(&datadir::datadir()),
            "total_chainwork": c.total_chainwork(),
        }))
    }

    fn m_getblockchaininfo(&self) -> RpcResponse {
        let c = self.state.chain.blocking_read();
        let tip = c.chain.last().unwrap();
        RpcResponse::ok(json!({
            "chain": self.state.network.name(),
            "blocks": c.height(),
            "headers": c.height(),
            "bestblockhash": tip.hash,
            "nbits": format!("{:#010x}", c.nbits),
            "difficulty": pow::nbits_to_difficulty(c.nbits),
            "pow_limit": consensus::pow_limit(),
            "mediantime": tip.timestamp,
            "verificationprogress": 1.0,
            "initialblockdownload": false,
            "pruned": false,
            "checkpoints_count": blockchain::checkpoint::CHECKPOINTS.len(),
            "checkpoints_highest": blockchain::checkpoint::highest(),
            "mempool_len": c.mempool.len(),
            "mempool_size_bytes": c.mempool_size_bytes(),
            "min_relay_fee_per_byte": blockchain::MIN_RELAY_FEE_PER_BYTE,
            "circulating": to_coin(c.circulating_supply()),
            "max_supply": to_coin(consensus::max_supply()),
            "era": c.current_era(),
            "subsidy_halving_interval": consensus::subsidy_halving_interval(),
            "blocks_until_halving": c.blocks_until_halving(),
            "difficulty_adjust_interval": consensus::difficulty_adjust_interval(),
            "consensus_fingerprint": self.state.fingerprint(),
            "datadir": datadir::as_str(&datadir::datadir()),
            "total_chainwork": c.total_chainwork(),
        }))
    }

    fn m_getblocktemplate(&self) -> RpcResponse {
        let c = self.state.chain.blocking_read();
        let next_height = c.height() + 1;
        let subsidy = Blockchain::calculate_reward(next_height);
        let prev = c.chain.last().unwrap();
        let prev_hash = prev.hash.clone();
        let previous_chainwork = prev.chainwork;
        let total_fees: u64 = c.mempool.iter().map(|tx| tx.fee).sum();
        let reward = subsidy.saturating_add(total_fees);
        RpcResponse::ok(json!({
            "height": next_height,
            "previous_hash": prev_hash,
            "previous_chainwork": previous_chainwork,
            "nbits": c.nbits,
            "reward": to_coin(reward),
            "subsidy": to_coin(subsidy),
            "total_fees": to_coin(total_fees),
            "reward_satoshi": reward,
            "subsidy_satoshi": subsidy,
            "total_fees_satoshi": total_fees,
            "mempool": c.mempool,
            "min_relay_fee_per_byte": blockchain::MIN_RELAY_FEE_PER_BYTE,
        }))
    }

    fn m_submitblock(&mut self, req: &RpcRequest) -> RpcResponse {
        if req.params.is_empty() {
            return RpcResponse::err("submitblock requires a block parameter");
        }
        let block: crate::block::Block = match serde_json::from_value(req.params[0].clone()) {
            Ok(b) => b,
            Err(e) => return RpcResponse::err(&format!("invalid block: {}", e)),
        };

        let mut chain = self.state.chain.blocking_write();
        let prev = chain.chain.last().unwrap().clone();

        if let Err(e) = validation::validate_block(&chain, &prev, &block) {
            return RpcResponse::err(&format!("block rejected: {}", e));
        }

        for tx in &block.transactions {
            if let Err(e) = chain.apply_transaction(tx) {
                return RpcResponse::err(&format!("failed to apply tx: {}", e));
            }
        }
        if let Err(e) = chain.credit_reward(&block.mined_for, block.reward, block.index) {
            return RpcResponse::err(&format!("failed to credit reward: {}", e));
        }

        chain.chain.push(block.clone());
        chain.adjust_difficulty();
        chain.mempool.clear();
        chain.mempool_timestamps.clear();

        block.log_add_to_wallet();

        if let Err(e) = storage::save_chain(&chain) {
            return RpcResponse::err(&format!("failed to save chain: {}", e));
        }

        drop(chain);

        let tx = self.state.network_cmd_tx.clone();
        let hash = block.hash.clone();
        let _ = tx.blocking_send(NetworkCommand::BroadcastBlock(hash));

        RpcResponse::ok(json!({
            "accepted": true,
            "height": block.index,
            "hash": block.hash,
            "txid": block.coinbase_txid,
            "merkle_root": block.merkle_root,
            "mined_at": block.mined_at,
            "mined_for": block.mined_for,
            "nbits": format!("{:#010x}", block.nbits),
        }))
    }

    fn m_getbalance(&self, req: &RpcRequest) -> RpcResponse {
        if let Some(addr) = req.params.first().and_then(|v| v.as_str()) {
            if !validation::is_valid_address(addr) {
                return RpcResponse::err("invalid address format");
            }
            let chain = self.state.chain.blocking_read();
            let balance = chain.balance_of(addr);
            let spendable = chain.spendable_balance_of(addr);
            let nonce = chain.nonce_of(addr);
            return RpcResponse::ok(json!({
                "address": addr,
                "balance": to_coin(balance),
                "balance_satoshi": balance,
                "spendable_balance": to_coin(spendable),
                "spendable_balance_satoshi": spendable,
                "nonce": nonce,
            }));
        }

        let w = match Wallet::load_default() {
            Ok(w) => w,
            Err(_) => return RpcResponse::err("wallet not found"),
        };
        let chain = self.state.chain.blocking_read();
        let mut total: u64 = 0;
        for entry in &w.entries {
            total = total.saturating_add(chain.balance_of(&entry.address));
        }
        RpcResponse::ok(json!({
            "wallet_total_balance": to_coin(total),
            "wallet_total_balance_satoshi": total,
            "addresses": w.entries.len(),
            "blockchain_height": chain.height(),
        }))
    }

    fn m_getnonce(&self, req: &RpcRequest) -> RpcResponse {
        let addr = match req.params.first().and_then(|v| v.as_str()) {
            Some(a) => a.to_string(),
            None => match Wallet::load_default() {
                Ok(w) => match w.default_address() {
                    Some(e) => e.address.clone(),
                    None => return RpcResponse::err("wallet has no addresses"),
                },
                Err(_) => return RpcResponse::err("wallet not found"),
            },
        };

        if !validation::is_valid_address(&addr) {
            return RpcResponse::err("invalid address format");
        }

        let chain = self.state.chain.blocking_read();
        RpcResponse::ok(json!({
            "address": addr,
            "nonce": chain.nonce_of(&addr),
            "next_nonce": chain.next_nonce_for(&addr),
        }))
    }

    fn m_getblockhash(&self, req: &RpcRequest) -> RpcResponse {
        let height = match req.params.first().and_then(|v| v.as_u64()) {
            Some(h) => h,
            None => return RpcResponse::err("getblockhash requires a height"),
        };
        let chain = self.state.chain.blocking_read();
        if height as usize >= chain.chain.len() {
            return RpcResponse::err("height out of range");
        }
        RpcResponse::ok(json!(chain.chain[height as usize].hash))
    }

    fn m_getblock(&self, req: &RpcRequest) -> RpcResponse {
        let hash = match req.params.first().and_then(|v| v.as_str()) {
            Some(h) => h,
            None => return RpcResponse::err("getblock requires a hash"),
        };
        let chain = self.state.chain.blocking_read();
        let block = match chain.chain.iter().find(|b| b.hash == hash) {
            Some(b) => b,
            None => return RpcResponse::err("block not found"),
        };
        RpcResponse::ok(json!({
            "nversion": block.nversion,
            "hash": block.hash,
            "height": block.index,
            "previous_hash": block.previous_hash,
            "merkle_root": block.merkle_root,
            "state_root": block.state_root,
            "timestamp": block.timestamp,
            "mined_at": block.mined_at,
            "mined_for": block.mined_for,
            "nonce": block.nonce,
            "nbits": format!("{:#010x}", block.nbits),
            "reward": to_coin(block.reward),
            "subsidy": to_coin(block.subsidy),
            "total_fees": to_coin(block.total_fees),
            "coinbase_txid": block.coinbase_txid,
            "blockchain_size": block.blockchain_size,
            "genesis_output_script": block.genesis_output_script,
            "miner_pubkey": block.miner_pubkey,
            "miner_signature": block.miner_signature,
            "chainwork": block.chainwork,
            "transactions": block.transactions,
        }))
    }

    fn m_getblockheader(&self, req: &RpcRequest) -> RpcResponse {
        let hash = match req.params.first().and_then(|v| v.as_str()) {
            Some(h) => h,
            None => return RpcResponse::err("getblockheader requires a hash"),
        };

        let chain = self.state.chain.blocking_read();
        let block = match chain.chain.iter().find(|b| b.hash == hash) {
            Some(b) => b,
            None => return RpcResponse::err("block not found"),
        };

        RpcResponse::ok(json!({
            "hash": block.hash,
            "confirmations": chain.height() - block.index + 1,
            "height": block.index,
            "version": block.nversion,
            "merkleroot": block.merkle_root,
            "state_root": block.state_root,
            "time": block.timestamp,
            "mediantime": block.timestamp,
            "nonce": block.nonce,
            "bits": format!("{:#010x}", block.nbits),
            "previousblockhash": block.previous_hash,
            "chainwork": block.chainwork,
            "nextblockhash": if block.index + 1 < chain.chain.len() as u64 {
                Some(chain.chain[(block.index + 1) as usize].hash.clone())
            } else {
                None
            }
        }))
    }

    fn m_getblockcount(&self) -> RpcResponse {
        let chain = self.state.chain.blocking_read();
        RpcResponse::ok(json!(chain.height()))
    }

    fn m_getbestblockhash(&self) -> RpcResponse {
        let chain = self.state.chain.blocking_read();
        RpcResponse::ok(json!(chain.chain.last().unwrap().hash))
    }

    fn m_getdifficulty(&self) -> RpcResponse {
        let chain = self.state.chain.blocking_read();
        let diff = pow::nbits_to_difficulty(chain.nbits);
        RpcResponse::ok(json!(diff))
    }

    fn m_getchaintips(&self) -> RpcResponse {
        let chain = self.state.chain.blocking_read();
        let tip = chain.chain.last().unwrap();

        RpcResponse::ok(json!([{
            "height": chain.height(),
            "hash": tip.hash,
            "branchlen": 0,
            "status": "active",
            "chainwork": tip.chainwork,
        }]))
    }

    fn m_getgenesishash(&self) -> RpcResponse {
        let chain = self.state.chain.blocking_read();
        match chain.chain.first() {
            Some(g) => RpcResponse::ok(json!(g.hash)),
            None => RpcResponse::err("chain is empty"),
        }
    }

    fn m_getgenesis(&self) -> RpcResponse {
        let chain = self.state.chain.blocking_read();
        match chain.chain.first() {
            Some(g) => RpcResponse::ok(json!({
                "nversion": g.nversion,
                "hash": g.hash,
                "index": g.index,
                "timestamp": g.timestamp,
                "mined_at": g.mined_at,
                "previous_hash": g.previous_hash,
                "merkle_root": g.merkle_root,
                "state_root": g.state_root,
                "nonce": g.nonce,
                "nbits": format!("{:#010x}", g.nbits),
                "reward": to_coin(g.reward),
                "subsidy": to_coin(g.subsidy),
                "total_fees": to_coin(g.total_fees),
                "coinbase_txid": g.coinbase_txid,
                "blockchain_size": g.blockchain_size,
                "genesis_output_script": g.genesis_output_script,
                "network": self.state.network.name(),
                "consensus_fingerprint": self.state.fingerprint(),
                "transactions": g.transactions,
                "chainwork": g.chainwork,
            })),
            None => RpcResponse::err("chain is empty"),
        }
    }

    fn m_verifychain(&self) -> RpcResponse {
        let chain = self.state.chain.blocking_read();
        match validation::validate_chain(&chain) {
            Ok(_) => RpcResponse::ok(json!({
                "valid": true,
                "blocks_checked": chain.chain.len(),
                "height": chain.height(),
                "nbits": format!("{:#010x}", chain.nbits),
                "pow_limit": consensus::pow_limit(),
                "signature_algo": consensus::signature_algo(),
                "checkpoints_count": blockchain::checkpoint::CHECKPOINTS.len(),
                "consensus_fingerprint": self.state.fingerprint(),
            })),
            Err(e) => RpcResponse::ok(json!({
                "valid": false,
                "code": e.code,
                "message": e.message,
                "blocks_checked": chain.chain.len(),
                "height": chain.height(),
            })),
        }
    }

    fn m_getrawtransaction(&self, req: &RpcRequest) -> RpcResponse {
        let txid = match req.params.first().and_then(|v| v.as_str()) {
            Some(t) => t.to_string(),
            None => return RpcResponse::err("getrawtransaction requires a txid"),
        };

        let chain = self.state.chain.blocking_read();
        for block in &chain.chain {
            for tx in &block.transactions {
                if tx.txid == txid {
                    return RpcResponse::ok(json!({
                        "txid": tx.txid,
                        "from": tx.from,
                        "to": tx.to,
                        "amount_satoshi": tx.amount,
                        "amount_zenac": to_coin(tx.amount),
                        "fee_satoshi": tx.fee,
                        "fee_zenac": to_coin(tx.fee),
                        "fee_rate": blockchain::Blockchain::fee_rate(tx),
                        "nonce": tx.nonce,
                        "note": tx.note,
                        "public_key": tx.public_key,
                        "signature": tx.signature,
                        "output_script": tx.output_script,
                        "in_mempool": false,
                        "block_hash": block.hash,
                        "block_height": block.index,
                        "block_mined_at": block.mined_at,
                    }));
                }
            }
            if block.coinbase_txid == txid {
                return RpcResponse::ok(json!({
                    "txid": block.coinbase_txid,
                    "from": "COINBASE",
                    "to": block.mined_for,
                    "amount_satoshi": block.reward,
                    "amount_zenac": to_coin(block.reward),
                    "subsidy_satoshi": block.subsidy,
                    "subsidy_zenac": to_coin(block.subsidy),
                    "total_fees_satoshi": block.total_fees,
                    "total_fees_zenac": to_coin(block.total_fees),
                    "fee_satoshi": 0,
                    "in_mempool": false,
                    "block_hash": block.hash,
                    "block_height": block.index,
                    "block_mined_at": block.mined_at,
                    "is_coinbase": true,
                }));
            }
        }

        for tx in &chain.mempool {
            if tx.txid == txid {
                return RpcResponse::ok(json!({
                    "txid": tx.txid,
                    "from": tx.from,
                    "to": tx.to,
                    "amount_satoshi": tx.amount,
                    "amount_zenac": to_coin(tx.amount),
                    "fee_satoshi": tx.fee,
                    "fee_zenac": to_coin(tx.fee),
                    "fee_rate": blockchain::Blockchain::fee_rate(tx),
                    "nonce": tx.nonce,
                    "note": tx.note,
                    "public_key": tx.public_key,
                    "signature": tx.signature,
                    "output_script": tx.output_script,
                    "in_mempool": true,
                }));
            }
        }

        RpcResponse::err("transaction not found")
    }

    fn m_decoderawtransaction(&self, req: &RpcRequest) -> RpcResponse {
        let txid = match req.params.first().and_then(|v| v.as_str()) {
            Some(t) => t.to_string(),
            None => return RpcResponse::err("decoderawtransaction requires a txid"),
        };

        let chain = self.state.chain.blocking_read();
        for block in &chain.chain {
            for tx in &block.transactions {
                if tx.txid == txid {
                    return RpcResponse::ok(json!({
                        "txid": tx.txid,
                        "from": tx.from,
                        "to": tx.to,
                        "amount": to_coin(tx.amount),
                        "fee": to_coin(tx.fee),
                        "fee_rate": blockchain::Blockchain::fee_rate(tx),
                        "nonce": tx.nonce,
                        "note": tx.note,
                        "public_key": tx.public_key,
                        "signature": tx.signature,
                        "block_hash": block.hash,
                        "block_height": block.index,
                    }));
                }
            }
        }

        for tx in &chain.mempool {
            if tx.txid == txid {
                return RpcResponse::ok(json!({
                    "txid": tx.txid,
                    "from": tx.from,
                    "to": tx.to,
                    "amount": to_coin(tx.amount),
                    "fee": to_coin(tx.fee),
                    "fee_rate": blockchain::Blockchain::fee_rate(tx),
                    "nonce": tx.nonce,
                    "note": tx.note,
                    "public_key": tx.public_key,
                    "signature": tx.signature,
                    "in_mempool": true,
                }));
            }
        }

        RpcResponse::err("transaction not found")
    }

    fn m_testmempoolaccept(&self, req: &RpcRequest) -> RpcResponse {
        let txid = match req.params.first().and_then(|v| v.as_str()) {
            Some(t) => t.to_string(),
            None => return RpcResponse::err("testmempoolaccept requires a txid"),
        };

        let chain = self.state.chain.blocking_read();

        let tx = chain.mempool.iter().find(|t| t.txid == txid).cloned();

        if tx.is_none() {
            return RpcResponse::ok(json!([{
                "txid": txid,
                "allowed": false,
                "reject-reason": "transaction not found in mempool"
            }]));
        }

        let tx = tx.unwrap();

        match validation::validate_transaction(&chain, &tx) {
            Ok(_) => {
                let fee_rate = blockchain::Blockchain::fee_rate(&tx);
                RpcResponse::ok(json!([{
                    "txid": txid,
                    "allowed": true,
                    "vsize": blockchain::Blockchain::tx_size_bytes(&tx),
                    "fees": {
                        "base": to_coin(tx.fee)
                    },
                    "fee_rate": fee_rate,
                }]))
            }
            Err(e) => RpcResponse::ok(json!([{
                "txid": txid,
                "allowed": false,
                "reject-reason": e.message
            }])),
        }
    }

    fn m_getmempoolinfo(&self) -> RpcResponse {
        let chain = self.state.chain.blocking_read();
        let txs = &chain.mempool;
        let count = txs.len();
        let mut total_amount: u64 = 0;
        let mut total_fee: u64 = 0;
        for tx in txs {
            total_amount = total_amount.saturating_add(tx.amount);
            total_fee = total_fee.saturating_add(tx.fee);
        }
        let avg_fee = if count > 0 {
            total_fee / count as u64
        } else {
            0
        };
        RpcResponse::ok(json!({
            "size": count,
            "size_bytes": chain.mempool_size_bytes(),
            "max_size_bytes": blockchain::MAX_MEMPOOL_SIZE_BYTES,
            "min_relay_fee_per_byte": blockchain::MIN_RELAY_FEE_PER_BYTE,
            "total_amount_satoshi": total_amount,
            "total_amount_zenac": to_coin(total_amount),
            "total_fee_satoshi": total_fee,
            "total_fee_zenac": to_coin(total_fee),
            "avg_fee_satoshi": avg_fee,
            "avg_fee_zenac": to_coin(avg_fee),
            "default_fee_satoshi": DEFAULT_FEE_SATOSHI,
            "default_fee_zenac": to_coin(DEFAULT_FEE_SATOSHI),
        }))
    }

    fn m_validatemempool(&self) -> RpcResponse {
        let chain = self.state.chain.blocking_read();
        let errors = validation::validate_mempool(&chain);
        let items: Vec<Value> = errors
            .iter()
            .map(|e| json!({"code": e.code, "message": e.message}))
            .collect();
        RpcResponse::ok(json!({
            "valid": items.is_empty(),
            "mempool_len": chain.mempool.len(),
            "errors": items,
        }))
    }

    fn m_generatetoaddress(&mut self, req: &RpcRequest) -> RpcResponse {
        let n = match req.params.first().and_then(|v| v.as_u64()) {
            Some(n) => n,
            None => return RpcResponse::err("generatetoaddress requires a block count"),
        };
        let addr = match req.params.get(1).and_then(|v| v.as_str()) {
            Some(a) => a.to_string(),
            None => return RpcResponse::err("generatetoaddress requires an address"),
        };

        if !validation::is_valid_address(&addr) {
            return RpcResponse::err("invalid address format");
        }

        self.state.interrupt.store(false, Ordering::SeqCst);

        let mut hashes: Vec<String> = Vec::new();

        for _ in 0..n {
            if self.state.interrupt.load(Ordering::SeqCst) {
                println!("[rpc] generatetoaddress interrupted by stop");
                break;
            }

            let (next_index, prev_hash, subsidy, nbits, mempool_txs, previous_chainwork) = {
                let mut chain = self.state.chain.blocking_write();
                let prev = chain.chain.last().unwrap().clone();
                let next_index = prev.index + 1;
                let subsidy = Blockchain::calculate_reward(next_index);

                if chain.circulating_supply().saturating_add(subsidy) > consensus::max_supply() {
                    return RpcResponse::err("max supply reached");
                }

                let txs = std::mem::take(&mut chain.mempool);
                chain.mempool_timestamps.clear();
                (next_index, prev.hash, subsidy, chain.nbits, txs, prev.chainwork)
            };

            let total_fees = mempool_txs.iter().map(|tx| tx.fee).sum::<u64>();

            let mut block = crate::block::Block::new_for(
                next_index,
                prev_hash,
                subsidy,
                total_fees,
                nbits,
                mempool_txs.clone(),
                addr.clone(),
                String::new(),
                previous_chainwork,
            );

            if !pow::mine_interruptible(&mut block, nbits, Some(&self.state.interrupt)) {
                println!("[rpc] mining interrupted at height {}", next_index);
                break;
            }

            {
                let mut chain = self.state.chain.blocking_write();
                let mut applied = 0usize;
                for tx in &mempool_txs {
                    if let Err(e) = chain.apply_transaction(tx) {
                        eprintln!("[rpc] failed to apply tx {}: {}", tx.txid, e);
                        for tx in mempool_txs.iter().skip(applied) {
                            chain.mempool.push(tx.clone());
                        }
                        return RpcResponse::err(&format!("failed to apply tx: {}", e));
                    }
                    applied += 1;
                }
                if let Err(e) = chain.credit_reward(&addr, block.reward, block.index) {
                    return RpcResponse::err(&format!("failed to credit reward: {}", e));
                }

                if let Ok(root) = chain.state.root_hash() {
                    block.state_root = hex::encode(root);
                    block.hash = block.calculate_hash();
                }

                chain.chain.push(block.clone());
                chain.adjust_difficulty();
                block.log_add_to_wallet();
                if let Err(e) = storage::save_chain(&chain) {
                    return RpcResponse::err(&format!("failed to save chain: {}", e));
                }
            }

            let tx = self.state.network_cmd_tx.clone();
            let hash = block.hash.clone();
            let _ = tx.blocking_send(NetworkCommand::BroadcastBlock(hash));

            hashes.push(block.hash.clone());
        }

        RpcResponse::ok(json!(hashes))
    }

    fn m_sendtoaddress(&mut self, req: &RpcRequest) -> RpcResponse {
        if req.params.len() < 2 {
            return RpcResponse::err("sendtoaddress requires an amount and an address");
        }

        let is_sendall = req.params[0].as_str() == Some("sendall");

        let to = match req.params[1].as_str() {
            Some(a) => a.to_string(),
            None => return RpcResponse::err("invalid destination address"),
        };

        if !validation::is_valid_address(&to) {
            return RpcResponse::err("invalid destination address format");
        }

        let mut wallet = match Wallet::load_default() {
            Ok(w) => w,
            Err(_) => return RpcResponse::err("wallet not found"),
        };

        if wallet.encrypted {
            let pass_opt = self.state.wallet_password.blocking_read().clone();
            match pass_opt {
                Some(pass) => match wallet.decrypt(&pass) {
                    Ok(decrypted) => wallet = decrypted,
                    Err(_) => {
                        return RpcResponse::err(
                            "wallet is locked. Run walletpassphrase first.",
                        )
                    }
                },
                None => {
                    return RpcResponse::err(
                        "wallet is locked. Run walletpassphrase first.",
                    )
                }
            }
        }

        let from_address = if req.params.len() >= 3 {
            req.params[2].as_str().unwrap_or("").to_string()
        } else {
            let chain = self.state.chain.blocking_read();
            let mut best: Option<(String, u64)> = None;
            for entry in &wallet.entries {
                if entry.address == to {
                    continue;
                }
                let bal = chain.spendable_balance_of(&entry.address);
                match &best {
                    Some((_, b)) if *b >= bal => {}
                    _ => best = Some((entry.address.clone(), bal)),
                }
            }
            drop(chain);
            match best {
                Some((addr, _)) => addr,
                None => {
                    return RpcResponse::err(
                        "wallet has no addresses other than the destination",
                    )
                }
            }
        };

        if from_address == to {
            return RpcResponse::err("sender and recipient addresses are identical");
        }

        let entry = match wallet.get_entry(&from_address) {
            Some(e) => e.clone(),
            None => return RpcResponse::err("from address not found in wallet"),
        };

        let estimated_tx_size = 250;
        let fee = if let Some(custom_fee) = req.params.get(3).and_then(|v| v.as_u64()) {
            custom_fee
        } else {
            (blockchain::MIN_RELAY_FEE_PER_BYTE * estimated_tx_size).max(DEFAULT_FEE_SATOSHI)
        };

        let amount_sat = if is_sendall {
            let chain = self.state.chain.blocking_read();
            let bal = chain.spendable_balance_of(&entry.address);
            drop(chain);
            if bal <= fee {
                return RpcResponse::err(&format!(
                    "spendable balance {} sat is too small to cover the fee of {} sat",
                    bal, fee
                ));
            }
            bal - fee
        } else {
            match req.params[0].as_str() {
                Some(s) => match parse_zenac_to_satoshi(s) {
                    Ok(v) => v,
                    Err(e) => return RpcResponse::err(&format!("invalid amount: {}", e)),
                },
                None => match req.params[0].as_f64() {
                    Some(v) => {
                        if v <= 0.0 {
                            return RpcResponse::err("amount must be greater than zero");
                        }
                        let s = format!("{:.8}", v);
                        match parse_zenac_to_satoshi(&s) {
                            Ok(v) => v,
                            Err(e) => {
                                return RpcResponse::err(&format!("invalid amount: {}", e))
                            }
                        }
                    }
                    None => return RpcResponse::err("invalid amount"),
                },
            }
        };

        if amount_sat == 0 {
            return RpcResponse::err("amount must be greater than zero");
        }

        if amount_sat > consensus::max_supply() {
            return RpcResponse::err(&format!(
                "amount exceeds the maximum supply of {} satoshi",
                consensus::max_supply()
            ));
        }

        let need = match amount_sat.checked_add(fee) {
            Some(v) => v,
            None => return RpcResponse::err("amount + fee overflows u64"),
        };
        {
            let chain = self.state.chain.blocking_read();
            let bal = chain.spendable_balance_of(&entry.address);
            drop(chain);
            if bal < need {
                let chain = self.state.chain.blocking_read();
                let total: u64 = wallet
                    .entries
                    .iter()
                    .map(|e| chain.spendable_balance_of(&e.address))
                    .sum();
                drop(chain);
                if total < need {
                    return RpcResponse::err(&format!(
                        "insufficient spendable wallet balance: required {} sat (amount {} + fee {}), available {} sat",
                        need, amount_sat, fee, total
                    ));
                }
                return RpcResponse::err(&format!(
                    "selected address {} has insufficient spendable balance: required {} sat, available {} sat; use a different source address",
                    entry.address, need, bal
                ));
            }
        }

        let nonce = if let Some(custom_nonce) = req.params.get(4).and_then(|v| v.as_u64()) {
            custom_nonce
        } else {
            let chain = self.state.chain.blocking_read();
            chain.next_nonce_for(&entry.address)
        };

        let mut tx = Transaction::new(
            &entry.address,
            &to,
            amount_sat,
            fee,
            nonce,
        );

        if let Err(e) = tx.sign(&entry.secret) {
            return RpcResponse::err(&format!("failed to sign: {}", e));
        }

        let mut chain = self.state.chain.blocking_write();
        
        if !chain.add_transaction(tx.clone()) {
            return RpcResponse::err("transaction rejected by mempool (RBF, eviction, or min relay fee)");
        }

        let txid = tx.txid.clone();

        let now = now_millis();
        println!(
            "[mempool] {} Accepted transaction {} from {} to {} amount={} fee={} nonce={}",
            format_datetime(now),
            txid,
            entry.address,
            to,
            to_coin(amount_sat),
            to_coin(fee),
            nonce
        );

        if let Err(e) = storage::save_chain(&chain) {
            return RpcResponse::err(&format!("failed to save chain: {}", e));
        }

        drop(chain);

        let tx = self.state.network_cmd_tx.clone();
        let tid = txid.clone();
        let _ = tx.blocking_send(NetworkCommand::BroadcastTransaction(tid));

        RpcResponse::ok(json!({
            "txid": txid,
            "from": entry.address,
            "to": to,
            "amount_satoshi": amount_sat,
            "amount_zenac": to_coin(amount_sat),
            "fee_satoshi": fee,
            "fee_zenac": to_coin(fee),
            "fee_rate": fee / estimated_tx_size,
            "nonce": nonce,
            "sendall": is_sendall,
            "mempool_len": self.state.chain.blocking_read().mempool.len(),
        }))
    }

    fn m_getnewwallet(&mut self) -> RpcResponse {
        if let Err(e) = datadir::ensure_dirs() {
            return RpcResponse::err(&format!("failed to prepare datadir: {}", e));
        }

        let mut w = Wallet::load_default().unwrap_or_else(|_| Wallet {
            entries: Vec::new(),
            created_at: now_millis(),
            encrypted: false,
            salt: String::new(),
            nonce: String::new(),
        });

        if w.encrypted {
            let pass_opt = self.state.wallet_password.blocking_read().clone();
            match pass_opt {
                Some(pass) => match w.decrypt(&pass) {
                    Ok(decrypted) => w = decrypted,
                    Err(_) => {
                        return RpcResponse::err(
                            "wallet is locked. Run walletpassphrase first.",
                        )
                    }
                },
                None => {
                    return RpcResponse::err(
                        "wallet is locked. Run walletpassphrase first.",
                    )
                }
            }
        }

        let addr = w.add_new_address("default".to_string());
        let entry = w.get_entry(&addr).unwrap().clone();

        match w.save_default() {
            Ok(_) => RpcResponse::ok(json!({
                "address": entry.address,
                "public_key": entry.public_key,
                "secret": entry.secret,
                "created_at": entry.created_at,
            })),
            Err(e) => RpcResponse::err(&format!("failed to save wallet: {}", e)),
        }
    }

    fn m_walletpassphrase(&mut self, req: &RpcRequest) -> RpcResponse {
        let pass = match req.params.first().and_then(|v| v.as_str()) {
            Some(p) => p.to_string(),
            None => return RpcResponse::err("walletpassphrase requires a password"),
        };

        let w = match Wallet::load_default() {
            Ok(w) => w,
            Err(_) => return RpcResponse::err("wallet not found"),
        };

        match w.decrypt(&pass) {
            Ok(_) => {
                let mut lock = self.state.wallet_password.blocking_write();
                *lock = Some(pass);

                RpcResponse::ok(json!({
                    "message": "wallet decrypted successfully",
                    "addresses": w.entries.len(),
                }))
            }
            Err(e) => RpcResponse::err(&format!("failed to decrypt wallet: {}", e)),
        }
    }
}

fn m_getcheckpoints() -> RpcResponse {
    let items: Vec<Value> = blockchain::checkpoint::CHECKPOINTS
        .iter()
        .map(|c| json!({"height": c.height, "hash": c.hash}))
        .collect();
    RpcResponse::ok(json!(items))
}

fn m_getlistwallet(chain_arc: Arc<RwLock<Blockchain>>) -> RpcResponse {
    let w = match Wallet::load_default() {
        Ok(w) => w,
        Err(_) => return RpcResponse::err("wallet not found"),
    };

    let chain = chain_arc.blocking_read();
    let current_height = chain.height();

    let mut addresses = Vec::new();
    let mut total_balance: u64 = 0;

    for entry in &w.entries {
        let balance = chain.balance_of(&entry.address);
        let spendable = chain.spendable_balance_of(&entry.address);
        total_balance = total_balance.saturating_add(balance);

        addresses.push(json!({
            "address": entry.address,
            "public_key": entry.public_key,
            "label": entry.label,
            "created_at": entry.created_at,
            "balance": to_coin(balance),
            "balance_satoshi": balance,
            "spendable_balance": to_coin(spendable),
            "spendable_balance_satoshi": spendable,
            "nonce": chain.nonce_of(&entry.address),
        }));
    }

    RpcResponse::ok(json!({
        "addresses": addresses,
        "count": w.entries.len(),
        "wallet_total_balance": to_coin(total_balance),
        "wallet_total_balance_satoshi": total_balance,
        "blockchain_height": current_height,
    }))
}

fn m_dumpprivkey(req: &RpcRequest) -> RpcResponse {
    let target = match req.params.first().and_then(|v| v.as_str()) {
        Some(a) => a.to_string(),
        None => return RpcResponse::err("dumpprivkey requires an address"),
    };

    let w = match Wallet::load_default() {
        Ok(w) => w,
        Err(_) => return RpcResponse::err("wallet not found"),
    };

    let entry = match w.get_entry(&target) {
        Some(e) => e,
        None => return RpcResponse::err("address not found in wallet"),
    };

    RpcResponse::ok(json!({
        "address": entry.address,
        "privkey": entry.secret,
    }))
}

fn m_importprivkey(req: &RpcRequest) -> RpcResponse {
    let secret = match req.params.first().and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("importprivkey requires a private key"),
    };

    if secret.len() != 64 || !secret.chars().all(|c| c.is_ascii_hexdigit()) {
        return RpcResponse::err("private key must be 64 hexadecimal characters");
    }

    let entry = match Wallet::from_secret(&secret, "imported".to_string()) {
        Ok(e) => e,
        Err(e) => return RpcResponse::err(&format!("invalid private key: {}", e)),
    };

    let mut w = Wallet::load_default().unwrap_or_else(|_| Wallet {
        entries: Vec::new(),
        created_at: now_millis(),
        encrypted: false,
        salt: String::new(),
        nonce: String::new(),
    });

    if w.encrypted {
        return RpcResponse::err(
            "wallet is encrypted. Use walletpassphrase to unlock it first.",
        );
    }

    if w.get_entry(&entry.address).is_none() {
        w.entries.push(entry.clone());
    }

    match w.save_default() {
        Ok(_) => RpcResponse::ok(json!({
            "address": entry.address,
            "public_key": entry.public_key,
            "message": "private key imported successfully",
        })),
        Err(e) => RpcResponse::err(&format!("failed to save wallet: {}", e)),
    }
}

fn m_encryptwallet(req: &RpcRequest) -> RpcResponse {
    let pass = match req.params.first().and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return RpcResponse::err("encryptwallet requires a password"),
    };

    let mut w = match Wallet::load_default() {
        Ok(w) => w,
        Err(_) => return RpcResponse::err("wallet not found"),
    };

    if let Err(e) = w.encrypt(&pass) {
        return RpcResponse::err(&format!("failed to encrypt wallet: {}", e));
    }

    match w.save_default() {
        Ok(_) => RpcResponse::ok(json!({
            "message": "wallet encrypted successfully",
        })),
        Err(e) => RpcResponse::err(&format!("failed to save wallet: {}", e)),
    }
}

fn m_getaddressinfo(req: &RpcRequest) -> RpcResponse {
    let target = match req.params.first().and_then(|v| v.as_str()) {
        Some(a) => a.to_string(),
        None => return RpcResponse::err("getaddressinfo requires an address"),
    };

    let w = match Wallet::load_default() {
        Ok(w) => w,
        Err(_) => return RpcResponse::err("wallet not found"),
    };

    let entry = w.get_entry(&target);

    RpcResponse::ok(json!({
        "address": target,
        "ismine": entry.is_some(),
        "label": entry.map(|e| e.label.clone()).unwrap_or_default(),
        "public_key": entry.map(|e| e.public_key.clone()).unwrap_or_default(),
        "created_at": entry.map(|e| e.created_at).unwrap_or(0),
    }))
}

fn m_setlabel(req: &RpcRequest) -> RpcResponse {
    if req.params.len() < 2 {
        return RpcResponse::err("setlabel requires an address and a label");
    }
    let target = match req.params[0].as_str() {
        Some(a) => a.to_string(),
        None => return RpcResponse::err("invalid address"),
    };
    let label = match req.params[1].as_str() {
        Some(l) => l.to_string(),
        None => return RpcResponse::err("invalid label"),
    };

    let mut w = match Wallet::load_default() {
        Ok(w) => w,
        Err(_) => return RpcResponse::err("wallet not found"),
    };

    let mut found = false;
    for e in w.entries.iter_mut() {
        if e.address == target {
            e.label = label.clone();
            found = true;
            break;
        }
    }

    if !found {
        return RpcResponse::err("address not found in wallet");
    }

    match w.save_default() {
        Ok(_) => RpcResponse::ok(json!({
            "address": target,
            "label": label,
            "message": "label updated successfully",
        })),
        Err(e) => RpcResponse::err(&format!("failed to save wallet: {}", e)),
    }
}

fn m_backupwallet(req: &RpcRequest) -> RpcResponse {
    let dest = match req.params.first().and_then(|v| v.as_str()) {
        Some(d) => d.to_string(),
        None => return RpcResponse::err("backupwallet requires a destination path"),
    };

    let src = datadir::as_str(&datadir::wallet_path());

    if !std::path::Path::new(&src).exists() {
        return RpcResponse::err("wallet file not found");
    }

    match std::fs::copy(&src, &dest) {
        Ok(_) => RpcResponse::ok(json!({
            "source": src,
            "destination": dest,
            "message": "wallet backed up successfully",
        })),
        Err(e) => RpcResponse::err(&format!("failed to backup wallet: {}", e)),
    }
}

fn m_getaddtowallet() -> RpcResponse {
    let w = match Wallet::load_default() {
        Ok(w) => w,
        Err(_) => return RpcResponse::err("wallet not found"),
    };
    let entry = match w.default_address() {
        Some(e) => e,
        None => return RpcResponse::err("wallet has no addresses"),
    };
    let mut cfg = Config::load_or_default();
    cfg.mining_address = entry.address.clone();
    match cfg.save_default() {
        Ok(_) => RpcResponse::ok(json!({
            "mining_address": entry.address,
            "config_file": Config::default_path(),
        })),
        Err(e) => RpcResponse::err(&format!("failed to save config: {}", e)),
    }
}

fn m_getaddtoaddress(req: &RpcRequest) -> RpcResponse {
    let addr = match req.params.first().and_then(|v| v.as_str()) {
        Some(a) => a.to_string(),
        None => return RpcResponse::err("getaddtoaddress requires an address"),
    };
    if !validation::is_valid_address(&addr) {
        return RpcResponse::err("invalid address format");
    }
    let mut cfg = Config::load_or_default();
    cfg.mining_address = addr.clone();
    match cfg.save_default() {
        Ok(_) => RpcResponse::ok(json!({
            "mining_address": addr,
            "config_file": Config::default_path(),
        })),
        Err(e) => RpcResponse::err(&format!("failed to save config: {}", e)),
    }
}

fn write_http_response(
    mut stream: TcpStream,
    status: u16,
    resp: &RpcResponse,
) -> std::io::Result<()> {
    let body = serde_json::to_string(resp).unwrap();
    let status_text = if status == 200 { "OK" } else { "Error" };
    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        status,
        status_text,
        body.len(),
        body
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()
}

pub fn call_and_print(method: &str, params: Vec<Value>) {
    match call(method, params) {
        Ok(resp) => {
            if resp.ok {
                println!("{}", serde_json::to_string_pretty(&resp.result).unwrap());
            } else {
                println!("error: {}", resp.error);
            }
        }
        Err(e) => {
            let cfg = Config::load_or_default();
            println!("[rpc] cannot connect to node at {}: {}", cfg.rpc_addr, e);
            println!("[rpc] start zenachaind first.");
        }
    }
}
