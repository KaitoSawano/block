use zenachain::block::Block;
use zenachain::blockchain::*;

#[test]
fn test_genesis_secret_tertanam() {
    let g = Block::genesis();
    let secret_tx = g
        .transactions
        .iter()
        .find(|tx| tx.from == "GENESIS")
        .expect("Pesan rahasia genesis tidak ditemukan");
    assert_eq!(secret_tx.note, "ID Today 15/Oct/2026");
}

#[test]
fn test_genesis_hash_mengikat_pesan() {
    let mut g1 = Block::genesis();
    let h1 = g1.calculate_hash();

    g1.transactions[1].note = "DIUBAH".into();
    let h2 = g1.calculate_hash();

    assert_ne!(h1, h2, "Hash genesis harus berubah jika pesan diubah");
}

#[test]
fn test_genesis_timestamp() {
    let g = Block::genesis();
    assert_eq!(g.timestamp, GENESIS_TIMESTAMP);
}

#[test]
fn test_genesis_index_nol() {
    let g = Block::genesis();
    assert_eq!(g.index, 0);
}
