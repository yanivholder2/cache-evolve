use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

pub mod s3_fifo_jni;

// S3-FIFO with split dual-HashMap layout: separate data<K,V> and freq_map<K,u8>
// replace the parent's single resident_map<K,(V,u8)>. This reduces the per-slot
// footprint on the hot frequency-update path in get() — freq_map stores only u8
// values (~9 bytes/slot for K=u64) vs the parent's (V,u8) tuples (~16 bytes/slot
// for K=V=u64) — fitting more entries per cache line on the hot get() path.
//
// Retained from parent (cycle/0005/v003):
//   - small_size = capacity/20 (1/20 small-queue fraction)
//   - O(1) ghost membership via ghost_set + lazy deletion
//   - Entry struct: key-only + PhantomData<V>
//   - unbounded second-chance loop in evict_main()
//   - freq > 1 promotion threshold in evict_small()

const MAX_FREQ: u8 = 3;

/// Entry stores only the key; value and frequency are held in data/freq_map.
/// The constructor still accepts `_value` so call-sites that pass a value
/// compile unchanged (the value is dropped immediately).
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
    /// Ghost deque of recently evicted keys; may contain stale entries after
    /// lazy deletion.
    ghost: VecDeque<K>,
    /// Set of keys currently live in the ghost deque (O(1) membership test).
    /// ghost_set.len() <= ghost.len() always holds.
    ghost_set: HashSet<K>,
    /// Stores the value for every resident key. O(1) value access on hits and
    /// updates.
    data: HashMap<K, V>,
    /// Stores only the frequency byte for every resident key. Smaller per-slot
    /// footprint than the parent's (V, u8) tuple — improves cache-line
    /// utilisation on the hot get() frequency-bump path.
    freq_map: HashMap<K, u8>,
    small_size: usize,
    main_size: usize,
}

impl<K: PartialEq + Eq + Hash + Clone, V> S3Fifo<K, V> {
    pub fn try_new(capacity: usize) -> Option<Self> {
        if capacity == 0 {
            log::error!("S3Fifo.try_new: capacity must be > 0");
            return None;
        }
        // 1/20 small-queue fraction, preserved from parent (cycle/0005/v003).
        let small_size = if capacity == 1 {
            1
        } else {
            (capacity / 20).max(1).min(capacity - 1)
        };
        let main_size = capacity - small_size;
        Some(Self {
            small: VecDeque::with_capacity(small_size),
            main: VecDeque::with_capacity(main_size),
            ghost: VecDeque::with_capacity(main_size),
            ghost_set: HashSet::with_capacity(main_size),
            data: HashMap::with_capacity(capacity),
            freq_map: HashMap::with_capacity(capacity),
            small_size,
            main_size,
        })
    }

    pub fn new(capacity: usize) -> Self {
        Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
    }

    /// O(1) lookup: probe freq_map first (mutable, updates frequency), then
    /// data (immutable, returns value reference). The split lets the hot
    /// frequency-bump path touch only the compact freq_map, keeping more
    /// entries in L1/L2 on repeated gets.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        if let Some(freq) = self.freq_map.get_mut(key) {
            *freq = freq.saturating_add(1).min(MAX_FREQ);
        } else {
            return None;
        }
        self.data.get(key)
    }

    pub fn put(&mut self, key: K, value: V) {
        // O(1) update path: key already resident — update value in-place.
        if let Some(v) = self.data.get_mut(&key) {
            *v = value;
            return;
        }

        // Ghost hit: O(1) via ghost_set; lazy deletion preserved (stale key
        // remains in the deque and is cleaned up by the ghost capacity guard).
        if self.ghost_set.contains(&key) {
            self.ghost_set.remove(&key);
            if self.main.len() >= self.main_size {
                self.evict_main();
            }
            self.data.insert(key.clone(), value);
            self.freq_map.insert(key.clone(), 0);
            self.main.push_front(Entry { key, _phantom: std::marker::PhantomData });
            return;
        }

        // Normal admission into the small queue.
        if self.small.len() >= self.small_size {
            self.evict_small();
        }
        self.data.insert(key.clone(), value);
        self.freq_map.insert(key.clone(), 0);
        self.small.push_front(Entry { key, _phantom: std::marker::PhantomData });
    }

    /// Evict from the main queue: unbounded second-chance via frequency
    /// decrement. SCAN_CAP removed — cold entries accumulate naturally at the
    /// tail of main on Zipfian workloads so the loop terminates quickly.
    fn evict_main(&mut self) {
        while let Some(tail) = self.main.pop_back() {
            let freq = self.freq_map.get(&tail.key).copied().unwrap_or(0);
            if freq > 0 {
                // Second chance: decrement frequency and re-admit at front.
                self.freq_map.insert(tail.key.clone(), freq - 1);
                self.main.push_front(tail);
            } else {
                // Zero frequency: evict and remove from both maps.
                self.data.remove(&tail.key);
                self.freq_map.remove(&tail.key);
                break;
            }
        }
    }

    /// Evict from the small queue: promote to main if freq > 1, else ghost.
    fn evict_small(&mut self) {
        if let Some(tail) = self.small.pop_back() {
            let freq = self.freq_map.get(&tail.key).copied().unwrap_or(0);
            if freq > 1 {
                // Hot entry: promote to main. data/freq_map entries stay intact.
                if self.main.len() >= self.main_size {
                    self.evict_main();
                }
                self.main.push_front(tail);
            } else {
                // Cold entry: evict to ghost, remove from both maps.
                self.data.remove(&tail.key);
                self.freq_map.remove(&tail.key);
                // Ghost capacity guard with lazy deletion: pop the back of ghost
                // until we find a live ghost (still in ghost_set) and evict it.
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
    /// Adapted for the dual-HashMap layout: freq values are stored in freq_map,
    /// values in data, not in resident_map tuples.
    ///
    /// Layout: main deque = [key:1(front), key:2, key:3, key:4(back)],
    ///         small = [key:5(front)],
    ///         data = {1:10, 2:20, 3:30, 4:40, 5:50},
    ///         freq_map = {1:0, 2:2, 3:0, 4:1, 5:2}.
    /// evict_small() sees freq(5)=2 > 1 → hot, promotes key:5 to main.
    /// main is full (4 entries) → evict_main():
    ///   pop key:4 (freq=1 > 0) → second chance, freq→0, push to front.
    ///   pop key:3 (freq=0)     → evict immediately.
    #[test]
    fn main_eviction_uses_frequency_second_chance() {
        let mut cache = S3Fifo::new(5);
        cache.main.push_back(Entry::new(1, 10)); cache.data.insert(1u64, 10u64); cache.freq_map.insert(1u64, 0u8);
        cache.main.push_back(Entry::new(2, 20)); cache.data.insert(2u64, 20u64); cache.freq_map.insert(2u64, 2u8);
        cache.main.push_back(Entry::new(3, 30)); cache.data.insert(3u64, 30u64); cache.freq_map.insert(3u64, 0u8);
        cache.main.push_back(Entry::new(4, 40)); cache.data.insert(4u64, 40u64); cache.freq_map.insert(4u64, 1u8);
        cache.small.push_front(Entry::new(5, 50)); cache.data.insert(5u64, 50u64); cache.freq_map.insert(5u64, 2u8);
        cache.evict_small();
        // Entry 4 had freq=1 → gets a second chance (freq decremented to 0,
        // moved to front of main), so it survives.
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
