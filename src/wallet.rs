// Placeholder wallet ZENAC.
// Akan diisi keypair + address Base58Check pada tahap berikutnya.

#[derive(Debug, Clone)]
pub struct Wallet {
    pub address: String,
}

impl Wallet {
    pub fn new(address: &str) -> Self {
        Wallet {
            address: address.into(),
        }
    }
}
