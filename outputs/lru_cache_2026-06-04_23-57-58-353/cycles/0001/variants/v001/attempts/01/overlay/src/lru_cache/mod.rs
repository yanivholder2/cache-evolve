// SLRU (Segmented LRU) cache using HashMap + arena-based doubly-linked list.
//
// Two segments share the same arena and HashMap:
//   - Probationary: new entries land here (one-hit scan pollution stays here)
//   - Protected:    entries promoted from probationary on a second access
//
// Protected gets ~80 % of capacity; probationary gets the rest (at least 1 slot).
// On a probationary hit the node is promoted to protected-MRU; if that overflows
// protected, the protected-LRU is demoted back to probationary-MRU.
// On a protected hit the node simply moves to protected-MRU.
// New inserts always enter at probationary-MRU, evicting probationary-LRU when
// the probationary segment is at its cap.

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

    /// Returns the value for `key` if present, applying SLRU promotion.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        let &idx = self.map.get(key)?;

        if self.nodes[idx].in_protected {
            // Protected hit: move to protected MRU (counts unchanged).
            self.unlink(idx);
            self.push_back_prot(idx);
        } else {
            // Probationary hit: promote to protected MRU.
            self.unlink(idx); // removes from probationary (in_protected still false here)
            self.prob_count -= 1;
            self.nodes[idx].in_protected = true;
            self.push_back_prot(idx);
            self.prot_count += 1;

            // If protected overflows its cap, demote its LRU to probationary MRU.
            if self.prot_count > self.prot_cap {
                let demote = self.prot_head.expect("prot_head must exist when prot_count > 0");
                self.unlink(demote); // removes from protected (in_protected still true)
                self.prot_count -= 1;
                self.nodes[demote].in_protected = false;
                self.push_back_prob(demote);
                self.prob_count += 1;
                // prob_count can never exceed prob_cap here because the promotion
                // decremented prob_count before incrementing prot_count.  The safety
                // assertion below documents the invariant.
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

    /// Inserts `key` (new or existing).  If at probationary cap (or total capacity),
    /// evicts the probationary LRU; falls back to protected LRU if probationary is empty.
    pub fn put(&mut self, key: K, value: V) {
        // Existing key: update value then apply the same promotion as get.
        if let Some(&idx) = self.map.get(&key) {
            self.nodes[idx].value = value;
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
                    let demote =
                        self.prot_head.expect("prot_head must exist when prot_count > 0");
                    self.unlink(demote);
                    self.prot_count -= 1;
                    self.nodes[demote].in_protected = false;
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
            // Reuse evicted slot for the new entry.
            self.nodes[evict].key = key.clone();
            self.nodes[evict].value = value;
            self.nodes[evict].in_protected = false;
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
}
