use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
//
// This variant builds on cycle/0007/v003 and collapses the three separate maps
// (data, freq_map, ghost_set) into a single unified state_map:
//   state_map: HashMap<K, Option<(V, u8)>>
//   - Some((value, freq)) → key is resident (in small or main queue)
//   - None               → key is a ghost (evicted from small, awaiting promotion)
//   - absent             → cold (never seen, or fully expired ghost)
//
// This single map replaces the two-probe (data.contains_key + ghost_set.contains)
// pattern on every put() with a single lookup, and eliminates ghost_set's
// per-put remove() on ghost hits. The ghost VecDeque is retained for
// age-ordered eviction with lazy deletion.
//
// Invariants preserved from v003:
//   1. small_size = capacity/40 — even smaller admission queue.
//   2. Lazy deletion — ghost deque may contain stale entries (re-admitted keys).
//   3. MAX_FREQ = 5 (raised second-chance window for hot entries).
//   4. evict_main() unbounded second-chance loop.
//   5. evict_small() freq>1 promotion threshold.
const MAX_FREQ: u8 = 5;

/// Queue entry — stores only the key; value and frequency live in state_map.
/// `Entry::new(key, _value)` accepts but discards the value argument so
/// white-box test call-sites compile unchanged.
#[derive(Debug)]
pub struct Entry<K, V> {
    key: K,
    _phantom: std::marker::PhantomData<V>,
}

impl<K, V> Entry<K, V> {
    fn new(key: K, _value: V) -> Self {
        Self { key, _phantom: std::marker::PhantomData }
    }
}

#[derive(Debug)]
pub struct S3Fifo<K: PartialEq + Eq + Hash + Clone, V> {
    /// Small admission FIFO queue (newest at front, oldest at back).
    small: VecDeque<Entry<K, V>>,
    /// Main resident FIFO queue.
    main: VecDeque<Entry<K, V>>,
    /// Unified state map:
    ///   Some((v, f)) → resident key with value v and access frequency f
    ///   None         → ghost key (value evicted, slot retained for ghost-hit detection)
    ///   absent       → cold (never inserted or ghost slot fully expired)
    state_map: HashMap<K, Option<(V, u8)>>,
    /// Ghost deque for age-ordered expiry of ghost slots; may contain stale
    /// entries (keys re-admitted via ghost hit) that are cleaned up lazily.
    ghost: VecDeque<K>,
    small_size: usize,
    main_size: usize,
}

impl<K: PartialEq + Eq + Hash + Clone, V> S3Fifo<K, V> {
    pub fn try_new(capacity: usize) -> Option<Self> {
        if capacity == 0 {
            log::error!("S3Fifo.try_new: capacity must be > 0");
            return None;
        }
        // small_size = capacity/40, clamped to [1, capacity-1]
        let small_size = if capacity == 1 {
            1
        } else {
            (capacity / 40).max(1).min(capacity - 1)
        };
        let main_size = capacity - small_size;
        Some(Self {
            small: VecDeque::with_capacity(small_size),
            main: VecDeque::with_capacity(main_size),
            state_map: HashMap::with_capacity(capacity),
            ghost: VecDeque::with_capacity(main_size),
            small_size,
            main_size,
        })
    }

    pub fn new(capacity: usize) -> Self {
        Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
    }

    /// Look up a key, increment its frequency, and return its value.
    /// Single probe via state_map: one get_mut resolves both the resident check
    /// and the frequency update before returning the value reference.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        match self.state_map.get_mut(key) {
            Some(Some((v, f))) => {
                // Resident: update frequency and return value reference.
                *f = f.saturating_add(1).min(MAX_FREQ);
                // Coerce &mut V → &V; lifetime is tied to &mut self.
                Some(v)
            }
            // None (ghost) or absent (cold): not accessible.
            _ => None,
        }
    }

    pub fn put(&mut self, key: K, value: V) {
        // Single probe to classify state.
        let state = match self.state_map.get(&key) {
            Some(Some(_)) => 0, // resident
            Some(None) => 1,    // ghost
            None => 2,          // cold
        };

        match state {
            0 => {
                // Resident update: overwrite value in place, no queue change.
                if let Some(Some((v, _))) = self.state_map.get_mut(&key) {
                    *v = value;
                }
            }
            1 => {
                // Ghost hit: convert None slot to resident, admit to main queue.
                // No ghost_set.remove() needed — the slot is overwritten in place.
                self.state_map.insert(key.clone(), Some((value, 0)));
                if self.main.len() >= self.main_size {
                    self.evict_main();
                }
                self.main.push_front(Entry { key, _phantom: std::marker::PhantomData });
            }
            _ => {
                // Cold admission into the small queue.
                if self.small.len() >= self.small_size {
                    self.evict_small();
                }
                self.state_map.insert(key.clone(), Some((value, 0)));
                self.small.push_front(Entry { key, _phantom: std::marker::PhantomData });
            }
        }
    }

    /// Evict from main: second-chance via frequency decrement.
    /// Each iteration reads state_map for the tail's freq; if > 0, decrements
    /// and re-admits to front; if 0, removes from state_map and breaks.
    fn evict_main(&mut self) {
        while let Some(tail) = self.main.pop_back() {
            let freq = match self.state_map.get(&tail.key) {
                Some(Some((_, f))) => *f,
                _ => 0,
            };
            if freq > 0 {
                // Second chance: decrement frequency, re-admit at front.
                if let Some(Some((_, f))) = self.state_map.get_mut(&tail.key) {
                    *f -= 1;
                }
                self.main.push_front(tail);
            } else {
                // freq == 0: evict for real; remove from state_map entirely.
                self.state_map.remove(&tail.key);
                break;
            }
        }
    }

    /// Evict from small: promote to main if freq > 1, else convert to ghost.
    fn evict_small(&mut self) {
        if let Some(tail) = self.small.pop_back() {
            let freq = match self.state_map.get(&tail.key) {
                Some(Some((_, f))) => *f,
                _ => 0,
            };
            if freq > 1 {
                // Hot entry: promote to main; state_map slot stays as Some((v, f)).
                if self.main.len() >= self.main_size {
                    self.evict_main();
                }
                self.main.push_front(tail);
            } else {
                // Cold entry: convert resident slot to ghost slot in place.
                // state_map[tail.key]: Some((v,f)) → None (drops value).
                self.state_map.insert(tail.key.clone(), None);

                // Ghost deque capacity guard with lazy deletion:
                // While the ghost deque is at capacity, pop from the back.
                // If the popped key still maps to None (live ghost), remove it
                // from state_map (aged expiry) and break.
                // If the popped key is absent or maps to Some (re-admitted lazily),
                // it is stale — skip it and keep popping.
                while self.ghost.len() >= self.main_size {
                    if let Some(old_key) = self.ghost.pop_back() {
                        if matches!(self.state_map.get(&old_key), Some(None)) {
                            // Live ghost: expire it.
                            self.state_map.remove(&old_key);
                            break;
                        }
                        // Stale (re-admitted or already removed): loop again.
                    }
                }
                self.ghost.push_front(tail.key);
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
        // capacity=4 → small_size=1, main_size=3
        let mut cache = S3Fifo::new(4);
        cache.put(1, 10);
        cache.get(&1); // freq → 1
        cache.get(&1); // freq → 2
        // put(2) triggers evict_small; key 1 has freq=2 > 1 → promoted to main.
        cache.put(2, 20);
        assert!(cache.main.iter().any(|entry| entry.key == 1));
    }

    #[test]
    fn ghost_hit_promotes_back_to_main() {
        // capacity=4 → small_size=1, main_size=3
        let mut cache = S3Fifo::new(4);
        cache.put(1, 10); // small=[1]
        cache.put(2, 20); // evicts 1 → ghost; small=[2]
        cache.put(3, 30); // evicts 2 → ghost; small=[3]
        cache.put(4, 40); // evicts 3 → ghost; small=[4]
        // key 1 is now a ghost (state_map[1] = None).
        cache.put(1, 10); // ghost hit → main=[1]
        assert!(cache.main.iter().any(|entry| entry.key == 1));
    }

    /// White-box test for second-chance eviction in main.
    /// Updated for state_map layout: insert Some((value, freq)) alongside queue entries.
    #[test]
    fn main_eviction_uses_frequency_second_chance() {
        let mut cache = S3Fifo::<u64, u64>::new(5);
        // capacity=5 → small_size=1, main_size=4.
        // Populate main queue and state_map directly.
        cache.main.push_back(Entry::new(1, 10)); cache.state_map.insert(1u64, Some((10u64, 0u8)));
        cache.main.push_back(Entry::new(2, 20)); cache.state_map.insert(2u64, Some((20u64, 2u8)));
        cache.main.push_back(Entry::new(3, 30)); cache.state_map.insert(3u64, Some((30u64, 0u8)));
        cache.main.push_back(Entry::new(4, 40)); cache.state_map.insert(4u64, Some((40u64, 1u8)));
        // Seed small with a hot entry (freq=2) to trigger evict_small → evict_main.
        cache.small.push_front(Entry::new(5, 50)); cache.state_map.insert(5u64, Some((50u64, 2u8)));
        cache.evict_small();
        // Entry 4 had freq=1 → second chance (freq decremented to 0, moved to front);
        // it survives evict_main.
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

    /// The number of live ghost slots (state_map values that are None) must
    /// not exceed ghost.len(). The ghost deque may additionally hold stale
    /// entries (keys that were re-admitted via a ghost hit) that are cleaned
    /// up lazily, so ghost.len() >= live_ghost_count always holds.
    #[test]
    fn ghost_set_len_bounded_after_ghost_hit() {
        // capacity=4 → small_size=1, main_size=3
        let mut cache = S3Fifo::<i64, i64>::new(4);
        // Push four keys through the small queue to populate the ghost deque.
        cache.put(1, 10); // small=[1]
        cache.put(2, 20); // evicts 1 → ghost=[1]; small=[2]
        cache.put(3, 30); // evicts 2 → ghost=[2,1]; small=[3]
        cache.put(4, 40); // evicts 3 → ghost=[3,2,1]; small=[4]
        // Ghost hit on key 1: state_map[1] = None → Some((10,0));
        // the stale deque entry for 1 remains in ghost (lazy deletion).
        cache.put(1, 10); // main=[1]; ghost=[3,2,1(stale)]
        // live_ghost_count = count of None values in state_map = {2,3} → 2
        // ghost.len() = 3 (including stale entry for 1)
        let live_ghost_count = cache.state_map.values().filter(|v| v.is_none()).count();
        assert!(
            live_ghost_count <= cache.ghost.len(),
            "live ghost count ({}) must not exceed ghost.len() ({})",
            live_ghost_count,
            cache.ghost.len()
        );
        // Also verify the ghost hit promoted key 1 to main.
        assert!(cache.main.iter().any(|e| e.key == 1));
    }
}
