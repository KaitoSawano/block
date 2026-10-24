// Copyright (c) 2026 Aldianokto
// Distributed under the Apache License, see the accompanying
// file LICENSE or https://www.apache.org/licenses/LICENSE-2.0.

//! Bech32 encoding/decoding for ZenaChain Native SegWit addresses.

use bech32::{segwit, Fe32};

pub fn encode(hrp: &str, witness_program: &[u8]) -> Result<String, String> {
    if witness_program.len() != 20 {
        return Err(format!(
            "witness program must be 20 bytes, got {}",
            witness_program.len()
        ));
    }
    segwit::encode(hrp, Fe32::Q, witness_program)
        .map_err(|e| format!("bech32 encoding failed: {}", e))
}

pub fn decode(address: &str) -> Option<(String, Vec<u8>)> {
    let (hrp, variant, data) = segwit::decode(address).ok()?;
    if variant != Fe32::Q {
        return None;
    }
    Some((hrp.as_str().to_string(), data))
}
