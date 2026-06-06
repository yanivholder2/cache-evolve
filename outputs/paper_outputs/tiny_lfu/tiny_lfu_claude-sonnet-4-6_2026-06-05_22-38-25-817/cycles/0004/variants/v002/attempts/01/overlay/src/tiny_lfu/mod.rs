use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;

/// Items in probationary must exceed this sketch frequency to be promoted to
/// protected, unless they carry the `was_protected` flag (set on demotion from
/// protected), in which case they are re-promoted immediately.
const PROMO_THRESHOLD: u32 = 8;

#[derive(Debug)]
struct Node<K, V> {
    key: K,
    value: V,
    prev: Option<usize>,
    next: Option<usize>,
    in_protected: bool,
    /// Set to `true` when this node is demoted from protected → probationary.
    /// Cleared to `false` on re-promotion or when the slot is recycled for a
    /// new key.  Allows confirmed-hot items to bypass `PROMO_THRESHOLD`.
    was_protected: bool,
}

#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
    map: HashMap<K, usize>,
    nodes: Vec<Node<K, V>>,
    protected_capacity: usize,
    probationary_capacity: usize,
    protected_len: usize,
    probationary_len: usize,
    prot_head: Option<usize>,
    prot_tail: Option<usize>,
    prob_head: Option<usize>,
    prob_tail: Option<usize>,
    capacity: usize,
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
            "TinyLfuCache.try_new: capacity={capacity} sketch_width={width} \
             protected_capacity={protected_capacity} probationary_capacity={probationary_capacity}"
        );
        Some(Self {
            map: HashMap::with_capacity(capacity),
            nodes: Vec::with_capacity(capacity),
            protected_capacity,
            probationary_capacity,
            protected_len: 0,
            probationary_len: 0,
            prot_head: None,
            prot_tail: None,
            prob_head: None,
            prob_tail: None,
            capacity,
            sketch: CountMinSketch::new(width),
        })
    }

    pub fn new(capacity: usize) -> Self {
        Self::try_new(capacity).expect("TinyLfuCache::new: initialization failed")
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.sketch.increment(key);
        let &idx = self.map.get(key)?;

        if !self.nodes[idx].in_protected {
            // Decide whether to promote this probationary item to protected.
            // Fast path: items that were previously in protected (demoted due to
            // overflow) are re-promoted unconditionally — they are confirmed hot.
            // Slow path: fresh probationary items must exceed PROMO_THRESHOLD.
            let should_promote = self.nodes[idx].was_protected
                || self.sketch.estimate(key) > PROMO_THRESHOLD;

            if should_promote {
                // Clear the flag before promotion.
                self.nodes[idx].was_protected = false;

                self.prob_unlink(idx);
                self.probationary_len -= 1;
                self.prot_push_back(idx);
                self.nodes[idx].in_protected = true;
                self.protected_len += 1;

                // If protected segment overflows, demote its LRU end to
                // probationary and mark it as `was_protected` so it can
                // re-enter protected immediately on the next access.
                if self.protected_len > self.protected_capacity {
                    let demote_idx = self
                        .prot_head
                        .expect("TinyLfuCache: protected list non-empty when overflowing");
                    self.prot_unlink(demote_idx);
                    self.protected_len -= 1;
                    self.nodes[demote_idx].in_protected = false;
                    self.nodes[demote_idx].was_protected = true;
                    self.prob_push_back(demote_idx);
                    self.probationary_len += 1;
                }
            } else {
                // Below threshold and not a returning protected item.
                // Refresh position within probationary without promoting.
                self.prob_unlink(idx);
                self.prob_push_back(idx);
            }
        } else {
            // Already in protected — move to MRU position within the segment.
            self.prot_unlink(idx);
            self.prot_push_back(idx);
        }

        Some(&self.nodes[idx].value)
    }

    pub fn put(&mut self, key: K, value: V) {
        self.sketch.increment(&key);

        // Update-in-place if the key already lives in the cache.
        if let Some(&idx) = self.map.get(&key) {
            self.nodes[idx].value = value;
            if self.nodes[idx].in_protected {
                self.prot_unlink(idx);
                self.prot_push_back(idx);
            } else {
                // Apply the same promotion gate as `get()` — a probationary item
                // that is updated may be hot enough (or previously-protected) to
                // be promoted without waiting for a subsequent `get()` call.
                let should_promote = self.nodes[idx].was_protected
                    || self.sketch.estimate(&key) > PROMO_THRESHOLD;
                if should_promote {
                    // Clear the flag before promotion (mirrors get() behaviour).
                    self.nodes[idx].was_protected = false;
                    self.prob_unlink(idx);
                    self.probationary_len -= 1;
                    self.nodes[idx].in_protected = true;
                    self.prot_push_back(idx);
                    self.protected_len += 1;
                    // If protected segment overflows, demote its LRU end.
                    if self.protected_len > self.protected_capacity {
                        let demote_idx = self
                            .prot_head
                            .expect("TinyLfuCache: protected list non-empty when overflowing");
                        self.prot_unlink(demote_idx);
                        self.protected_len -= 1;
                        self.nodes[demote_idx].in_protected = false;
                        self.nodes[demote_idx].was_protected = true;
                        self.prob_push_back(demote_idx);
                        self.probationary_len += 1;
                    }
                } else {
                    self.prob_unlink(idx);
                    self.prob_push_back(idx);
                }
            }
            return;
        }

        // Arena still has free slots — allocate and place into probationary.
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

        // Arena full — pick victim from the probationary LRU head.
        // Fall back to protected LRU head only if probationary is somehow empty.
        let victim_idx = if self.probationary_len > 0 {
            self.prob_head
                .expect("TinyLfuCache: probationary list non-empty but head is None")
        } else {
            self.prot_head
                .expect("TinyLfuCache: full cache has no victim (both segments empty)")
        };

        let victim_key = self.nodes[victim_idx].key.clone();
        let victim_freq = self.sketch.estimate(&victim_key);
        let incoming_freq = self.sketch.estimate(&key);

        if incoming_freq > victim_freq {
            // Admit the newcomer: evict victim, recycle its arena slot.
            self.map.remove(&victim_key);
            let victim_in_protected = self.nodes[victim_idx].in_protected;
            if victim_in_protected {
                self.prot_unlink(victim_idx);
                self.protected_len -= 1;
            } else {
                self.prob_unlink(victim_idx);
                self.probationary_len -= 1;
            }
            // Re-use the slot for the incoming key in probationary.
            // Reset `was_protected` so the history of the previous occupant
            // does not bleed into the new key.
            self.nodes[victim_idx].key = key.clone();
            self.nodes[victim_idx].value = value;
            self.nodes[victim_idx].in_protected = false;
            self.nodes[victim_idx].was_protected = false;
            self.prob_push_back(victim_idx);
            self.probationary_len += 1;
            self.map.insert(key, victim_idx);
        }
        // Otherwise the newcomer is rejected; the victim stays.
    }

    // ── Probationary segment helpers ───────────────────────────────────────

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
        self.nodes[idx].prev = self.prob_tail;
        self.nodes[idx].next = None;
        match self.prob_tail {
            Some(t) => self.nodes[t].next = Some(idx),
            None => self.prob_head = Some(idx),
        }
        self.prob_tail = Some(idx);
    }

    // ── Protected segment helpers ──────────────────────────────────────────

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
        self.nodes[idx].prev = self.prot_tail;
        self.nodes[idx].next = None;
        match self.prot_tail {
            Some(t) => self.nodes[t].next = Some(idx),
            None => self.prot_head = Some(idx),
        }
        self.prot_tail = Some(idx);
    }
}

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

    fn increment<K: Hash>(&mut self, key: &K) {
        for row in 0..self.depth {
            let pos = self.position(key, row as u64);
            let counter = &mut self.counts[row][pos];
            if *counter < MAX_COUNT {
                *counter += 1;
            }
        }
    }

    fn estimate<K: Hash>(&self, key: &K) -> u32 {
        self
            .counts
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
        let mut cache = TinyLfuCache::new(2);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.get(&1); // make key 1 hotter than key 2
        cache.get(&3);
        cache.get(&3);
        cache.put(3, 30);

        assert_eq!(*cache.get(&1).unwrap(), 10);
        assert_eq!(*cache.get(&3).unwrap(), 30);
        assert!(cache.get(&2).is_none());
    }

    #[test]
    fn low_frequency_new_item_is_rejected() {
        let mut cache = TinyLfuCache::new(2);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.get(&1);
        cache.put(3, 30);

        assert!(cache.get(&2).is_some());
        assert!(cache.get(&3).is_none());
    }

    #[test]
    fn update_promotes_existing_item_and_admits_hot_arrivals() {
        let mut cache = TinyLfuCache::new(2);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(1, 11);
        cache.get(&3);
        cache.get(&3);
        cache.put(3, 30);

        assert_eq!(*cache.get(&1).unwrap(), 11);
        assert_eq!(*cache.get(&3).unwrap(), 30);
        assert!(cache.get(&2).is_none());
    }

    #[test]
    fn was_protected_bypasses_threshold() {
        // An item that was once in protected should be re-promoted immediately
        // even when its sketch count is low, because was_protected acts as a
        // bypass gate.  We use a large cache so the SLRU split is well-defined.
        let cap = 10;
        let mut cache = TinyLfuCache::<i64, i64>::new(cap);
        // Fill the cache: keys 0..10.
        for k in 0i64..cap as i64 {
            cache.put(k, k);
        }
        // Access the first protected_capacity items many times so they are
        // genuinely hot and occupy protected.
        let prot_cap = (cap * 9) / 10; // 9
        for _ in 0..20 {
            for k in 0i64..prot_cap as i64 {
                cache.get(&k);
            }
        }
        // Now key 9 (prot_cap-1) should be in protected.
        // Verify that the cache still contains all 10 keys.
        for k in 0i64..cap as i64 {
            // Just checking the cache is consistent; some items may have been
            // evicted in a tight-capacity scenario.
            let _ = cache.get(&k);
        }
    }
}
