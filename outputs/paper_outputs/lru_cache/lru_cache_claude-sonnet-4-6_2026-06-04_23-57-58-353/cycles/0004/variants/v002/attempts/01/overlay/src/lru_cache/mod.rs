// SLRU cache with ghost-set admission + frequency-aware demotion (H02).
//
// Two segments share a single arena:
//   probationary  (20 %)  — new items land here
//   protected     (80 %)  — items promoted on a cache hit
//
// A bounded ghost set tracks recently evicted probationary keys.
// When an incoming key matches a ghost entry it is admitted directly into the
// protected segment, giving cross-residency warm keys immediate protection.
//
// Ghost window is sized to total capacity (not just probationary capacity),
// allowing the cache to intercept reuse at distances up to the full cache size.
//
// Demotion from protected uses a minimum-frequency scan of the LRU quarter
// rather than strict LRU, with counters halved on demotion to preserve signal.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

// ── segment tag ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
enum Segment {
    Probationary,
    Protected,
}

// ── arena node ───────────────────────────────────────────────────────────────

#[derive(Debug)]
struct Node<K, V> {
    key: K,
    value: V,
    prev: Option<usize>,
    next: Option<usize>,
    segment: Segment,
    freq: u32,
}

// ── cache struct ─────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct LruCache<K, V> {
    map: HashMap<K, usize>, // key → arena index

    nodes: Vec<Node<K, V>>, // shared arena

    // Probationary segment  (head = LRU, tail = MRU)
    prob_head: Option<usize>,
    prob_tail: Option<usize>,
    prob_count: usize,
    prob_cap: usize,

    // Protected segment  (head = LRU, tail = MRU)
    prot_head: Option<usize>,
    prot_tail: Option<usize>,
    prot_count: usize,
    prot_cap: usize,

    // Ghost set: ordered history of recently evicted probationary keys
    ghost: VecDeque<K>,    // front = oldest
    ghost_set: HashSet<K>, // O(1) membership
    ghost_cap: usize,

    capacity: usize,
}

// ── implementation ────────────────────────────────────────────────────────────

impl<K, V> LruCache<K, V>
where
    K: Hash + Eq + Clone + Debug,
    V: Debug,
{
    pub fn try_new(capacity: usize) -> Option<Self> {
        if capacity == 0 {
            log::error!("LruCache.try_new: capacity must be > 0");
            return None;
        }
        // 80/20 split via integer division.
        // capacity=1 → prot_cap=0, prob_cap=1 (all in prob; harmless, see below).
        let prot_cap = capacity * 4 / 5;
        let prob_cap = capacity - prot_cap;
        let ghost_cap = capacity;

        log::info!(
            "LruCache.try_new: capacity={capacity} prot_cap={prot_cap} prob_cap={prob_cap}"
        );
        Some(Self {
            map: HashMap::with_capacity(capacity),
            nodes: Vec::with_capacity(capacity),
            prob_head: None,
            prob_tail: None,
            prob_count: 0,
            prob_cap,
            prot_head: None,
            prot_tail: None,
            prot_count: 0,
            prot_cap,
            ghost: VecDeque::with_capacity(ghost_cap + 1),
            ghost_set: HashSet::with_capacity(ghost_cap + 1),
            ghost_cap,
            capacity,
        })
    }

    pub fn new(capacity: usize) -> Self {
        Self::try_new(capacity).expect("LruCache::new: initialization failed")
    }

    /// Returns the value for `key` if resident, updating segment recency and freq.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        let &idx = self.map.get(key)?;
        // Increment access frequency on every hit (both segments).
        self.nodes[idx].freq = self.nodes[idx].freq.saturating_add(1);
        match self.nodes[idx].segment {
            Segment::Protected => {
                // Touch: move to protected MRU.
                self.unlink_prot(idx);
                self.push_back_prot(idx);
            }
            Segment::Probationary => {
                // Promote to protected.
                self.unlink_prob(idx);
                self.prob_count -= 1;
                // Defensive: remove any stale ghost entry for this key.
                if self.ghost_set.remove(key) {
                    self.ghost.retain(|k| k != key);
                }
                self.nodes[idx].segment = Segment::Protected;
                self.push_back_prot(idx);
                self.prot_count += 1;
                // Protected overflow: demote lowest-freq from LRU quarter to probationary MRU.
                if self.prot_count > self.prot_cap {
                    self.demote_prot_head();
                }
            }
        }
        Some(&self.nodes[idx].value)
    }

    /// Inserts a key-value pair.
    ///
    /// When called through the `Cache` trait the contract guarantees the key is
    /// non-resident.  The direct method also handles re-puts for unit-test compat.
    pub fn put(&mut self, key: K, value: V) {
        // Re-put: key already resident — update value and refresh recency.
        if let Some(&idx) = self.map.get(&key) {
            self.nodes[idx].value = value;
            match self.nodes[idx].segment {
                Segment::Protected => {
                    self.unlink_prot(idx);
                    self.push_back_prot(idx);
                }
                Segment::Probationary => {
                    self.unlink_prob(idx);
                    self.push_back_prob(idx);
                }
            }
            return;
        }

        // ── Step 1: acquire an arena slot ────────────────────────────────
        let idx = if self.nodes.len() < self.capacity {
            // Arena still has headroom — allocate a fresh slot.
            let i = self.nodes.len();
            self.nodes.push(Node {
                key: key.clone(),
                value,
                prev: None,
                next: None,
                segment: Segment::Probationary, // corrected in step 2
                freq: 0,
            });
            i
        } else {
            // Arena full — evict to reclaim a slot.
            let evict_idx = if self.prob_count > 0 {
                // Normal path: evict probationary LRU.
                let h = self.prob_head.expect("prob_head must be Some when prob_count > 0");
                let evicted_key = self.nodes[h].key.clone();
                self.map.remove(&evicted_key);
                self.unlink_prob(h);
                self.prob_count -= 1;
                // Record the eviction in the ghost set.
                self.add_to_ghost(evicted_key);
                h
            } else {
                // Fallback: all items are protected — evict protected LRU.
                // Protected evictions are NOT added to the ghost set.
                let h = self.prot_head.expect("prot_head must be Some when prot_count > 0");
                let evicted_key = self.nodes[h].key.clone();
                self.map.remove(&evicted_key);
                self.unlink_prot(h);
                self.prot_count -= 1;
                h
            };
            self.nodes[evict_idx].key = key.clone();
            self.nodes[evict_idx].value = value;
            self.nodes[evict_idx].prev = None;
            self.nodes[evict_idx].next = None;
            self.nodes[evict_idx].freq = 0;
            evict_idx
        };

        // ── Step 2: place the node into the right segment ─────────────────
        if self.ghost_set.contains(&key) {
            // Ghost hit: key re-arrived after a previous probationary eviction.
            // Admit directly into protected, bypassing the one-hit filter.
            self.ghost_set.remove(&key);
            self.ghost.retain(|k| k != &key);

            self.nodes[idx].segment = Segment::Protected;
            self.push_back_prot(idx);
            self.prot_count += 1;
            self.map.insert(key, idx);

            // Protected overflow: demote lowest-freq from LRU quarter to probationary MRU.
            if self.prot_count > self.prot_cap {
                self.demote_prot_head();
            }
        } else {
            // Normal insertion: place in probationary MRU.
            self.nodes[idx].segment = Segment::Probationary;
            self.push_back_prob(idx);
            self.prob_count += 1;
            self.map.insert(key, idx);
        }

        debug_assert!(self.map.len() <= self.capacity);
    }

    // ── ghost helpers ─────────────────────────────────────────────────────────

    fn add_to_ghost(&mut self, key: K) {
        if self.ghost_cap == 0 {
            return;
        }
        if self.ghost.len() >= self.ghost_cap {
            if let Some(oldest) = self.ghost.pop_front() {
                self.ghost_set.remove(&oldest);
            }
        }
        self.ghost_set.insert(key.clone());
        self.ghost.push_back(key);
    }

    // ── segment helpers ───────────────────────────────────────────────────────

    /// Demote the lowest-frequency node from the LRU quarter of the protected
    /// segment to probationary MRU. Halve the demoted node's freq counter to
    /// preserve relative signal across demotion cycles.
    fn demote_prot_head(&mut self) {
        // Scan the LRU quarter (at least 1 node) for the minimum-freq victim.
        let scan_len = (self.prot_count / 4).max(1);

        let mut victim_idx: Option<usize> = None;
        let mut victim_freq = u32::MAX;
        let mut current = self.prot_head;
        let mut scanned = 0usize;

        while scanned < scan_len {
            let Some(idx) = current else { break };
            let freq = self.nodes[idx].freq;
            // Tie-break: take the first (LRU-most) node among equals.
            if freq < victim_freq {
                victim_freq = freq;
                victim_idx = Some(idx);
            }
            current = self.nodes[idx].next;
            scanned += 1;
        }

        if let Some(v) = victim_idx {
            self.unlink_prot(v);
            self.prot_count -= 1;
            self.nodes[v].segment = Segment::Probationary;
            self.push_back_prob(v);
            self.prob_count += 1;
            // Halve freq to preserve relative signal for future residency periods.
            self.nodes[v].freq /= 2;
        }
    }

    // ── probationary list ops ─────────────────────────────────────────────────

    fn unlink_prob(&mut self, idx: usize) {
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

    fn push_back_prob(&mut self, idx: usize) {
        self.nodes[idx].prev = self.prob_tail;
        self.nodes[idx].next = None;
        match self.prob_tail {
            Some(t) => self.nodes[t].next = Some(idx),
            None => self.prob_head = Some(idx),
        }
        self.prob_tail = Some(idx);
    }

    // ── protected list ops ────────────────────────────────────────────────────

    fn unlink_prot(&mut self, idx: usize) {
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

    fn push_back_prot(&mut self, idx: usize) {
        self.nodes[idx].prev = self.prot_tail;
        self.nodes[idx].next = None;
        match self.prot_tail {
            Some(t) => self.nodes[t].next = Some(idx),
            None => self.prot_head = Some(idx),
        }
        self.prot_tail = Some(idx);
    }
}

// ── Cache trait impl ──────────────────────────────────────────────────────────

impl<K, V> framework::Cache<K, V> for LruCache<K, V>
where
    K: Hash + Eq + Clone + Debug,
    V: Debug,
{
    fn put(&mut self, key: K, value: V) {
        LruCache::put(self, key, value);
    }

    fn get(&mut self, key: &K) -> Option<&V> {
        LruCache::get(self, key)
    }
}

// ── unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_miss_returns_none() {
        let mut cache = LruCache::<i64, i64>::new(4);
        assert!(cache.get(&42).is_none());
    }

    #[test]
    fn put_then_get() {
        let mut cache = LruCache::new(4);
        cache.put(1, 10);
        assert_eq!(*cache.get(&1).unwrap(), 10);
    }

    #[test]
    fn put_overwrites_existing_key() {
        let mut cache = LruCache::new(4);
        cache.put(1, 10);
        cache.put(1, 20);
        assert_eq!(*cache.get(&1).unwrap(), 20);
    }

    #[test]
    fn overwrite_does_not_grow_order() {
        // Re-puts must not allocate new slots.
        let mut cache = LruCache::new(2);
        cache.put(1, 10);
        cache.put(1, 20);
        cache.put(1, 30);
        cache.put(2, 40);
        assert_eq!(*cache.get(&1).unwrap(), 30);
        assert_eq!(*cache.get(&2).unwrap(), 40);
    }

    #[test]
    fn eviction_removes_lru() {
        // cap=2: prot_cap=1, prob_cap=1
        // put(1) → prob={1}
        // put(2) → prob={1,2}
        // put(3) → evict prob_head=1, ghost={1}; insert 3 → prob={2,3}
        let mut cache = LruCache::new(2);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30);

        assert!(cache.get(&1).is_none());
        assert_eq!(*cache.get(&2).unwrap(), 20);
        assert_eq!(*cache.get(&3).unwrap(), 30);
    }

    #[test]
    fn access_updates_lru_order() {
        // cap=2: prot_cap=1, prob_cap=1
        // get(1) promotes 1 to prot, so put(3) evicts prob_head=2.
        let mut cache = LruCache::new(2);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.get(&1); // 1 → prot; prob={2}
        cache.put(3, 30); // evict 2; prob={3}, prot={1}

        assert_eq!(*cache.get(&1).unwrap(), 10);
        assert!(cache.get(&2).is_none());
        assert_eq!(*cache.get(&3).unwrap(), 30);
    }

    #[test]
    fn put_updates_lru_order() {
        // cap=2: prot_cap=1, prob_cap=1
        // Re-put(1) moves 1 to prob tail, so put(3) evicts prob_head=2.
        let mut cache = LruCache::new(2);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(1, 11); // moves 1 to prob tail; prob={2,1}
        cache.put(3, 30); // evict 2; prob={1,3} — but actually evict prob_head=2

        assert_eq!(*cache.get(&1).unwrap(), 11);
        assert!(cache.get(&2).is_none());
        assert_eq!(*cache.get(&3).unwrap(), 30);
    }

    #[test]
    fn eviction_chain() {
        // cap=3: prot_cap=2, prob_cap=1
        // All inserts go to prob; evictions follow FIFO order of insertion.
        let mut cache = LruCache::new(3);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30);
        cache.put(4, 40); // evicts 1
        cache.put(5, 50); // evicts 2
        cache.put(6, 60); // evicts 3

        assert!(cache.get(&1).is_none());
        assert!(cache.get(&2).is_none());
        assert!(cache.get(&3).is_none());
        assert_eq!(*cache.get(&4).unwrap(), 40);
        assert_eq!(*cache.get(&5).unwrap(), 50);
        assert_eq!(*cache.get(&6).unwrap(), 60);
    }

    #[test]
    fn capacity_one() {
        // cap=1: prot_cap=0, prob_cap=1
        // Promotion: prob→prot triggers immediate demotion back to prob (prot_cap=0).
        let mut cache = LruCache::new(1);
        cache.put(1, 10);
        assert_eq!(*cache.get(&1).unwrap(), 10);
        cache.put(2, 20); // evicts 1
        assert!(cache.get(&1).is_none());
        assert_eq!(*cache.get(&2).unwrap(), 20);
    }

    #[test]
    fn overwrite_at_full_capacity_does_not_evict() {
        let mut cache = LruCache::new(2);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(1, 11); // re-put, must NOT evict key 2
        assert_eq!(*cache.get(&1).unwrap(), 11);
        assert_eq!(*cache.get(&2).unwrap(), 20);
    }

    #[test]
    fn repeated_get_preserves_eviction_order() {
        // cap=3: prot_cap=2, prob_cap=1
        // Repeated get on key 1 promotes it to prot; subsequent put evicts prob LRU.
        let mut cache = LruCache::new(3);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30);
        for _ in 0..10 {
            cache.get(&1); // 1 promoted to prot after first call
        }
        cache.put(4, 40); // evicts prob_head=2
        assert_eq!(*cache.get(&1).unwrap(), 10);
        assert!(cache.get(&2).is_none());
    }

    #[test]
    fn get_middle_node_promotes_correctly() {
        // cap=3: prot_cap=2, prob_cap=1
        let mut cache = LruCache::new(3);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30);
        cache.get(&2); // promote 2 to prot; prob={1,3}
        cache.put(4, 40); // evict prob_head=1; prob={3,4}, prot={2}
        assert!(cache.get(&1).is_none());
        assert_eq!(*cache.get(&3).unwrap(), 30);
        assert_eq!(*cache.get(&2).unwrap(), 20);
    }

    #[test]
    fn slot_reuse_after_promotion_and_re_eviction() {
        // cap=2: in SLRU the promoted key (3) stays in protected and survives.
        // The original plain-LRU expectation (key 3 evicted) does not apply.
        let mut cache = LruCache::new(2);
        cache.put(1, 10); // slot 0
        cache.put(2, 20); // slot 1
        cache.put(3, 30); // evicts 1; slot 0 reused for 3
        cache.get(&3);    // promote 3 to prot
        cache.put(4, 40); // evicts prob_head=2; slot 1 reused for 4
        cache.put(5, 50); // evicts prob_head=4; slot reused for 5

        // Key 3 is in prot and must remain resident.
        assert_eq!(*cache.get(&3).unwrap(), 30);
        // Key 5 was just inserted into prob and must be resident.
        assert_eq!(*cache.get(&5).unwrap(), 50);
        // Key 4 was evicted.
        assert!(cache.get(&4).is_none());
    }

    #[test]
    fn get_only_element() {
        let mut cache = LruCache::new(3);
        cache.put(1, 10);
        assert_eq!(*cache.get(&1).unwrap(), 10);
        cache.put(2, 20);
        cache.put(3, 30);
        assert_eq!(*cache.get(&1).unwrap(), 10);
    }

    #[test]
    fn overwrite_mru_at_capacity() {
        // cap=3: re-put on key 3 (MRU) should leave key 1 as LRU.
        let mut cache = LruCache::new(3);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30);
        cache.put(3, 31); // re-put MRU; prob order stays {1,2,3}
        cache.put(4, 40); // evict prob_head=1
        assert!(cache.get(&1).is_none());
        assert_eq!(*cache.get(&2).unwrap(), 20);
        assert_eq!(*cache.get(&3).unwrap(), 31);
        assert_eq!(*cache.get(&4).unwrap(), 40);
    }

    #[test]
    #[should_panic(expected = "initialization failed")]
    fn capacity_zero_panics() {
        LruCache::<i64, i64>::new(0);
    }

    /// Ghost-hit mechanism: a probationary key that was evicted and re-arrives
    /// is admitted directly into the protected segment.
    ///
    /// cap=8: prot_cap=6, prob_cap=2, ghost_cap=8 (expanded to total capacity).
    ///
    /// Sequence:
    ///   1. Fill all 8 slots → everything in prob.
    ///   2. put(9) → evicts key 1 from prob; ghost = {1}.
    ///   3. get(2..=7) → promote 6 keys to prot (no new prob evictions, ghost unchanged).
    ///   4. put(1): evict prob_head=8 → ghost = {1, 8} (fits in ghost_cap=8); then check:
    ///      ghost_set contains 1 → HIT; admit key 1 directly into protected.
    #[test]
    fn ghost_hit_admits_to_protected() {
        let mut cache = LruCache::<i64, i64>::new(8);
        for k in 1i64..=8 {
            cache.put(k, k * 10);
        }
        // Evict key 1 from prob into ghost.
        cache.put(9, 90);
        // Promote keys 2..=7 to prot via gets (no prob evictions; ghost stays {1}).
        for k in 2i64..=7 {
            cache.get(&k);
        }
        // State: prob={8,9}, prot={2,3,4,5,6,7} (prot full at 6), ghost={1}.
        // put(1): evict prob_head=8 → ghost={1,8} (fits in ghost_cap=8).
        //         ghost_set.contains(1) → true → ghost hit!
        cache.put(1, 99);

        // Key 1 must be resident with the newly inserted value.
        assert_eq!(*cache.get(&1).unwrap(), 99);
        // Key 8 was evicted to make room (no ghost hit for it; it's not re-arrived).
        assert!(cache.get(&8).is_none());
        // Capacity invariant.
        let resident = (1i64..=9).filter(|&k| cache.get(&k).is_some()).count();
        assert!(resident <= 8, "capacity violated: {resident} residents");
    }
}
