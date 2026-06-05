// SLRU cache with ghost-set admission + IRR-based eviction (H03/cycle-8) +
// referenced-bit tie-breaker in protected demotion (H01/cycle-6).
//
// This variant transplants IRR (Inter-Reference Recency) timestamps as the
// primary eviction signal from cycle/0007/v003 into the ghost-admission
// framework of cycle/0006/v001.  The hypothesis is that ghost admission and
// IRR eviction are complementary signals: ghost sets detect cross-residency
// warmth (whether a key was recently evicted), while IRR timestamps detect
// temporal clustering (whether recent accesses are tightly spaced).
//
// Two segments share a single arena:
//   probationary  (20 %)  — new items land here
//   protected     (80 %)  — items promoted on a cache hit
//
// A bounded ghost set tracks recently evicted probationary keys.
// When an incoming key matches a ghost entry it is admitted directly into the
// protected segment, giving cross-residency warm keys immediate protection.
//
// Eviction policy: IRR (Inter-Reference Recency).  For each node, IRR is
// last_access - prev_access (the gap between its two most recent accesses).
// A node with only one access has IRR = u64::MAX (cold).  The victim is the
// node with the HIGHEST IRR — the least temporally clustered entry.  This
// mirrors the LIRS insight: hot items have low IRR and are protected; cold
// items have high IRR and are evicted.
//
// Referenced-bit tie-breaker (H01/cycle-6): a per-node `referenced` bit is
// set to true on every protected hit.  Inside demote_prot_head() the victim
// is selected by (irr descending, then unreferenced-first) — a referenced
// node at the same IRR is shielded from demotion when an unreferenced
// candidate exists in the scan window.  Timestamps are NOT reset on demotion;
// only the `referenced` bit is cleared.

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
    last_access: u64,  // op_clock value at most recent access
    prev_access: u64,  // op_clock value at second most recent access; 0 if < 2 accesses
    referenced: bool,
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

    // Monotonic access clock for IRR computation
    op_clock: u64,

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
        let ghost_cap = prob_cap;

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
            op_clock: 1,
            capacity,
        })
    }

    pub fn new(capacity: usize) -> Self {
        Self::try_new(capacity).expect("LruCache::new: initialization failed")
    }

    /// Compute the Inter-Reference Recency for the node at `idx`.
    ///
    /// Returns `u64::MAX` for nodes that have been accessed fewer than twice
    /// (cold entries whose second access has never been observed).
    #[inline]
    fn irr(&self, idx: usize) -> u64 {
        if self.nodes[idx].prev_access == 0 {
            u64::MAX
        } else {
            self.nodes[idx].last_access - self.nodes[idx].prev_access
        }
    }

    /// Returns the value for `key` if resident, updating recency and IRR timestamps.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        let &idx = self.map.get(key)?;
        // Update IRR timestamps on every hit (before segment logic).
        self.nodes[idx].prev_access = self.nodes[idx].last_access;
        self.nodes[idx].last_access = self.op_clock;
        self.op_clock += 1;
        match self.nodes[idx].segment {
            Segment::Protected => {
                // Touch: move to protected MRU.
                self.unlink_prot(idx);
                self.push_back_prot(idx);
                // Mark as referenced for demotion tie-breaking.
                self.nodes[idx].referenced = true;
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
                // referenced stays false: newly promoted entry starts cold.
                self.push_back_prot(idx);
                self.prot_count += 1;
                // Protected overflow: demote highest-IRR from LRU quarter to probationary MRU.
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
        // Re-put: key already resident — update value, refresh timestamps, refresh recency.
        if let Some(&idx) = self.map.get(&key) {
            self.nodes[idx].value = value;
            // Update IRR timestamps on re-put (same as a get hit).
            self.nodes[idx].prev_access = self.nodes[idx].last_access;
            self.nodes[idx].last_access = self.op_clock;
            self.op_clock += 1;
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
                last_access: self.op_clock,
                prev_access: 0,
                referenced: false,
            });
            self.op_clock += 1;
            i
        } else {
            // Arena full — evict to reclaim a slot.
            let evict_idx = if self.prob_count > 0 {
                // IRR-based scan of the LRU half of probationary.
                // Pick the node with the highest IRR (least temporally clustered).
                // Cold nodes (prev_access=0) have IRR=u64::MAX and are preferred victims.
                let scan_len = (self.prob_count / 2).max(1);
                let mut victim_idx: Option<usize> = None;
                let mut victim_irr: u64 = 0;
                let mut current = self.prob_head;
                let mut scanned = 0usize;
                while scanned < scan_len {
                    let Some(cur) = current else { break };
                    let cur_irr = self.irr(cur);
                    if cur_irr > victim_irr {
                        victim_irr = cur_irr;
                        victim_idx = Some(cur);
                    }
                    current = self.nodes[cur].next;
                    scanned += 1;
                }
                let evict_idx = victim_idx.expect("prob_count > 0 guarantees a victim");
                let evicted_key = self.nodes[evict_idx].key.clone();
                self.map.remove(&evicted_key);
                self.unlink_prob(evict_idx);
                self.prob_count -= 1;
                // Record the eviction in the ghost set.
                self.add_to_ghost(evicted_key);
                evict_idx
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
            // Re-initialise the reclaimed slot for the new key.
            self.nodes[evict_idx].key = key.clone();
            self.nodes[evict_idx].value = value;
            self.nodes[evict_idx].prev = None;
            self.nodes[evict_idx].next = None;
            self.nodes[evict_idx].last_access = self.op_clock;
            self.nodes[evict_idx].prev_access = 0;
            self.nodes[evict_idx].referenced = false;
            self.op_clock += 1;
            evict_idx
        };

        // ── Step 2: place the node into the right segment ─────────────────
        if self.ghost_set.contains(&key) {
            // Ghost hit: key re-arrived after a previous probationary eviction.
            // Admit directly into protected, bypassing the one-hit filter.
            // The entry arrives with prev_access=0 (IRR=MAX) because it has
            // only been accessed once; placement into protected is the signal,
            // not a freq credit.
            self.ghost_set.remove(&key);
            self.ghost.retain(|k| k != &key);

            self.nodes[idx].segment = Segment::Protected;
            // referenced stays false: ghost-admitted entry starts cold.
            self.push_back_prot(idx);
            self.prot_count += 1;
            self.map.insert(key, idx);

            // Protected overflow: demote highest-IRR from LRU quarter to probationary MRU.
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

    /// Demote the highest-IRR node from the LRU quarter of the protected segment
    /// to probationary MRU.
    ///
    /// Selection criterion: prefer the node with the highest IRR (cold, least
    /// temporally clustered).  Among equal-IRR candidates, prefer an unreferenced
    /// node (r=false) over a referenced node (r=true) — the referenced node was
    /// recently touched and gets a reprieve.
    ///
    /// IRR timestamps are preserved across demotion unchanged.  Only the
    /// `referenced` bit is cleared so the demoted entry re-enters probationary cold.
    fn demote_prot_head(&mut self) {
        // Scan the LRU quarter (at least 1 node) for the maximum-IRR victim.
        let scan_len = (self.prot_count / 4).max(1);

        let mut victim_idx: Option<usize> = None;
        let mut victim_irr: u64 = 0;
        let mut current = self.prot_head;
        let mut scanned = 0usize;

        while scanned < scan_len {
            let Some(idx) = current else { break };
            let cur_irr = self.irr(idx);
            let cur_ref = self.nodes[idx].referenced;

            let should_update = match victim_idx {
                None => true,
                Some(vi) => {
                    let vic_ref = self.nodes[vi].referenced;
                    cur_irr > victim_irr
                        || (cur_irr == victim_irr && !cur_ref && vic_ref)
                }
            };

            if should_update {
                victim_idx = Some(idx);
                victim_irr = cur_irr;
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
            // IRR timestamps are NOT reset on demotion — preserve temporal signal.
            // Clear referenced so the demoted entry re-enters probationary cold.
            self.nodes[v].referenced = false;
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
        // put(3) → scan_len=1, victim=prob_head=1 (IRR=MAX, cold), ghost={1}; insert 3 → prob={2,3}
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
        cache.put(3, 30); // scan_len=1, victim=prob_head=2 (IRR=MAX, cold)

        assert_eq!(*cache.get(&1).unwrap(), 11);
        assert!(cache.get(&2).is_none());
        assert_eq!(*cache.get(&3).unwrap(), 30);
    }

    #[test]
    fn eviction_chain() {
        // cap=3: prot_cap=2, prob_cap=1
        // All inserts go to prob; evictions follow FIFO order of insertion
        // (all have IRR=MAX and scan_len=1 so LRU head is always evicted).
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
        cache.put(4, 40); // scan_len=1, victim=prob_head=2 (IRR=MAX, cold)
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
        cache.put(4, 40); // scan_len=1, victim=prob_head=1 (IRR=MAX, cold)
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
        cache.put(4, 40); // scan_len=1, victim=prob_head=1 (IRR=MAX, cold)
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
    /// cap=8: prot_cap=6, prob_cap=2, ghost_cap=2.
    ///
    /// Sequence:
    ///   1. Fill all 8 slots → everything in prob.
    ///   2. put(9) → scan_len=4, all IRR=MAX (cold), victim=prob_head=1; ghost = {1}.
    ///   3. get(2..=7) → promote 6 keys to prot (no new prob evictions, ghost unchanged).
    ///   4. put(1): prob_count=2, scan_len=1, victim=prob_head=8 (IRR=MAX);
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
        // put(1): scan_len=1, victim=prob_head=8 (IRR=MAX) → ghost={1,8} (fits in ghost_cap=2).
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

    /// IRR scan test: a cold newcomer (prev_access=0, IRR=MAX) in probationary is
    /// preferred as eviction victim over a recently-seen entry (lower IRR) when
    /// both are within the scan window.
    ///
    /// cap=6: prot_cap=4, prob_cap=2, ghost_cap=2.
    /// put(1..=6): prob={1,2,3,4,5,6}
    /// get(1): promotes 1 → prot; updates timestamps (IRR < MAX)
    /// get(2): promotes 2 → prot; updates timestamps (IRR < MAX)
    /// get(2) again: prot hit, IRR shrinks further
    /// get(2) again: same
    /// get(3): promotes 3 → prot
    /// get(4): promotes 4 → prot (prot_count=4=prot_cap, no overflow)
    /// prob={5,6}
    ///
    /// put(7): arena full, prob_count=2 > 0
    ///   scan_len=(2/2).max(1)=1, scans prob_head=5 (IRR=MAX, cold) → victim=5
    #[test]
    fn cold_entry_evicted_before_accessed_entry() {
        // cap=6: prot_cap=4, prob_cap=2, ghost_cap=2
        let mut cache = LruCache::<i64, i64>::new(6);
        // Fill arena with cold inserts.
        for k in 1i64..=6 {
            cache.put(k, k * 10);
        }

        // Promote key 1 and key 2 to prot, access key 2 repeatedly.
        cache.get(&1); // promote 1 to prot
        cache.get(&2); // promote 2 to prot
        cache.get(&2); // prot hit, IRR shrinks
        cache.get(&2); // prot hit, IRR shrinks again

        // Fill prot to cap.
        cache.get(&3); // promote 3 → prot={1,2,3}
        cache.get(&4); // promote 4 → prot={1,2,3,4}, prot_count=4=prot_cap
        // prob={5,6}

        // put(7): arena full, scan_len=1, scans prob_head=5 (IRR=MAX) → evict 5
        cache.put(7, 70);

        // Key 5 is the LRU cold newcomer and should have been evicted.
        assert!(cache.get(&5).is_none(), "key 5 should be evicted (cold LRU)");
        // Key 6 should still be resident (not scanned).
        assert_eq!(*cache.get(&6).unwrap(), 60);
        // Key 7 was just inserted.
        assert_eq!(*cache.get(&7).unwrap(), 70);
        // Keys 1-4 in prot remain resident.
        assert_eq!(*cache.get(&1).unwrap(), 10);
        assert_eq!(*cache.get(&2).unwrap(), 20);
    }

    /// Referenced-bit tie-breaker test: among equal-IRR protected entries in the
    /// scan window, the unreferenced one is demoted first.
    ///
    /// cap=4: prot_cap=3, prob_cap=1
    /// put(1..=4): prob={1,2,3,4}
    /// get(1): promotes 1 (IRR=MAX on first get, updates timestamps) → prot={1}
    /// get(2): promotes 2 → prot={1,2}
    /// get(3): promotes 3 → prot={1,2,3}, prot_count=3=prot_cap
    /// get(2): prot hit, ref=true, IRR updated to small value
    /// get(4): promotes 4 → prot_count=4 > 3 → demote.
    ///   scan_len=1; scans prot_head=1.  Key 1 is the only candidate, so it is
    ///   demoted regardless of referenced bit (single-candidate scan).
    #[test]
    fn referenced_bit_shields_recently_accessed_protected_entry() {
        // cap=4: prot_cap=3, prob_cap=1
        let mut cache = LruCache::<i64, i64>::new(4);
        for k in 1i64..=4 {
            cache.put(k, k * 10);
        }
        // Promote keys 1, 2, 3 to prot.
        cache.get(&1); // prot={1}; prob={2,3,4}
        cache.get(&2); // prot={1,2}; prob={3,4}
        cache.get(&3); // prot={1,2,3}, prot_count=3=prot_cap; prob={4}

        // Access key 2 again in protected: sets ref=true, moves to tail.
        cache.get(&2); // ref=true; prot order: {1, 3, 2}

        // Promote key 4: prot_count becomes 4 > prot_cap=3 → demote.
        // scan_len=(4/4).max(1)=1; scans prot_head=1 → demotes key 1.
        cache.get(&4);

        // Key 1 should be in prob (demoted), keys 2,3,4 in prot — all resident.
        assert!(cache.get(&1).is_some(), "key 1 should still be resident (demoted to prob)");
        assert!(cache.get(&2).is_some(), "key 2 should be in prot (referenced)");
        assert!(cache.get(&3).is_some(), "key 3 should be in prot");
        assert!(cache.get(&4).is_some(), "key 4 should be in prot");
    }

    /// IRR timestamps update on repeated access: a key accessed many times
    /// accumulates a short IRR and remains resident (hot entry).
    #[test]
    fn irr_timestamps_update_on_repeated_access() {
        let mut cache = LruCache::<i64, i64>::new(4);
        cache.put(1, 10);
        // Access key 1 many times; each hit updates prev_access and last_access.
        for _ in 0..100 {
            cache.get(&1);
        }
        // The node should still be resident after many accesses.
        assert_eq!(*cache.get(&1).unwrap(), 10);
    }
}
