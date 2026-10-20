// O(1) two-queue (2Q) cache using HashMap + arena-based doubly-linked list.
// Nodes are stored in a Vec (arena) indexed by usize. On eviction the slot is reused in-place.
//
// H02 (cycle/0007): prev/next link fields are moved out of Node<K,V> into a dedicated
// Vec<Link>, shrinking Node<u64,u64> from 24 to 16 bytes.
//
// H01 (cycle/0006): probationary capacity is reduced from capacity/4 to capacity/8,
// raising the protected quota from 75% to 87.5%.
//
// H02 (cycle/0007): A persistent 64-byte CountMin frequency sketch gates graduation
// from probationary to protected at freq >= 2.
//
// H03 (cycle/0010): Deferred demotion via DEMOTION_FLAG.
//   - Probationary quota tightened to capacity/10 (90% protected, 10% probationary).
//   - Bit 31 of `links[i].next` (DEMOTION_FLAG) marks the protected-queue LRU head as
//     pending demotion. The demotion is deferred from the `get()` hot path to `put()`.
//   - `get()` lazy path: when a probationary graduation would overflow protected and no
//     pending demotion exists, set DEMOTION_FLAG on prot_head's next field instead of
//     immediately demoting it. `prot_len` rises to prot_cap + 1 transiently.
//   - `get()` eager fallback: when the flag is already set (at most one pending demotion
//     at a time), execute the deferred demotion synchronously before promoting the new item.
//   - `put()` finalization: before selecting a victim, if prot_head's next has
//     DEMOTION_FLAG, clear the flag, unlink prot_head, and push it to probationary's
//     MRU tail. This moves the linked-list pointer work from `get()` to `put()`.
//
// Sentinel / bit constants:
//   NULL          = 0x7FFF_FFFF  (no valid index; bit 31 always 0 in raw arena indices)
//   PROT_BIT      = 0x8000_0000  (bit 31 of prev: node is in the protected queue)
//   IDX_MASK      = 0x7FFF_FFFF  (strip PROT_BIT to get the raw index from prev)
//   DEMOTION_FLAG = 0x8000_0000  (bit 31 of next: deferred demotion pending on this node)
//
// Policy sketch:
//   • New entries enter the probationary queue (FIFO, evicted oldest-first).
//   • A probationary hit increments the CountMin sketch; if freq >= 2, the entry
//     graduates to the protected queue (LRU). Otherwise it stays in probationary.
//   • A protected hit refreshes the entry to the MRU end of protected.
//   • When protected exceeds its quota (capacity - prob_cap), the LRU of
//     protected is demoted to the MRU end of probationary — lazily (deferred
//     to the next put()) if no existing demotion is pending, eagerly otherwise.
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
/// Also used to strip `DEMOTION_FLAG` from `next` fields.
const IDX_MASK: u32 = 0x7FFF_FFFF;

/// Bit 31 of the `next` field: when set on the protected-queue LRU head, a
/// demotion of that node to probationary is deferred to the next `put()`.
/// Capacity values are at most a few thousand, so bit 31 is always 0 in the
/// original code for any real arena index; stealing it is safe.
const DEMOTION_FLAG: u32 = 1 << 31;

/// Return the arena index stored in a `next` field, stripping any DEMOTION_FLAG.
#[inline(always)]
fn next_idx(next: u32) -> u32 {
    next & IDX_MASK
}

/// Return true when the DEMOTION_FLAG is set on a `next` field.
#[inline(always)]
fn is_pending_demotion(next: u32) -> bool {
    next & DEMOTION_FLAG != 0
}

/// Per-slot doubly-linked-list pointers, stored separately from key/value data.
/// 8 bytes each; 8 entries fit in a 64-byte cache line.
#[derive(Debug)]
struct Link {
    /// bit 31 = PROT_BIT; bits 0-30 = prev index (or NULL = 0x7FFF_FFFF)
    prev: u32,
    /// bit 31 = DEMOTION_FLAG (only valid on protected-queue LRU head);
    /// bits 0-30 = next index (or NULL = 0x7FFF_FFFF)
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
    /// Probationary capacity = (capacity / 10).max(1).
    /// Protected capacity = capacity - prob_cap (≈ 90% of total).
    prob_cap: usize,
    capacity: usize,
    /// 4-row × 16-col CountMin frequency sketch; zero-initialized.
    /// Stored as a 64-byte array (one cache line) for compact, persistent
    /// frequency estimation across cache fills and evictions.
    freq_sketch: [u8; 64],
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
        let prob_cap = (capacity / 10).max(1);
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
            freq_sketch: [0u8; 64],
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
    /// Finalizes any pending deferred demotion before selecting an eviction victim.
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

    /// Increment the CountMin frequency sketch for `key` and return the
    /// post-increment minimum across all four rows (the frequency estimate).
    ///
    /// The sketch is 4 rows × 16 columns = 64 bytes (one cache line).
    /// Four independent column indices are derived from a single hash by
    /// multiplying with four different mixing constants and masking to 4 bits.
    fn sketch_inc(&mut self, key: &K) -> u8 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        key.hash(&mut h);
        let v = h.finish();
        // Derive four independent column indices (0..16) from one hash
        // by multiplying with four different primes and taking low 4 bits.
        let cols: [usize; 4] = [
            (v & 0xF) as usize,
            ((v.wrapping_mul(0xbf58476d1ce4e5b9)) & 0xF) as usize,
            ((v.wrapping_mul(0x94d049bb133111eb)) & 0xF) as usize,
            ((v.wrapping_mul(0x9e3779b97f4a7c15)) & 0xF) as usize,
        ];
        let mut min_val = u8::MAX;
        for (row, &col) in cols.iter().enumerate() {
            let slot = &mut self.freq_sketch[row * 16 + col];
            *slot = slot.saturating_add(1);
            if *slot < min_val {
                min_val = *slot;
            }
        }
        min_val
    }

    /// Promote a node: refresh in protected (if already protected), or attempt
    /// to graduate from probationary to protected using the frequency sketch.
    ///
    /// H03 lazy-demotion: when a graduation would overflow the protected quota
    /// and no deferred demotion is pending, set DEMOTION_FLAG on the protected
    /// LRU head's `next` field and proceed with the promotion (deferred to put()).
    /// If a deferred demotion is already pending (flag set), execute it eagerly
    /// instead, ensuring at most one item carries the flag at any time.
    fn promote(&mut self, idx: usize) {
        let raw_prev = self.links[idx].prev;
        let protected = (raw_prev & PROT_BIT) != 0;
        if protected {
            // Protected hit: refresh to the MRU end of the protected queue.
            // unlink() strips DEMOTION_FLAG from `next`, so if prot_head had the
            // flag and is the item being refreshed, the deferred demotion is
            // implicitly cancelled (the item is hot again — no demotion needed).
            self.unlink(idx);
            self.push_back_prot(idx);
            // prot_len is unchanged (item stays in protected).
        } else {
            // Probationary hit: consult the frequency sketch.
            // Clone key to satisfy the borrow checker.
            let freq = {
                let k = self.nodes[idx].key.clone();
                self.sketch_inc(&k)
            };
            if freq < 2 {
                // Below threshold: item has only been accessed once globally.
                return;
            }
            // freq >= 2: graduate to protected.
            let prot_cap = self.capacity - self.prob_cap;

            // H03: before promoting, check whether the protected queue is at or
            // above its quota (prot_len >= prot_cap) and, if so, apply either
            // lazy or eager demotion of the protected LRU head.
            if self.prot_len >= prot_cap && self.prot_head != NULL {
                let head_idx = (self.prot_head & IDX_MASK) as usize;
                if is_pending_demotion(self.links[head_idx].next) {
                    // A deferred demotion is already pending (flag set on prot_head).
                    // Eager fallback: execute the deferred demotion now on the get path.
                    // Clear the flag before calling unlink() so it reads a clean index.
                    self.links[head_idx].next &= !DEMOTION_FLAG;
                    self.unlink(head_idx);
                    self.prot_len -= 1;
                    self.push_back_prob(head_idx);
                    self.prob_len += 1;
                    // After eager demotion, prot_len = prot_cap (or prot_cap - 1 if it
                    // was prot_cap + 1 before). We then promote the new item below,
                    // bringing prot_len back up to prot_cap + 1 at most.
                } else {
                    // No pending demotion: lazy path. Set DEMOTION_FLAG on prot_head's
                    // `next` field. The actual pointer manipulation is deferred to the
                    // next put() via evict_one(). prot_len will transiently become
                    // prot_cap + 1 after the promotion below.
                    self.links[head_idx].next |= DEMOTION_FLAG;
                }
            }

            // Graduate the probationary item to the protected queue.
            self.unlink(idx);
            self.prob_len -= 1;
            self.push_back_prot(idx);
            self.prot_len += 1;
        }
    }

    /// Evict one entry via the 2Q policy (probationary first, protected as
    /// fallback), remove it from the map, and return the freed slot index.
    ///
    /// H03 finalization: before selecting a victim, if the protected queue's
    /// LRU head has DEMOTION_FLAG set on its `next` field, finalize the
    /// deferred demotion — move that node to the MRU end of probationary —
    /// so it may be the next probationary eviction candidate.
    fn evict_one(&mut self) -> usize {
        // Finalize any deferred demotion from the protected-LRU head.
        if self.prot_len > 0 {
            let head_idx = (self.prot_head & IDX_MASK) as usize;
            if is_pending_demotion(self.links[head_idx].next) {
                // Clear flag before unlink() so it reads a clean successor index.
                self.links[head_idx].next &= !DEMOTION_FLAG;
                self.unlink(head_idx);
                self.prot_len -= 1;
                self.push_back_prob(head_idx);
                self.prob_len += 1;
            }
        }

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
    ///
    /// DEMOTION_FLAG is stripped from `next` before using it as an index or
    /// storing it in head/tail pointers, so the flag is implicitly cancelled
    /// when any flagged node is unlinked (e.g., on a protected hit that refreshes
    /// the prot_head node, or during finalization after the flag is pre-cleared).
    fn unlink(&mut self, idx: usize) {
        let raw_prev = self.links[idx].prev;
        let protected = (raw_prev & PROT_BIT) != 0;
        let prev_idx = raw_prev & IDX_MASK; // raw index (or NULL sentinel)
        // Strip DEMOTION_FLAG before using `next` as an arena index.
        let next = next_idx(self.links[idx].next);

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

        // Clear the unlinked node's link fields (NULL has PROT_BIT = DEMOTION_FLAG = 0).
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
    fn capacity_bound_holds_with_sketch() {
        // Verify that with the frequency sketch, the cache never exceeds capacity.
        // With sketch, first probationary hit does NOT graduate (freq=1 < 2).
        // Second hit graduates to protected (freq=2).
        let capacity = 4;
        let mut cache = LruCache::<i64, i64>::new(capacity);
        for key in 0i64..20 {
            if cache.get(&key).is_none() {
                cache.put(key, key * 10);
            }
        }
        let resident = (0i64..20).filter(|k| cache.get(k).is_some()).count();
        assert!(resident <= capacity, "capacity exceeded: {resident} > {capacity}");
    }

    #[test]
    fn sketch_gates_graduation() {
        // With the frequency sketch, a probationary item requires 2 hits to graduate.
        // First get on a probationary item: freq=1, stays in prob.
        // Second get: freq=2, graduates to protected.
        // With prob_cap = (4/10).max(1) = 1, prot_cap = 3.
        let mut cache = LruCache::<i64, i64>::new(4);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30);
        // Access key 1 twice to ensure it graduates to protected.
        cache.get(&1); // freq=1, stays in prob
        cache.get(&1); // freq=2, graduates to protected (prot_len=1 < prot_cap=3)
        // Put 3 more items — key 1 should be shielded by protected.
        cache.put(4, 40); // arena full (4 items); no eviction needed yet; 4 goes to prob
        // Arena is full now: put(5) must evict.
        cache.put(5, 50); // evicts prob LRU (key 2)
        cache.put(6, 60); // evicts prob LRU (key 3)
        // Key 1 should still be in protected
        assert_eq!(*cache.get(&1).unwrap(), 10);
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
    fn get_only_element() {
        let mut cache = LruCache::new(3);
        cache.put(1, 10);
        // Single node: first get keeps in prob (freq=1), second get graduates to protected.
        assert_eq!(*cache.get(&1).unwrap(), 10);
        // Verify list is still intact after first hit (stays in prob).
        cache.put(2, 20);
        cache.put(3, 30);
        assert_eq!(*cache.get(&1).unwrap(), 10);
    }

    #[test]
    #[should_panic(expected = "initialization failed")]
    fn capacity_zero_panics() {
        LruCache::<i64, i64>::new(0);
    }

    #[test]
    fn lazy_demotion_deferred_to_put() {
        // Verify that deferred demotion is correctly finalised on the put() path.
        // capacity=10, prob_cap=1, prot_cap=9.
        // Graduate 9 items to fill prot to quota, then graduate one more (triggers
        // lazy demotion with DEMOTION_FLAG), then put a new item (triggers finalisation).
        let capacity = 10usize;
        let mut cache = LruCache::<i64, i64>::new(capacity);
        // Fill the arena: keys 0..10 → all in probationary.
        for k in 0i64..capacity as i64 {
            cache.put(k, k * 10);
        }
        // Graduate keys 0..9 to protected (2 hits each via sketch).
        // prot_cap = 10 - 1 = 9.
        for k in 0i64..9 {
            cache.get(&k); // freq=1, stays in prob
            cache.get(&k); // freq=2, graduates
        }
        // Now prot_len = 9 = prot_cap. Key 9 is still in prob.
        // Graduate key 9: prot_len(9) >= prot_cap(9) → lazy demotion (set flag on prot_head).
        // prot_len becomes 10 transiently.
        cache.get(&9); // freq=1, stays in prob (freq < 2)
        cache.get(&9); // freq=2, graduates → lazy demotion triggered
        // Now put a new key: this triggers finalization (flag cleared, prot LRU demoted to prob).
        // Then eviction takes the newly-demoted item from prob.
        cache.put(100, 1000);
        // The cache should still have at most `capacity` items and all values correct.
        let resident = (0i64..10)
            .chain(std::iter::once(100i64))
            .filter(|k| cache.get(k).is_some())
            .count();
        assert!(resident <= capacity, "resident={resident} exceeds capacity={capacity}");
    }
}
