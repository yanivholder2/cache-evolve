// O(1) two-queue (2Q) cache using HashMap + arena-based doubly-linked list.
// Extends v002's u32-sentinel node layout with a probationary FIFO queue and a
// protected LRU queue. Bits 30-31 of the `prev` field encode queue membership
// and first-hit state, keeping Node<K,V> at 24 bytes with no additional field.
//
// Sentinel / bit constants:
//   NULL     = 0x3FFF_FFFF  (bits 0-29 all set; bits 30-31 reserved for flags)
//   PROT_BIT = 0x8000_0000  (bit 31 of prev: node is in the protected queue)
//   SEEN_BIT = 0x4000_0000  (bit 30 of prev: probationary node accessed once)
//   IDX_MASK = 0x3FFF_FFFF  (strips both PROT_BIT and SEEN_BIT)
//
// Policy sketch:
//   • New entries enter the probationary queue (FIFO, evicted oldest-first).
//   • First probationary hit: set SEEN_BIT in prev (no relinking, O(1) bit-OR).
//   • Second probationary hit: graduate the entry to the protected queue (LRU).
//   • A protected hit refreshes the entry to the MRU end of protected.
//   • When protected exceeds its quota (capacity - prob_cap), the LRU of
//     protected is demoted to the MRU end of probationary.
//   • Eviction always drains probationary first; protected is a fallback.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

/// Null sentinel for u32 link fields.
/// Bits 0-29 are all set; bits 30-31 are reserved for SEEN_BIT and PROT_BIT.
const NULL: u32     = 0x3FFF_FFFF;  // bits 0-29 all set; bits 30-31 reserved for flags

/// Bit 31 of the `prev` field: set when the node lives in the protected queue.
const PROT_BIT: u32 = 0x8000_0000;  // bit 31: node is in the protected queue

/// Bit 30 of the `prev` field: set when a probationary node has been accessed once.
const SEEN_BIT: u32 = 0x4000_0000;  // bit 30: probationary node has been accessed once

/// Mask to extract the raw index from a `prev` field (strips both PROT_BIT and SEEN_BIT).
const IDX_MASK: u32 = 0x3FFF_FFFF;  // strips both PROT_BIT and SEEN_BIT

#[derive(Debug)]
pub struct Node<K, V> {
    key: K,
    value: V,
    /// Dual-purpose u32:
    ///   bit 31    = PROT_BIT  (which queue this node belongs to)
    ///   bit 30    = SEEN_BIT  (probationary first-hit marker)
    ///   bits 0-29 = prev index (or NULL = 0x3FFF_FFFF when no predecessor)
    prev: u32,
    next: u32,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
    map: HashMap<K, usize>, // key -> node index
    nodes: Vec<Node<K, V>>, // arena
    /// LRU end of probationary queue (eviction candidate); NULL if empty.
    prob_head: u32,
    /// MRU end of probationary queue; NULL if empty.
    prob_tail: u32,
    /// LRU end of protected queue; NULL if empty.
    prot_head: u32,
    /// MRU end of protected queue; NULL if empty.
    prot_tail: u32,
    prob_len: usize,
    prot_len: usize,
    /// Probationary capacity = (capacity / 4).max(1).
    prob_cap: usize,
    capacity: usize,
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
        // Valid indices run 0 ..= 0x3FFF_FFFE; NULL = 0x3FFF_FFFF is the sentinel.
        if capacity > 0x3FFF_FFFFusize - 1 {
            log::error!("LruCache.try_new: capacity exceeds u32 sentinel range");
            return None;
        }
        log::info!("LruCache.try_new: capacity={capacity}");
        let prob_cap = (capacity / 4).max(1);
        Some(Self {
            map: HashMap::with_capacity(capacity),
            nodes: Vec::with_capacity(capacity),
            prob_head: NULL,
            prob_tail: NULL,
            prot_head: NULL,
            prot_tail: NULL,
            prob_len: 0,
            prot_len: 0,
            prob_cap,
            capacity,
        })
    }

    pub fn new(capacity: usize) -> Self {
        Self::try_new(capacity).expect("LruCache::new: initialization failed")
    }

    /// Returns the value for `key` if present, promoting it through the 2Q queues.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        let &idx = self.map.get(key)?;
        self.promote(idx);
        Some(&self.nodes[idx].value)
    }

    /// Inserts or updates `key`. If at capacity, evicts according to the 2Q policy.
    pub fn put(&mut self, key: K, value: V) {
        // Existing key: update value in-place and promote through the 2Q queues.
        if let Some(&idx) = self.map.get(&key) {
            self.nodes[idx].value = value;
            self.promote(idx);
            return;
        }
        // New key — need an arena slot.
        let idx = if self.nodes.len() >= self.capacity {
            // Arena full: evict via 2Q policy and reuse slot.
            let slot = self.evict_one();
            self.nodes[slot].key = key.clone();
            self.nodes[slot].value = value;
            slot
        } else {
            // Arena has room: allocate a new slot.
            let new_idx = self.nodes.len();
            self.nodes.push(Node {
                key: key.clone(),
                value,
                prev: NULL,
                next: NULL,
            });
            new_idx
        };
        // All new insertions enter the probationary queue at its MRU end.
        self.push_back_prob(idx);
        self.prob_len += 1;
        self.map.insert(key, idx);
        debug_assert!(self.map.len() <= self.capacity);
        debug_assert_eq!(self.prob_len + self.prot_len, self.map.len());
    }

    /// Promote a node: refresh in protected (if already protected), or apply the
    /// two-hit filter for probationary nodes. First probationary hit marks SEEN_BIT
    /// with a single bitwise OR (no relinking). Second probationary hit graduates
    /// the node to protected. Enforces the protected-queue budget.
    fn promote(&mut self, idx: usize) {
        let raw_prev = self.nodes[idx].prev;
        let protected = (raw_prev & PROT_BIT) != 0;
        if protected {
            // Protected hit: refresh to the MRU end of the protected queue.
            self.unlink(idx);
            self.push_back_prot(idx);
            // prot_len is unchanged (item stays in protected).
        } else {
            // Probationary hit: check SEEN_BIT to decide whether to graduate.
            let has_seen = (self.nodes[idx].prev & SEEN_BIT) != 0;
            if !has_seen {
                // First hit: mark as seen with a single bit-OR.
                // Do NOT relink — the node stays at its current FIFO position in
                // the probationary queue, keeping this path a branchless write.
                self.nodes[idx].prev |= SEEN_BIT;
                return;
            }
            // Second+ hit: graduate to protected (SEEN_BIT is cleared automatically
            // when push_back_prot writes PROT_BIT | prot_tail into prev).
            self.unlink(idx);
            self.prob_len -= 1;
            self.push_back_prot(idx);
            self.prot_len += 1;
            // Enforce the protected budget.
            let prot_cap = self.capacity - self.prob_cap;
            if self.prot_len > prot_cap {
                let h = (self.prot_head & IDX_MASK) as usize;
                self.unlink(h);
                self.prot_len -= 1;
                self.push_back_prob(h);
                self.prob_len += 1;
            }
        }
    }

    /// Evict one entry via the 2Q policy (probationary first, protected as
    /// fallback), remove it from the map, and return the freed slot index.
    fn evict_one(&mut self) -> usize {
        if self.prob_len > 0 {
            let h = (self.prob_head & IDX_MASK) as usize;
            let evicted_key = self.nodes[h].key.clone();
            self.map.remove(&evicted_key);
            self.unlink(h);
            self.prob_len -= 1;
            h
        } else {
            debug_assert!(self.prot_len > 0, "no items to evict");
            let h = (self.prot_head & IDX_MASK) as usize;
            let evicted_key = self.nodes[h].key.clone();
            self.map.remove(&evicted_key);
            self.unlink(h);
            self.prot_len -= 1;
            h
        }
    }

    /// Remove a node from its queue's linked list.
    /// Dispatches on PROT_BIT in `prev` to update the correct head/tail pointer.
    fn unlink(&mut self, idx: usize) {
        let raw_prev = self.nodes[idx].prev;
        let protected = (raw_prev & PROT_BIT) != 0;
        let prev_idx = raw_prev & IDX_MASK; // raw index (or NULL sentinel)
        let next = self.nodes[idx].next;

        // Fix the forward link of the predecessor (or update the queue head).
        if prev_idx != NULL {
            self.nodes[prev_idx as usize].next = next;
        } else if protected {
            self.prot_head = next;
        } else {
            self.prob_head = next;
        }

        // Fix the backward link of the successor (or update the queue tail).
        if next != NULL {
            // Preserve the successor's own PROT_BIT and SEEN_BIT; replace only its
            // index part. A successor that has itself been accessed once will have
            // SEEN_BIT set; clearing it during an unlink of a different node would
            // silently reset its admission state.
            let succ_flags = self.nodes[next as usize].prev & (PROT_BIT | SEEN_BIT);
            self.nodes[next as usize].prev = succ_flags | prev_idx;
        } else if protected {
            self.prot_tail = prev_idx;
        } else {
            self.prob_tail = prev_idx;
        }

        // Clear the unlinked node's link fields (NULL has PROT_BIT = SEEN_BIT = 0).
        self.nodes[idx].prev = NULL;
        self.nodes[idx].next = NULL;
    }

    /// Append a node to the MRU end of the probationary queue.
    /// Clears PROT_BIT and SEEN_BIT in `prev` to mark the node as probationary.
    fn push_back_prob(&mut self, idx: usize) {
        let idx32 = idx as u32;
        // prob_tail is a raw index (no PROT_BIT, no SEEN_BIT), so assigning it
        // directly to prev leaves both flag bits = 0 (probationary, unseen).
        self.nodes[idx].prev = self.prob_tail;
        self.nodes[idx].next = NULL;
        if self.prob_tail != NULL {
            self.nodes[self.prob_tail as usize].next = idx32;
        } else {
            self.prob_head = idx32;
        }
        self.prob_tail = idx32;
    }

    /// Append a node to the MRU end of the protected queue.
    /// Sets PROT_BIT in `prev` to mark the node as protected (SEEN_BIT stays 0).
    fn push_back_prot(&mut self, idx: usize) {
        let idx32 = idx as u32;
        // OR-in PROT_BIT so the node is marked as belonging to the protected queue.
        // prot_tail is a raw index with no flags, so SEEN_BIT is naturally 0.
        self.nodes[idx].prev = PROT_BIT | self.prot_tail;
        self.nodes[idx].next = NULL;
        if self.prot_tail != NULL {
            self.nodes[self.prot_tail as usize].next = idx32;
        } else {
            self.prot_head = idx32;
        }
        self.prot_tail = idx32;
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
    fn eviction_removes_prob_lru() {
        // All three entries start in prob; eviction takes the LRU of prob (key 1).
        let mut cache = LruCache::new(2);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30); // evicts key 1 (prob head)

        assert!(cache.get(&1).is_none());
        assert_eq!(*cache.get(&2).unwrap(), 20);
        assert_eq!(*cache.get(&3).unwrap(), 30);
    }

    #[test]
    fn first_hit_sets_seen_bit_no_relink() {
        // With SEEN_BIT filter: a single hit only marks the node as seen;
        // it stays at its FIFO position in probationary (does not graduate).
        // So a subsequent eviction still drains the oldest prob entry first.
        //
        // capacity=2, prob_cap=1, prot_cap=1
        let mut cache = LruCache::new(2);
        cache.put(1, 10); // prob=[1]
        cache.put(2, 20); // prob=[1,2]
        cache.get(&1);    // first hit: SEEN_BIT on key 1, still in prob (LRU position)
        cache.put(3, 30); // arena full; evicts prob_head = key 1 (SEEN_BIT doesn't protect it)
        assert!(cache.get(&1).is_none()); // key 1 evicted
        assert_eq!(*cache.get(&2).unwrap(), 20);
        assert_eq!(*cache.get(&3).unwrap(), 30);
    }

    #[test]
    fn two_hits_graduate_to_protected() {
        // With SEEN_BIT filter: two probationary hits graduate to protected.
        // capacity=3, prob_cap=1, prot_cap=2
        let mut cache = LruCache::new(3);
        cache.put(1, 10); // prob=[1]
        cache.put(2, 20); // prob=[1,2]
        cache.put(3, 30); // prob=[1,2,3]
        cache.get(&1);    // first hit on key 1: SEEN_BIT, prob=[1(SEEN),2,3]
        cache.get(&1);    // second hit: graduate key 1 to protected
        // prob=[2,3], prot=[1], prot_len=1 <= prot_cap=2, no demotion
        cache.put(4, 40); // evicts key 2 (prob LRU)
        assert_eq!(*cache.get(&1).unwrap(), 10); // key 1 is in protected
        assert!(cache.get(&2).is_none());         // key 2 was evicted
        assert_eq!(*cache.get(&3).unwrap(), 30);
        assert_eq!(*cache.get(&4).unwrap(), 40);
    }

    #[test]
    fn protected_hit_refreshes_mru() {
        // After two hits, key 1 is in protected (MRU end).
        // A protected hit (get) moves it back to MRU, shielding it from demotion.
        let mut cache = LruCache::new(3);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30);
        // Graduate key 1 and key 2 to protected (2 hits each).
        cache.get(&1); cache.get(&1); // prot=[1]
        cache.get(&2); cache.get(&2); // prot=[1,2], prob_cap=1, prot_cap=2 → no demotion yet
        // prot=[1,2], prot_len=2 == prot_cap=2; next graduation forces demotion.
        cache.get(&3); cache.get(&3); // graduate key 3 → prot=[1,2,3], len=3 > cap=2
        // Demote prot LRU = key 1 back to prob.
        // prob=[3→1 demoted], prot=[2,3]
        cache.put(4, 40); // evicts prob_head = key 1
        assert!(cache.get(&1).is_none());
        assert_eq!(*cache.get(&2).unwrap(), 20);
        assert_eq!(*cache.get(&3).unwrap(), 30);
    }

    #[test]
    fn put_updates_eviction_order() {
        // put on an existing key counts as a second hit → graduation to protected.
        // capacity=3, prob_cap=1, prot_cap=2
        let mut cache = LruCache::new(3);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30);
        cache.get(&1);    // first hit on key 1: SEEN_BIT
        cache.put(1, 11); // second access (put on existing): graduates key 1 to protected
        // prob=[2,3], prot=[1]
        cache.put(4, 40); // evicts key 2 (prob LRU)

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
    fn repeated_get_keeps_item_from_eviction() {
        // Two hits on key 1 graduate it to protected; repeated hits keep it as MRU
        // of protected so it is never demoted.
        // capacity=3, prob_cap=1, prot_cap=2
        let mut cache = LruCache::new(3);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30);
        for _ in 0..10 {
            // first iteration: SEEN_BIT; second+ graduation + protected refresh
            cache.get(&1);
        }
        // key 1 is in protected (MRU). Eviction drains prob first.
        cache.put(4, 40); // evicts key 2 (prob LRU), not key 1
        assert_eq!(*cache.get(&1).unwrap(), 10);
        assert!(cache.get(&2).is_none());
    }

    #[test]
    fn get_middle_node_promotes_correctly() {
        // capacity=3, prob_cap=1, prot_cap=2
        let mut cache = LruCache::new(3);
        cache.put(1, 10); // prob LRU
        cache.put(2, 20); // prob middle
        cache.put(3, 30); // prob MRU
        cache.get(&2);    // first hit: SEEN_BIT on key 2
        cache.get(&2);    // second hit: graduate key 2 to protected
        cache.put(4, 40); // evicts key 1 (prob LRU)
        assert!(cache.get(&1).is_none());
        assert_eq!(*cache.get(&3).unwrap(), 30);
        assert_eq!(*cache.get(&2).unwrap(), 20);
    }

    #[test]
    fn slot_reuse_with_2q_protection() {
        // With 2Q (capacity=2, prob_cap=1, prot_cap=1):
        //   Two hits on key 3 graduate it to protected.
        let mut cache = LruCache::new(2);
        cache.put(1, 10); // slot 0
        cache.put(2, 20); // slot 1
        cache.put(3, 30); // evicts 1 → slot 0 reused for key 3
        cache.get(&3);    // first hit: SEEN_BIT on key 3
        cache.get(&3);    // second hit: promote slot 0 (key 3) to protected
        cache.put(4, 40); // evicts 2 (prob) → slot 1 reused for key 4
        cache.put(5, 50); // evicts 4 (prob) → slot 1 reused for key 5; key 3 stays protected
        assert!(cache.get(&1).is_none());
        assert!(cache.get(&2).is_none());
        assert!(cache.get(&4).is_none());        // 4 was in prob and got evicted
        assert_eq!(*cache.get(&3).unwrap(), 30); // 3 is still protected
        assert_eq!(*cache.get(&5).unwrap(), 50);
    }

    #[test]
    fn get_only_element() {
        let mut cache = LruCache::new(3);
        cache.put(1, 10);
        // Single node: two gets graduate it to protected (no budget overflow since prot_cap=2).
        cache.get(&1); // first hit: SEEN_BIT
        cache.get(&1); // second hit: graduate to protected
        // Verify list is still intact after promotion.
        cache.put(2, 20);
        cache.put(3, 30);
        assert_eq!(*cache.get(&1).unwrap(), 10);
    }

    #[test]
    fn overwrite_mru_at_capacity() {
        // capacity=3, prob_cap=1, prot_cap=2
        let mut cache = LruCache::new(3);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30); // full; 3 is MRU (tail of prob)
        cache.get(&3);    // first hit on key 3: SEEN_BIT
        cache.put(3, 31); // second access (put overwrites) → graduate key 3 to protected
                          // prob: {key1, key2}; prot: {key3}
        cache.put(4, 40); // evicts key 1 (prob LRU)
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
    fn seen_bit_preserved_through_unlink_of_predecessor() {
        // Regression: unlinking a node that precedes a SEEN_BIT node must not
        // clear the successor's SEEN_BIT.
        // capacity=4, prob_cap=1, prot_cap=3
        let mut cache = LruCache::new(4);
        cache.put(1, 10); // prob: [1]
        cache.put(2, 20); // prob: [1,2]
        cache.put(3, 30); // prob: [1,2,3]
        cache.get(&3);    // first hit on key 3: SEEN_BIT; prob=[1,2,3(SEEN)]
        cache.put(4, 40); // prob: [1,2,3(SEEN),4], nodes.len()=4 == capacity; no eviction yet
        // Now give node 3 its second hit — SEEN_BIT must still be set despite
        // push_back_prob(4) having updated node 3's next field.
        cache.get(&3);    // second hit: graduate key 3 to protected
        // prob=[1,2,4], prot=[3], prot_len=1 <= prot_cap=3, no demotion
        cache.put(5, 50); // nodes full: evict prob_head = key 1
        assert_eq!(*cache.get(&3).unwrap(), 30); // key 3 in protected
        assert!(cache.get(&1).is_none());         // key 1 evicted
        assert_eq!(*cache.get(&2).unwrap(), 20);
        assert_eq!(*cache.get(&4).unwrap(), 40);
        assert_eq!(*cache.get(&5).unwrap(), 50);
    }
}
