//! Cache fixture for the microbench harness.
//!
//! This is a **fixture, not the cache under test.** It exists so the per-op-time
//! plugin → bench → JSON → FitnessVector → Pareto pipeline is exercisable before
//! any real variant exists. When variants land, the fixture is replaced (or
//! shadowed under a build flag) by the variant's cache; the measurement code in
//! `main.rs` stays unchanged.
//!
//! Chosen properties: std-only, deterministic, exercises both hit/miss and
//! insert/evict paths so the harness emits realistic p50/p99 numbers.

use std::collections::{HashMap, VecDeque};

pub struct CacheFixture {
    cap: usize,
    order: VecDeque<u64>,
    store: HashMap<u64, u64>,
}

impl CacheFixture {
    /// New empty cache bounded to `cap` entries.
    pub fn new(cap: usize) -> Self {
        Self {
            cap,
            order: VecDeque::with_capacity(cap),
            store: HashMap::new(),
        }
    }

    /// Lookup `key`. On hit, promote it to MRU (back of `order`) — this is what
    /// makes the policy LRU rather than FIFO. `O(n)` due to `retain`; acceptable
    /// at the bench's small capacity and keeps the fixture dependency-free.
    pub fn get(&mut self, key: u64) -> Option<u64> {
        if let Some(&v) = self.store.get(&key) {
            self.order.retain(|&k| k != key);
            self.order.push_back(key);
            Some(v)
        } else {
            None
        }
    }

    /// Insert or update `key`. On update, refresh MRU position. On insert at
    /// capacity, evict the LRU entry (front of `order`) before inserting.
    pub fn put(&mut self, key: u64, value: u64) {
        if self.store.contains_key(&key) {
            self.order.retain(|&k| k != key);
        } else if self.store.len() >= self.cap {
            if let Some(evicted) = self.order.pop_front() {
                self.store.remove(&evicted);
            }
        }
        self.store.insert(key, value);
        self.order.push_back(key);
    }
}
