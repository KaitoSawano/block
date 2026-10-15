use std::path::{Path, PathBuf};

pub const DATADIR_NAME: &str = ".chain";
pub const BLOCKS_DIR: &str = "blocks";
pub const CHAINSTATE_DIR: &str = "chainstate";
pub const WALLET_FILE: &str = "wallet.dat";
pub const CONFIG_FILE: &str = "zenachain.conf";
pub const PEERS_FILE: &str = "peers.json";
pub const MEMPOOL_FILE: &str = "mempool.json";
pub const DEBUG_LOG: &str = "debug.log";
pub const BLOCK_FILE_PREFIX: &str = "blk";

pub fn datadir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(DATADIR_NAME)
}

pub fn blocks_dir() -> PathBuf { datadir().join(BLOCKS_DIR) }
pub fn chainstate_dir() -> PathBuf { datadir().join(CHAINSTATE_DIR) }
pub fn wallet_path() -> PathBuf { datadir().join(WALLET_FILE) }
pub fn config_path() -> PathBuf { datadir().join(CONFIG_FILE) }
pub fn peers_path() -> PathBuf { datadir().join(PEERS_FILE) }
pub fn mempool_path() -> PathBuf { datadir().join(MEMPOOL_FILE) }
pub fn debug_log_path() -> PathBuf { datadir().join(DEBUG_LOG) }

pub fn block_file(index: u32) -> PathBuf {
    blocks_dir().join(format!("{}{:05}.dat", BLOCK_FILE_PREFIX, index))
}

pub fn chainstate_file() -> PathBuf {
    chainstate_dir().join("state.json")
}

pub fn ensure_dirs() -> std::io::Result<()> {
    std::fs::create_dir_all(datadir())?;
    std::fs::create_dir_all(blocks_dir())?;
    std::fs::create_dir_all(chainstate_dir())?;
    Ok(())
}

pub fn exists() -> bool { datadir().exists() }

pub fn as_str(p: &Path) -> String {
    p.to_string_lossy().to_string()
}

pub fn append_debug_log(line: &str) {
    let _ = ensure_dirs();
    let path = debug_log_path();
    let ts = crate::utils::now_millis();
    let entry = format!("{} {}\n", ts, line);
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = f.write_all(entry.as_bytes());
    }
}
