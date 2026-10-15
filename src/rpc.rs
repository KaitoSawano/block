use crate::blockchain::{self, Blockchain};
use crate::config::Config;
use crate::datadir;
use crate::pow;
use crate::storage;
use crate::transaction::Transaction;
use crate::utils::{now_millis, zenac_to_satoshi};
use crate::validation;
use crate::wallet::Wallet;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub const RPC_ADDR: &str = "127.0.0.1:8332";
pub const DEFAULT_FEE_SATOSHI: u64 = 100_000; // 0.001 ZENAC

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

fn to_coin(satoshi: u64) -> String {
    let whole = satoshi / 100_000_000;
    let frac = satoshi % 100_000_000;
    format!("{}.{:08}", whole, frac)
}

pub struct RpcServer {
    chain: Blockchain,
    shutdown: Arc<AtomicBool>,
}

impl RpcServer {
    pub fn new() -> Self {
        let chain = storage::load_chain().unwrap_or_else(|_| Blockchain::new());
        RpcServer {
            chain,
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn listen(&mut self) {
        let listener = match TcpListener::bind(RPC_ADDR) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("[rpc] gagal bind {}: {}", RPC_ADDR, e);
                return;
            }
        };

        let _ = listener.set_nonblocking(true);

        loop {
            if self.shutdown.load(Ordering::SeqCst) {
                let _ = storage::save_chain(&self.chain);
                eprintln!("[rpc] node berhenti.");
                break;
            }

            match listener.accept() {
                Ok((stream, _)) => { let _ = self.handle(stream); }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(e) => eprintln!("[rpc] koneksi gagal: {}", e),
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
        match req.method.as_str() {
            "getinfo" => self.m_getinfo(),
            "getblockchaininfo" => self.m_getblockchaininfo(),
            "getcheckpoints" => m_getcheckpoints(),
            "getblocktemplate" => self.m_getblocktemplate(),
            "submitblock" => self.m_submitblock(req),
            "getbalance" => self.m_getbalance(req),
            "getblockhash" => self.m_getblockhash(req),
            "getblock" => self.m_getblock(req),
            "getblockcount" => self.m_getblockcount(),
            "getbestblockhash" => self.m_getbestblockhash(),
            "verifychain" => self.m_verifychain(),
            "getwallet" => m_getwallet(),
            "getprivkey" => m_getprivkey(req),
            "dumpprivkey" => m_dumpprivkey(req),
            "getnewwallet" => m_getnewwallet(),
            "getaddtowallet" => m_getaddtowallet(),
            "getaddtoaddress" => m_getaddtoaddress(req),
            "generatetoaddress" => self.m_generatetoaddress(req),
            "sendtoaddress" => self.m_sendtoaddress(req),
            "stop" => self.m_stop(),
            "getrawtransaction" => self.m_getrawtransaction(req),
            "listunspent" => self.m_listunspent(req),
            "getmempoolinfo" => self.m_getmempoolinfo(),
            "validatemempool" => self.m_validatemempool(),
            _ => RpcResponse::err("method tidak dikenal"),
        }
    }

    fn m_stop(&mut self) -> RpcResponse {
        if let Err(e) = storage::save_chain(&self.chain) {
            return RpcResponse::err(&format!("gagal simpan chain: {}", e));
        }

        self.shutdown.store(true, Ordering::SeqCst);

        RpcResponse::ok(json!({
            "stopping": true,
            "height": self.chain.height(),
            "chain_len": self.chain.chain.len(),
            "saved": true,
        }))
    }

    fn m_getinfo(&self) -> RpcResponse {
        let c = &self.chain;
        let circulating = c.circulating_supply();
        let max_supply = blockchain::max_supply();
        RpcResponse::ok(json!({
            "coin_name": blockchain::coin_name(),
            "symbol": blockchain::symbol(),
            "version": blockchain::version(),
            "protocol_version": blockchain::protocol_version(),
            "hash_algo": blockchain::hash_algo(),
            "signature_algo": blockchain::signature_algo(),
            "height": c.height(),
            "chain_len": c.chain.len(),
            "nbits": format!("{:#010x}", c.nbits),
            "pow_limit": blockchain::pow_limit(),
            "circulating": to_coin(circulating),
            "max_supply": to_coin(max_supply),
            "era": c.current_era(),
            "subsidy_halving_interval": blockchain::subsidy_halving_interval(),
            "blocks_until_halving": c.blocks_until_halving(),
            "pow_target_spacing_secs": blockchain::pow_target_spacing_secs(),
            "pow_target_timespan_secs": blockchain::pow_target_timespan_secs(),
            "miner_confirmation_window": blockchain::miner_confirmation_window(),
            "difficulty_adjust_interval": blockchain::difficulty_adjust_interval(),
            "mempool_len": c.mempool.len(),
            "valid": c.is_valid(),
            "checkpoints_count": blockchain::checkpoint::CHECKPOINTS.len(),
            "checkpoints_highest": blockchain::checkpoint::highest(),
            "datadir": datadir::as_str(&datadir::datadir()),
        }))
    }

    fn m_getblockchaininfo(&self) -> RpcResponse {
        let c = &self.chain;
        let tip = c.chain.last().unwrap();
        RpcResponse::ok(json!({
            "chain": "main",
            "blocks": c.height(),
            "headers": c.height(),
            "bestblockhash": tip.hash,
            "nbits": format!("{:#010x}", c.nbits),
            "pow_limit": blockchain::pow_limit(),
            "mediantime": tip.timestamp,
            "verificationprogress": 1.0,
            "initialblockdownload": false,
            "pruned": false,
            "checkpoints_count": blockchain::checkpoint::CHECKPOINTS.len(),
            "checkpoints_highest": blockchain::checkpoint::highest(),
            "mempool_len": c.mempool.len(),
            "circulating": to_coin(c.circulating_supply()),
            "max_supply": to_coin(blockchain::max_supply()),
            "era": c.current_era(),
            "subsidy_halving_interval": blockchain::subsidy_halving_interval(),
            "blocks_until_halving": c.blocks_until_halving(),
            "difficulty_adjust_interval": blockchain::difficulty_adjust_interval(),
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
            "nbits": c.nbits,
            "reward": to_coin(reward),
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

        let prev = self.chain.chain.last().unwrap().clone();

        if let Err(e) = validation::validate_block(&self.chain, &prev, &block) {
            return RpcResponse::err(&format!("blok ditolak: {}", e));
        }

        self.chain.chain.push(block.clone());
        self.chain.mempool.clear();

        block.log_add_to_wallet();

        if let Err(e) = storage::save_chain(&self.chain) {
            return RpcResponse::err(&format!("gagal simpan chain: {}", e));
        }

        RpcResponse::ok(json!({
            "accepted": true,
            "height": block.index,
            "hash": block.hash,
            "txid": block.coinbase_txid,
            "mined_at": block.mined_at,
            "mined_for": block.mined_for,
            "nbits": format!("{:#010x}", block.nbits),
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

        if !validation::is_valid_address(&addr) {
            return RpcResponse::err("format alamat tidak valid");
        }

        let mature = validation::mature_balance(&self.chain, &addr);
        let immature = validation::immature_balance(&self.chain, &addr);

        RpcResponse::ok(json!({
            "address": addr,
            "balance": to_coin(mature),
            "mature": to_coin(mature),
            "immature": to_coin(immature),
        }))
    }

    fn m_getblockhash(&self, req: &RpcRequest) -> RpcResponse {
        let height = match req.params.first().and_then(|v| v.as_u64()) {
            Some(h) => h,
            None => return RpcResponse::err("getblockhash butuh height"),
        };
        if height as usize >= self.chain.chain.len() {
            return RpcResponse::err("height di luar rentang chain");
        }
        let block = &self.chain.chain[height as usize];
        if block.index != height {
            return RpcResponse::err("index blok tidak cocok dengan height");
        }
        RpcResponse::ok(json!(block.hash))
    }

    fn m_getblock(&self, req: &RpcRequest) -> RpcResponse {
        let hash = match req.params.first().and_then(|v| v.as_str()) {
            Some(h) => h,
            None => return RpcResponse::err("getblock butuh hash"),
        };
        let block = match self.chain.chain.iter().find(|b| b.hash == hash) {
            Some(b) => b,
            None => return RpcResponse::err("blok tidak ditemukan"),
        };
        RpcResponse::ok(json!({
            "hash": block.hash,
            "height": block.index,
            "previous_hash": block.previous_hash,
            "timestamp": block.timestamp,
            "mined_at": block.mined_at,
            "mined_for": block.mined_for,
            "nonce": block.nonce,
            "nbits": format!("{:#010x}", block.nbits),
            "reward": to_coin(block.reward),
            "coinbase_txid": block.coinbase_txid,
            "transactions": block.transactions,
        }))
    }

    fn m_getblockcount(&self) -> RpcResponse {
        RpcResponse::ok(json!(self.chain.chain.len() as u64))
    }

    fn m_getbestblockhash(&self) -> RpcResponse {
        RpcResponse::ok(json!(self.chain.chain.last().unwrap().hash))
    }

    fn m_verifychain(&self) -> RpcResponse {
        RpcResponse::ok(json!({
            "valid": self.chain.is_valid(),
            "blocks_checked": self.chain.chain.len(),
            "height": self.chain.height(),
            "nbits": format!("{:#010x}", self.chain.nbits),
            "pow_limit": blockchain::pow_limit(),
            "signature_algo": blockchain::signature_algo(),
            "checkpoints_count": blockchain::checkpoint::CHECKPOINTS.len(),
        }))
    }

    fn m_getrawtransaction(&self, req: &RpcRequest) -> RpcResponse {
        let txid = match req.params.first().and_then(|v| v.as_str()) {
            Some(t) => t.to_string(),
            None => return RpcResponse::err("getrawtransaction butuh txid"),
        };

        for block in &self.chain.chain {
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
                        "note": tx.note,
                        "inputs": tx.inputs,
                        "public_key": tx.public_key,
                        "signature": tx.signature,
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
                    "fee_satoshi": 0,
                    "fee_zenac": "0.00000000",
                    "note": "",
                    "inputs": [],
                    "public_key": "",
                    "signature": "",
                    "in_mempool": false,
                    "block_hash": block.hash,
                    "block_height": block.index,
                    "block_mined_at": block.mined_at,
                    "is_coinbase": true,
                }));
            }
        }

        for tx in &self.chain.mempool {
            if tx.txid == txid {
                return RpcResponse::ok(json!({
                    "txid": tx.txid,
                    "from": tx.from,
                    "to": tx.to,
                    "amount_satoshi": tx.amount,
                    "amount_zenac": to_coin(tx.amount),
                    "fee_satoshi": tx.fee,
                    "fee_zenac": to_coin(tx.fee),
                    "note": tx.note,
                    "inputs": tx.inputs,
                    "public_key": tx.public_key,
                    "signature": tx.signature,
                    "in_mempool": true,
                }));
            }
        }

        RpcResponse::err("transaksi tidak ditemukan")
    }

    fn m_listunspent(&self, req: &RpcRequest) -> RpcResponse {
        let addr = match req.params.first().and_then(|v| v.as_str()) {
            Some(a) => a.to_string(),
            None => match Wallet::load_default() {
                Ok(w) => w.address,
                Err(_) => return RpcResponse::err("wallet tidak ada"),
            },
        };

        if !validation::is_valid_address(&addr) {
            return RpcResponse::err("format alamat tidak valid");
        }

        let spent = validation::spent_inputs(&self.chain);
        let current_height = self.chain.height();
        let mut entries: Vec<Value> = Vec::new();

        for block in &self.chain.chain {
            for tx in &block.transactions {
                if tx.to == addr {
                    let outpoint = format!("{}:0", tx.txid);
                    if !spent.contains(&outpoint) {
                        entries.push(json!({
                            "txid": tx.txid,
                            "vout": 0,
                            "address": addr,
                            "amount_satoshi": tx.amount,
                            "amount_zenac": to_coin(tx.amount),
                            "from": tx.from,
                            "block_height": block.index,
                            "block_hash": block.hash,
                            "mined_at": block.mined_at,
                            "mature": true,
                            "source": "tx",
                        }));
                    }
                }
            }

            if block.mined_for == addr && block.reward > 0 {
                let mature_at = block.index.saturating_add(validation::COINBASE_MATURITY);
                let is_mature = mature_at <= current_height;
                entries.push(json!({
                    "txid": block.coinbase_txid,
                    "vout": 0,
                    "address": addr,
                    "amount_satoshi": block.reward,
                    "amount_zenac": to_coin(block.reward),
                    "from": "COINBASE",
                    "block_height": block.index,
                    "block_hash": block.hash,
                    "mined_at": block.mined_at,
                    "mature": is_mature,
                    "mature_at": mature_at,
                    "source": "coinbase",
                }));
            }
        }

        RpcResponse::ok(json!({
            "address": addr,
            "height": current_height,
            "coinbase_maturity": validation::COINBASE_MATURITY,
            "count": entries.len(),
            "utxos": entries,
        }))
    }

    fn m_getmempoolinfo(&self) -> RpcResponse {
        let txs = &self.chain.mempool;
        let count = txs.len();

        let mut total_amount: u64 = 0;
        let mut total_fee: u64 = 0;
        let mut min_fee: u64 = u64::MAX;
        let mut max_fee: u64 = 0;

        let mut by_from: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
        let mut by_to: std::collections::HashMap<String, u64> = std::collections::HashMap::new();

        for tx in txs {
            total_amount = total_amount.saturating_add(tx.amount);
            total_fee = total_fee.saturating_add(tx.fee);

            if tx.fee < min_fee {
                min_fee = tx.fee;
            }
            if tx.fee > max_fee {
                max_fee = tx.fee;
            }

            *by_from.entry(tx.from.clone()).or_insert(0) += 1;
            *by_to.entry(tx.to.clone()).or_insert(0) += 1;
        }

        if count == 0 {
            min_fee = 0;
        }

        let avg_fee = if count > 0 {
            total_fee / count as u64
        } else {
            0
        };

        RpcResponse::ok(json!({
            "size": count,
            "total_amount_satoshi": total_amount,
            "total_amount_zenac": to_coin(total_amount),
            "total_fee_satoshi": total_fee,
            "total_fee_zenac": to_coin(total_fee),
            "min_fee_satoshi": min_fee,
            "min_fee_zenac": to_coin(min_fee),
            "max_fee_satoshi": max_fee,
            "max_fee_zenac": to_coin(max_fee),
            "avg_fee_satoshi": avg_fee,
            "avg_fee_zenac": to_coin(avg_fee),
            "default_fee_satoshi": DEFAULT_FEE_SATOSHI,
            "default_fee_zenac": to_coin(DEFAULT_FEE_SATOSHI),
            "by_from": by_from,
            "by_to": by_to,
        }))
    }

    fn m_validatemempool(&self) -> RpcResponse {
        let errors = validation::validate_mempool(&self.chain);
        let items: Vec<Value> = errors
            .iter()
            .map(|e| json!({"code": e.code, "message": e.message}))
            .collect();
        RpcResponse::ok(json!({
            "valid": items.is_empty(),
            "mempool_len": self.chain.mempool.len(),
            "errors": items,
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

        if !validation::is_valid_address(&addr) {
            return RpcResponse::err("format alamat tidak valid");
        }

        let mut hashes: Vec<String> = Vec::new();

        for _ in 0..n {
            let prev = self.chain.chain.last().unwrap().clone();
            let next_index = prev.index + 1;
            let reward = Blockchain::calculate_reward(next_index);

            if self.chain.circulating_supply().saturating_add(reward)
                > blockchain::max_supply()
            {
                return RpcResponse::err("max supply reached");
            }

            let mut txs = std::mem::take(&mut self.chain.mempool);
            txs.push(Transaction::new(
                "NETWORK",
                &addr,
                blockchain::satoshi() / 100,
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

            let mut block = crate::block::Block::new_for(
                next_index,
                prev.hash.clone(),
                reward,
                self.chain.nbits,
                txs,
                addr.clone(),
            );
            if !pow::mine(&mut block, self.chain.nbits) {
                return RpcResponse::err("mining gagal: nonce habis");
            }

            let hash = block.hash.clone();
            self.chain.chain.push(block.clone());
            self.chain.adjust_difficulty();

            block.log_add_to_wallet();

            hashes.push(hash);
        }

        if let Err(e) = storage::save_chain(&self.chain) {
            return RpcResponse::err(&format!("gagal simpan chain: {}", e));
        }

        RpcResponse::ok(json!(hashes))
    }

    fn m_sendtoaddress(&mut self, req: &RpcRequest) -> RpcResponse {
        if req.params.len() < 2 {
            return RpcResponse::err("sendtoaddress butuh jumlah dan alamat");
        }

        let amount_zenac: f64 = match req.params[0].as_f64() {
            Some(v) => v,
            None => match req.params[0].as_str().and_then(|s| s.parse().ok()) {
                Some(v) => v,
                None => return RpcResponse::err("jumlah tidak valid"),
            },
        };

        if amount_zenac <= 0.0 {
            return RpcResponse::err("jumlah harus lebih dari nol");
        }

        let to = match req.params[1].as_str() {
            Some(a) => a.to_string(),
            None => return RpcResponse::err("alamat tujuan tidak valid"),
        };

        let wallet = match Wallet::load_default() {
            Ok(w) => w,
            Err(_) => return RpcResponse::err("wallet tidak ditemukan"),
        };

        let amount_sat = zenac_to_satoshi(amount_zenac);
        if amount_sat == 0 {
            return RpcResponse::err("jumlah terlalu kecil");
        }

        let mut tx = Transaction::new(
            &wallet.address,
            &to,
            amount_sat,
            DEFAULT_FEE_SATOSHI,
        );

        if let Err(e) = tx.sign(&wallet.secret) {
            return RpcResponse::err(&format!("gagal tanda tangan: {}", e));
        }

        if let Err(e) = validation::validate_transaction(&self.chain, &tx) {
            return RpcResponse::err(&format!("transaksi tidak valid: {}", e));
        }

        let txid = tx.txid.clone();
        self.chain.mempool.push(tx);

        if let Err(e) = storage::save_chain(&self.chain) {
            return RpcResponse::err(&format!("gagal simpan chain: {}", e));
        }

        RpcResponse::ok(json!({
            "txid": txid,
            "from": wallet.address,
            "to": to,
            "amount_satoshi": amount_sat,
            "amount_zenac": to_coin(amount_sat),
            "fee_satoshi": DEFAULT_FEE_SATOSHI,
            "fee_zenac": to_coin(DEFAULT_FEE_SATOSHI),
            "mempool_len": self.chain.mempool.len(),
        }))
    }
}

fn m_getcheckpoints() -> RpcResponse {
    let items: Vec<Value> = blockchain::checkpoint::CHECKPOINTS
        .iter()
        .map(|c| json!({"height": c.height, "hash": c.hash}))
        .collect();
    RpcResponse::ok(json!(items))
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

fn m_getprivkey(req: &RpcRequest) -> RpcResponse {
    let target = match req.params.first().and_then(|v| v.as_str()) {
        Some(a) => a.to_string(),
        None => return RpcResponse::err("wajib memasukkan \"alamat tujuan\""),
    };

    let w = match Wallet::load_default() {
        Ok(w) => w,
        Err(_) => return RpcResponse::err("wallet tidak ditemukan"),
    };

    if w.address != target {
        return RpcResponse::err("alamat tidak cocok dengan wallet tersimpan");
    }

    RpcResponse::ok(json!({
        "address": w.address,
        "public_key": w.public_key,
        "privkey": w.secret,
        "created_at": w.created_at,
    }))
}

fn m_dumpprivkey(req: &RpcRequest) -> RpcResponse {
    let secret = match req.params.first().and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("dumpprivkey butuh private key"),
    };

    if secret.len() != 64 || !secret.chars().all(|c| c.is_ascii_hexdigit()) {
        return RpcResponse::err("private key harus 64 karakter hex");
    }

    let w = match Wallet::from_secret(&secret) {
        Ok(w) => w,
        Err(e) => return RpcResponse::err(&format!("private key tidak valid: {}", e)),
    };

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
        protocol_version: blockchain::protocol_version(),
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
    if !validation::is_valid_address(&addr) {
        return RpcResponse::err("format alamat tidak valid");
    }
    let cfg = Config {
        mining_address: addr.clone(),
        created_at: now_millis(),
        protocol_version: blockchain::protocol_version(),
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
