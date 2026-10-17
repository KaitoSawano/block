// Copyright (c) 2026 Aldianokto
// Copyright (c) 2026-present Zenachain Core developers
// Distributed under the MIT software license, see the accompanying
// file LICENSE or https://opensource.org/license/mit.

use std::net::SocketAddr;

/// Daftar DNS seed ZenaChain.
/// Isi dengan hostname nyata saat sudah di-deploy.
pub const DNS_SEEDS: &[&str] = &[];

/// Query semua DNS seed dan kembalikan daftar SocketAddr.
pub async fn resolve_seeds() -> Vec<SocketAddr> {
    let mut out = Vec::new();
    for seed in DNS_SEEDS {
        match tokio::net::lookup_host(seed).await {
            Ok(iter) => {
                for addr in iter {
                    out.push(addr);
                }
            }
            Err(e) => {
                eprintln!("[dns_seed] gagal resolve {}: {}", seed, e);
            }
        }
    }
    out
}

/// Query DNS seed tertentu.
pub async fn resolve_one(seed: &str) -> Vec<SocketAddr> {
    match tokio::net::lookup_host(seed).await {
        Ok(iter) => iter.collect(),
        Err(e) => {
            eprintln!("[dns_seed] gagal resolve {}: {}", seed, e);
            Vec::new()
        }
    }
}
