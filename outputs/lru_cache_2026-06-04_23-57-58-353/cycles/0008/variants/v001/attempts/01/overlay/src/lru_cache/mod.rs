// SLRU cache with IRR-based victim selection.
//
// Architecture:
// - Protected segment (80% of capacity): promoted entries with demonstrated reuse.
// - Probationary segment (20% of capacity): new entries and demoted entries.
// - New entries land in probationary; probationary hits promote to protected.
// - Victim selection uses inter-reference recency (IRR) rather than frequency.
//
// IRR(node) = last_access - prev_access   if prev_access > 0
//           = u64::MAX                    if fewer than 2 accesses recorded
//
// Lower IRR = hotter (two recent accesses were close together).
// Higher IRR (or MAX) = colder — eviction/demotion target.
//
// Scan windows:
//   Probationary eviction: LRU quarter — scan_len = (prob_count / 4).max(1)
//   Protected demotion:    LRU quarter — scan_len = (prot_count / 4).max(1)
//
// Timestamps are preserved across segment transitions so a hot entry retains
// its small IRR after demotion, letting it naturally compete against cold
// newcomers (IRR = MAX) without any counter adjustment.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Segment {
    Protected,
    Probationary,
}

#[derive(Debug)]
struct Node<K, V> {
    key: K,
    value: V,
    prev: Option<usize>,
    next: Option<usize>,
    segment: Segment,
    /// op_clock value at the most recent access (get hit or put for new key).
    last_access: u64,
    /// op_clock value at the second most recent access; 0 if fewer than 2 accesses.
    prev_access: u64,
}

/// SLRU cache with IRR-based victim selection.
///
/// Replaces accumulated frequency counters with inter-reference recency (IRR)
/// timestamps so that entries accessed twice in quick succession (low IRR,
/// "hot") are protected while entries with a large gap between last two
/// accesses (high IRR, "cold") are preferentially evicted.
#[derive(Debug)]
pub struct LruCache<K, V> {
    map: HashMap<K, usize>, // key -> arena index
    nodes: Vec<Node<K, V>>, // arena: indices are stable; slots are reused after eviction
    free_slots: Vec<usize>, // indices of evicted (reusable) slots

    /// Protected segment (80% of capacity).  Head = LRU end, tail = MRU end.
    prot_head: Option<usize>,
    prot_tail: Option<usize>,
    prot_count: usize,
    prot_cap: usize, // = capacity * 4 / 5

    /// Probationary segment (remaining 20%).  Head = LRU end, tail = MRU end.
    prob_head: Option<usize>,
    prob_tail: Option<usize>,
    prob_count: usize,

    capacity: usize,
    /// Monotonic counter: incremented on every get hit and every put for a new key.
    op_clock: u64,
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
        let prot_cap = capacity * 4 / 5;
        log::info!("LruCache.try_new: capacity={capacity} prot_cap={prot_cap}");
        Some(Self {
            map: HashMap::with_capacity(capacity),
            nodes: Vec::with_capacity(capacity),
            free_slots: Vec::new(),
            prot_head: None,
            prot_tail: None,
            prot_count: 0,
            prot_cap,
            prob_head: None,
            prob_tail: None,
            prob_count: 0,
            capacity,
            op_clock: 1,
        })
    }

    pub fn new(capacity: usize) -> Self {
        Self::try_new(capacity).expect("LruCache::new: initialization failed")
    }

    /// Inter-reference recency for the node at `idx`.
    /// Returns `u64::MAX` if fewer than two accesses have been recorded (cold newcomer).
    #[inline]
    fn irr(&self, idx: usize) -> u64 {
        if self.nodes[idx].prev_access == 0 {
            u64::MAX
        } else {
            self.nodes[idx].last_access - self.nodes[idx].prev_access
        }
    }

    /// Returns the value for `key` if present.
    ///
    /// On a hit:
    /// - Updates IRR timestamps (`prev_access`, `last_access`, `op_clock`).
    /// - If probationary: promotes to protected MRU tail; demotes coldest
    ///   protected entry if the segment overflows.
    /// - If protected: re-orders to protected MRU tail.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        let &idx = self.map.get(key)?;

        // Update IRR timestamps.
        self.nodes[idx].prev_access = self.nodes[idx].last_access;
        self.nodes[idx].last_access = self.op_clock;
        self.op_clock += 1;

        match self.nodes[idx].segment {
            Segment::Protected => {
                // Re-order within protected: move to MRU tail.
                self.unlink_protected(idx);
                self.push_prot_tail(idx);
            }
            Segment::Probationary => {
                // Promote to protected MRU tail.
                self.unlink_probationary(idx);
                self.prob_count -= 1;
                self.nodes[idx].segment = Segment::Protected;
                self.push_prot_tail(idx);
                self.prot_count += 1;
                // Demote if protected overflows its cap.
                if self.prot_count > self.prot_cap {
                    self.demote_prot_head();
                }
            }
        }

        Some(&self.nodes[idx].value)
    }

    /// Inserts `key` with `value`.
    ///
    /// Per the `Cache` contract, `put` is called only for non-resident keys.
    /// If at capacity, the coldest entry in probationary is evicted first.
    /// The new entry is placed at the probationary MRU tail.
    ///
    /// If `key` is already resident (contract violation), the value is updated
    /// and the node is moved to the MRU of its current segment.
    pub fn put(&mut self, key: K, value: V) {
        // Defensive: handle the (contract-violating) case of a resident key.
        if let Some(&idx) = self.map.get(&key) {
            self.nodes[idx].value = value;
            self.nodes[idx].prev_access = self.nodes[idx].last_access;
            self.nodes[idx].last_access = self.op_clock;
            self.op_clock += 1;
            match self.nodes[idx].segment {
                Segment::Protected => {
                    self.unlink_protected(idx);
                    self.push_prot_tail(idx);
                }
                Segment::Probationary => {
                    self.unlink_probationary(idx);
                    self.push_prob_tail(idx);
                }
            }
            return;
        }

        // Make room if at capacity.
        if self.prot_count + self.prob_count >= self.capacity {
            self.evict_from_probationary();
        }

        // Allocate an arena slot (reuse a freed slot if available).
        let idx = if let Some(slot) = self.free_slots.pop() {
            self.nodes[slot].key = key.clone();
            self.nodes[slot].value = value;
            self.nodes[slot].prev = None;
            self.nodes[slot].next = None;
            self.nodes[slot].segment = Segment::Probationary;
            self.nodes[slot].last_access = self.op_clock;
            self.nodes[slot].prev_access = 0;
            slot
        } else {
            let slot = self.nodes.len();
            self.nodes.push(Node {
                key: key.clone(),
                value,
                prev: None,
                next: None,
                segment: Segment::Probationary,
                last_access: self.op_clock,
                prev_access: 0,
            });
            slot
        };
        self.op_clock += 1;

        // Place in probationary at MRU tail.
        self.push_prob_tail(idx);
        self.prob_count += 1;
        self.map.insert(key, idx);
    }

    /// Scan the LRU quarter of probationary and evict the coldest node (max IRR).
    ///
    /// `scan_len = (prob_count / 4).max(1)` nodes are examined starting from
    /// `prob_head`. The one with the highest IRR is removed.
    fn evict_from_probationary(&mut self) {
        debug_assert!(
            self.prob_count > 0,
            "evict_from_probationary called with empty probationary segment"
        );
        let scan_len = (self.prob_count / 4).max(1);

        let start = self.prob_head.expect("prob_head is Some when prob_count > 0");
        let mut victim_idx = start;
        let mut victim_irr = self.irr(start);
        let mut cursor = self.nodes[start].next;

        for _ in 1..scan_len {
            match cursor {
                None => break,
                Some(cur) => {
                    let cur_irr = self.irr(cur);
                    if cur_irr > victim_irr {
                        victim_idx = cur;
                        victim_irr = cur_irr;
                    }
                    cursor = self.nodes[cur].next;
                }
            }
        }

        let evicted_key = self.nodes[victim_idx].key.clone();
        self.unlink_probationary(victim_idx);
        self.prob_count -= 1;
        self.map.remove(&evicted_key);
        self.free_slots.push(victim_idx);
    }

    /// Scan the LRU quarter of protected and demote the coldest node (max IRR)
    /// to the probationary MRU tail.
    ///
    /// `scan_len = (prot_count / 4).max(1)` nodes are examined starting from
    /// `prot_head`. Timestamps are preserved — a previously hot entry retains
    /// its small IRR in probationary, competing naturally against cold newcomers.
    fn demote_prot_head(&mut self) {
        debug_assert!(
            self.prot_count > 0,
            "demote_prot_head called with empty protected segment"
        );
        let scan_len = (self.prot_count / 4).max(1);

        let start = self.prot_head.expect("prot_head is Some when prot_count > 0");
        let mut victim_idx = start;
        let mut victim_irr = self.irr(start);
        let mut cursor = self.nodes[start].next;

        for _ in 1..scan_len {
            match cursor {
                None => break,
                Some(cur) => {
                    let cur_irr = self.irr(cur);
                    if cur_irr > victim_irr {
                        victim_idx = cur;
                        victim_irr = cur_irr;
                    }
                    cursor = self.nodes[cur].next;
                }
            }
        }

        // Demote: move from protected to probationary MRU tail.
        // Timestamps are deliberately NOT reset.
        self.unlink_protected(victim_idx);
        self.prot_count -= 1;
        self.nodes[victim_idx].segment = Segment::Probationary;
        self.push_prob_tail(victim_idx);
        self.prob_count += 1;
    }

    // ── Per-segment linked-list helpers ──────────────────────────────────────

    fn unlink_protected(&mut self, idx: usize) {
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

    fn push_prot_tail(&mut self, idx: usize) {
        self.nodes[idx].prev = self.prot_tail;
        self.nodes[idx].next = None;
        match self.prot_tail {
            Some(t) => self.nodes[t].next = Some(idx),
            None => self.prot_head = Some(idx),
        }
        self.prot_tail = Some(idx);
    }

    fn unlink_probationary(&mut self, idx: usize) {
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

    fn push_prob_tail(&mut self, idx: usize) {
        self.nodes[idx].prev = self.prob_tail;
        self.nodes[idx].next = None;
        match self.prob_tail {
            Some(t) => self.nodes[t].next = Some(idx),
            None => self.prob_head = Some(idx),
        }
        self.prob_tail = Some(idx);
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
        // Probationary scan_len = (2/4).max(1) = 1: evicts from prob_head.
        let mut cache = LruCache::new(2);
        cache.put(1, 10); // prob: [1]
        cache.put(2, 20); // prob: [1, 2]
        cache.put(3, 30); // evicts 1 (prob_head, irr=MAX)

        assert!(cache.get(&1).is_none());
        assert_eq!(*cache.get(&2).unwrap(), 20);
        assert_eq!(*cache.get(&3).unwrap(), 30);
    }

    #[test]
    fn access_updates_order() {
        // get(1) promotes 1 from probationary to protected, so put(3) evicts 2.
        let mut cache = LruCache::new(2);
        cache.put(1, 10); // prob: [1]
        cache.put(2, 20); // prob: [1, 2]
        cache.get(&1);    // promote 1 to protected; prob: [2], prot: [1]
        cache.put(3, 30); // evicts 2 (only prob entry)

        assert_eq!(*cache.get(&1).unwrap(), 10);
        assert!(cache.get(&2).is_none());
        assert_eq!(*cache.get(&3).unwrap(), 30);
    }

    #[test]
    fn put_updates_order() {
        // Defensive put on existing key moves it to probationary MRU tail.
        let mut cache = LruCache::new(2);
        cache.put(1, 10); // prob: [1]
        cache.put(2, 20); // prob: [1, 2]
        cache.put(1, 11); // update key 1 -> moved to prob tail: [2, 1]
        cache.put(3, 30); // evicts 2 (prob_head)

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
        cache.put(4, 40); // evicts 1 (prob_head)
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
    fn repeated_get_protects_hot_entry() {
        // key 1 is repeatedly accessed -> promoted to protected with small IRR.
        // When put(4) needs to evict, it takes from probationary, not from protected.
        let mut cache = LruCache::new(3);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30);
        for _ in 0..10 {
            cache.get(&1); // promote and warm key 1 in protected
        }
        cache.put(4, 40); // evicts from probationary (key 2 or 3), not key 1
        assert_eq!(*cache.get(&1).unwrap(), 10);
    }

    #[test]
    fn get_middle_node_promotes_correctly() {
        // cap=3, prot_cap=2.  get(2) promotes 2 to protected.
        // put(4) evicts from probationary: [1, 3] -> prob_head=1 (irr=MAX), evicted.
        let mut cache = LruCache::new(3);
        cache.put(1, 10); // prob: [1]
        cache.put(2, 20); // prob: [1, 2]
        cache.put(3, 30); // prob: [1, 2, 3]
        cache.get(&2);    // promote 2 to prot; prob: [1, 3]
        cache.put(4, 40); // evicts 1 (prob_head)
        assert!(cache.get(&1).is_none());
        assert_eq!(*cache.get(&3).unwrap(), 30);
        assert_eq!(*cache.get(&2).unwrap(), 20);
    }

    #[test]
    fn slot_reuse_and_protection() {
        // cap=2, prot_cap=1.
        // After get(3), key 3 is in protected (small IRR).
        // Subsequent puts evict from probationary, leaving the protected entry intact.
        let mut cache = LruCache::new(2);
        cache.put(1, 10); // slot 0 -> prob
        cache.put(2, 20); // slot 1 -> prob
        cache.put(3, 30); // evicts key 1 (prob_head) -> slot 0 reused for key 3
        cache.get(&3);    // promote key 3 to protected (small IRR)
        cache.put(4, 40); // evicts key 2 (only prob entry) -> slot reused for key 4
        cache.put(5, 50); // evicts key 4 (probationary, irr=MAX) -> key 3 (protected) survives
        assert!(cache.get(&3).is_some()); // protected: small IRR, not evicted
        assert!(cache.get(&4).is_none()); // probationary: cold, evicted
        assert_eq!(*cache.get(&5).unwrap(), 50);
    }

    #[test]
    fn get_only_element() {
        let mut cache = LruCache::new(3);
        cache.put(1, 10);
        assert_eq!(*cache.get(&1).unwrap(), 10);
        // Verify list is still intact after single-node promotion
        cache.put(2, 20);
        cache.put(3, 30);
        assert_eq!(*cache.get(&1).unwrap(), 10);
    }

    #[test]
    fn overwrite_mru_at_capacity() {
        let mut cache = LruCache::new(3);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30); // full; 3 is prob MRU tail
        cache.put(3, 31); // overwrite MRU (defensive path, no eviction)
        cache.put(4, 40); // evicts 1 (prob_head)
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
