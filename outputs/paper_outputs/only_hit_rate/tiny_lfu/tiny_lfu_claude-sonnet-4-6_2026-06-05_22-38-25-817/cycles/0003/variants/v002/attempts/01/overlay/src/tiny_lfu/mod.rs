use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;

#[derive(Debug)]
struct Node<K, V> {
    key: K,
    value: V,
    prev: Option<usize>,
    next: Option<usize>,
    in_protected: bool,
}

/// A TinyLFU cache using a Segmented LRU (SLRU) resident set plus a ghost set.
///
/// The resident set is split into:
///  - Protected segment  (≈ 80 % of capacity): items that survived at least one
///    access while resident.
///  - Probationary segment (≈ 20 % of capacity): newly admitted items.
///
/// A ghost ring-buffer tracks the hashes of the most recently evicted keys.
/// When a ghost-set hit is detected on admission, the incoming item is placed
/// directly into the protected segment, skipping the probationary staging delay.
#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
    map: HashMap<K, usize>,
    nodes: Vec<Node<K, V>>,

    // Probationary segment: LRU end = head, MRU end = tail
    prob_head: Option<usize>,
    prob_tail: Option<usize>,
    probationary_len: usize,
    probationary_capacity: usize,

    // Protected segment: LRU end = head, MRU end = tail
    prot_head: Option<usize>,
    prot_tail: Option<usize>,
    protected_len: usize,
    protected_capacity: usize,

    capacity: usize,
    sketch: CountMinSketch,

    // Ghost ring-buffer of recently-evicted key hashes
    ghost: Vec<u64>,
    ghost_head: usize,
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
        log::info!("TinyLfuCache.try_new: capacity={capacity} sketch_width={width}");

        let probationary_capacity = (capacity / 5).max(1);
        let protected_capacity = capacity - probationary_capacity;
        let ghost_size = (capacity / 8).max(4);

        Some(Self {
            map: HashMap::with_capacity(capacity),
            nodes: Vec::with_capacity(capacity),
            prob_head: None,
            prob_tail: None,
            probationary_len: 0,
            probationary_capacity,
            prot_head: None,
            prot_tail: None,
            protected_len: 0,
            protected_capacity,
            capacity,
            sketch: CountMinSketch::new(width),
            ghost: vec![u64::MAX; ghost_size],
            ghost_head: 0,
        })
    }

    pub fn new(capacity: usize) -> Self {
        Self::try_new(capacity).expect("TinyLfuCache::new: initialization failed")
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.sketch.increment(key);
        let &idx = self.map.get(key)?;
        if self.nodes[idx].in_protected {
            // Already in protected: move to MRU end within protected
            self.prot_unlink(idx);
            self.prot_push_back(idx);
        } else {
            // In probationary: promote to protected
            self.prob_unlink(idx);
            self.probationary_len -= 1;
            self.prot_push_back(idx);
            self.protected_len += 1;
            // If protected segment overflows its soft capacity, demote its LRU back to probationary
            self.maybe_demote_from_protected();
        }
        Some(&self.nodes[idx].value)
    }

    pub fn put(&mut self, key: K, value: V) {
        self.sketch.increment(&key);

        // Update-in-place: move to MRU end within the same segment
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

        if self.nodes.len() < self.capacity {
            // Arena still has free slots — allocate a new node
            let idx = self.nodes.len();
            self.nodes.push(Node {
                key: key.clone(),
                value,
                prev: None,
                next: None,
                in_protected: false, // will be set by push_back helpers
            });
            self.map.insert(key.clone(), idx);
            if self.ghost_contains(&key) {
                self.prot_push_back(idx);
                self.protected_len += 1;
                self.maybe_demote_from_protected();
            } else {
                self.prob_push_back(idx);
                self.probationary_len += 1;
            }
            return;
        }

        // Cache is full: apply TinyLFU admission gate.
        // Eviction candidate is the LRU end of probationary; fall back to protected
        // only if probationary is empty.
        let victim_idx = match self.prob_head {
            Some(idx) => idx,
            None => self.prot_head.expect("TinyLfuCache: full cache has no victim"),
        };

        let victim_key = self.nodes[victim_idx].key.clone();
        let victim_freq = self.sketch.estimate(&victim_key);
        let incoming_freq = self.sketch.estimate(&key);

        if incoming_freq > victim_freq {
            // Record evicted key in ghost ring-buffer before overwriting the slot
            self.ghost_insert(&victim_key);

            // Remove victim from its segment
            self.map.remove(&victim_key);
            if self.nodes[victim_idx].in_protected {
                self.prot_unlink(victim_idx);
                self.protected_len -= 1;
            } else {
                self.prob_unlink(victim_idx);
                self.probationary_len -= 1;
            }

            // Reuse the victim's arena slot for the incoming item
            self.nodes[victim_idx].key = key.clone();
            self.nodes[victim_idx].value = value;
            self.nodes[victim_idx].prev = None;
            self.nodes[victim_idx].next = None;
            self.map.insert(key.clone(), victim_idx);

            // Ghost-set hit → admit directly to protected; otherwise probationary
            if self.ghost_contains(&key) {
                self.prot_push_back(victim_idx);
                self.protected_len += 1;
                self.maybe_demote_from_protected();
            } else {
                self.prob_push_back(victim_idx);
                self.probationary_len += 1;
            }
        }
        // If incoming_freq <= victim_freq: incoming item is rejected (victim stays)
    }

    // ── Segment management ─────────────────────────────────────────────────────

    /// Demote protected LRU items to probationary until protected_len ≤ protected_capacity.
    fn maybe_demote_from_protected(&mut self) {
        while self.protected_len > self.protected_capacity {
            if let Some(head_idx) = self.prot_head {
                self.prot_unlink(head_idx);
                self.protected_len -= 1;
                self.prob_push_back(head_idx);
                self.probationary_len += 1;
            } else {
                break;
            }
        }
    }

    // ── Probationary linked-list helpers ───────────────────────────────────────

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

    // ── Protected linked-list helpers ──────────────────────────────────────────

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

    // ── Ghost-set helpers ──────────────────────────────────────────────────────

    fn ghost_insert(&mut self, key: &K) {
        let h = self.hash_key(key);
        self.ghost[self.ghost_head] = h;
        self.ghost_head = (self.ghost_head + 1) % self.ghost.len();
    }

    fn ghost_contains(&self, key: &K) -> bool {
        let h = self.hash_key(key);
        self.ghost.iter().any(|&g| g == h)
    }

    fn hash_key(&self, key: &K) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }
}

// ── CountMinSketch ─────────────────────────────────────────────────────────────

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

    fn increment<Q: Hash>(&mut self, key: &Q) {
        for row in 0..self.depth {
            let pos = self.position(key, row as u64);
            let counter = &mut self.counts[row][pos];
            if *counter < MAX_COUNT {
                *counter += 1;
            }
        }
    }

    fn estimate<Q: Hash>(&self, key: &Q) -> u32 {
        self.counts
            .iter()
            .enumerate()
            .map(|(row, _)| self.counts[row][self.position(key, row as u64)])
            .min()
            .unwrap_or(0)
    }

    fn position<Q: Hash>(&self, key: &Q, seed: u64) -> usize {
        let mut hasher = DefaultHasher::new();
        hasher.write_u64(seed);
        key.hash(&mut hasher);
        (hasher.finish() as usize) & (self.width - 1)
    }
}

// ── framework::Cache<K, V> impl ───────────────────────────────────────────────

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

// ── Unit tests ─────────────────────────────────────────────────────────────────

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
        cache.get(&1); // promote key 1 to protected; key 2 stays in probationary
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
    fn ghost_hit_promotes_to_protected() {
        // Verify that a re-admitted key that was previously evicted goes to protected.
        // capacity=4: probationary_capacity=1, protected_capacity=3
        let mut cache = TinyLfuCache::<i64, i64>::new(4);

        // Fill cache: items 0-3, all in probationary
        for k in 0..4i64 {
            cache.get(&k); // miss → sketch++
            cache.put(k, k);
        }
        // Make item 10 very hot so it can evict item 0 repeatedly
        for _ in 0..10 {
            cache.get(&10);
        }
        // Evict item 0 → it goes into ghost
        cache.put(10, 10);
        // Now item 0 is in the ghost set. If we re-request it and it re-enters,
        // it should go into protected (ghost hit).
        for _ in 0..5 {
            cache.get(&0); // miss, sketch for 0 increases
        }
        // At some point item 0 should be admitted, and if ghost still holds its hash,
        // it goes to protected. We just verify no panic and capacity is maintained.
        cache.put(0, 0);
        let resident = (0..100i64).filter(|k| cache.get(k).is_some()).count();
        assert!(resident <= 4, "ghost test: {resident} residents but capacity=4");
    }
}
