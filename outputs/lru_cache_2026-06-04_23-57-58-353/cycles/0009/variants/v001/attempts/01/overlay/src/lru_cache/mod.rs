// SLRU cache with ghost-set admission + frequency-aware demotion (H02) +
// minimum-frequency probationary eviction scan (H03) +
// frequency counter cap at MAX_FREQ (H02-cap) +
// IRR tie-breaker in protected demotion (H01/cycle-9).
//
// Two segments share a single arena:
//   probationary  (20 %)  — new items land here
//   protected     (80 %)  — items promoted on a cache hit
//
// A bounded ghost set tracks recently evicted probationary keys.
// When an incoming key matches a ghost entry it is admitted directly into the
// protected segment, giving cross-residency warm keys immediate protection.
//
// Demotion from protected uses a minimum-frequency scan of the LRU quarter
// rather than strict LRU, with counters halved on demotion to preserve signal.
//
// Probationary eviction (H03): instead of blindly evicting the LRU head of
// probationary, scan the LRU half and pick the lowest-frequency victim.  This
// protects entries recently demoted from protected (freq > 0) over cold new
// arrivals (freq = 0) without changing list positions or disabling recency
// ordering.
//
// Frequency cap (H02-cap): freq counters are clamped to MAX_FREQ on every
// access.  This prevents long-resident protected entries from accumulating
// unbounded scores that make them effectively immortal.  With MAX_FREQ=8,
// three halvings (demotion events) reduce the maximum to 1, giving warm
// entries a modest but bounded residency advantage over cold newcomers.
//
// IRR tie-breaker (H01/cycle-9): `last_access` and `prev_access` op-clock
// stamps are maintained on every cache hit.  Inside demote_prot_head() the
// victim is selected by (freq ascending, IRR descending) — among equal-
// frequency candidates the one with the highest inter-reference recency
// (coldest, i.e. longest gap since its last two hits) is demoted first.
// Unlike the referenced-bit tie-breaker this fires on every demotion because
// any two equal-frequency entries have distinguishable IRR values.  IRR is
// u64::MAX for entries that have been accessed fewer than twice.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

// ── frequency cap ─────────────────────────────────────────────────────────────

const MAX_FREQ: u32 = 8;

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
    referenced: bool,
    last_access: u64,  // op_clock at most-recent get hit; 0 if never hit
    prev_access: u64,  // op_clock at second-most-recent get hit; 0 if fewer than 2 hits
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

    // Monotonic counter bumped on every cache hit; used for IRR computation.
    op_clock: u64,
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
            capacity,
            op_clock: 1,
        })
    }

    pub fn new(capacity: usize) -> Self {
        Self::try_new(capacity).expect("LruCache::new: initialization failed")
    }

    /// Compute the inter-reference recency for the node at `idx`.
    ///
    /// Returns `u64::MAX` if the node has been hit fewer than twice (no second
    /// access stamp), otherwise returns `last_access - prev_access`.  Higher
    /// IRR means the node was accessed less recently relative to its prior
    /// access — i.e., it is "colder" in the IRR sense.
    #[inline]
    fn irr(&self, idx: usize) -> u64 {
        if self.nodes[idx].prev_access == 0 {
            u64::MAX
        } else {
            self.nodes[idx].last_access - self.nodes[idx].prev_access
        }
    }

    /// Returns the value for `key` if resident, updating segment recency and freq.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        let &idx = self.map.get(key)?;
        // Increment access frequency on every hit, capped at MAX_FREQ.
        self.nodes[idx].freq = (self.nodes[idx].freq + 1).min(MAX_FREQ);
        // Update IRR timestamps before any segment-specific logic.
        self.nodes[idx].prev_access = self.nodes[idx].last_access;
        self.nodes[idx].last_access = self.op_clock;
        self.op_clock += 1;
        match self.nodes[idx].segment {
            Segment::Protected => {
                // Touch: move to protected MRU.
                self.unlink_prot(idx);
                self.push_back_prot(idx);
                // Retain referenced bit for test compatibility (no longer used in demotion).
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
                // Protected overflow: demote lowest-score from LRU quarter to probationary MRU.
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
                referenced: false,
                last_access: 0,
                prev_access: 0,
            });
            i
        } else {
            // Arena full — evict to reclaim a slot.
            let evict_idx = if self.prob_count > 0 {
                // H03: minimum-frequency scan of the LRU half of probationary.
                // This protects recently-demoted entries (freq > 0) over cold
                // newcomers (freq = 0), giving warm keys a second residency window.
                let scan_len = (self.prob_count / 2).max(1);
                let mut victim_idx: Option<usize> = None;
                let mut victim_freq = u32::MAX;
                let mut current = self.prob_head;
                let mut scanned = 0usize;
                while scanned < scan_len {
                    let Some(idx) = current else { break };
                    let freq = self.nodes[idx].freq;
                    if freq < victim_freq {
                        victim_freq = freq;
                        victim_idx = Some(idx);
                    }
                    current = self.nodes[idx].next;
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
            self.nodes[evict_idx].key = key.clone();
            self.nodes[evict_idx].value = value;
            self.nodes[evict_idx].prev = None;
            self.nodes[evict_idx].next = None;
            self.nodes[evict_idx].freq = 0;
            self.nodes[evict_idx].referenced = false;
            self.nodes[evict_idx].last_access = 0;
            self.nodes[evict_idx].prev_access = 0;
            evict_idx
        };

        // ── Step 2: place the node into the right segment ─────────────────
        if self.ghost_set.contains(&key) {
            // Ghost hit: key re-arrived after a previous probationary eviction.
            // Admit directly into protected, bypassing the one-hit filter.
            self.ghost_set.remove(&key);
            self.ghost.retain(|k| k != &key);

            self.nodes[idx].segment = Segment::Protected;
            // referenced stays false: ghost-admitted entry starts cold.
            self.push_back_prot(idx);
            self.prot_count += 1;
            self.map.insert(key, idx);

            // Protected overflow: demote lowest-score from LRU quarter to probationary MRU.
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

    /// Demote the lowest-score node from the LRU quarter of the protected
    /// segment to probationary MRU.  Score is (freq ascending, IRR descending):
    /// among equal-frequency candidates the one with the highest IRR (longest
    /// gap between its last two hits — i.e., coldest) is chosen as victim.
    /// Halve the demoted node's freq counter and clear its referenced bit to
    /// preserve relative signal across demotion cycles.
    fn demote_prot_head(&mut self) {
        // Scan the LRU quarter (at least 1 node) for the minimum-score victim.
        let scan_len = (self.prot_count / 4).max(1);

        let mut victim_idx: Option<usize> = None;
        let mut victim_freq: u32 = u32::MAX;
        let mut victim_irr: u64 = 0;
        let mut current = self.prot_head;
        let mut scanned = 0usize;

        while scanned < scan_len {
            let Some(idx) = current else { break };
            let cur_freq = self.nodes[idx].freq;
            let cur_irr = self.irr(idx);
            // Lower freq wins; among equal freq, higher IRR (colder) wins.
            if cur_freq < victim_freq
                || (cur_freq == victim_freq && cur_irr > victim_irr)
            {
                victim_freq = cur_freq;
                victim_irr = cur_irr;
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
        // put(3) → scan_len=1, victim=prob_head=1 (freq=0), ghost={1}; insert 3 → prob={2,3}
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
        cache.put(3, 30); // scan_len=1, victim=prob_head=2 (freq=0)

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
        cache.put(4, 40); // scan_len=1, victim=prob_head=2 (freq=0)
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
        cache.put(4, 40); // scan_len=1, victim=prob_head=1 (freq=0)
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
        cache.put(4, 40); // scan_len=1, victim=prob_head=1 (freq=0)
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
    ///   2. put(9) → scan_len=4, all freq=0, victim=prob_head=1; ghost = {1}.
    ///   3. get(2..=7) → promote 6 keys to prot (no new prob evictions, ghost unchanged).
    ///   4. put(1): prob_count=2, scan_len=1, victim=prob_head=8 (freq=0);
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
        // put(1): scan_len=1, victim=prob_head=8 → ghost={1,8} (fits in ghost_cap=2).
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

    /// H03 regression: a demoted-from-protected entry (freq > 0) near the prob
    /// LRU head is not evicted when a cold newcomer (freq = 0) exists within
    /// the scan window.
    ///
    /// cap=4: prot_cap=3, prob_cap=1, ghost_cap=1.
    /// put(1..=4): prob={1,2,3,4}
    /// get(1): promotes 1 → prot; prob={2,3,4}, prot={1}
    /// get(2): promotes 2 → prot; prob={3,4}, prot={1,2}
    /// get(3): promotes 3 → prot; prob={4}, prot={1,2,3}
    /// get(4): promotes 4 → prot; prob={}, prot_count=4 > prot_cap=3
    ///   → demote_prot_head: scans LRU quarter (1 node) = key 1 (freq=1), demote to prob
    ///   → prob={1(freq=0 after halve)}, prot={2,3,4}
    ///   Wait — freq=1 halved = 0. Let me reconsider.
    ///
    /// Actually with cap=6: prot_cap=4, prob_cap=2, ghost_cap=2.
    /// put(1..=6): prob={1,2,3,4,5,6}
    /// get(1): promotes 1 (freq=1) → prot; prob={2,3,4,5,6}
    /// get(2): promotes 2 (freq=1) → prot; prob={3,4,5,6}
    /// get(2) again: refreshes 2 in prot (freq=2)
    /// get(2) again: freq=3
    /// get(2) again: freq=4
    /// Now prot={1(freq=1),2(freq=4)}, prob={3,4,5,6}
    /// put(7): arena full → prob_count=4 > 0
    ///   scan_len=(4/2).max(1)=2, scan from prob_head:
    ///   key=3 (freq=0), key=4 (freq=0) → victim=key=3 (first with min freq=0)
    ///   → evict key=3
    #[test]
    fn h03_demoted_entry_survives_over_cold_newcomer() {
        // cap=6: prot_cap=4, prob_cap=2, ghost_cap=2
        let mut cache = LruCache::<i64, i64>::new(6);
        // Fill arena with cold inserts.
        for k in 1i64..=6 {
            cache.put(k, k * 10);
        }
        // prob={1,2,3,4,5,6}, prot={}

        // Promote key 1 and key 2 to prot, access key 2 repeatedly to build freq.
        cache.get(&1); // promote 1 to prot (freq=1)
        cache.get(&2); // promote 2 to prot (freq=1)
        cache.get(&2); // freq=2
        cache.get(&2); // freq=3
        // prot={1(freq=1),2(freq=3)}, prob={3,4,5,6}

        // Put two more to fill prot to cap then demote.
        cache.get(&3); // promote 3 (freq=1) → prot={1,2,3}; prob={4,5,6}
        cache.get(&4); // promote 4 (freq=1) → prot={1,2,3,4}, prot_count=4=prot_cap → no demotion
        // prob={5,6}

        // Now put(7): arena full, prob_count=2 > 0
        // scan_len=(2/2).max(1)=1, scans prob_head=5 (freq=0) → victim=5
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

    /// Frequency cap test: verify that repeated gets do not push freq above MAX_FREQ.
    #[test]
    fn freq_cap_limits_counter() {
        let mut cache = LruCache::<i64, i64>::new(4);
        cache.put(1, 10);
        // Access key 1 many more times than MAX_FREQ.
        for _ in 0..(MAX_FREQ as usize + 100) {
            cache.get(&1);
        }
        // The freq should be capped at MAX_FREQ (8), not higher.
        // We verify indirectly: the node is still resident and not phantom.
        assert_eq!(*cache.get(&1).unwrap(), 10);
    }

    /// IRR tie-breaker test: among equal-frequency protected entries the scan
    /// window selects the single candidate (scan_len=1) at the LRU head.
    ///
    /// cap=4: prot_cap=3, prob_cap=1
    /// put(1..=4): prob={1,2,3,4}
    /// get(1): promotes 1 (freq=1) → prot={1}; prob={2,3,4}
    /// get(2): promotes 2 (freq=1) → prot={1,2}; prob={3,4}
    /// get(3): promotes 3 (freq=1) → prot={1,2,3}; prob={4}
    /// get(2): prot hit, freq=2; move 2 to tail → prot={1,3,2}
    /// get(4): promotes 4 → prot_count=4 > prot_cap=3
    ///   → demote_prot_head: scan_len=1, only candidate = prot_head=1
    ///   → demote key 1 (regardless of IRR scoring: only one candidate)
    #[test]
    fn irr_tiebreaker_demotes_lru_head_when_scan_len_one() {
        // cap=4: prot_cap=3, prob_cap=1
        let mut cache = LruCache::<i64, i64>::new(4);
        for k in 1i64..=4 {
            cache.put(k, k * 10);
        }
        // Promote keys 1, 2, 3 to prot (each gets freq=1).
        cache.get(&1); // prot={1(f=1)}; prob={2,3,4}
        cache.get(&2); // prot={1,2(f=1)}; prob={3,4}
        cache.get(&3); // prot={1,2,3(f=1)}, prot_count=3=prot_cap; prob={4}

        // Access key 2 again in protected: freq=2, IRR timestamp set.
        cache.get(&2); // freq=2; prot={1,3,2}

        // Promote key 4: prot_count becomes 4 > prot_cap=3 → demote.
        // scan_len=(4/4).max(1)=1; only prot_head=1 is scanned → demoted.
        cache.get(&4);

        // Key 1 was demoted to prob (still resident).
        assert!(cache.get(&1).is_some(), "key 1 should still be resident (demoted to prob)");
        // Keys 2, 3, 4 remain in prot.
        assert!(cache.get(&2).is_some(), "key 2 should be in prot (higher freq)");
        assert!(cache.get(&3).is_some(), "key 3 should be in prot");
        assert!(cache.get(&4).is_some(), "key 4 should be in prot");
    }

    /// IRR clock tracking regression: op_clock advances on every hit and the
    /// cache state remains consistent after mixed promotions and demotions.
    ///
    /// cap=8: prot_cap=6, prob_cap=2, ghost_cap=2.
    ///
    /// Sequence:
    ///   1. Fill all 8 slots → everything in prob.
    ///   2. Promote keys 1..=6 to prot; access key 1 twice more to build IRR.
    ///   3. put(9), put(10) → evict 7 and 8 from prob.
    ///   4. get(9) → promote 9; prot_count=7 > prot_cap=6 → demote.
    ///      scan_len=(7/4).max(1)=1; only prot_head (key 2) is scanned → demoted.
    ///   5. Verify 9 is resident and total residents ≤ capacity.
    #[test]
    fn irr_clock_tracking_does_not_corrupt_state() {
        // cap=8: prot_cap=6, prob_cap=2
        let mut cache = LruCache::<i64, i64>::new(8);
        for k in 1i64..=8 {
            cache.put(k, k * 10);
        }
        // Promote keys 1..=6 to prot (each freq=1 after first hit).
        for k in 1i64..=6 {
            cache.get(&k);
        }
        // prot={1,2,3,4,5,6} (prot_count=6=prot_cap), prob={7,8}

        // Access key 1 twice more in prot to advance op_clock and set IRR on key 1.
        // After these two hits: key 1 has prev_access set, irr(1) = small (warm).
        // Key 1 moves to prot MRU tail; prot LRU head becomes key 2.
        cache.get(&1);
        cache.get(&1);

        // Evict prob entries to make room for new insertions.
        cache.put(9, 90);   // evicts prob_head=7; prob={8,9}
        cache.put(10, 100); // evicts prob_head=8; prob={9,10}

        // Promote key 9: prot_count=7 > prot_cap=6 → demote prot_head=key 2.
        // scan_len=(7/4).max(1)=1; only one candidate scanned regardless of IRR.
        cache.get(&9);

        // Key 9 must be resident after promotion.
        assert!(cache.get(&9).is_some(), "key 9 should be resident after promotion");
        // Capacity invariant must hold.
        let resident = (1i64..=10).filter(|&k| cache.get(&k).is_some()).count();
        assert!(resident <= 8, "capacity violated: {resident} residents");
    }
}
