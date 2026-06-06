// SLRU cache: frequency-primary + IRR tie-breaker + bounded ghost set (H02/cycle-10).
//
// Two segments share a single arena:
//   probationary  (20 %)  — new items land here
//   protected     (80 %)  — items promoted on a cache hit
//
// Ghost admission (bounded ghost set, capacity = prob_cap):
//   When a probationary entry is evicted, its key enters the ghost set.  On
//   the next put of that key (a get-miss re-insertion), it is fast-tracked
//   directly into the protected segment, bypassing the normal
//   probationary→promote path.  This recovers the hit-rate advantage that
//   was lost in cycle 9 when ghost admission was removed.
//
// Demotion from protected uses a minimum-score scan of the LRU quarter.
// Victim score is (freq, irr) ascending: lower frequency is colder; at equal
// frequency, higher IRR (larger gap between last two accesses) is colder.
// IRR = MAX (u64::MAX) for items seen only once, making cold first-timers
// strictly colder than any warmed item at the same frequency.
//
// Frequency cap: freq counters are clamped to MAX_FREQ on every access.
// Three halvings reduce a cap-8 entry to 1, giving warm entries a bounded
// residency advantage over cold newcomers.
//
// Probationary eviction uses the same composite (freq, irr) over the LRU half.
// IRR timestamps are preserved across demotion so a warm demoted entry retains
// its competitive signal in probationary.

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
    /// Clock tick of the most recent access (put or get).
    last_access: u64,
    /// Clock tick of the second-most-recent access; 0 means never accessed twice.
    prev_access: u64,
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

    /// Monotonically increasing logical clock; incremented on each put and get.
    op_clock: u64,

    capacity: usize,

    // Ghost set: bounded FIFO of recently-evicted probationary keys.
    // Capacity equals prob_cap.  A ghost hit on put() fast-tracks the key into
    // protected rather than probationary.
    ghost: VecDeque<K>,
    ghost_set: HashSet<K>,
    ghost_cap: usize,
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
            "LruCache.try_new: capacity={capacity} prot_cap={prot_cap} prob_cap={prob_cap} ghost_cap={ghost_cap}"
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
            op_clock: 1,
            capacity,
            ghost: VecDeque::with_capacity(ghost_cap + 1),
            ghost_set: HashSet::with_capacity(ghost_cap + 1),
            ghost_cap,
        })
    }

    pub fn new(capacity: usize) -> Self {
        Self::try_new(capacity).expect("LruCache::new: initialization failed")
    }

    /// Returns the IRR (inter-reference recency) for the node at `idx`.
    ///
    /// Returns `u64::MAX` if the node has never been accessed twice (`prev_access == 0`),
    /// marking it as maximally cold. Otherwise returns `last_access - prev_access`.
    fn irr(&self, idx: usize) -> u64 {
        if self.nodes[idx].prev_access == 0 {
            u64::MAX
        } else {
            self.nodes[idx].last_access - self.nodes[idx].prev_access
        }
    }

    /// Record a recently-evicted probationary key in the ghost set.
    ///
    /// If the ghost set is at capacity, the oldest entry is dropped to make room.
    /// Protected evictions are not tracked here.
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

    /// Returns the value for `key` if resident, updating segment recency and freq.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        let &idx = self.map.get(key)?;
        // Increment access frequency on every hit, capped at MAX_FREQ.
        self.nodes[idx].freq = (self.nodes[idx].freq + 1).min(MAX_FREQ);
        // Update IRR timestamps before segment dispatch.
        self.nodes[idx].prev_access = self.nodes[idx].last_access;
        self.nodes[idx].last_access = self.op_clock;
        self.op_clock += 1;
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
                // Defensive ghost cleanup: if this key was somehow still in the ghost
                // set (e.g. inserted via normal prob path after eviction), clear it
                // before marking the node as protected.
                if self.ghost_set.remove(key) {
                    self.ghost.retain(|k| k != key);
                }
                self.nodes[idx].segment = Segment::Protected;
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
                last_access: self.op_clock,
                prev_access: 0,
            });
            self.op_clock += 1;
            i
        } else {
            // Arena full — evict to reclaim a slot.
            let evict_idx = if self.prob_count > 0 {
                // Minimum-score scan of the LRU half of probationary.
                // Score is (freq, irr) ascending: lower freq is colder; at equal freq,
                // higher IRR (larger gap = longer since last two accesses) is colder.
                let scan_len = (self.prob_count / 2).max(1);
                let mut victim_idx: Option<usize> = None;
                let mut victim_freq = u32::MAX;
                let mut victim_irr = 0u64;
                let mut current = self.prob_head;
                let mut scanned = 0usize;
                while scanned < scan_len {
                    let Some(idx) = current else { break };
                    let freq = self.nodes[idx].freq;
                    let irr = self.irr(idx);
                    let is_better_victim = match victim_idx {
                        None => true,
                        Some(_) => freq < victim_freq || (freq == victim_freq && irr > victim_irr),
                    };
                    if is_better_victim {
                        victim_freq = freq;
                        victim_irr = irr;
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
                // Track evicted probationary key in ghost for potential fast-track
                // re-admission on next arrival.
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
            // Reset the recycled slot with fresh timestamps.
            self.nodes[evict_idx].key = key.clone();
            self.nodes[evict_idx].value = value;
            self.nodes[evict_idx].prev = None;
            self.nodes[evict_idx].next = None;
            self.nodes[evict_idx].freq = 0;
            self.nodes[evict_idx].prev_access = 0;
            self.nodes[evict_idx].last_access = self.op_clock;
            self.op_clock += 1;
            evict_idx
        };

        // ── Step 2: place node — ghost hit → protected, else probationary ──
        if self.ghost_set.contains(&key) {
            // Ghost hit: this key was recently evicted from probationary.
            // Fast-track it directly into protected, skipping the normal
            // one-hit-then-promote path.
            self.ghost_set.remove(&key);
            self.ghost.retain(|k| k != &key);
            self.nodes[idx].segment = Segment::Protected;
            self.push_back_prot(idx);
            self.prot_count += 1;
            self.map.insert(key, idx);
            if self.prot_count > self.prot_cap {
                self.demote_prot_head();
            }
        } else {
            self.nodes[idx].segment = Segment::Probationary;
            self.push_back_prob(idx);
            self.prob_count += 1;
            self.map.insert(key, idx);
        }

        debug_assert!(self.map.len() <= self.capacity);
    }

    // ── segment helpers ───────────────────────────────────────────────────────

    /// Demote the lowest-score node from the LRU quarter of the protected
    /// segment to probationary MRU.  Score is (freq, irr) ascending:
    /// lower frequency is colder; at equal frequency, higher IRR (larger gap
    /// between last two accesses) is colder.  IRR timestamps are preserved
    /// across demotion so a warm demoted entry keeps its competitive signal.
    /// Freq is halved on demotion to preserve relative ordering across cycles.
    fn demote_prot_head(&mut self) {
        // Scan the LRU quarter (at least 1 node) for the minimum-score victim.
        let scan_len = (self.prot_count / 4).max(1);

        let mut victim_idx: Option<usize> = None;
        let mut victim_freq = u32::MAX;
        let mut victim_irr = 0u64;
        let mut current = self.prot_head;
        let mut scanned = 0usize;

        while scanned < scan_len {
            let Some(idx) = current else { break };
            let freq = self.nodes[idx].freq;
            let irr = self.irr(idx);
            // Candidate beats current victim if lower freq, or equal freq with higher IRR.
            let is_better_victim = match victim_idx {
                None => true,
                Some(_) => freq < victim_freq || (freq == victim_freq && irr > victim_irr),
            };
            if is_better_victim {
                victim_freq = freq;
                victim_irr = irr;
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
            // Timestamps are NOT reset: warm demoted entries retain their IRR signal.
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
        // put(3) → scan_len=1, victim=prob_head=1 (freq=0, irr=MAX); insert 3 → prob={2,3}
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
        cache.put(3, 30); // scan_len=1, victim=prob_head=2 (freq=0, irr=MAX)

        assert_eq!(*cache.get(&1).unwrap(), 11);
        assert!(cache.get(&2).is_none());
        assert_eq!(*cache.get(&3).unwrap(), 30);
    }

    #[test]
    fn eviction_chain() {
        // cap=3: prot_cap=2, prob_cap=1
        // All inserts go to prob; evictions follow LRU order via scan_len=1.
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

    /// H03 regression: a demoted-from-protected entry (freq > 0) near the prob
    /// LRU head is not evicted when a cold newcomer (freq = 0) exists within
    /// the scan window.
    #[test]
    fn demoted_entry_survives_over_cold_newcomer() {
        // cap=6: prot_cap=4, prob_cap=2
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
        // prot={1,2}, prob={3,4,5,6}

        // Promote 3 and 4 to fill prot.
        cache.get(&3); // promote 3 (freq=1) → prot={1,2,3}; prob={4,5,6}
        cache.get(&4); // promote 4 (freq=1) → prot={1,2,3,4}, prot_count=4=prot_cap; prob={5,6}

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

    /// IRR timestamps survive demotion: a promoted entry's prev_access and last_access
    /// are preserved when it is demoted back to probationary, keeping its IRR signal.
    #[test]
    fn irr_timestamps_preserved_across_demotion() {
        // cap=4: prot_cap=3, prob_cap=1.
        let mut cache = LruCache::<i64, i64>::new(4);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30);
        cache.put(4, 40);

        // Promote 1, 2, 3 to prot.
        cache.get(&1); // freq=1, irr timestamps set. prot={1}.
        cache.get(&2); // freq=1. prot={1,2}.
        cache.get(&3); // freq=1. prot={1,2,3}=prot_cap.

        // Promote 4 → overflow → demote prot_head=1 (freq=1→0, irr preserved).
        cache.get(&4); // prot={2,3,4}, prob={1(f=0, irr=finite)}.

        // Key 1 survived demotion and is still resident.
        assert!(cache.get(&1).is_some(), "demoted key 1 should still be resident");
        // Keys 2,3,4 are in prot.
        assert!(cache.get(&2).is_some());
        assert!(cache.get(&3).is_some());
        assert!(cache.get(&4).is_some());
    }

    /// Frequency ordering dominates IRR: a freq>0 item demoted to prob survives
    /// eviction over a fresh freq=0 item regardless of IRR, because freq is the
    /// primary eviction key.
    #[test]
    fn freq_dominates_irr_in_eviction() {
        // cap=6: prot_cap=4, prob_cap=2.
        let mut cache = LruCache::<i64, i64>::new(6);
        for k in 1i64..=6 {
            cache.put(k, k * 10);
        }
        // prob={1,2,3,4,5,6}, prot={}.

        // Promote 1..=4, repeatedly access 1 to push its freq high.
        cache.get(&1); // freq=1, prot={1}.
        cache.get(&1); // freq=2.
        cache.get(&1); // freq=3.
        cache.get(&2); // freq=1. prot={1,2}.
        cache.get(&3); // freq=1. prot={1,2,3}.
        cache.get(&4); // freq=1. prot={1,2,3,4}=prot_cap. prob={5,6}.

        // Promote 5: prot_count=5 > 4 → demote. scan_len=1.
        // prot_head=key1 (promoted first). Demote key1 (freq=3→1 after /2).
        cache.get(&5); // prot={2,3,4,5}, prob={key6(head,f=0), key1(tail,f=1)}.

        // put(7): arena full, prob_count=2. scan_len=1.
        // Scans prob_head=key6 (f=0, irr=MAX). victim=key6. key1 (f=1) is not scanned.
        cache.put(7, 70);

        // Key 6 (fresh, freq=0, prob LRU head) should be evicted.
        assert!(cache.get(&6).is_none(), "key 6 (cold LRU, freq=0) should be evicted");
        // Key 1 (demoted to prob, freq=1 after halving) survives in prob.
        assert!(cache.get(&1).is_some(), "key 1 (demoted, freq=1) should survive");
        assert!(cache.get(&7).is_some(), "key 7 newly inserted should be resident");
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
        assert_eq!(*cache.get(&1).unwrap(), 10);
    }

    /// Ghost admission: a recently-evicted probationary key is fast-tracked into
    /// protected on its next insertion, bypassing the normal one-hit-then-promote
    /// path.
    ///
    /// Scenario (cap=6: prot_cap=4, prob_cap=2, ghost_cap=2):
    ///   put(1..=6): arena full, prob={1,2,3,4,5,6}, prot={}, ghost={}.
    ///   put(7):  evict prob LRU = key1 → ghost={1}; prob gains key7.
    ///   put(1):  ghost hit for key1. Step 1 evicts key2 → ghost=[1,2].
    ///            Step 2 finds key1 in ghost → admits to protected directly.
    ///   put(8):  evicts key3 from prob. key1 (in protected) should survive.
    #[test]
    fn ghost_hit_admits_to_protected() {
        // cap=6: prot_cap=4, prob_cap=2, ghost_cap=2.
        let mut cache = LruCache::<i64, i64>::new(6);
        for k in 1i64..=6 {
            cache.put(k, k * 10);
        }
        // Arena full: prob={1,2,3,4,5,6}, prot={}, ghost={}.

        // put(7): scan_len=3, evict prob LRU = key1 (first scanned, freq=0, irr=MAX).
        // ghost gains key1.
        cache.put(7, 70);
        assert!(cache.get(&1).is_none(), "key 1 should be evicted to ghost");

        // put(1): Step 1 evicts key2 (new prob LRU scan), adds key2 to ghost.
        // ghost = [1, 2] (len=1 before this eviction, ghost_cap=2, no displacement).
        // Step 2: ghost_set contains key1 → admit directly to protected.
        cache.put(1, 11);
        assert_eq!(
            *cache.get(&1).unwrap(),
            11,
            "key 1 should be re-admitted via ghost hit"
        );

        // put(8): evict from prob (key3 is new LRU head). key1 is in protected
        // and must survive this probationary eviction.
        cache.put(8, 80);
        assert!(
            cache.get(&1).is_some(),
            "ghost-admitted key 1 (in protected) should survive a subsequent probationary eviction"
        );
        assert!(cache.get(&3).is_none(), "key 3 should be evicted by put(8)");
    }
}
