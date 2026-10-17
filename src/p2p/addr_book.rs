//! Buku alamat peer (AddrMan sederhana ala Bitcoin).

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

/// Satu entri alamat peer.
#[derive(Debug, Clone)]
pub struct AddrEntry {
    pub addr: SocketAddr,
    pub last_seen: Instant,
    pub last_try: Option<Instant>,
    pub attempts: u32,
}

/// Buku alamat peer.
#[derive(Debug, Default)]
pub struct AddrBook {
    entries: HashMap<SocketAddr, AddrEntry>,
    max_entries: usize,
}

impl AddrBook {
    pub fn new(max_entries: usize) -> Self {
        AddrBook {
            entries: HashMap::new(),
            max_entries,
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Tambah alamat. Return `true` jika baru.
    pub fn add(&mut self, addr: SocketAddr) -> bool {
        if self.entries.contains_key(&addr) {
            if let Some(entry) = self.entries.get_mut(&addr) {
                entry.last_seen = Instant::now();
            }
            return false;
        }

        if self.entries.len() >= self.max_entries {
            self.evict_oldest();
        }

        self.entries.insert(
            addr,
            AddrEntry {
                addr,
                last_seen: Instant::now(),
                last_try: None,
                attempts: 0,
            },
        );
        true
    }

    pub fn add_many(&mut self, addrs: &[SocketAddr]) -> usize {
        let mut added = 0;
        for addr in addrs {
            if self.add(*addr) {
                added += 1;
            }
        }
        added
    }

    /// Tandai alamat sudah dicoba.
    pub fn mark_tried(&mut self, addr: &SocketAddr) {
        if let Some(entry) = self.entries.get_mut(addr) {
            entry.last_try = Some(Instant::now());
            entry.attempts = entry.attempts.saturating_add(1);
        }
    }

    /// Ambil alamat yang belum dicoba atau sudah lama tidak dicoba.
    pub fn candidates(&self, min_interval: Duration, limit: usize) -> Vec<SocketAddr> {
        let now = Instant::now();
        let mut out: Vec<SocketAddr> = self
            .entries
            .values()
            .filter(|e| match e.last_try {
                None => true,
                Some(t) => now.duration_since(t) > min_interval,
            })
            .map(|e| e.addr)
            .collect();

        out.sort_by_key(|addr| {
            self.entries
                .get(addr)
                .and_then(|e| e.last_try)
                .map(|t| now.duration_since(t))
                .unwrap_or(Duration::MAX)
        });

        out.truncate(limit);
        out
    }

    /// Hapus entri yang sudah lama tidak terlihat.
    pub fn prune(&mut self, max_age: Duration) {
        let now = Instant::now();
        self.entries
            .retain(|_, e| now.duration_since(e.last_seen) <= max_age);
    }

    fn evict_oldest(&mut self) {
        if let Some(oldest) = self
            .entries
            .iter()
            .min_by_key(|(_, e)| e.last_seen)
            .map(|(addr, _)| *addr)
        {
            self.entries.remove(&oldest);
        }
    }

    pub fn all_addrs(&self) -> Vec<SocketAddr> {
        self.entries.keys().copied().collect()
    }
}
