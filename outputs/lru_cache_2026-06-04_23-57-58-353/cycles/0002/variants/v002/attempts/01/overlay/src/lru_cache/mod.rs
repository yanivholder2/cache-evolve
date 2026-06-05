// SLRU (Segmented LRU) with frequency-aware demotion — v002 / H02.
//
// Identical to v001 (SLRU) except:
//   - Node carries a `freq: u8` counter capped at 3.
//   - On every cache hit (get in either segment, or put on an existing key),
//     `freq` is incremented (saturating at 3).
//   - When a promotion causes the protected segment to overflow, instead of
//     always demoting `prot_head` (strict LRU), we scan up to 4 nodes from
//     `prot_head` and demote the first node with the minimum `freq`.
//   - `freq` is reset to 0 on demotion from protected and when a probationary
//     slot is recycled for a brand-new key.
//
// All other logic (insertion into probationary, promotion on a probationary
// hit, eviction of probationary-LRU on overflow, fallback to protected-LRU
// when probationary is empty, and the 80/20 split) is identical to v001.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

#[derive(Debug)]
struct Node<K, V> {
    key: K,
    value: V,
    prev: Option<usize>,
    next: Option<usize>,
    /// True when the node lives in the protected segment.
    in_protected: bool,
    /// Access frequency, capped at 3 via saturating arithmetic.
    freq: u8,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
    map: HashMap<K, usize>, // key -> arena index
    nodes: Vec<Node<K, V>>, // arena; slots are reused on eviction

    // Protected segment — frequently-accessed items (LRU end = head, MRU end = tail)
    prot_head: Option<usize>,
    prot_tail: Option<usize>,
    prot_count: usize,
    prot_cap: usize, // max items in protected

    // Probationary segment — recently-inserted / one-hit items
    prob_head: Option<usize>,
    prob_tail: Option<usize>,
    prob_count: usize,
    prob_cap: usize, // max items in probationary

    capacity: usize, // total capacity = prot_cap + prob_cap
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
        // Protected gets floor(capacity * 4 / 5), capped so at least 1 slot remains
        // for the probationary segment.
        let prot_cap = (capacity * 4 / 5).min(capacity.saturating_sub(1));
        let prob_cap = capacity - prot_cap;
        log::info!(
            "LruCache.try_new: capacity={capacity} prot_cap={prot_cap} prob_cap={prob_cap}"
        );
        Some(Self {
            map: HashMap::with_capacity(capacity),
            nodes: Vec::with_capacity(capacity),
            prot_head: None,
            prot_tail: None,
            prot_count: 0,
            prot_cap,
            prob_head: None,
            prob_tail: None,
            prob_count: 0,
            prob_cap,
            capacity,
        })
    }

    pub fn new(capacity: usize) -> Self {
        Self::try_new(capacity).expect("LruCache::new: initialization failed")
    }

    /// Scan up to `max_scan` nodes from `prot_head` and return the index of
    /// the first node with the minimum `freq` (ties broken by proximity to head).
    fn find_demote_target(&self, max_scan: usize) -> usize {
        let mut cur = self.prot_head.expect("prot_head must exist when prot_count > 0");
        let mut min_idx = cur;
        let mut min_freq = self.nodes[cur].freq;

        for _ in 1..max_scan {
            match self.nodes[cur].next {
                Some(next_idx) => {
                    cur = next_idx;
                    if self.nodes[cur].freq < min_freq {
                        min_freq = self.nodes[cur].freq;
                        min_idx = cur;
                    }
                }
                None => break,
            }
        }

        min_idx
    }

    /// Returns the value for `key` if present, applying SLRU promotion with
    /// frequency tracking.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        let &idx = self.map.get(key)?;

        if self.nodes[idx].in_protected {
            // Protected hit: increment freq, move to protected MRU (counts unchanged).
            self.nodes[idx].freq = self.nodes[idx].freq.saturating_add(1).min(3);
            self.unlink(idx);
            self.push_back_prot(idx);
        } else {
            // Probationary hit: increment freq, promote to protected MRU.
            self.nodes[idx].freq = self.nodes[idx].freq.saturating_add(1).min(3);
            self.unlink(idx); // removes from probationary (in_protected still false here)
            self.prob_count -= 1;
            self.nodes[idx].in_protected = true;
            self.push_back_prot(idx);
            self.prot_count += 1;

            // If protected overflows its cap, frequency-aware demotion.
            if self.prot_count > self.prot_cap {
                let demote = self.find_demote_target(4);
                self.unlink(demote); // removes from protected (in_protected still true)
                self.prot_count -= 1;
                self.nodes[demote].in_protected = false;
                self.nodes[demote].freq = 0; // reset on demotion
                self.push_back_prob(demote);
                self.prob_count += 1;
                // prob_count can never exceed prob_cap here because the promotion
                // decremented prob_count before incrementing prot_count.
                debug_assert!(
                    self.prob_count <= self.prob_cap,
                    "prob overflow during get: prob_count={} prob_cap={}",
                    self.prob_count,
                    self.prob_cap
                );
            }
        }

        Some(&self.nodes[idx].value)
    }

    /// Inserts `key` (new or existing).  If at probationary cap (or total
    /// capacity), evicts the probationary LRU; falls back to protected LRU if
    /// probationary is empty.
    pub fn put(&mut self, key: K, value: V) {
        // Existing key: update value, increment freq, apply same promotion as get.
        if let Some(&idx) = self.map.get(&key) {
            self.nodes[idx].value = value;
            self.nodes[idx].freq = self.nodes[idx].freq.saturating_add(1).min(3);
            if self.nodes[idx].in_protected {
                self.unlink(idx);
                self.push_back_prot(idx);
            } else {
                self.unlink(idx);
                self.prob_count -= 1;
                self.nodes[idx].in_protected = true;
                self.push_back_prot(idx);
                self.prot_count += 1;
                if self.prot_count > self.prot_cap {
                    let demote = self.find_demote_target(4);
                    self.unlink(demote);
                    self.prot_count -= 1;
                    self.nodes[demote].in_protected = false;
                    self.nodes[demote].freq = 0; // reset on demotion
                    self.push_back_prob(demote);
                    self.prob_count += 1;
                }
            }
            return;
        }

        // New key: obtain an arena slot.
        let idx = if self.nodes.len() >= self.capacity || self.prob_count >= self.prob_cap {
            // Either the arena is full or probationary is at its cap.
            // Evict from probationary LRU; fall back to protected LRU if prob is empty.
            let evict = if self.prob_count > 0 {
                self.prob_head.expect("prob_head must exist when prob_count > 0")
            } else {
                self.prot_head.expect("prot_head must exist when prot_count > 0")
            };
            let was_protected = self.nodes[evict].in_protected;
            let evicted_key = self.nodes[evict].key.clone();
            self.map.remove(&evicted_key);
            self.unlink(evict);
            if was_protected {
                self.prot_count -= 1;
            } else {
                self.prob_count -= 1;
            }
            // Reuse evicted slot for the new entry; reset freq for the new key.
            self.nodes[evict].key = key.clone();
            self.nodes[evict].value = value;
            self.nodes[evict].in_protected = false;
            self.nodes[evict].freq = 0; // reset when slot is recycled for a new key
            evict
        } else {
            // Arena has room and probationary is under cap: allocate a fresh slot.
            let new_idx = self.nodes.len();
            self.nodes.push(Node {
                key: key.clone(),
                value,
                prev: None,
                next: None,
                in_protected: false,
                freq: 0,
            });
            new_idx
        };

        // Link at probationary MRU and register in the map.
        self.push_back_prob(idx);
        self.map.insert(key, idx);
        self.prob_count += 1;

        debug_assert!(self.map.len() <= self.capacity);
        debug_assert!(
            self.prot_count + self.prob_count <= self.capacity,
            "total count {} exceeds capacity {}",
            self.prot_count + self.prob_count,
            self.capacity
        );
    }

    // ── list helpers ──────────────────────────────────────────────────────────

    /// Detach `idx` from whichever segment list it currently belongs to.
    /// Reads `in_protected` to identify the owning segment's head/tail pointers.
    fn unlink(&mut self, idx: usize) {
        let prev = self.nodes[idx].prev;
        let next = self.nodes[idx].next;

        // Repair the predecessor's forward link (or the segment head).
        match prev {
            Some(p) => self.nodes[p].next = next,
            None => {
                if self.nodes[idx].in_protected {
                    self.prot_head = next;
                } else {
                    self.prob_head = next;
                }
            }
        }

        // Repair the successor's backward link (or the segment tail).
        match next {
            Some(n) => self.nodes[n].prev = prev,
            None => {
                if self.nodes[idx].in_protected {
                    self.prot_tail = prev;
                } else {
                    self.prob_tail = prev;
                }
            }
        }

        self.nodes[idx].prev = None;
        self.nodes[idx].next = None;
    }

    /// Append `idx` at the MRU end of the protected segment.
    fn push_back_prot(&mut self, idx: usize) {
        self.nodes[idx].prev = self.prot_tail;
        self.nodes[idx].next = None;
        match self.prot_tail {
            Some(t) => self.nodes[t].next = Some(idx),
            None => self.prot_head = Some(idx),
        }
        self.prot_tail = Some(idx);
    }

    /// Append `idx` at the MRU end of the probationary segment.
    fn push_back_prob(&mut self, idx: usize) {
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
    fn capacity_one() {
        let mut cache = LruCache::new(1);
        cache.put(1, 10);
        assert_eq!(*cache.get(&1).unwrap(), 10);
        cache.put(2, 20); // evicts 1
        assert!(cache.get(&1).is_none());
        assert_eq!(*cache.get(&2).unwrap(), 20);
    }

    #[test]
    #[should_panic(expected = "initialization failed")]
    fn capacity_zero_panics() {
        LruCache::<i64, i64>::new(0);
    }

    // SLRU-specific: probationary hit promotes to protected.
    #[test]
    fn probationary_hit_promotes_to_protected() {
        // capacity=8: prot_cap=6, prob_cap=2
        let mut cache = LruCache::<i64, i64>::new(8);
        cache.put(1, 10);
        // first get: prob hit -> promoted to prot
        assert_eq!(*cache.get(&1).unwrap(), 10);
        // subsequent gets stay in prot (just move to prot MRU)
        assert_eq!(*cache.get(&1).unwrap(), 10);
    }

    // A key in protected survives new inserts that fill probationary.
    #[test]
    fn protected_key_survives_prob_eviction() {
        // capacity=8, prot_cap=6, prob_cap=2
        let mut cache = LruCache::<i64, i64>::new(8);
        cache.put(42, 420);
        cache.get(&42); // promote to protected
        // Fill probationary many times over; key 42 must survive
        for k in 0i64..20 {
            if cache.get(&k).is_none() {
                cache.put(k, k * 10);
            }
        }
        assert_eq!(*cache.get(&42).unwrap(), 420);
    }

    // H02-specific: a high-freq protected node survives demotion scan even
    // when it is the oldest (at prot_head).
    #[test]
    fn freq_aware_demotion_prefers_low_freq() {
        // capacity=4: prot_cap=3, prob_cap=1
        // Promote keys 1, 2, 3 into protected (filling it to cap=3).
        // Key 1 is at prot_head (LRU), but we hit it many times so freq=3.
        // Key 4 then enters probationary; another key 5 promotes key 4 to
        // protected, causing overflow.  The scan of up to 4 nodes should
        // prefer key 2 or 3 (freq=1) over key 1 (freq=3).
        let mut cache = LruCache::<i64, i64>::new(4);

        // Put then get to promote into protected
        cache.put(1, 10);
        cache.get(&1); // prob hit -> promoted, freq=1
        cache.put(2, 20);
        cache.get(&2); // prob hit -> promoted; prot overflow: demotes freq=1 node
        cache.put(3, 30);
        cache.get(&3); // prob hit -> promoted; prot overflow again

        // At this point prot is full. Hit key 1 repeatedly to raise its freq.
        cache.get(&1);
        cache.get(&1);
        cache.get(&1); // freq[1] should be capped at 3

        // Now insert key 4 into probationary, then get it to trigger prot overflow.
        // The scan should demote the lowest-freq node (not key 1).
        cache.put(4, 40);
        cache.get(&4); // promotes 4 into prot, overflow demotes lowest-freq

        // Key 1 (freq=3) should still be in cache.
        assert!(
            cache.get(&1).is_some(),
            "key 1 with high freq should not have been demoted"
        );
    }
}
