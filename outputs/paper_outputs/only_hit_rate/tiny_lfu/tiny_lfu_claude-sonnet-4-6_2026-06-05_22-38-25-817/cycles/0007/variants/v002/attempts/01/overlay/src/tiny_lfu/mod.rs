use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;

/// Number of LRU-end probationary candidates to scan for minimum-frequency eviction.
const EVICTION_SCAN_K: usize = 4;

/// Minimum sketch frequency required to promote a probationary item to protected.
/// Items that were previously in protected (was_protected == true) bypass this gate.
/// Lowered from 8 to 4 to recalibrate the promotion gate for the tighter frequency
/// estimates produced by conservative CMS update: conservative update keeps estimates
/// close to the true access count, so medium-hot items (true count ~4-5) that would
/// have cleared the old threshold via collision inflation now stall without this
/// adjustment.
const PROMO_THRESHOLD: u32 = 4;

#[derive(Debug)]
struct Node<K, V> {
    key: K,
    value: V,
    prev: Option<usize>,
    next: Option<usize>,
    /// Whether this node currently lives in the protected segment.
    in_protected: bool,
    /// Whether this node was previously in the protected segment (demoted overflow).
    was_protected: bool,
}

/// Two-segment SLRU cache with frequency-sampled eviction.
///
/// Items enter via the **probationary** segment.  On access they are
/// promoted to the **protected** segment (≈90 % of `capacity`) if their
/// sketch frequency meets PROMO_THRESHOLD, or if they were previously
/// demoted from protected (`was_protected` bypass).  When protected
/// overflows its budget the least-recently-used protected item is demoted
/// back to the MRU end of probationary.
///
/// When the arena is full and a new item must be admitted, the eviction
/// candidate is selected by scanning the *k = 4* least-recently-used
/// entries at the LRU end of probationary and evicting the one with the
/// lowest estimated access frequency (TinyLFU sketch).  The TinyLFU
/// admission gate then decides whether the newcomer beats the victim; if
/// not, the newcomer is dropped.
///
/// The CountMinSketch uses **conservative update** semantics: only rows
/// whose counter is at the current minimum estimate are incremented,
/// reducing overcounting due to hash collisions.
#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
    map: HashMap<K, usize>,
    nodes: Vec<Node<K, V>>,
    capacity: usize,
    protected_capacity: usize,
    #[allow(dead_code)]
    probationary_capacity: usize,
    protected_len: usize,
    probationary_len: usize,
    prot_head: Option<usize>,
    prot_tail: Option<usize>,
    prob_head: Option<usize>,
    prob_tail: Option<usize>,
    sketch: CountMinSketch,
}

impl<K, V> TinyLfuCache<K, V>
where
    K: Hash + Eq + Clone + Debug,
    V: Debug,
{
    pub fn try_new(capacity: usize) -> Option<Self> {
        if capacity == 0 {
            log::error!("TinyLfuCache.try_new: capacity must be > 0");
            return None;
        }
        let width = (capacity * SKETCH_WIDTH_FACTOR)
            .max(MIN_SKETCH_WIDTH)
            .next_power_of_two();
        let protected_capacity = (capacity * 9) / 10;
        let probationary_capacity = capacity - protected_capacity;
        log::info!(
            "TinyLfuCache.try_new: capacity={capacity} \
             protected_capacity={protected_capacity} \
             probationary_capacity={probationary_capacity} \
             sketch_width={width}"
        );
        Some(Self {
            map: HashMap::with_capacity(capacity),
            nodes: Vec::with_capacity(capacity),
            capacity,
            protected_capacity,
            probationary_capacity,
            protected_len: 0,
            probationary_len: 0,
            prot_head: None,
            prot_tail: None,
            prob_head: None,
            prob_tail: None,
            sketch: CountMinSketch::new(width),
        })
    }

    pub fn new(capacity: usize) -> Self {
        Self::try_new(capacity).expect("TinyLfuCache::new: initialization failed")
    }

    // ── probationary list helpers ────────────────────────────────────────────

    fn prob_unlink(&mut self, idx: usize) {
        let prev = self.nodes[idx].prev;
        let next = self.nodes[idx].next;
        match prev {
            Some(p) => self.nodes[p].next = next,
            None => self.prob_head = next,
        }
        match next {
            Some(n) => self.nodes[n].prev = prev,
            None => self.prob_tail = prev,
        }
        self.nodes[idx].prev = None;
        self.nodes[idx].next = None;
    }

    fn prob_push_back(&mut self, idx: usize) {
        self.nodes[idx].in_protected = false;
        self.nodes[idx].prev = self.prob_tail;
        self.nodes[idx].next = None;
        match self.prob_tail {
            Some(t) => self.nodes[t].next = Some(idx),
            None => self.prob_head = Some(idx),
        }
        self.prob_tail = Some(idx);
    }

    // ── protected list helpers ───────────────────────────────────────────────

    fn prot_unlink(&mut self, idx: usize) {
        let prev = self.nodes[idx].prev;
        let next = self.nodes[idx].next;
        match prev {
            Some(p) => self.nodes[p].next = next,
            None => self.prot_head = next,
        }
        match next {
            Some(n) => self.nodes[n].prev = prev,
            None => self.prot_tail = prev,
        }
        self.nodes[idx].prev = None;
        self.nodes[idx].next = None;
    }

    fn prot_push_back(&mut self, idx: usize) {
        self.nodes[idx].in_protected = true;
        self.nodes[idx].prev = self.prot_tail;
        self.nodes[idx].next = None;
        match self.prot_tail {
            Some(t) => self.nodes[t].next = Some(idx),
            None => self.prot_head = Some(idx),
        }
        self.prot_tail = Some(idx);
    }

    // ── cross-segment helpers ────────────────────────────────────────────────

    /// Demote the LRU end of protected to probationary until the protected
    /// segment is within its capacity budget.
    fn maybe_demote_from_protected(&mut self) {
        while self.protected_len > self.protected_capacity {
            if let Some(h) = self.prot_head {
                self.prot_unlink(h);
                self.protected_len -= 1;
                self.nodes[h].was_protected = true;
                self.prob_push_back(h);
                self.probationary_len += 1;
            } else {
                break;
            }
        }
    }

    // ── public cache interface ───────────────────────────────────────────────

    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.sketch.increment(key);
        let &idx = self.map.get(key)?;
        if self.nodes[idx].in_protected {
            // Refresh position within protected (move to MRU end).
            self.prot_unlink(idx);
            self.prot_push_back(idx);
        } else {
            // Promote probationary → protected only if frequency meets threshold
            // or item was previously demoted from protected (was_protected bypass).
            let freq = self.sketch.estimate(&self.nodes[idx].key);
            if freq >= PROMO_THRESHOLD || self.nodes[idx].was_protected {
                self.nodes[idx].was_protected = false;
                self.prob_unlink(idx);
                self.probationary_len -= 1;
                self.prot_push_back(idx);
                self.protected_len += 1;
                self.maybe_demote_from_protected();
            } else {
                // Below threshold and not a protected overflow-demotee:
                // refresh MRU position within probationary only.
                self.prob_unlink(idx);
                self.prob_push_back(idx);
            }
        }
        Some(&self.nodes[idx].value)
    }

    pub fn put(&mut self, key: K, value: V) {
        self.sketch.increment(&key);

        // ── existing key: update value and refresh segment position ──────────
        if let Some(&idx) = self.map.get(&key) {
            self.nodes[idx].value = value;
            if self.nodes[idx].in_protected {
                self.prot_unlink(idx);
                self.prot_push_back(idx);
            } else {
                self.prob_unlink(idx);
                self.prob_push_back(idx);
            }
            return;
        }

        // ── arena not yet full: insert new node into probationary ────────────
        if self.nodes.len() < self.capacity {
            let idx = self.nodes.len();
            self.nodes.push(Node {
                key: key.clone(),
                value,
                prev: None,
                next: None,
                in_protected: false,
                was_protected: false,
            });
            self.prob_push_back(idx);
            self.probationary_len += 1;
            self.map.insert(key, idx);
            return;
        }

        // ── arena full: frequency-sampled victim selection ───────────────────
        //
        // Scan up to EVICTION_SCAN_K items from the LRU end of probationary
        // and pick the one with the lowest estimated access frequency.
        // Fall back to the protected LRU head if probationary is empty.

        let (victim_idx, victim_in_protected) = if let Some(ph) = self.prob_head {
            let mut victim_idx = ph;
            let mut victim_freq = self.sketch.estimate(&self.nodes[ph].key);

            let mut cursor = self.nodes[ph].next;
            for _ in 1..EVICTION_SCAN_K {
                let Some(c) = cursor else { break };
                let f = self.sketch.estimate(&self.nodes[c].key);
                if f < victim_freq {
                    victim_freq = f;
                    victim_idx = c;
                }
                cursor = self.nodes[c].next;
            }
            (victim_idx, false)
        } else {
            // Probationary is empty; fall back to the protected LRU head.
            let ph = self.prot_head
                .expect("TinyLfuCache: full cache has no victim");
            (ph, true)
        };

        let victim_freq = self.sketch.estimate(&self.nodes[victim_idx].key);
        let incoming_freq = self.sketch.estimate(&key);

        if incoming_freq > victim_freq {
            let victim_key = self.nodes[victim_idx].key.clone();
            self.map.remove(&victim_key);

            // Unlink victim from its current segment.
            if victim_in_protected {
                self.prot_unlink(victim_idx);
                self.protected_len -= 1;
            } else {
                self.prob_unlink(victim_idx);
                self.probationary_len -= 1;
            }

            // Recycle the slot for the incoming key (always enters probationary).
            self.nodes[victim_idx].key = key.clone();
            self.nodes[victim_idx].value = value;
            self.nodes[victim_idx].was_protected = false;
            self.prob_push_back(victim_idx);
            self.probationary_len += 1;
            self.map.insert(key, victim_idx);
        }
        // else: incoming key is rejected; victim stays in place.
    }
}

// ── framework::Cache<K, V> impl ─────────────────────────────────────────────

impl<K, V> framework::Cache<K, V> for TinyLfuCache<K, V>
where
    K: Hash + Eq + Clone + Debug,
    V: Debug,
{
    fn put(&mut self, key: K, value: V) {
        TinyLfuCache::put(self, key, value);
    }

    fn get(&mut self, key: &K) -> Option<&V> {
        TinyLfuCache::get(self, key)
    }
}

// ── CountMinSketch ───────────────────────────────────────────────────────────

#[derive(Debug)]
struct CountMinSketch {
    width: usize,
    depth: usize,
    counts: Vec<Vec<u32>>,
}

impl CountMinSketch {
    fn new(width: usize) -> Self {
        let depth = SKETCH_DEPTH;
        let counts = vec![vec![0; width]; depth];
        Self { width, depth, counts }
    }

    /// Conservative update: only increment rows whose counter equals the
    /// current minimum estimate for the key.  This avoids inflating rows
    /// that are already overcounting due to hash collisions, tightening
    /// future estimates without widening the sketch.
    fn increment<K: Hash>(&mut self, key: &K) {
        let current_min = self.estimate(key);
        for row in 0..self.depth {
            let pos = self.position(key, row as u64);
            let counter = &mut self.counts[row][pos];
            if *counter <= current_min && *counter < MAX_COUNT {
                *counter += 1;
            }
        }
    }

    fn estimate<K: Hash>(&self, key: &K) -> u32 {
        self.counts
            .iter()
            .enumerate()
            .map(|(row, _)| self.counts[row][self.position(key, row as u64)])
            .min()
            .unwrap_or(0)
    }

    fn position<K: Hash>(&self, key: &K, seed: u64) -> usize {
        let mut hasher = DefaultHasher::new();
        hasher.write_u64(seed);
        key.hash(&mut hasher);
        (hasher.finish() as usize) & (self.width - 1)
    }
}

// ── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_miss_returns_none() {
        let mut cache = TinyLfuCache::<i64, i64>::new(4);
        assert!(cache.get(&42).is_none());
    }

    #[test]
    fn put_then_get() {
        let mut cache = TinyLfuCache::new(4);
        cache.put(1, 10);
        assert_eq!(*cache.get(&1).unwrap(), 10);
    }

    #[test]
    fn eviction_prefers_hot_items() {
        // capacity=2: protected_capacity=1, probationary_capacity=1
        let mut cache = TinyLfuCache::new(2);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.get(&1); // freq(1)=2 after get, still below PROMO_THRESHOLD=4, stays in prob
        cache.get(&3); // miss, heat up key 3
        cache.get(&3); // miss, heat up key 3
        cache.put(3, 30);

        assert_eq!(*cache.get(&1).unwrap(), 10);
        assert_eq!(*cache.get(&3).unwrap(), 30);
        assert!(cache.get(&2).is_none());
    }

    #[test]
    fn low_frequency_new_item_is_rejected() {
        // capacity=2: protected_capacity=1, probationary_capacity=1
        let mut cache = TinyLfuCache::new(2);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.get(&1); // key 1 access (freq below threshold, stays in prob)
        cache.put(3, 30); // key 3 has freq=1, victim (key 1 or key 2) has freq>=1 → rejected

        // Both keys should still be present or one was evicted; key 3 should not be present
        // since freq(3)=1 is not > freq(victim)=1
        assert!(cache.get(&3).is_none());
    }

    #[test]
    fn update_promotes_existing_item_and_admits_hot_arrivals() {
        // capacity=2: protected_capacity=1, probationary_capacity=1
        let mut cache = TinyLfuCache::new(2);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(1, 11); // update: value changes, stays in probationary (MRU)
        cache.get(&3);    // miss ×1
        cache.get(&3);    // miss ×2
        cache.put(3, 30); // freq(3)=3, victim is key 2 (LRU of prob, freq=1) → admitted

        assert_eq!(*cache.get(&1).unwrap(), 11);
        assert_eq!(*cache.get(&3).unwrap(), 30);
        assert!(cache.get(&2).is_none());
    }

    /// Verifies that the k=4 scan evicts the *minimum-frequency* probationary
    /// item rather than blindly evicting the LRU head.
    ///
    /// Setup: key 1 is inserted after three get-misses, giving it freq=4.
    /// Because it was inserted first it sits at the LRU head of probationary.
    /// Keys 2-8 are inserted with freq=1.  Key 9 is pre-heated to freq=4 via
    /// get-misses.  When put(9) is called:
    ///   • k=4 scan: key1(4), key2(1), key3(1), key4(1) → victim = key2 (min).
    ///   • Admission gate: freq(9)=4 > freq(key2)=1 → admit.
    ///
    /// A pure-LRU eviction would remove key1 instead.
    #[test]
    fn min_freq_victim_preferred_over_lru_head() {
        // capacity=8: protected_capacity=7, sketch_width=32
        let mut cache = TinyLfuCache::new(8);

        // Pre-heat key 1 before inserting so it accumulates high sketch frequency.
        cache.get(&1); // miss, sketch[1]=1
        cache.get(&1); // miss, sketch[1]=2
        cache.get(&1); // miss, sketch[1]=3

        // Fill the arena: key1 enters with freq=4, keys 2-8 enter with freq=1.
        cache.put(1, 10); // sketch[1]=4; prob: [key1 (head/LRU)]
        cache.put(2, 20);
        cache.put(3, 30);
        cache.put(4, 40);
        cache.put(5, 50);
        cache.put(6, 60);
        cache.put(7, 70);
        cache.put(8, 80); // prob: [key1(LRU), key2, key3, key4, key5, key6, key7, key8(MRU)]

        // Pre-heat key 9 before putting it.
        cache.get(&9); // miss, sketch[9]=1
        cache.get(&9); // miss, sketch[9]=2
        cache.get(&9); // miss, sketch[9]=3

        // Arena full → frequency-sampled eviction.
        // k=4 scan: key1(freq=4), key2(freq=1), key3(freq=1), key4(freq=1).
        // Min = key2; incoming freq(key9)=4 > 1 → evict key2, admit key9.
        cache.put(9, 90); // sketch[9]=4

        // Key1 survives because its frequency (4) shields it from eviction
        // even though it is the LRU head of probationary.
        assert!(cache.get(&1).is_some(), "key1 should be retained (high frequency)");
        // Key2 is evicted as the lowest-frequency candidate in the scan window.
        assert!(cache.get(&2).is_none(), "key2 should be evicted (min freq in k=4 window)");
        // Key9 was admitted.
        assert!(cache.get(&9).is_some(), "key9 should be admitted");
    }

    /// Verifies conservative-update semantics: after many increments the
    /// estimate should stay at the true count for a key with no collisions.
    #[test]
    fn conservative_update_does_not_overcount_collision_free() {
        let mut sketch = CountMinSketch::new(64); // wide enough to avoid collisions
        for _ in 0..10 {
            sketch.increment(&42i64);
        }
        // With conservative update and no collisions, every row should be exactly 10.
        assert_eq!(sketch.estimate(&42i64), 10);
    }
}
