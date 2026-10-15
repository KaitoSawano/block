use crate::blockchain::Blockchain;
use crate::config::Config;
use crate::datadir;
use crate::pow;
use crate::storage;
use crate::transaction::Transaction;
use crate::utils::now_millis;
use crate::wallet::Wallet;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

pub const RPC_ADDR: &str = "127.0.0.1:8332";
pub const COINBASE_MATURITY: u64 = 100;

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
        RpcResponse { ok: true, result, error: String::new() }
    }
    pub fn err(msg: &str) -> Self {
        RpcResponse { ok: false, result: Value::Null, error: msg.to_string() }
    }
}

pub fn call(method: &str, params: Vec<Value>) -> std::io::Result<RpcResponse> {
    let mut stream = TcpStream::connect(RPC_ADDR)?;
    let req = RpcRequest { method: method.to_string(), params };
    let line = serde_json::to_string(&req).unwrap();
    stream.write_all(line.as_bytes())?;
    stream.write_all(b"\n")?;
    stream.flush()?;

    let mut reader = BufReader::new(stream);
    let mut buf = String::new();
    reader.read_line(&mut buf)?;

    let resp: RpcResponse = serde_json::from_str(buf.trim())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    Ok(resp)
}

pub struct RpcServer {
    chain: Blockchain,
}

impl RpcServer {
    pub fn new() -> Self {
        println!("[rpc] new: memuat chain untuk server RPC.");
        let chain = storage::load_chain().unwrap_or_else(|_| Blockchain::new());
        RpcServer { chain }
    }

    pub fn listen(&mut self) {
        println!("[rpc] listen: server RPC di {}.", RPC_ADDR);
        let listener = match TcpListener::bind(RPC_ADDR) {
            Ok(l) => l,
            Err(e) => {
                println!("[rpc] gagal bind {}: {}", RPC_ADDR, e);
                return;
            }
        };

        for incoming in listener.incoming() {
            match incoming {
                Ok(stream) => { let _ = self.handle(stream); }
                Err(e) => println!("[rpc] koneksi gagal: {}", e),
            }
        }
    }

    fn handle(&mut self, stream: TcpStream) -> std::io::Result<()> {
        let mut reader = BufReader::new(stream.try_clone()?);
        let mut line = String::new();
        reader.read_line(&mut line)?;

        let req: RpcRequest = match serde_json::from_str(line.trim()) {
            Ok(r) => r,
            Err(_) => {
                let resp = RpcResponse::err("request tidak valid");
                write_response(stream, &resp)?;
                return Ok(());
            }
        };

        let resp = self.dispatch(&req);
        write_response(stream, &resp)
    }

    fn dispatch(&mut self, req: &RpcRequest) -> RpcResponse {
        println!("[rpc] dispatch: method={}", req.method);
        match req.method.as_str() {
            "getinfo" => self.m_getinfo(),
            "getblocktemplate" => self.m_getblocktemplate(),
            "submitblock" => self.m_submitblock(req),
            "getbalance" => self.m_getbalance(req),
            "getwallet" => m_getwallet(),
            "getprivkey" => m_getprivkey(),
            "dumpprivkey" => m_dumpprivkey(req),
            "getnewwallet" => m_getnewwallet(),
            "getaddtowallet" => m_getaddtowallet(),
            "getaddtoaddress" => m_getaddtoaddress(req),
            "generatetoaddress" => self.m_generatetoaddress(req),
            _ => RpcResponse::err("method tidak dikenal"),
        }
    }

    fn m_getinfo(&self) -> RpcResponse {
        let c = &self.chain;
        RpcResponse::ok(json!({
            "coin_name": crate::blockchain::coin_name(),
            "symbol": crate::blockchain::symbol(),
            "protocol_version": crate::blockchain::protocol_version(),
            "hash_algo": crate::blockchain::hash_algo(),
            "height": c.height(),
            "chain_len": c.chain.len(),
            "difficulty": c.difficulty,
            "circulating": c.circulating_supply(),
            "max_supply": crate::blockchain::max_supply(),
            "era": c.current_era(),
            "until_halving": c.blocks_until_halving(),
            "mempool_len": c.mempool.len(),
            "valid": c.is_valid(),
            "datadir": datadir::as_str(&datadir::datadir()),
        }))
    }

    fn m_getblocktemplate(&self) -> RpcResponse {
        let c = &self.chain;
        let next_height = c.height() + 1;
        let reward = Blockchain::calculate_reward(next_height);
        let prev_hash = c.chain.last().unwrap().hash.clone();
        RpcResponse::ok(json!({
            "height": next_height,
            "previous_hash": prev_hash,
            "difficulty": c.difficulty,
            "reward": reward,
            "mempool": c.mempool,
        }))
    }

    fn m_submitblock(&mut self, req: &RpcRequest) -> RpcResponse {
        if req.params.is_empty() {
            return RpcResponse::err("submitblock butuh parameter blok");
        }
        let block: crate::block::Block = match serde_json::from_value(req.params[0].clone()) {
            Ok(b) => b,
            Err(e) => return RpcResponse::err(&format!("blok tidak valid: {}", e)),
        };

        if block.hash != block.calculate_hash() {
            return RpcResponse::err("hash blok tidak cocok");
        }

        let prev = self.chain.chain.last().unwrap();
        if block.previous_hash != prev.hash {
            return RpcResponse::err("previous_hash tidak cocok");
        }

        if !pow::hash_meets_target(&block.hash, self.chain.difficulty) {
            return RpcResponse::err("pow tidak memenuhi target");
        }

        if block.reward != Blockchain::calculate_reward(block.index) {
            return RpcResponse::err("reward tidak sesuai era halving");
        }

        self.chain.chain.push(block.clone());
        self.chain.mempool.clear();

        if let Err(e) = storage::save_chain(&self.chain) {
            return RpcResponse::err(&format!("gagal simpan chain: {}", e));
        }

        datadir::append_debug_log(&format!(
            "block accepted height={} hash={} mined_for={} mined_at={}",
            block.index, block.hash, block.mined_for, block.mined_at
        ));

        RpcResponse::ok(json!({
            "accepted": true,
            "height": block.index,
            "hash": block.hash,
            "mined_for": block.mined_for,
            "mined_at": block.mined_at,
        }))
    }

    fn m_getbalance(&self, req: &RpcRequest) -> RpcResponse {
        let addr = match req.params.first().and_then(|v| v.as_str()) {
            Some(a) => a.to_string(),
            None => match Wallet::load_default() {
                Ok(w) => w.address,
                Err(_) => return RpcResponse::err("wallet tidak ada"),
            },
        };

        let mut spent: HashSet<String> = HashSet::new();
        for block in &self.chain.chain {
            for tx in &block.transactions {
                for input in &tx.inputs {
                    spent.insert(input.clone());
                }
            }
        }

        let current_height = self.chain.height();

        let mut mature: u64 = 0;
        let mut immature: u64 = 0;
        let mut received: u64 = 0;

        for block in &self.chain.chain {
            for tx in &block.transactions {
                if tx.to == addr {
                    let outpoint = format!("{}:0", tx.txid);
                    if !spent.contains(&outpoint) {
                        received += tx.amount;
                        mature += tx.amount;
                    }
                }
            }

            if block.mined_for == addr && block.reward > 0 {
                if block.index + COINBASE_MATURITY <= current_height {
                    mature += block.reward;
                } else {
                    immature += block.reward;
                }
            }
        }

        let balance = mature;
        RpcResponse::ok(json!({
            "address": addr,
            "balance": balance,
            "mature": mature,
            "immature": immature,
            "received": received,
            "spent_outputs": spent.len(),
        }))
    }

    fn m_generatetoaddress(&mut self, req: &RpcRequest) -> RpcResponse {
        let n = match req.params.first().and_then(|v| v.as_u64()) {
            Some(n) => n,
            None => return RpcResponse::err("generatetoaddress butuh jumlah blok"),
        };
        let addr = match req.params.get(1).and_then(|v| v.as_str()) {
            Some(a) => a.to_string(),
            None => return RpcResponse::err("generatetoaddress butuh alamat"),
        };

        let mut hashes: Vec<String> = Vec::new();

        for _ in 0..n {
            let prev = self.chain.chain.last().unwrap().clone();
            let next_index = prev.index + 1;
            let reward = Blockchain::calculate_reward(next_index);

            if self.chain.circulating_supply() + reward > crate::blockchain::max_supply() {
                return RpcResponse::err("max supply reached");
            }

            let mut txs = std::mem::take(&mut self.chain.mempool);
            txs.push(Transaction::new(
                "NETWORK",
                &addr,
                crate::blockchain::satoshi() / 100,
                0,
            ));
            txs.push(Transaction::new_with_inputs(
                "MEMPOOL",
                "CreateNewBlock",
                0,
                0,
                Vec::new(),
                format!("height={} target={}", next_index, addr),
            ));

            let mut block =
                crate::block::Block::new_for(next_index, prev.hash.clone(), reward, txs, addr.clone());
            pow::mine(&mut block, self.chain.difficulty);

            let hash = block.hash.clone();
            let height = block.index;
            let mined_at = block.mined_at.clone();
            self.chain.chain.push(block);
            self.chain.adjust_difficulty();

            datadir::append_debug_log(&format!(
                "block mined (generatetoaddress) height={} hash={} mined_for={} mined_at={}",
                height, hash, addr, mined_at
            ));

            hashes.push(hash);
        }

        if let Err(e) = storage::save_chain(&self.chain) {
            return RpcResponse::err(&format!("gagal simpan chain: {}", e));
        }

        RpcResponse::ok(json!(hashes))
    }
}

fn m_getwallet() -> RpcResponse {
    match Wallet::load_default() {
        Ok(w) => RpcResponse::ok(json!({
            "address": w.address,
            "public_key": w.public_key,
            "secret": w.secret,
            "created_at": w.created_at,
        })),
        Err(_) => RpcResponse::err("wallet tidak ditemukan"),
    }
}

fn m_getprivkey() -> RpcResponse {
    match Wallet::load_default() {
        Ok(w) => RpcResponse::ok(json!({
            "address": w.address,
            "privkey": w.secret,
        })),
        Err(_) => RpcResponse::err("wallet tidak ditemukan"),
    }
}

fn m_dumpprivkey(req: &RpcRequest) -> RpcResponse {
    let secret = match req.params.first().and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("dumpprivkey butuh private key"),
    };

    if secret.len() != 64 || !secret.chars().all(|c| c.is_ascii_hexdigit()) {
        return RpcResponse::err("private key harus 64 karakter hex");
    }

    let w = Wallet::from_secret(&secret);
    match w.save_default() {
        Ok(_) => RpcResponse::ok(json!({
            "address": w.address,
            "public_key": w.public_key,
            "created_at": w.created_at,
        })),
        Err(e) => RpcResponse::err(&format!("gagal simpan wallet: {}", e)),
    }
}

fn m_getnewwallet() -> RpcResponse {
    if let Err(e) = datadir::ensure_dirs() {
        return RpcResponse::err(&format!("gagal siapkan .chain: {}", e));
    }
    let w = Wallet::new();
    match w.save_default() {
        Ok(_) => RpcResponse::ok(json!({
            "address": w.address,
            "public_key": w.public_key,
            "secret": w.secret,
            "created_at": w.created_at,
        })),
        Err(e) => RpcResponse::err(&format!("gagal simpan wallet: {}", e)),
    }
}

fn m_getaddtowallet() -> RpcResponse {
    let w = match Wallet::load_default() {
        Ok(w) => w,
        Err(_) => return RpcResponse::err("wallet tidak ditemukan"),
    };
    let cfg = Config {
        mining_address: w.address.clone(),
        created_at: now_millis(),
        protocol_version: crate::blockchain::protocol_version(),
        network: "main".to_string(),
    };
    match cfg.save_default() {
        Ok(_) => RpcResponse::ok(json!({
            "mining_address": w.address,
            "config_file": datadir::as_str(&datadir::config_path()),
        })),
        Err(e) => RpcResponse::err(&format!("gagal simpan config: {}", e)),
    }
}

fn m_getaddtoaddress(req: &RpcRequest) -> RpcResponse {
    let addr = match req.params.first().and_then(|v| v.as_str()) {
        Some(a) => a.to_string(),
        None => return RpcResponse::err("getaddtoaddress butuh alamat"),
    };
    let cfg = Config {
        mining_address: addr.clone(),
        created_at: now_millis(),
        protocol_version: crate::blockchain::protocol_version(),
        network: "main".to_string(),
    };
    match cfg.save_default() {
        Ok(_) => RpcResponse::ok(json!({
            "mining_address": addr,
            "config_file": datadir::as_str(&datadir::config_path()),
        })),
        Err(e) => RpcResponse::err(&format!("gagal simpan config: {}", e)),
    }
}

fn write_response(mut stream: TcpStream, resp: &RpcResponse) -> std::io::Result<()> {
    let line = serde_json::to_string(resp).unwrap();
    stream.write_all(line.as_bytes())?;
    stream.write_all(b"\n")?;
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
            println!("[rpc] tidak dapat menghubungi node di {}: {}", RPC_ADDR, e);
            println!("[rpc] jalankan zenachaind terlebih dahulu.");
        }
    }
}
