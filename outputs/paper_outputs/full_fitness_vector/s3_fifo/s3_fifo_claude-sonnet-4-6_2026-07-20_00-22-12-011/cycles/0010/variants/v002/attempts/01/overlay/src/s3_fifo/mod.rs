use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
//
// This variant (cycle/0010/H02) adds a compact 512-byte admission sketch on top
// of cycle/0007/v003. First-time-seen cold keys are rejected from the small queue
// (sketch count == 0 → set to 1, return early). On subsequent puts the key is
// admitted normally. The ghost-hit path bypasses the sketch entirely so that
// re-accessed (ghost-promoted) keys are never filtered. The sketch is reset after
// every main_size evictions from small to prevent counter saturation and adapt to
// workload shifts.
//
// All other v003 properties are preserved:
//   1. small_size = capacity/40 — tight admission queue.
//   2. O(1) ghost membership + lazy deletion via ghost_set HashSet.
//   3. MAX_FREQ = 5 second-chance window in main.
//   4. evict_main() unbounded second-chance loop.
//   5. evict_small() freq>1 promotion threshold.
//   6. O(1) lookups via side HashMaps (data, freq_map).
const MAX_FREQ: u8 = 5;

/// Entry stores only the key; value and frequency are kept in side HashMaps.
/// The constructor still accepts a value argument (ignored) so white-box test
/// call-sites compile unchanged.
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

/// Minimal inline hasher for the admission sketch.  Uses a cheap LCG-style mix
/// so we avoid pulling in a full `RandomState` hasher.
struct MixHasher(u64);

impl Hasher for MixHasher {
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = self.0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(b as u64 ^ 1442695040888963407);
        }
    }
    fn finish(&self) -> u64 {
        self.0
    }
}

#[derive(Debug)]
pub struct S3Fifo<K: PartialEq + Eq + Hash + Clone, V> {
    /// Small admission FIFO queue (newest at front, oldest at back).
    small: VecDeque<Entry<K, V>>,
    /// Main resident FIFO queue.
    main: VecDeque<Entry<K, V>>,
    /// Side map: resident key → value (O(1) access, replaces Entry.value and
    /// the parallel small_freq/main_freq VecDeques).
    data: HashMap<K, V>,
    /// Side map: resident key → frequency (O(1) access).
    freq_map: HashMap<K, u8>,
    /// Ghost deque of recently evicted keys; may contain stale entries after
    /// lazy deletion.
    ghost: VecDeque<K>,
    /// Set of keys currently live in the ghost deque (O(1) membership test).
    /// ghost_set.len() <= ghost.len() always holds.
    ghost_set: HashSet<K>,
    small_size: usize,
    main_size: usize,
    /// 512-byte admission sketch: saturating byte counters, one per hash bucket.
    /// A count of 0 means "never seen"; > 0 means "seen at least once".
    admit_sketch: Box<[u8; 512]>,
    /// Counts evictions from small since last sketch reset.
    evict_count: usize,
    /// Per-instance seed for the sketch hash; deterministic given capacity.
    sketch_seed: u64,
}

impl<K: PartialEq + Eq + Hash + Clone, V> S3Fifo<K, V> {
    pub fn try_new(capacity: usize) -> Option<Self> {
        if capacity == 0 {
            log::error!("S3Fifo.try_new: capacity must be > 0");
            return None;
        }
        let small_size = if capacity == 1 {
            1
        } else {
            (capacity / 40).max(1).min(capacity - 1)
        };
        let main_size = capacity - small_size;
        Some(Self {
            small: VecDeque::with_capacity(small_size),
            main: VecDeque::with_capacity(main_size),
            data: HashMap::with_capacity(capacity),
            freq_map: HashMap::with_capacity(capacity),
            ghost: VecDeque::with_capacity(main_size),
            ghost_set: HashSet::with_capacity(main_size),
            small_size,
            main_size,
            admit_sketch: Box::new([0u8; 512]),
            evict_count: 0,
            sketch_seed: capacity as u64 ^ 0xdeadbeefcafe1234,
        })
    }

    pub fn new(capacity: usize) -> Self {
        Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
    }

    /// Compute the sketch bucket index for a key using the per-instance seed.
    fn sketch_idx(&self, key: &K) -> usize {
        let mut h = MixHasher(self.sketch_seed);
        key.hash(&mut h);
        (h.finish() as usize) % 512
    }

    /// Look up a key and increment its frequency. O(1) via freq_map and data.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        // Update frequency first; end the mutable borrow before borrowing data.
        if let Some(freq) = self.freq_map.get_mut(key) {
            *freq = freq.saturating_add(1).min(MAX_FREQ);
        } else {
            return None;
        }
        self.data.get(key)
    }

    pub fn put(&mut self, key: K, value: V) {
        // O(1) update path: key already resident.
        if self.data.contains_key(&key) {
            self.data.insert(key, value);
            return;
        }

        // Ghost hit: O(1) via ghost_set; bypasses the admission sketch.
        // Ghost hits represent confirmed re-access and must always be admitted.
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

        // Cold admission gate: consult the sketch.
        let idx = self.sketch_idx(&key);
        let count = self.admit_sketch[idx];
        if count == 0 {
            // First time this hash bucket has been seen: record it and skip.
            // The key is not inserted into small; it will be admitted on the
            // next cold-path put (when count > 0).
            self.admit_sketch[idx] = 1;
            return;
        }
        // Seen before: saturating-increment, then fall through to normal admission.
        if count < 255 {
            self.admit_sketch[idx] = count + 1;
        }

        // Normal admission into the small queue.
        if self.small.len() >= self.small_size {
            self.evict_small();
        }
        self.data.insert(key.clone(), value);
        self.freq_map.insert(key.clone(), 0);
        self.small.push_front(Entry { key, _phantom: std::marker::PhantomData });
    }

    /// Evict from the main queue: second-chance via frequency decrement. O(1) per step.
    fn evict_main(&mut self) {
        while let Some(tail) = self.main.pop_back() {
            let freq = self.freq_map.get(&tail.key).copied().unwrap_or(0);
            if freq > 0 {
                // Give this entry a second chance: decrement freq, re-admit at front.
                self.freq_map.insert(tail.key.clone(), freq - 1);
                self.main.push_front(tail);
            } else {
                // freq == 0: evict for real.
                self.data.remove(&tail.key);
                self.freq_map.remove(&tail.key);
                break;
            }
        }
    }

    /// Evict from the small queue: promote to main if freq > 1, else ghost.
    /// After each eviction, increment evict_count and reset the sketch when
    /// evict_count reaches main_size (periodic counter reset).
    fn evict_small(&mut self) {
        if let Some(tail) = self.small.pop_back() {
            let freq = self.freq_map.get(&tail.key).copied().unwrap_or(0);
            if freq > 1 {
                // Hot entry: promote to main; entry stays in data and freq_map.
                if self.main.len() >= self.main_size {
                    self.evict_main();
                }
                self.main.push_front(tail);
            } else {
                // Cold entry: move key to ghost; remove from data and freq_map.
                self.data.remove(&tail.key);
                self.freq_map.remove(&tail.key);
                // Capacity guard with lazy deletion:
                //   Pop from the back of the ghost deque while it is at capacity.
                //   If the popped key is still in ghost_set it is a live ghost —
                //   remove it from ghost_set (normal aged eviction) and break.
                //   If not, it was already lazily deleted; continue popping.
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

            // Periodic sketch reset: after main_size evictions from small,
            // reset all counters to 0 to prevent saturation and adapt to
            // workload shifts.
            self.evict_count += 1;
            if self.evict_count >= self.main_size {
                self.evict_count = 0;
                self.admit_sketch.iter_mut().for_each(|c| *c = 0);
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

    /// With the admission sketch, the first cold put of a new key is rejected
    /// (sketch bucket was 0 → set to 1, return early without storing).
    /// The second put of the same key is admitted (bucket > 0 → proceed).
    #[test]
    fn sketch_rejects_first_put_admits_second() {
        let mut cache = S3Fifo::<i64, i64>::new(10);
        // First put: rejected — key not in small, data, or freq_map.
        cache.put(99, 990);
        assert!(!cache.small.iter().any(|e| e.key == 99));
        assert!(!cache.data.contains_key(&99));
        assert!(cache.get(&99).is_none());
        // Second put: admitted — sketch bucket now 1 > 0.
        cache.put(99, 990);
        assert!(cache.get(&99).is_some());
    }

    #[test]
    fn put_twice_then_get_finds_value() {
        // Two puts required for admission under the sketch.
        let mut cache = S3Fifo::new(4);
        cache.put(1i64, 10i64); // rejected (first time)
        cache.put(1, 10);       // admitted
        assert_eq!(*cache.get(&1).unwrap(), 10);
    }

    #[test]
    fn put_existing_updates_value() {
        let mut cache = S3Fifo::new(4);
        cache.put(1i64, 10i64); // rejected
        cache.put(1, 10);       // admitted
        cache.put(1, 20);       // resident-update path (data.contains_key → true)
        assert_eq!(*cache.get(&1).unwrap(), 20);
    }

    #[test]
    fn small_eviction_promotes_to_main_after_multiple_hits() {
        // capacity=4 → small_size=1, main_size=3
        let mut cache = S3Fifo::new(4);
        // Admit key 1 (two puts), then hit it twice so freq > 1.
        cache.put(1i64, 10i64); // rejected
        cache.put(1, 10);       // admitted to small=[1]
        cache.get(&1);          // freq[1] = 1
        cache.get(&1);          // freq[1] = 2
        // Admit key 2 (two puts) to trigger evict_small: freq[1]=2>1 → promotes.
        cache.put(2i64, 20i64); // rejected
        cache.put(2, 20);       // admitted, small full → evict_small(1) → main
        assert!(cache.main.iter().any(|entry| entry.key == 1));
    }

    #[test]
    fn ghost_hit_promotes_back_to_main() {
        // Ghost hits bypass the sketch and are always admitted to main.
        // capacity=4 → small_size=1, main_size=3
        let mut cache = S3Fifo::<i64, i64>::new(4);

        // Admit key 1 into small (two puts).
        cache.put(1, 10); // rejected
        cache.put(1, 10); // admitted → small=[1]

        // Admit key 2 → evict_small(1) to ghost (freq=0 ≤ 1).
        cache.put(2, 20); // rejected
        cache.put(2, 20); // admitted → small=[2], ghost=[1]

        // Ghost hit on key 1 (bypasses sketch → admitted to main).
        cache.put(1, 10);
        assert!(cache.main.iter().any(|entry| entry.key == 1));
    }

    /// White-box test for second-chance eviction in main.
    /// Updated for HashMap-based layout: freq values and data are pushed into
    /// freq_map and data alongside their entries in small/main.
    #[test]
    fn main_eviction_uses_frequency_second_chance() {
        let mut cache = S3Fifo::<u64, u64>::new(5);
        // Push entries directly into main, data, and freq_map.
        cache.main.push_back(Entry::new(1, 10)); cache.data.insert(1u64, 10u64); cache.freq_map.insert(1u64, 0u8);
        cache.main.push_back(Entry::new(2, 20)); cache.data.insert(2u64, 20u64); cache.freq_map.insert(2u64, 2u8);
        cache.main.push_back(Entry::new(3, 30)); cache.data.insert(3u64, 30u64); cache.freq_map.insert(3u64, 0u8);
        cache.main.push_back(Entry::new(4, 40)); cache.data.insert(4u64, 40u64); cache.freq_map.insert(4u64, 1u8);
        // Seed small with a hot entry so evict_small triggers evict_main.
        cache.small.push_front(Entry::new(5, 50)); cache.data.insert(5u64, 50u64); cache.freq_map.insert(5u64, 2u8);
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
        for key in 0..20i64 {
            // Two puts per key for admission.
            cache.put(key, key);
            cache.put(key, key);
        }
        assert!(cache.small.len() + cache.main.len() <= 4);
    }

    #[test]
    fn repeated_get_clamps_frequency() {
        let mut cache = S3Fifo::new(4);
        cache.put(1i64, 10i64); // rejected
        cache.put(1, 10);       // admitted
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
        // Admit keys 1..=4 with two puts each, forcing evictions into ghost.
        for key in [1i64, 2, 3, 4] {
            cache.put(key, key * 10); // rejected
            cache.put(key, key * 10); // admitted
        }
        // Ghost hit on key 1 (if it was evicted to ghost): removes from ghost_set.
        cache.put(1, 10);
        assert!(
            cache.ghost_set.len() <= cache.ghost.len(),
            "ghost_set.len() ({}) must not exceed ghost.len() ({})",
            cache.ghost_set.len(),
            cache.ghost.len()
        );
        // If key 1 is in main, verify it.
        if cache.main.iter().any(|e| e.key == 1) {
            assert!(cache.data.contains_key(&1));
        }
    }

    /// Sketch counters are reset after main_size evictions from small.
    #[test]
    fn sketch_resets_after_eviction_threshold() {
        // capacity=4 → small_size=1, main_size=3; sketch resets at 3 evictions.
        let mut cache = S3Fifo::<i64, i64>::new(4);
        // Seed the sketch for key 99.
        let idx = cache.sketch_idx(&99i64);
        cache.admit_sketch[idx] = 1; // pretend key 99 was seen before

        // Drive 3 evictions from small using white-box injection.
        for _ in 0..3 {
            cache.small.push_back(Entry::new(42i64, 420i64));
            cache.data.insert(42, 420);
            cache.freq_map.insert(42, 0);
            cache.evict_small(); // evicts key 42, increments evict_count
            // Clean up for next round.
            cache.data.remove(&42);
            cache.freq_map.remove(&42);
            // Remove stale ghost entries if any.
            cache.ghost_set.remove(&42);
        }

        // After main_size (3) evictions, sketch should be reset.
        assert_eq!(cache.admit_sketch[idx], 0, "sketch should reset after main_size evictions");
    }
}
