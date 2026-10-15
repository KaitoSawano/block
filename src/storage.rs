// Placeholder storage ZENAC.
// Akan diisi simpan/load chain.json + verifikasi genesis pada tahap berikutnya.

use crate::blockchain::Blockchain;

pub fn save(_chain: &Blockchain, _path: &str) -> std::io::Result<()> {
    // TODO: serialize chain ke JSON
    Ok(())
}

pub fn load(_path: &str) -> std::io::Result<Blockchain> {
    // TODO: deserialize chain dari JSON
    Ok(Blockchain::new())
}
