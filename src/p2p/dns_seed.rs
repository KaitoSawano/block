// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

//! DNS seed resolution for peer discovery.

use std::net::SocketAddr;

use crate::blockchain::dns_seed;

/// Resolve every DNS seed for the current network and return the
/// resulting socket addresses.
pub async fn resolve_seeds() -> Vec<SocketAddr> {
    let mut out = Vec::new();
    for seed in dns_seed::seeds_for_current_network() {
        match tokio::net::lookup_host(seed).await {
            Ok(iter) => {
                for addr in iter {
                    out.push(addr);
                }
            }
            Err(e) => {
                eprintln!("[dns_seed] failed to resolve {}: {}", seed, e);
            }
        }
    }
    out
}

/// Resolve a single DNS seed.
pub async fn resolve_one(seed: &str) -> Vec<SocketAddr> {
    match tokio::net::lookup_host(seed).await {
        Ok(iter) => iter.collect(),
        Err(e) => {
            eprintln!("[dns_seed] failed to resolve {}: {}", seed, e);
            Vec::new()
        }
    }
}
