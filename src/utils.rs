// Copyright (c) 2026 Aldianokto
// Distributed under the Apache License, see the accompanying
// file LICENSE or https://www.apache.org/licenses/LICENSE-2.0.

//! Utility functions shared across the crate.

use rand::RngCore;
use serde::{Deserialize, Deserializer, Serializer};

/// Return the current Unix timestamp in milliseconds.
pub fn now_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

/// Convert an amount in satoshis to a ZENAC string with eight decimal places.
pub fn satoshi_to_zenac(satoshi: u64) -> String {
    let whole = satoshi / 100_000_000;
    let frac = satoshi % 100_000_000;
    format!("{}.{:08}", whole, frac)
}

/// Convert an amount in ZENAC to satoshis.
pub fn zenac_to_satoshi(zenac: f64) -> u64 {
    (zenac * 100_000_000.0) as u64
}

/// Format a Unix timestamp in milliseconds as "YYYY-MM-DD HH:MM:SS" in UTC.
pub fn format_datetime(millis: u128) -> String {
    let secs = (millis / 1000) as i64;
    let (year, month, day, hour, minute, second) = civil_from_unix(secs);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hour, minute, second
    )
}

fn civil_from_unix(secs: i64) -> (i64, u32, u32, u32, u32, u32) {
    let days = secs.div_euclid(86_400);
    let secs_of_day = secs.rem_euclid(86_400);

    let hour = (secs_of_day / 3600) as u32;
    let minute = ((secs_of_day % 3600) / 60) as u32;
    let second = (secs_of_day % 60) as u32;

    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = if m <= 2 { y + 1 } else { y };

    (year, m, d, hour, minute, second)
}

/// Generate a random hex string of `n` bytes.
pub fn random_hex(n: usize) -> String {
    let mut buf = vec![0u8; n];
    rand::thread_rng().fill_bytes(&mut buf);
    hex::encode(buf)
}

/// Convert a hex string to bytes.
pub fn hex_to_bytes(hex_str: &str) -> Result<Vec<u8>, String> {
    hex::decode(hex_str).map_err(|e| e.to_string())
}

/// Serialize a `u128` as a decimal string in JSON.
pub mod u128_string {
    use super::*;

    pub fn serialize<S>(value: &u128, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u128, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<u128>().map_err(serde::de::Error::custom)
    }
}
