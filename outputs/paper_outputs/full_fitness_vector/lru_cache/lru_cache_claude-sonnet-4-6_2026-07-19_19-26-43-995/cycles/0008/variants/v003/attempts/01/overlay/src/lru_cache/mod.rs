// O(1) two-queue (2Q) cache using HashMap + arena-based doubly-linked list.
// Extends v002's u32-sentinel node layout with a probationary FIFO queue and a
// protected LRU queue.
//
// H02: prev/next link fields are moved out of Node<K,V> into a dedicated
// Vec<Link>, shrinking Node<u64,u64> from 24 to 16 bytes and packing 8 link
// pairs per 64-byte cache line on the hot traversal paths.
//
// H03: prob_cap is reduced from capacity/4 to capacity/16, giving 93.75%
// protected quota. At capacity=2048 this means prob_cap=128, leaving 1920
// protected slots.
//
// Sentinel / bit constants:
//   NULL     = 0x7FFF_FFFF  (no valid index; bit 31 clear so PROT_BIT is free)
//   PROT_BIT = 0x8000_0000  (bit 31 of prev: node is in the protected queue)
//   IDX_MASK = 0x7FFF_FFFF  (strip PROT_BIT to get the raw index from prev)
//
// Policy sketch:
//   • New entries enter the probationary queue (FIFO, evicted oldest-first).
//   • A probationary hit graduates the entry to the protected queue (LRU).
//   • A protected hit refreshes the entry to the MRU end of protected.
//   • When protected exceeds its quota (capacity - prob_cap), the LRU of
//     protected is demoted to the MRU end of probationary.
//   • Eviction always drains probationary first; protected is a fallback.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

/// Null sentinel for u32 link fields.
/// Bit 31 is clear so that `PROT_BIT` (bit 31 of `prev`) is distinguishable
/// from any valid index embedded in `prev`.
const NULL: u32 = 0x7FFF_FFFF;

/// Bit 31 of the `prev` field: set when the node lives in the protected queue.
const PROT_BIT: u32 = 0x8000_0000;

/// Mask to extract the raw index from a `prev` field (strips `PROT_BIT`).
const IDX_MASK: u32 = 0x7FFF_FFFF;

/// Per-slot doubly-linked-list pointers, stored separately from key/value data.
/// 8 bytes each; 8 entries fit in a 64-byte cache line.
#[derive(Debug)]
struct Link {
    /// bit 31 = PROT_BIT; bits 0-30 = prev index (or NULL = 0x7FFF_FFFF)
    prev: u32,
    next: u32,
}

#[derive(Debug)]
pub struct Node<K, V> {
    key: K,
    value: V,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
    map: HashMap<K, usize>,  // key -> node index
    nodes: Vec<Node<K, V>>,  // arena
    links: Vec<Link>,        // parallel to nodes; links[i] holds prev/next for nodes[i]
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
    /// Probationary capacity = (capacity / 16).max(1).
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
        // Valid indices run 0 ..= 0x7FFF_FFFE; NULL = 0x7FFF_FFFF is the sentinel.
        if capacity > 0x7FFF_FFFEusize {
            log::error!("LruCache.try_new: capacity exceeds u32 sentinel range");
            return None;
        }
        log::info!("LruCache.try_new: capacity={capacity}");
        let prob_cap = (capacity / 16).max(1);
        Some(Self {
            map: HashMap::with_capacity(capacity),
            nodes: Vec::with_capacity(capacity),
            links: Vec::with_capacity(capacity),
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
            });
            self.links.push(Link { prev: NULL, next: NULL });
            new_idx
        };
        // All new insertions enter the probationary queue at its MRU end.
        self.push_back_prob(idx);
        self.prob_len += 1;
        self.map.insert(key, idx);
        debug_assert!(self.map.len() <= self.capacity);
        debug_assert_eq!(self.prob_len + self.prot_len, self.map.len());
    }

    /// Promote a node: refresh in protected (if already protected), or graduate
    /// from probationary to protected. Enforces the protected-queue budget.
    fn promote(&mut self, idx: usize) {
        let raw_prev = self.links[idx].prev;
        let protected = (raw_prev & PROT_BIT) != 0;
        if protected {
            // Protected hit: refresh to the MRU end of the protected queue.
            self.unlink(idx);
            self.push_back_prot(idx);
            // prot_len is unchanged (item stays in protected).
        } else {
            // Probationary hit: graduate to the protected queue.
            self.unlink(idx);
            self.prob_len -= 1;
            self.push_back_prot(idx);
            self.prot_len += 1;
            // Enforce the protected budget: if protected exceeds its quota,
            // demote the LRU of protected back to the MRU of probationary.
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
        let raw_prev = self.links[idx].prev;
        let protected = (raw_prev & PROT_BIT) != 0;
        let prev_idx = raw_prev & IDX_MASK; // raw index (or NULL sentinel)
        let next = self.links[idx].next;

        // Fix the forward link of the predecessor (or update the queue head).
        if prev_idx != NULL {
            self.links[prev_idx as usize].next = next;
        } else if protected {
            self.prot_head = next;
        } else {
            self.prob_head = next;
        }

        // Fix the backward link of the successor (or update the queue tail).
        if next != NULL {
            // Preserve the successor's own PROT_BIT; replace only its index part.
            let succ_prot_bit = self.links[next as usize].prev & PROT_BIT;
            self.links[next as usize].prev = succ_prot_bit | prev_idx;
        } else if protected {
            self.prot_tail = prev_idx;
        } else {
            self.prob_tail = prev_idx;
        }

        // Clear the unlinked node's link fields (NULL has PROT_BIT = 0).
        self.links[idx].prev = NULL;
        self.links[idx].next = NULL;
    }

    /// Append a node to the MRU end of the probationary queue.
    /// Clears PROT_BIT in `prev` to mark the node as probationary.
    fn push_back_prob(&mut self, idx: usize) {
        let idx32 = idx as u32;
        // prob_tail is a raw index (no PROT_BIT), so assigning it directly
        // to prev leaves PROT_BIT = 0 (probationary).
        self.links[idx].prev = self.prob_tail;
        self.links[idx].next = NULL;
        if self.prob_tail != NULL {
            self.links[self.prob_tail as usize].next = idx32;
        } else {
            self.prob_head = idx32;
        }
        self.prob_tail = idx32;
    }

    /// Append a node to the MRU end of the protected queue.
    /// Sets PROT_BIT in `prev` to mark the node as protected.
    fn push_back_prot(&mut self, idx: usize) {
        let idx32 = idx as u32;
        // OR-in PROT_BIT so the node is marked as belonging to the protected queue.
        self.links[idx].prev = PROT_BIT | self.prot_tail;
        self.links[idx].next = NULL;
        if self.prot_tail != NULL {
            self.links[self.prot_tail as usize].next = idx32;
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
    fn eviction_removes_lru() {
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
    fn access_updates_eviction_order() {
        // get(1) graduates key 1 to protected; put(3) evicts key 2 from prob.
        let mut cache = LruCache::new(2);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.get(&1); // promotes key 1 from prob → protected
        cache.put(3, 30); // evicts key 2 (only item left in prob)

        assert_eq!(*cache.get(&1).unwrap(), 10); // key 1 still lives in protected
        assert!(cache.get(&2).is_none());         // key 2 was evicted
        assert_eq!(*cache.get(&3).unwrap(), 30);
    }

    #[test]
    fn put_updates_eviction_order() {
        // put(1,11) on an existing key promotes key 1 to protected;
        // put(3) then evicts key 2 from prob.
        let mut cache = LruCache::new(2);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(1, 11); // update key 1 → promoted to protected
        cache.put(3, 30); // evicts key 2 (prob head)

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
        // Repeated hits on key 1 graduate it to protected, shielding it from eviction.
        let mut cache = LruCache::new(3);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30);
        for _ in 0..10 {
            cache.get(&1);
        }
        // Eviction drains prob (keys 2 and 3 are there); key 1 is protected.
        cache.put(4, 40); // should evict key 2 (prob LRU), not key 1
        assert_eq!(*cache.get(&1).unwrap(), 10);
        assert!(cache.get(&2).is_none());
    }

    #[test]
    fn get_middle_node_promotes_correctly() {
        let mut cache = LruCache::new(3);
        cache.put(1, 10); // prob LRU
        cache.put(2, 20); // prob middle
        cache.put(3, 30); // prob MRU
        cache.get(&2);    // graduate key 2 to protected
        cache.put(4, 40); // evicts key 1 (prob LRU)
        assert!(cache.get(&1).is_none());
        assert_eq!(*cache.get(&3).unwrap(), 30);
        assert_eq!(*cache.get(&2).unwrap(), 20);
    }

    #[test]
    fn slot_reuse_with_2q_protection() {
        // With 2Q (capacity=2, prob_cap=1, prot_cap=1):
        //   key 3 is promoted to protected by get(&3);
        //   subsequent puts evict from prob (key 2, then key 4), not from protected.
        let mut cache = LruCache::new(2);
        cache.put(1, 10); // slot 0
        cache.put(2, 20); // slot 1
        cache.put(3, 30); // evicts 1 → slot 0 reused for key 3
        cache.get(&3);    // promote slot 0 (key 3) to protected
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
        // Single node: get promotes it to protected (no budget overflow since prot_cap=2).
        assert_eq!(*cache.get(&1).unwrap(), 10);
        // Verify list is still intact after promotion.
        cache.put(2, 20);
        cache.put(3, 30);
        assert_eq!(*cache.get(&1).unwrap(), 10);
    }

    #[test]
    fn overwrite_mru_at_capacity() {
        let mut cache = LruCache::new(3);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30); // full; 3 is MRU (tail of prob)
        cache.put(3, 31); // overwrite MRU → promotes key 3 to protected
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
}
