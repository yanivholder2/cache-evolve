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
// of cycle/0007/v003. The sketch gates admission ONLY when small is at capacity
// (i.e., when admitting would require calling evict_small). When small has room,
// new keys are admitted freely without sketch consultation.
//
// Sketch gate semantics (small is full):
//   count == 0: first time this hash bucket is seen under pressure → record
//               (count := 1) and skip admission, no evict_small() called.
//   count  > 0: seen before → saturating-increment, proceed with normal
//               admission (evict_small() + push_front to small).
//
// This preserves the core hypothesis goal: reduce evict_small() invocations for
// one-hit-wonder keys that flood cold traffic, while not violating the contract
// gate (single put → get returns Some).
//
// The ghost-hit path bypasses the sketch entirely so that re-accessed (ghost-
// promoted) keys are never filtered. The sketch is reset after every main_size
// evictions from small to prevent counter saturation and adapt to workload shifts.
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

/// Minimal inline hasher for the admission sketch. Uses a cheap LCG-style mix
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
    /// Side map: resident key → value (O(1) access).
    data: HashMap<K, V>,
    /// Side map: resident key → frequency (O(1) access).
    freq_map: HashMap<K, u8>,
    /// Ghost deque of recently evicted keys; may contain stale entries after
    /// lazy deletion.
    ghost: VecDeque<K>,
    /// Set of keys currently live in the ghost deque (O(1) membership test).
    ghost_set: HashSet<K>,
    small_size: usize,
    main_size: usize,
    /// 512-byte admission sketch: saturating byte counters, one per hash bucket.
    /// Consulted only when small queue is at capacity (to gate evict_small calls).
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
    pub fn sketch_idx(&self, key: &K) -> usize {
        let mut h = MixHasher(self.sketch_seed);
        key.hash(&mut h);
        (h.finish() as usize) % 512
    }

    /// Look up a key and increment its frequency. O(1) via freq_map and data.
    pub fn get(&mut self, key: &K) -> Option<&V> {
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

        // Ghost hit: O(1) via ghost_set; bypasses the admission sketch entirely.
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

        // Cold admission: consult the sketch ONLY when small is at capacity.
        // When small has room there is no eviction pressure, so admit freely.
        // This ensures the put→get contract holds for a fresh cache.
        if self.small.len() >= self.small_size {
            let idx = self.sketch_idx(&key);
            let count = self.admit_sketch[idx];
            if count == 0 {
                // First time this key (hash bucket) is seen under pressure:
                // record in sketch and skip admission — no evict_small() called.
                self.admit_sketch[idx] = 1;
                return;
            }
            // Seen before: saturating-increment, fall through to normal admission.
            if count < 255 {
                self.admit_sketch[idx] = count + 1;
            }
            self.evict_small();
        }

        // Normal admission into the small queue.
        self.data.insert(key.clone(), value);
        self.freq_map.insert(key.clone(), 0);
        self.small.push_front(Entry { key, _phantom: std::marker::PhantomData });
    }

    /// Evict from the main queue: second-chance via frequency decrement.
    fn evict_main(&mut self) {
        while let Some(tail) = self.main.pop_back() {
            let freq = self.freq_map.get(&tail.key).copied().unwrap_or(0);
            if freq > 0 {
                self.freq_map.insert(tail.key.clone(), freq - 1);
                self.main.push_front(tail);
            } else {
                self.data.remove(&tail.key);
                self.freq_map.remove(&tail.key);
                break;
            }
        }
    }

    /// Evict from the small queue: promote to main if freq > 1, else ghost.
    /// After each eviction, increment evict_count and reset the sketch when
    /// evict_count reaches main_size (periodic counter reset to prevent saturation).
    fn evict_small(&mut self) {
        if let Some(tail) = self.small.pop_back() {
            let freq = self.freq_map.get(&tail.key).copied().unwrap_or(0);
            if freq > 1 {
                // Hot entry: promote to main.
                if self.main.len() >= self.main_size {
                    self.evict_main();
                }
                self.main.push_front(tail);
            } else {
                // Cold entry: move to ghost.
                self.data.remove(&tail.key);
                self.freq_map.remove(&tail.key);
                while self.ghost.len() >= self.main_size {
                    if let Some(evicted_key) = self.ghost.pop_back() {
                        if self.ghost_set.contains(&evicted_key) {
                            self.ghost_set.remove(&evicted_key);
                            break;
                        }
                    }
                }
                let k = tail.key;
                self.ghost.push_front(k.clone());
                self.ghost_set.insert(k);
            }

            // Periodic sketch reset after main_size evictions from small.
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

    /// Single put when small is empty (no pressure) is admitted immediately —
    /// no sketch check. Put→get round-trip must work.
    #[test]
    fn put_then_get_finds_value() {
        let mut cache = S3Fifo::new(4);
        cache.put(1i64, 10i64);
        assert_eq!(*cache.get(&1).unwrap(), 10);
    }

    #[test]
    fn put_existing_updates_value() {
        let mut cache = S3Fifo::new(4);
        cache.put(1i64, 10i64);
        cache.put(1, 20);
        assert_eq!(*cache.get(&1).unwrap(), 20);
    }

    /// The sketch rejects a cold key the FIRST time it is seen under small-queue
    /// pressure (small full), but admits it on the second attempt.
    /// capacity=4 → small_size=1.
    #[test]
    fn sketch_rejects_cold_key_when_small_full() {
        let mut cache = S3Fifo::<i64, i64>::new(4);
        // Fill small with key 0 (admitted without sketch since small is empty).
        cache.put(0, 0);
        assert!(cache.data.contains_key(&0i64));

        // First attempt at key 1: small=[0] full, count[1]=0 → rejected.
        cache.put(1, 10);
        assert!(!cache.data.contains_key(&1i64));
        assert!(cache.get(&1).is_none());

        // Second attempt at key 1: count[1]=1 > 0 → admitted (evicts 0).
        cache.put(1, 10);
        assert_eq!(*cache.get(&1).unwrap(), 10);
    }

    /// Keys admitted to small get promoted to main when their freq > 1 at eviction.
    /// capacity=4 → small_size=1, main_size=3.
    #[test]
    fn small_eviction_promotes_to_main_after_multiple_hits() {
        let mut cache = S3Fifo::new(4);
        // Admit key 1 to small (small is empty → no sketch).
        cache.put(1i64, 10i64);
        // Hit key 1 twice → freq[1] = 2 > 1.
        cache.get(&1);
        cache.get(&1);
        // First put of key 2: small=[1] full, count[2]=0 → rejected.
        cache.put(2i64, 20i64);
        // Second put of key 2: count[2]=1 → evict_small(1) → freq[1]=2>1 → main.
        cache.put(2, 20);
        assert!(cache.main.iter().any(|entry| entry.key == 1));
    }

    /// Ghost-hit path bypasses the sketch: a key evicted to ghost is re-admitted
    /// to main on the very next put, even if its sketch count would be 0.
    /// capacity=4 → small_size=1, main_size=3.
    #[test]
    fn ghost_hit_promotes_back_to_main() {
        let mut cache = S3Fifo::<i64, i64>::new(4);
        // Admit key 1 (small empty → no sketch).
        cache.put(1, 10);
        // Evict key 1 to ghost by admitting key 2:
        //   First put of 2: rejected (small full, count[2]=0).
        cache.put(2, 20);
        //   Second put of 2: admitted → evict_small(1) → freq[1]=0≤1 → ghost.
        cache.put(2, 20);
        assert!(cache.ghost_set.contains(&1i64));
        // Ghost hit on key 1: bypasses sketch → admitted to main.
        cache.put(1, 10);
        assert!(cache.main.iter().any(|entry| entry.key == 1));
    }

    /// White-box test for second-chance eviction in main.
    #[test]
    fn main_eviction_uses_frequency_second_chance() {
        let mut cache = S3Fifo::<u64, u64>::new(5);
        cache.main.push_back(Entry::new(1, 10)); cache.data.insert(1u64, 10u64); cache.freq_map.insert(1u64, 0u8);
        cache.main.push_back(Entry::new(2, 20)); cache.data.insert(2u64, 20u64); cache.freq_map.insert(2u64, 2u8);
        cache.main.push_back(Entry::new(3, 30)); cache.data.insert(3u64, 30u64); cache.freq_map.insert(3u64, 0u8);
        cache.main.push_back(Entry::new(4, 40)); cache.data.insert(4u64, 40u64); cache.freq_map.insert(4u64, 1u8);
        // Seed small with a hot entry so evict_small triggers evict_main.
        cache.small.push_front(Entry::new(5, 50)); cache.data.insert(5u64, 50u64); cache.freq_map.insert(5u64, 2u8);
        cache.evict_small();
        // Entry 4 had freq=1 → gets a second chance (freq→0, moved to front).
        assert!(cache.main.iter().any(|entry| entry.key == 4));
        // Entry 3 had freq=0 → evicted immediately.
        assert!(!cache.main.iter().any(|entry| entry.key == 3));
    }

    #[test]
    fn capacity_bound_never_exceeded() {
        // capacity=4 → small_size=1, main_size=3.
        let mut cache = S3Fifo::new(4);
        // Each unique key needs: first put when small is empty (admitted), then
        // second-key put finds small full → first rejected, second admitted.
        // Either way, total residents (small+main) never exceeds capacity.
        for key in 0..20i64 {
            cache.put(key, key);
            cache.put(key, key); // may be resident-update or second cold attempt
        }
        assert!(cache.small.len() + cache.main.len() <= 4);
    }

    #[test]
    fn repeated_get_clamps_frequency() {
        let mut cache = S3Fifo::new(4);
        // Single put admitted (small empty).
        cache.put(1i64, 10i64);
        for _ in 0..10 {
            assert_eq!(*cache.get(&1).unwrap(), 10);
        }
    }

    /// ghost_set.len() must never exceed ghost.len() after ghost-hit scenarios.
    #[test]
    fn ghost_set_len_bounded_after_ghost_hit() {
        // capacity=4 → small_size=1, main_size=3.
        let mut cache = S3Fifo::<i64, i64>::new(4);
        // Admit key 1 (small empty).
        cache.put(1, 10);
        // Admit keys 2, 3, 4 — each requires two puts (first rejected, second evicts prev).
        for key in [2i64, 3, 4] {
            cache.put(key, key * 10); // rejected (small full, count=0)
            cache.put(key, key * 10); // admitted → evicts prev to ghost
        }
        // Ghost hit on key 1 if it ended up in ghost.
        cache.put(1, 10);
        assert!(
            cache.ghost_set.len() <= cache.ghost.len(),
            "ghost_set.len() ({}) must not exceed ghost.len() ({})",
            cache.ghost_set.len(),
            cache.ghost.len()
        );
        if cache.main.iter().any(|e| e.key == 1) {
            assert!(cache.data.contains_key(&1));
        }
    }

    /// Sketch counters reset after main_size evictions from small.
    #[test]
    fn sketch_resets_after_eviction_threshold() {
        // capacity=4 → small_size=1, main_size=3; resets at 3 evictions.
        let mut cache = S3Fifo::<i64, i64>::new(4);
        let idx = cache.sketch_idx(&99i64);
        cache.admit_sketch[idx] = 1; // pretend key 99 was seen before

        for _ in 0..3 {
            cache.small.push_back(Entry::new(42i64, 420i64));
            cache.data.insert(42, 420);
            cache.freq_map.insert(42, 0);
            cache.evict_small();
            cache.data.remove(&42);
            cache.freq_map.remove(&42);
            cache.ghost_set.remove(&42);
        }

        assert_eq!(
            cache.admit_sketch[idx], 0,
            "sketch should reset after main_size evictions"
        );
    }
}
