// SIEVE cache: a single-queue eviction policy (NSDI 2024).
//
// New items are inserted at the MRU tail. `get` marks an item visited without
// moving it in the list. When eviction is needed, a scanning "hand" starts at
// the tail (or its last position) and moves toward the head, clearing visited
// bits; the first unvisited item it encounters is evicted.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

#[derive(Debug)]
struct Node<K, V> {
    key: K,
    value: V,
    prev: Option<usize>, // toward LRU head (older)
    next: Option<usize>, // toward MRU tail (newer)
    visited: bool,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
    map: HashMap<K, usize>, // key -> node index
    nodes: Vec<Node<K, V>>, // arena
    head: Option<usize>,    // LRU/oldest end
    tail: Option<usize>,    // MRU/newest end
    hand: Option<usize>,    // eviction hand
    count: usize,           // number of live entries
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
        log::info!("LruCache.try_new: capacity={capacity}");
        Some(Self {
            map: HashMap::with_capacity(capacity),
            nodes: Vec::with_capacity(capacity),
            head: None,
            tail: None,
            hand: None,
            count: 0,
            capacity,
        })
    }

    pub fn new(capacity: usize) -> Self {
        Self::try_new(capacity).expect("LruCache::new: initialization failed")
    }

    /// Returns the value for `key` if present, marking it visited.
    /// Does **not** change list position.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        let &idx = self.map.get(key)?;
        self.nodes[idx].visited = true;
        Some(&self.nodes[idx].value)
    }

    /// Inserts or updates `key`. If at capacity, evicts using the SIEVE hand.
    pub fn put(&mut self, key: K, value: V) {
        // Key already present: update value in-place (no list movement, no eviction).
        if let Some(&idx) = self.map.get(&key) {
            self.nodes[idx].value = value;
            return;
        }
        // New key — acquire a slot.
        let idx = if self.count < self.capacity {
            // Arena has room: allocate a fresh slot.
            let idx = self.nodes.len();
            self.nodes.push(Node {
                key: key.clone(),
                value,
                prev: None,
                next: None,
                visited: false,
            });
            self.count += 1;
            idx
        } else {
            // Arena full: run SIEVE hand to find and evict a victim slot.
            let victim_idx = self.sieve_evict();
            self.nodes[victim_idx].key = key.clone();
            self.nodes[victim_idx].value = value;
            self.nodes[victim_idx].visited = false;
            victim_idx
        };
        // Link the slot at the MRU tail and register it.
        self.push_back(idx);
        self.map.insert(key, idx);
        debug_assert!(self.map.len() <= self.capacity);
    }

    /// Advance the SIEVE hand until an unvisited node is found, then evict it.
    ///
    /// The hand starts at `tail` when `None`. It moves toward the head (`prev`
    /// direction). Visited nodes have their bit cleared but are not evicted. If
    /// the hand steps past the head (reaches `None`), it wraps back to `tail`.
    /// Termination is guaranteed: at most one full cycle clears all visited bits,
    /// and the second pass through any node will find it unvisited.
    ///
    /// Returns the (now-unlinked) slot index of the evicted node.
    fn sieve_evict(&mut self) -> usize {
        loop {
            let idx = match self.hand {
                Some(h) => h,
                None => {
                    // Wrap (or initialize) to tail.
                    let t = self.tail.expect("non-empty cache must have a tail");
                    self.hand = Some(t);
                    t
                }
            };

            if self.nodes[idx].visited {
                // Clear visited bit and advance hand toward head.
                self.nodes[idx].visited = false;
                self.hand = self.nodes[idx].prev;
            } else {
                // Victim found: save hand successor, evict node, return its slot.
                let new_hand = self.nodes[idx].prev;
                let evicted_key = self.nodes[idx].key.clone();
                self.map.remove(&evicted_key);
                self.unlink(idx);
                self.hand = new_hand;
                return idx;
            }
        }
    }

    /// Detach node `idx` from its current list position.
    fn unlink(&mut self, idx: usize) {
        let prev = self.nodes[idx].prev;
        let next = self.nodes[idx].next;
        match prev {
            Some(p) => self.nodes[p].next = next,
            None => self.head = next,
        }
        match next {
            Some(n) => self.nodes[n].prev = prev,
            None => self.tail = prev,
        }
        self.nodes[idx].prev = None;
        self.nodes[idx].next = None;
    }

    /// Append node `idx` to the tail (MRU end) of the list.
    fn push_back(&mut self, idx: usize) {
        self.nodes[idx].prev = self.tail;
        self.nodes[idx].next = None;
        match self.tail {
            Some(t) => self.nodes[t].next = Some(idx),
            None => self.head = Some(idx),
        }
        self.tail = Some(idx);
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

    // ── Basic API contract ───────────────────────────────────────────────────

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
        // key 1 was updated in-place three times; the cache still has room for a second key
        cache.put(2, 40);
        assert_eq!(*cache.get(&1).unwrap(), 30);
        assert_eq!(*cache.get(&2).unwrap(), 40);
    }

    #[test]
    fn overwrite_at_full_capacity_does_not_evict() {
        let mut cache = LruCache::new(2);
        cache.put(1, 10);
        cache.put(2, 20); // cache full
        cache.put(1, 11); // overwrite: must NOT evict key 2
        assert_eq!(*cache.get(&1).unwrap(), 11);
        assert_eq!(*cache.get(&2).unwrap(), 20);
    }

    #[test]
    fn capacity_one() {
        let mut cache = LruCache::new(1);
        cache.put(1, 10);
        assert_eq!(*cache.get(&1).unwrap(), 10);
        // key 1 is visited; eviction clears the bit, wraps, then evicts key 1
        cache.put(2, 20);
        assert!(cache.get(&1).is_none());
        assert_eq!(*cache.get(&2).unwrap(), 20);
    }

    #[test]
    fn get_only_element() {
        let mut cache = LruCache::new(3);
        cache.put(1, 10);
        // Single-node get: list stays intact after marking visited
        assert_eq!(*cache.get(&1).unwrap(), 10);
        cache.put(2, 20);
        cache.put(3, 30);
        assert_eq!(*cache.get(&1).unwrap(), 10);
    }

    #[test]
    fn eviction_chain() {
        // Six sequential inserts into a capacity-3 cache (no re-access).
        // SIEVE hand starts at tail and walks toward head on each eviction,
        // so keys 1, 2, 3 are all eventually displaced.
        let mut cache = LruCache::new(3);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30);
        cache.put(4, 40);
        cache.put(5, 50);
        cache.put(6, 60);

        assert!(cache.get(&1).is_none());
        assert!(cache.get(&2).is_none());
        assert!(cache.get(&3).is_none());
        assert_eq!(*cache.get(&4).unwrap(), 40);
        assert_eq!(*cache.get(&5).unwrap(), 50);
        assert_eq!(*cache.get(&6).unwrap(), 60);
    }

    #[test]
    #[should_panic(expected = "initialization failed")]
    fn capacity_zero_panics() {
        LruCache::<i64, i64>::new(0);
    }

    // ── SIEVE-specific behavior ──────────────────────────────────────────────

    /// A `get` hit marks the item visited; on the next eviction the visited item
    /// is skipped and the nearest unvisited item is evicted instead.
    #[test]
    fn get_hit_marks_visited_prevents_eviction() {
        let mut cache = LruCache::new(2);
        cache.put(1, 10); // head (older)
        cache.put(2, 20); // tail (newer)
        cache.get(&1);    // mark key 1 visited

        // SIEVE: hand starts at tail (key 2, unvisited) -> evict key 2
        cache.put(3, 30);

        assert_eq!(*cache.get(&1).unwrap(), 10); // key 1 survived
        assert!(cache.get(&2).is_none());         // key 2 evicted
        assert_eq!(*cache.get(&3).unwrap(), 30);
    }

    /// When the hand scans a visited item it clears the bit and advances.
    /// If all items are visited the hand makes a full pass before evicting.
    #[test]
    fn visited_bit_cleared_after_hand_pass() {
        let mut cache = LruCache::new(2);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.get(&1); // both visited
        cache.get(&2);

        // Hand starts at tail (key 2, visited=true) -> clear, advance to key 1
        // key 1 (visited=true) -> clear, advance -> hand=None -> wrap to tail
        // key 2 (visited=false after clear) -> EVICT
        cache.put(3, 30);

        assert!(cache.get(&2).is_none());         // key 2 evicted
        assert_eq!(*cache.get(&1).unwrap(), 10);  // key 1 preserved
        assert_eq!(*cache.get(&3).unwrap(), 30);
    }

    /// The hand retains its position across successive evictions.
    #[test]
    fn hand_persists_across_evictions() {
        let mut cache = LruCache::new(3);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30);
        cache.get(&3); // mark tail visited

        // Eviction 1: hand starts at tail (key 3, visited=true) -> clear, advance
        //             hand at key 2 (visited=false) -> evict key 2; hand saved at key 1
        cache.put(4, 40);
        assert!(cache.get(&2).is_none());

        // Eviction 2: hand resumes at key 1 (visited=false) -> evict key 1
        cache.put(5, 50);
        assert!(cache.get(&1).is_none());

        assert_eq!(*cache.get(&3).unwrap(), 30);
        assert_eq!(*cache.get(&4).unwrap(), 40);
        assert_eq!(*cache.get(&5).unwrap(), 50);
    }
}
