use std::collections::{HashSet, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
//
// This variant composes three orthogonal improvements:
//   1. small_size = capacity/40 (H02) — even smaller admission queue gives
//      main queue ~97.5 % of capacity instead of 95 %.
//   2. Parallel freq deques (from v003) — freq removed from Entry<K,V> to
//      shrink it from 24 to 16 bytes; stored in sibling small_freq/main_freq
//      VecDeques indexed in lock-step with small/main.
//   3. O(1) ghost membership + lazy deletion (from v002 extended) — ghost_set
//      HashSet replaces the O(n) iter().any(); ghost hits remove only from
//      ghost_set (O(1)); the stale key remains in the deque and is cleaned up
//      lazily in the capacity guard inside evict_small().
const MAX_FREQ: u8 = 3;

/// Entry stores only key and value; frequency is kept in a parallel deque.
#[derive(Debug)]
pub struct Entry<K, V> {
    key: K,
    value: V,
}

impl<K, V> Entry<K, V> {
    fn new(key: K, value: V) -> Self {
        Self { key, value }
    }
}

#[derive(Debug)]
pub struct S3Fifo<K: PartialEq + Eq + Hash + Clone, V> {
    /// Small admission FIFO queue (newest at front, oldest at back).
    small: VecDeque<Entry<K, V>>,
    /// Frequency values parallel to `small` (small_freq[i] ↔ small[i]).
    small_freq: VecDeque<u8>,
    /// Main resident FIFO queue.
    main: VecDeque<Entry<K, V>>,
    /// Frequency values parallel to `main` (main_freq[i] ↔ main[i]).
    main_freq: VecDeque<u8>,
    /// Ghost deque of recently evicted keys; may contain stale entries after
    /// lazy deletion.
    ghost: VecDeque<K>,
    /// Set of keys currently live in the ghost deque (O(1) membership test).
    /// ghost_set.len() <= ghost.len() always holds.
    ghost_set: HashSet<K>,
    small_size: usize,
    main_size: usize,
}

impl<K: PartialEq + Eq + Hash + Clone, V> S3Fifo<K, V> {
    pub fn try_new(capacity: usize) -> Option<Self> {
        if capacity == 0 {
            log::error!("S3Fifo.try_new: capacity must be > 0");
            return None;
        }
        // H02 change: use 1/40 instead of 1/20 for the small queue, giving
        // main queue ~97.5 % of capacity on Zipfian workloads.
        let small_size = if capacity == 1 {
            1
        } else {
            (capacity / 40).max(1).min(capacity - 1)
        };
        let main_size = capacity - small_size;
        Some(Self {
            small: VecDeque::with_capacity(small_size),
            small_freq: VecDeque::with_capacity(small_size),
            main: VecDeque::with_capacity(main_size),
            main_freq: VecDeque::with_capacity(main_size),
            ghost: VecDeque::with_capacity(main_size),
            ghost_set: HashSet::with_capacity(main_size),
            small_size,
            main_size,
        })
    }

    pub fn new(capacity: usize) -> Self {
        Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
    }

    /// Look up a key and increment its frequency.
    /// Uses index-based position lookup so that the parallel freq deque can be
    /// updated without needing freq stored inside Entry.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        // Search small first.
        if let Some(pos) = self.small.iter().position(|e| &e.key == key) {
            let f = &mut self.small_freq[pos];
            *f = f.saturating_add(1).min(MAX_FREQ);
            return Some(&self.small[pos].value);
        }
        // Then main.
        if let Some(pos) = self.main.iter().position(|e| &e.key == key) {
            let f = &mut self.main_freq[pos];
            *f = f.saturating_add(1).min(MAX_FREQ);
            return Some(&self.main[pos].value);
        }
        None
    }

    pub fn put(&mut self, key: K, value: V) {
        // Update existing resident entry (value-update path).
        if let Some(pos) = self.small.iter().position(|e| e.key == key) {
            self.small[pos].value = value;
            return;
        }
        if let Some(pos) = self.main.iter().position(|e| e.key == key) {
            self.main[pos].value = value;
            return;
        }

        // Ghost hit: O(1) via ghost_set; lazy deletion — stale key stays in
        // the ghost deque and is cleaned up by the capacity guard later.
        if self.ghost_set.contains(&key) {
            self.ghost_set.remove(&key);
            if self.main.len() >= self.main_size {
                self.evict_main();
            }
            self.main.push_front(Entry::new(key, value));
            self.main_freq.push_front(0);
            return;
        }

        // Normal admission into the small queue.
        if self.small.len() >= self.small_size {
            self.evict_small();
        }
        self.small.push_front(Entry::new(key, value));
        self.small_freq.push_front(0);
    }

    /// Evict from the main queue: second-chance via frequency decrement.
    fn evict_main(&mut self) {
        while let Some(tail) = self.main.pop_back() {
            let freq = self.main_freq.pop_back().unwrap_or(0);
            if freq > 0 {
                // Give this entry a second chance: decrement freq, re-admit at front.
                self.main.push_front(tail);
                self.main_freq.push_front(freq - 1);
            } else {
                // freq == 0: evict for real.
                break;
            }
        }
    }

    /// Evict from the small queue: promote to main if freq > 1, else ghost.
    fn evict_small(&mut self) {
        if let Some(tail) = self.small.pop_back() {
            let freq = self.small_freq.pop_back().unwrap_or(0);
            if freq > 1 {
                // Hot entry: promote to main.
                if self.main.len() >= self.main_size {
                    self.evict_main();
                }
                self.main.push_front(tail);
                self.main_freq.push_front(freq);
            } else {
                // Cold entry: move key to ghost.
                // Capacity guard with lazy deletion:
                //   Pop from the back of the ghost deque while it is at capacity.
                //   If the popped key is still in ghost_set it is a live ghost —
                //   remove it from ghost_set (normal aged eviction) and break.
                //   If not, it was already lazily deleted; continue popping
                //   (the effective ghost count did not change).
                while self.ghost.len() >= self.main_size {
                    if let Some(evicted_key) = self.ghost.pop_back() {
                        if self.ghost_set.contains(&evicted_key) {
                            self.ghost_set.remove(&evicted_key);
                            break; // evicted one real ghost entry
                        }
                        // stale (lazily deleted): loop again
                    }
                }
                let k = tail.key;
                self.ghost.push_front(k.clone());
                self.ghost_set.insert(k);
            }
        }
    }
}

impl<K: PartialEq + Eq + Hash + Clone + Debug, V: Debug> framework::Cache<K, V>
    for S3Fifo<K, V>
{
    fn put(&mut self, key: K, value: V) {
        S3Fifo::put(self, key, value);
    }

    fn get(&mut self, key: &K) -> Option<&V> {
        S3Fifo::get(self, key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_miss_returns_none() {
        let mut cache = S3Fifo::<i64, i64>::new(4);
        assert!(cache.get(&42).is_none());
    }

    #[test]
    fn put_then_get_finds_value() {
        let mut cache = S3Fifo::new(4);
        cache.put(1, 10);
        assert_eq!(*cache.get(&1).unwrap(), 10);
    }

    #[test]
    fn put_existing_updates_value() {
        let mut cache = S3Fifo::new(4);
        cache.put(1, 10);
        cache.put(1, 20);
        assert_eq!(*cache.get(&1).unwrap(), 20);
    }

    #[test]
    fn small_eviction_promotes_to_main_after_multiple_hits() {
        let mut cache = S3Fifo::new(4);
        cache.put(1, 10);
        cache.get(&1);
        cache.get(&1);
        cache.put(2, 20);
        assert!(cache.main.iter().any(|entry| entry.key == 1));
    }

    #[test]
    fn ghost_hit_promotes_back_to_main() {
        let mut cache = S3Fifo::new(4);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30);
        cache.put(4, 40);
        cache.put(1, 10);
        assert!(cache.main.iter().any(|entry| entry.key == 1));
    }

    /// White-box test for second-chance eviction in main.
    /// Adapted for the parallel-freq-deque layout: freq values are pushed
    /// alongside their entries into the sibling deques.
    #[test]
    fn main_eviction_uses_frequency_second_chance() {
        let mut cache = S3Fifo::new(5);
        // Push entries directly into main and the parallel freq deque.
        // Entries are push_back'd so back == newest from pop_back perspective.
        cache.main.push_back(Entry::new(1, 10));
        cache.main_freq.push_back(0);
        cache.main.push_back(Entry::new(2, 20));
        cache.main_freq.push_back(2);
        cache.main.push_back(Entry::new(3, 30));
        cache.main_freq.push_back(0);
        cache.main.push_back(Entry::new(4, 40));
        cache.main_freq.push_back(1);
        // Seed small with a hot entry so evict_small triggers evict_main.
        cache.small.push_front(Entry::new(5, 50));
        cache.small_freq.push_front(2);
        cache.evict_small();
        // Entry 4 had freq=1 → gets a second chance (freq decremented to 0,
        //   moved to front of main), so it survives.
        assert!(cache.main.iter().any(|entry| entry.key == 4));
        // Entry 3 had freq=0 → evicted immediately (no second chance).
        assert!(!cache.main.iter().any(|entry| entry.key == 3));
    }

    #[test]
    fn capacity_bound_never_exceeded() {
        let mut cache = S3Fifo::new(4);
        for key in 0..20 {
            cache.put(key, key);
        }
        assert!(cache.small.len() + cache.main.len() <= 4);
    }

    #[test]
    fn repeated_get_clamps_frequency() {
        let mut cache = S3Fifo::new(4);
        cache.put(1, 10);
        for _ in 0..10 {
            assert_eq!(*cache.get(&1).unwrap(), 10);
        }
    }

    /// Lazy ghost deletion must not desync ghost_set and ghost deque:
    /// ghost_set.len() <= ghost.len() must hold after ghost-hit scenarios.
    #[test]
    fn ghost_set_len_bounded_after_ghost_hit() {
        // capacity=4 → small_size=1, main_size=3
        let mut cache = S3Fifo::<i64, i64>::new(4);
        // Fill ghost with entries 1, 2, 3 by pushing 4 entries through small.
        cache.put(1, 10); // small=[1]
        cache.put(2, 20); // evicts 1 → ghost=[1]
        cache.put(3, 30); // evicts 2 → ghost=[2,1]
        cache.put(4, 40); // evicts 3 → ghost=[3,2,1]
        // Ghost hit on 1: removes 1 from ghost_set (lazy), stays in deque.
        cache.put(1, 10);
        // ghost_set = {2, 3}, ghost deque = [3,2,1(stale)]
        assert!(
            cache.ghost_set.len() <= cache.ghost.len(),
            "ghost_set.len() ({}) must not exceed ghost.len() ({})",
            cache.ghost_set.len(),
            cache.ghost.len()
        );
        // Also verify the ghost hit actually promoted key 1 to main.
        assert!(cache.main.iter().any(|e| e.key == 1));
    }
}
