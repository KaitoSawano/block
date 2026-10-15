use zenachain::utils::sha3_256_hex;

#[test]
fn test_sha3_256_known_vector_empty() {
    // NIST test vector untuk SHA3-256("")
    let h = sha3_256_hex(b"");
    assert_eq!(
        h,
        "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a"
    );
}

#[test]
fn test_sha3_256_known_vector_abc() {
    // NIST test vector untuk SHA3-256("abc")
    let h = sha3_256_hex(b"abc");
    assert_eq!(
        h,
        "3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532"
    );
}

#[test]
fn test_sha3_panjang_64_hex() {
    let h = sha3_256_hex(b"ZenaChain");
    assert_eq!(h.len(), 64);
}
