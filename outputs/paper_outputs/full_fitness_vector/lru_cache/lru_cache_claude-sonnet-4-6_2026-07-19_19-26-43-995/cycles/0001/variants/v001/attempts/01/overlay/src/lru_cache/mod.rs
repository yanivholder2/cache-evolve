// Two-queue (2Q) cache: probationary FIFO queue + protected LRU queue.
//
// New items enter the probationary queue (FIFO). On a cache hit:
//   - Probationary hit → promoted to protected LRU tail; if protected exceeds
//     its budget, its LRU head is demoted back to the probationary tail.
//   - Protected hit → refreshed in place (LRU within protected queue).
//
// Eviction: always takes from the probationary head first (FIFO), falling back
// to the protected head (LRU of protected) only when probationary is empty.
//
// Budgets: prob_cap = (capacity / 4).max(1), prot_cap = capacity - prob_cap.
// The total number of resident items is always ≤ capacity.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

#[derive(Debug)]
pub struct Node<K, V> {
    key: K,
    value: V,
    prev: Option<usize>,
    next: Option<usize>,
    protected: bool, // false = probationary queue, true = protected queue
}

#[derive(Debug)]
pub struct LruCache<K, V> {
    map: HashMap<K, usize>, // key -> arena index
    nodes: Vec<Node<K, V>>, // arena

    // Probationary FIFO queue: head is LRU (eviction candidate), tail is MRU.
    prob_head: Option<usize>,
    prob_tail: Option<usize>,
    prob_len: usize,

    // Protected LRU queue: head is LRU (demotion candidate), tail is MRU.
    prot_head: Option<usize>,
    prot_tail: Option<usize>,
    prot_len: usize,

    prob_cap: usize, // probationary budget = (capacity / 4).max(1)
    capacity: usize, // total capacity
}

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
        let prob_cap = (capacity / 4).max(1);
        log::info!("LruCache.try_new: capacity={capacity} prob_cap={prob_cap}");
        Some(Self {
            map: HashMap::with_capacity(capacity),
            nodes: Vec::with_capacity(capacity),
            prob_head: None,
            prob_tail: None,
            prob_len: 0,
            prot_head: None,
            prot_tail: None,
            prot_len: 0,
            prob_cap,
            capacity,
        })
    }

    pub fn new(capacity: usize) -> Self {
        Self::try_new(capacity).expect("LruCache::new: initialization failed")
    }

    /// Returns the value for `key` if present, promoting it according to its queue.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        let &idx = self.map.get(key)?;
        self.promote(idx);
        Some(&self.nodes[idx].value)
    }

    /// Inserts or updates `key`. On a miss, inserts into probationary queue,
    /// evicting if necessary. On a hit, updates value and promotes as if accessed.
    ///
    /// Note: per the framework contract, `put` is only called for non-resident keys
    /// (get-miss then put). The existing-key path is retained for unit-test
    /// compatibility.
    pub fn put(&mut self, key: K, value: V) {
        // Existing key: update value, then promote as a get would.
        if let Some(&idx) = self.map.get(&key) {
            self.nodes[idx].value = value;
            self.promote(idx);
            return;
        }

        // New key: acquire an arena slot, evicting if necessary.
        let idx = if self.nodes.len() < self.capacity {
            // Arena still has free slots — allocate a new one.
            let new_idx = self.nodes.len();
            self.nodes.push(Node {
                key: key.clone(),
                value,
                prev: None,
                next: None,
                protected: false,
            });
            new_idx
        } else {
            // Arena full — evict one entry and reuse its slot.
            let slot = self.evict_one();
            self.nodes[slot].key = key.clone();
            self.nodes[slot].value = value;
            slot
        };

        // Link the slot at the probationary tail and register in the map.
        self.push_back_prob(idx);
        self.prob_len += 1;
        self.map.insert(key, idx);
        debug_assert!(self.map.len() <= self.capacity);
    }

    // ── Internal helpers ────────────────────────────────────────────────────

    /// Promote node `idx` according to its current queue:
    ///   - Probationary → move to protected tail; demote protected head if
    ///     the protected queue would exceed `capacity - prob_cap`.
    ///   - Protected → LRU refresh (unlink + push to protected tail).
    fn promote(&mut self, idx: usize) {
        if self.nodes[idx].protected {
            // Protected hit: refresh position within the protected LRU queue.
            self.unlink(idx);
            self.push_back_prot(idx);
        } else {
            // Probationary hit: graduate to protected queue.
            self.unlink(idx);
            self.prob_len -= 1;
            self.push_back_prot(idx);
            self.prot_len += 1;

            // Enforce the protected budget: demote the LRU of protected back
            // to the probationary tail if the budget is exceeded.
            let prot_cap = self.capacity - self.prob_cap;
            if self.prot_len > prot_cap {
                // Safety: prot_len > 0 implies prot_head is Some.
                let h = self.prot_head.expect("prot_len > 0 implies prot_head is Some");
                self.unlink(h);
                self.prot_len -= 1;
                self.push_back_prob(h);
                self.prob_len += 1;
            }
        }
    }

    /// Evict one entry and return its (now-free) arena slot index.
    ///
    /// Eviction order: probationary head (FIFO) → protected head (LRU fallback).
    fn evict_one(&mut self) -> usize {
        if self.prob_len > 0 {
            let h = self.prob_head.expect("prob_len > 0 implies prob_head is Some");
            let evicted_key = self.nodes[h].key.clone();
            self.map.remove(&evicted_key);
            self.unlink(h);
            self.prob_len -= 1;
            h
        } else {
            let h = self.prot_head.expect("non-empty cache must have items");
            let evicted_key = self.nodes[h].key.clone();
            self.map.remove(&evicted_key);
            self.unlink(h);
            self.prot_len -= 1;
            h
        }
    }

    /// Unlink node `idx` from whichever queue it currently belongs to.
    ///
    /// Uses `nodes[idx].protected` to update the correct head/tail pointers.
    /// Clears the node's `prev`/`next` links but leaves `protected` unchanged
    /// (the push helpers set it on re-insertion).
    fn unlink(&mut self, idx: usize) {
        let prev = self.nodes[idx].prev;
        let next = self.nodes[idx].next;
        let protected = self.nodes[idx].protected;

        match prev {
            Some(p) => self.nodes[p].next = next,
            None => {
                // `idx` was at the head of its queue.
                if protected {
                    self.prot_head = next;
                } else {
                    self.prob_head = next;
                }
            }
        }
        match next {
            Some(n) => self.nodes[n].prev = prev,
            None => {
                // `idx` was at the tail of its queue.
                if protected {
                    self.prot_tail = prev;
                } else {
                    self.prob_tail = prev;
                }
            }
        }

        self.nodes[idx].prev = None;
        self.nodes[idx].next = None;
    }

    /// Append node `idx` to the tail (MRU end) of the probationary queue.
    fn push_back_prob(&mut self, idx: usize) {
        self.nodes[idx].prev = self.prob_tail;
        self.nodes[idx].next = None;
        self.nodes[idx].protected = false;
        match self.prob_tail {
            Some(t) => self.nodes[t].next = Some(idx),
            None => self.prob_head = Some(idx),
        }
        self.prob_tail = Some(idx);
    }

    /// Append node `idx` to the tail (MRU end) of the protected queue.
    fn push_back_prot(&mut self, idx: usize) {
        self.nodes[idx].prev = self.prot_tail;
        self.nodes[idx].next = None;
        self.nodes[idx].protected = true;
        match self.prot_tail {
            Some(t) => self.nodes[t].next = Some(idx),
            None => self.prot_head = Some(idx),
        }
        self.prot_tail = Some(idx);
    }
}

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
        let mut cache = LruCache::new(2);
        cache.put(1, 10);
        cache.put(1, 20);
        cache.put(1, 30);
        // Should still have room for a second key without eviction
        cache.put(2, 40);
        assert_eq!(*cache.get(&1).unwrap(), 30);
        assert_eq!(*cache.get(&2).unwrap(), 40);
    }

    #[test]
    fn eviction_removes_probationary_head() {
        // With 2Q: all three items enter prob. Eviction takes prob head (1).
        let mut cache = LruCache::new(2);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30); // evicts 1 (prob head)

        assert!(cache.get(&1).is_none());
        assert_eq!(*cache.get(&2).unwrap(), 20);
        assert_eq!(*cache.get(&3).unwrap(), 30);
    }

    #[test]
    fn access_protects_item_from_eviction() {
        // Accessing 1 promotes it to protected; next eviction takes prob head (2).
        let mut cache = LruCache::new(2);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.get(&1); // promotes 1 to protected
        cache.put(3, 30); // evicts 2 (prob head), not 1

        assert_eq!(*cache.get(&1).unwrap(), 10);
        assert!(cache.get(&2).is_none());
        assert_eq!(*cache.get(&3).unwrap(), 30);
    }

    #[test]
    fn put_existing_key_promotes_to_protected() {
        // put on an existing key behaves like get-then-update: promotes.
        let mut cache = LruCache::new(2);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(1, 11); // update key 1, promotes it to protected
        cache.put(3, 30); // evicts 2 (prob head)

        assert_eq!(*cache.get(&1).unwrap(), 11);
        assert!(cache.get(&2).is_none());
        assert_eq!(*cache.get(&3).unwrap(), 30);
    }

    #[test]
    fn eviction_chain() {
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
        cache.put(2, 20); // cache full
        cache.put(1, 11); // overwrite, must NOT evict key 2
        assert_eq!(*cache.get(&1).unwrap(), 11);
        assert_eq!(*cache.get(&2).unwrap(), 20);
    }

    #[test]
    fn repeated_get_keeps_item_in_protected() {
        // Repeated accesses to key 1 keep it protected; new inserts evict prob items.
        let mut cache = LruCache::new(3);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30);
        for _ in 0..10 {
            cache.get(&1);
        }
        cache.put(4, 40); // should evict key 2 (prob head), not key 1
        assert_eq!(*cache.get(&1).unwrap(), 10);
        assert!(cache.get(&2).is_none());
    }

    #[test]
    fn get_middle_node_promotes_correctly() {
        let mut cache = LruCache::new(3);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30);
        cache.get(&2); // promote 2 to protected; prob = [1, 3]
        cache.put(4, 40); // evicts 1 (prob head)
        assert!(cache.get(&1).is_none());
        assert_eq!(*cache.get(&3).unwrap(), 30);
        assert_eq!(*cache.get(&2).unwrap(), 20);
    }

    #[test]
    fn protected_item_survives_subsequent_puts() {
        // 2Q: key 3 is promoted to protected and survives two subsequent puts.
        let mut cache = LruCache::new(2);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30); // evicts 1; prob = [2, 3]
        cache.get(&3);    // promote 3 to protected; prob = [2], prot = [3]
        cache.put(4, 40); // evicts 2 (prob head); prob = [4], prot = [3]
        cache.put(5, 50); // evicts 4 (prob head); prob = [5], prot = [3]
        assert_eq!(*cache.get(&3).unwrap(), 30); // still resident (protected)
        assert!(cache.get(&4).is_none());
        assert_eq!(*cache.get(&5).unwrap(), 50);
    }

    #[test]
    fn get_only_element() {
        let mut cache = LruCache::new(3);
        cache.put(1, 10);
        assert_eq!(*cache.get(&1).unwrap(), 10);
        // After get, 1 is in protected. Verify list is still intact.
        cache.put(2, 20);
        cache.put(3, 30);
        assert_eq!(*cache.get(&1).unwrap(), 10);
    }

    #[test]
    fn overwrite_mru_at_capacity() {
        let mut cache = LruCache::new(3);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30); // full
        cache.put(3, 31); // overwrite 3 (prob) → promotes 3 to protected
        cache.put(4, 40); // evicts 1 (prob head)
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

    #[test]
    fn probationary_budget_maintained() {
        // With capacity=4, prob_cap=1, prot_cap=3.
        // After filling with new items, prot overflow causes demotion.
        let mut cache = LruCache::new(4);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30);
        cache.put(4, 40);
        // All in prob. Now access 1, 2, 3 to promote them.
        cache.get(&1); // prob=[2,3,4], prot=[1]. prot_len=1 ≤ prot_cap=3. OK.
        cache.get(&2); // prob=[3,4], prot=[1,2]. prot_len=2 ≤ 3. OK.
        cache.get(&3); // prob=[4], prot=[1,2,3]. prot_len=3 ≤ 3. OK.
        // All values still accessible.
        assert_eq!(*cache.get(&1).unwrap(), 10);
        assert_eq!(*cache.get(&2).unwrap(), 20);
        assert_eq!(*cache.get(&3).unwrap(), 30);
        assert_eq!(*cache.get(&4).unwrap(), 40);
    }

    #[test]
    fn demotion_on_protected_overflow() {
        // capacity=2, prob_cap=1, prot_cap=1.
        // When a probationary item is promoted, the existing protected item
        // is demoted back to probationary.
        let mut cache = LruCache::new(2);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.get(&1); // promote 1 to prot. prot_len=1 ≤ 1. prob=[2], prot=[1].
        cache.get(&2); // promote 2 to prot. prot_len=2 > 1. Demote prot head (1) to prob.
                       // prob=[1], prot=[2].
        // Both items still accessible.
        assert_eq!(*cache.get(&1).unwrap(), 10);
        assert_eq!(*cache.get(&2).unwrap(), 20);
    }
}
