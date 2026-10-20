# Agent log: variant-implementer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0009\variants\v003\attempts\01
- allow_edits: True
- started: 2026-07-21T07:44:28+00:00

## timeline

### 07:44:33 thinking

Let me start by reading the task.md file and understanding what I need to do. The task is already provided in the prompt, so let me read the hypothesis file and other relevant files.

Let me gather all the necessary information:
1. Read the hypothesis file
2. Read the subject manifest
3. Read the parent variant's overlay
4. Read the parent's fitness-vector and insight
5. Read the subject source files

### 07:44:34 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0009\\hypotheses\\H03.md"
}
```

### 07:44:34 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\lru_cache\\manifest.toml"
}
```

### 07:44:40 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0006\\variants\\v001\\attempts\\01\\overlay"
}
```

### 07:44:41 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0006\\variants\\v001\\eval\\fitness-vector.json"
}
```

### 07:44:41 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0006\\variants\\v001\\insight.md"
}
```

### 07:44:49 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0006\\variants\\v001\\attempts\\01\\overlay"
}
```

### 07:44:50 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\lru_cache\\src"
}
```

### 07:44:52 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0006\\variants\\v001\\attempts\\01\\overlay\\src\\lru_cache\\mod.rs"
}
```

### 07:44:57 thinking

Now I have all the information I need. I need to:

1. Start with the parent variant's `mod.rs` (from cycle/0006/v001)
2. Apply the three changes from the hypothesis:
   - Change `prob_cap = (capacity / 8).max(1)` to `prob_cap = (capacity / 16).max(1)`
   - Add a custom FNV-1a hasher and builder
   - Change the `map` field type to use `FnvBuildHasher` instead of the default hasher

Let me create the modified file:

### 07:44:58 chat

Now I have all the information I need. Let me create the overlay file by applying the three changes from the hypothesis to the parent variant's `mod.rs`:

1. Change `prob_cap = (capacity / 8).max(1)` → `prob_cap = (capacity / 16).max(1)`
2. Add FNV-1a hasher and builder
3. Change `map` field type to use `FnvBuildHasher`

### 07:45:00 tool: Bash

```json
{
  "command": "mkdir -p \"C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0009\\variants\\v003\\attempts\\01\\overlay\\src\\lru_cache\"",
  "description": "Create overlay directory structure"
}
```

### 07:46:30 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0009\\variants\\v003\\attempts\\01\\overlay\\src\\lru_cache\\mod.rs",
  "content": "// O(1) two-queue (2Q) cache using HashMap + arena-based doubly-linked list.\n// Extends v002's u32-sentinel node layout with a probationary FIFO queue and a\n// protected LRU queue.\n//\n// H02: prev/next link fields are moved out of Node<K,V> into a dedicated\n// Vec<Link>, shrinking Node<u64,u64> from 24 to 16 bytes and packing 8 link\n// pairs per 64-byte cache line on the hot traversal paths.\n//\n// H01 (cycle/0006): probationary capacity is reduced from capacity/4 to\n// capacity/8, raising the protected quota from 75% to 87.5% of total capacity.\n// This reduces demotion pressure and keeps moderately-frequent Zipfian items\n// resident longer, testing whether the hit-rate improvement seen in cycle/0005/v003\n// was genuine when measured against a clean (non-zero p50) baseline.\n//\n// H03 (cycle/0009): probationary capacity is further reduced from capacity/8 to\n// capacity/16, raising the protected quota to 93.75%. A custom FNV-1a hasher\n// replaces SipHash-1-3 to reduce per-map-lookup latency and offset the added\n// demotion overhead at this tighter quota.\n//\n// Sentinel / bit constants:\n//   NULL     = 0x7FFF_FFFF  (no valid index; bit 31 clear so PROT_BIT is free)\n//   PROT_BIT = 0x8000_0000  (bit 31 of prev: node is in the protected queue)\n//   IDX_MASK = 0x7FFF_FFFF  (strip PROT_BIT to get the raw index from prev)\n//\n// Policy sketch:\n//   • New entries enter the probationary queue (FIFO, evicted oldest-first).\n//   • A probationary hit graduates the entry to the protected queue (LRU).\n//   • A protected hit refreshes the entry to the MRU end of protected.\n//   • When protected exceeds its quota (capacity - prob_cap), the LRU of\n//     protected is demoted to the MRU end of probationary.\n//   • Eviction always drains probationary first; protected is a fallback.\n\nuse std::collections::HashMap;\nuse std::fmt::Debug;\nuse std::hash::Hash;\n\npub mod lru_cache_jni;\n\n/// Null sentinel for u32 link fields.\n/// Bit 31 is clear so that `PROT_BIT` (bit 31 of `prev`) is distinguishable\n/// from any valid index embedded in `prev`.\nconst NULL: u32 = 0x7FFF_FFFF;\n\n/// Bit 31 of the `prev` field: set when the node lives in the protected queue.\nconst PROT_BIT: u32 = 0x8000_0000;\n\n/// Mask to extract the raw index from a `prev` field (strips `PROT_BIT`).\nconst IDX_MASK: u32 = 0x7FFF_FFFF;\n\n/// FNV-1a hasher: processes 8 xor+multiply steps for an 8-byte i64 key,\n/// avoiding SipHash-1-3's more expensive mixing rounds.\nstruct FnvHasher(u64);\n\nimpl std::hash::Hasher for FnvHasher {\n    #[inline(always)]\n    fn finish(&self) -> u64 { self.0 }\n    #[inline(always)]\n    fn write(&mut self, bytes: &[u8]) {\n        for &b in bytes {\n            self.0 ^= b as u64;\n            self.0 = self.0.wrapping_mul(0x00000100000001b3);\n        }\n    }\n}\n\n#[derive(Clone, Default)]\nstruct FnvBuildHasher;\n\nimpl std::hash::BuildHasher for FnvBuildHasher {\n    type Hasher = FnvHasher;\n    #[inline(always)]\n    fn build_hasher(&self) -> FnvHasher { FnvHasher(0xcbf29ce484222325) }\n}\n\n/// Per-slot doubly-linked-list pointers, stored separately from key/value data.\n/// 8 bytes each; 8 entries fit in a 64-byte cache line.\n#[derive(Debug)]\nstruct Link {\n    /// bit 31 = PROT_BIT; bits 0-30 = prev index (or NULL = 0x7FFF_FFFF)\n    prev: u32,\n    next: u32,\n}\n\n#[derive(Debug)]\npub struct Node<K, V> {\n    key: K,\n    value: V,\n}\n\n#[derive(Debug)]\npub struct LruCache<K, V> {\n    map: HashMap<K, usize, FnvBuildHasher>,  // key -> node index\n    nodes: Vec<Node<K, V>>,  // arena\n    links: Vec<Link>,        // parallel to nodes; links[i] holds prev/next for nodes[i]\n    /// LRU end of probationary queue (eviction candidate); NULL if empty.\n    prob_head: u32,\n    /// MRU end of probationary queue; NULL if empty.\n    prob_tail: u32,\n    /// LRU end of protected queue; NULL if empty.\n    prot_head: u32,\n    /// MRU end of protected queue; NULL if empty.\n    prot_tail: u32,\n    prob_len: usize,\n    prot_len: usize,\n    /// Probationary capacity = (capacity / 16).max(1).\n    prob_cap: usize,\n    capacity: usize,\n}\n\nimpl<K, V> LruCache<K, V>\nwhere\n    K: Hash + Eq + Clone + Debug,\n    V: Debug,\n{\n    pub fn try_new(capacity: usize) -> Option<Self> {\n        if capacity == 0 {\n            log::error!(\"LruCache.try_new: capacity must be > 0\");\n            return None;\n        }\n        // Valid indices run 0 ..= 0x7FFF_FFFE; NULL = 0x7FFF_FFFF is the sentinel.\n        if capacity > 0x7FFF_FFFEusize {\n            log::error!(\"LruCache.try_new: capacity exceeds u32 sentinel range\");\n            return None;\n        }\n        log::info!(\"LruCache.try_new: capacity={capacity}\");\n        let prob_cap = (capacity / 16).max(1);\n        Some(Self {\n            map: HashMap::with_capacity_and_hasher(capacity, FnvBuildHasher),\n            nodes: Vec::with_capacity(capacity),\n            links: Vec::with_capacity(capacity),\n            prob_head: NULL,\n            prob_tail: NULL,\n            prot_head: NULL,\n            prot_tail: NULL,\n            prob_len: 0,\n            prot_len: 0,\n            prob_cap,\n            capacity,\n        })\n    }\n\n    pub fn new(capacity: usize) -> Self {\n        Self::try_new(capacity).expect(\"LruCache::new: initialization failed\")\n    }\n\n    /// Returns the value for `key` if present, promoting it through the 2Q queues.\n    pub fn get(&mut self, key: &K) -> Option<&V> {\n        let &idx = self.map.get(key)?;\n        self.promote(idx);\n        Some(&self.nodes[idx].value)\n    }\n\n    /// Inserts or updates `key`. If at capacity, evicts according to the 2Q policy.\n    pub fn put(&mut self, key: K, value: V) {\n        // Existing key: update value in-place and promote through the 2Q queues.\n        if let Some(&idx) = self.map.get(&key) {\n            self.nodes[idx].value = value;\n            self.promote(idx);\n            return;\n        }\n        // New key — need an arena slot.\n        let idx = if self.nodes.len() >= self.capacity {\n            // Arena full: evict via 2Q policy and reuse slot.\n            let slot = self.evict_one();\n            self.nodes[slot].key = key.clone();\n            self.nodes[slot].value = value;\n            slot\n        } else {\n            // Arena has room: allocate a new slot.\n            let new_idx = self.nodes.len();\n            self.nodes.push(Node {\n                key: key.clone(),\n                value,\n            });\n            self.links.push(Link { prev: NULL, next: NULL });\n            new_idx\n        };\n        // All new insertions enter the probationary queue at its MRU end.\n        self.push_back_prob(idx);\n        self.prob_len += 1;\n        self.map.insert(key, idx);\n        debug_assert!(self.map.len() <= self.capacity);\n        debug_assert_eq!(self.prob_len + self.prot_len, self.map.len());\n    }\n\n    /// Promote a node: refresh in protected (if already protected), or graduate\n    /// from probationary to protected. Enforces the protected-queue budget.\n    fn promote(&mut self, idx: usize) {\n        let raw_prev = self.links[idx].prev;\n        let protected = (raw_prev & PROT_BIT) != 0;\n        if protected {\n            // Protected hit: refresh to the MRU end of the protected queue.\n            self.unlink(idx);\n            self.push_back_prot(idx);\n            // prot_len is unchanged (item stays in protected).\n        } else {\n            // Probationary hit: graduate to the protected queue.\n            self.unlink(idx);\n            self.prob_len -= 1;\n            self.push_back_prot(idx);\n            self.prot_len += 1;\n            // Enforce the protected budget: if protected exceeds its quota,\n            // demote the LRU of protected back to the MRU of probationary.\n            let prot_cap = self.capacity - self.prob_cap;\n            if self.prot_len > prot_cap {\n                let h = (self.prot_head & IDX_MASK) as usize;\n                self.unlink(h);\n                self.prot_len -= 1;\n                self.push_back_prob(h);\n                self.prob_len += 1;\n            }\n        }\n    }\n\n    /// Evict one entry via the 2Q policy (probationary first, protected as\n    /// fallback), remove it from the map, and return the freed slot index.\n    fn evict_one(&mut self) -> usize {\n        if self.prob_len > 0 {\n            let h = (self.prob_head & IDX_MASK) as usize;\n            let evicted_key = self.nodes[h].key.clone();\n            self.map.remove(&evicted_key);\n            self.unlink(h);\n            self.prob_len -= 1;\n            h\n        } else {\n            debug_assert!(self.prot_len > 0, \"no items to evict\");\n            let h = (self.prot_head & IDX_MASK) as usize;\n            let evicted_key = self.nodes[h].key.clone();\n            self.map.remove(&evicted_key);\n            self.unlink(h);\n            self.prot_len -= 1;\n            h\n        }\n    }\n\n    /// Remove a node from its queue's linked list.\n    /// Dispatches on PROT_BIT in `prev` to update the correct head/tail pointer.\n    fn unlink(&mut self, idx: usize) {\n        let raw_prev = self.links[idx].prev;\n        let protected = (raw_prev & PROT_BIT) != 0;\n        let prev_idx = raw_prev & IDX_MASK; // raw index (or NULL sentinel)\n        let next = self.links[idx].next;\n\n        // Fix the forward link of the predecessor (or update the queue head).\n        if prev_idx != NULL {\n            self.links[prev_idx as usize].next = next;\n        } else if protected {\n            self.prot_head = next;\n        } else {\n            self.prob_head = next;\n        }\n\n        // Fix the backward link of the successor (or update the queue tail).\n        if next != NULL {\n            // Preserve the successor's own PROT_BIT; replace only its index part.\n            let succ_prot_bit = self.links[next as usize].prev & PROT_BIT;\n            self.links[next as usize].prev = succ_prot_bit | prev_idx;\n        } else if protected {\n            self.prot_tail = prev_idx;\n        } else {\n            self.prob_tail = prev_idx;\n        }\n\n        // Clear the unlinked node's link fields (NULL has PROT_BIT = 0).\n        self.links[idx].prev = NULL;\n        self.links[idx].next = NULL;\n    }\n\n    /// Append a node to the MRU end of the probationary queue.\n    /// Clears PROT_BIT in `prev` to mark the node as probationary.\n    fn push_back_prob(&mut self, idx: usize) {\n        let idx32 = idx as u32;\n        // prob_tail is a raw index (no PROT_BIT), so assigning it directly\n        // to prev leaves PROT_BIT = 0 (probationary).\n        self.links[idx].prev = self.prob_tail;\n        self.links[idx].next = NULL;\n        if self.prob_tail != NULL {\n            self.links[self.prob_tail as usize].next = idx32;\n        } else {\n            self.prob_head = idx32;\n        }\n        self.prob_tail = idx32;\n    }\n\n    /// Append a node to the MRU end of the protected queue.\n    /// Sets PROT_BIT in `prev` to mark the node as protected.\n    fn push_back_prot(&mut self, idx: usize) {\n        let idx32 = idx as u32;\n        // OR-in PROT_BIT so the node is marked as belonging to the protected queue.\n        self.links[idx].prev = PROT_BIT | self.prot_tail;\n        self.links[idx].next = NULL;\n        if self.prot_tail != NULL {\n            self.links[self.prot_tail as usize].next = idx32;\n        } else {\n            self.prot_head = idx32;\n        }\n        self.prot_tail = idx32;\n    }\n}\n\nimpl<K, V> framework::Cache<K, V> for LruCache<K, V>\nwhere\n    K: Hash + Eq + Clone + Debug,\n    V: Debug,\n{\n    fn put(&mut self, key: K, value: V) {\n        LruCache::put(self, key, value);\n    }\n\n    fn get(&mut self, key: &K) -> Option<&V> {\n        LruCache::get(self, key)\n    }\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn get_miss_returns_none() {\n        let mut cache = LruCache::<i64, i64>::new(4);\n        assert!(cache.get(&42).is_none());\n    }\n\n    #[test]\n    fn put_then_get() {\n        let mut cache = LruCache::new(4);\n        cache.put(1, 10);\n        assert_eq!(*cache.get(&1).unwrap(), 10);\n    }\n\n    #[test]\n    fn put_overwrites_existing_key() {\n        let mut cache = LruCache::new(4);\n        cache.put(1, 10);\n        cache.put(1, 20);\n        assert_eq!(*cache.get(&1).unwrap(), 20);\n    }\n\n    #[test]\n    fn overwrite_does_not_grow_order() {\n        let mut cache = LruCache::new(2);\n        cache.put(1, 10);\n        cache.put(1, 20);\n        cache.put(1, 30);\n        // Should still have room for a second key without eviction\n        cache.put(2, 40);\n        assert_eq!(*cache.get(&1).unwrap(), 30);\n        assert_eq!(*cache.get(&2).unwrap(), 40);\n    }\n\n    #[test]\n    fn eviction_removes_lru() {\n        // All three entries start in prob; eviction takes the LRU of prob (key 1).\n        let mut cache = LruCache::new(2);\n        cache.put(1, 10);\n        cache.put(2, 20);\n        cache.put(3, 30); // evicts key 1 (prob head)\n\n        assert!(cache.get(&1).is_none());\n        assert_eq!(*cache.get(&2).unwrap(), 20);\n        assert_eq!(*cache.get(&3).unwrap(), 30);\n    }\n\n    #[test]\n    fn access_updates_eviction_order() {\n        // get(1) graduates key 1 to protected; put(3) evicts key 2 from prob.\n        let mut cache = LruCache::new(2);\n        cache.put(1, 10);\n        cache.put(2, 20);\n        cache.get(&1); // promotes key 1 from prob → protected\n        cache.put(3, 30); // evicts key 2 (only item left in prob)\n\n        assert_eq!(*cache.get(&1).unwrap(), 10); // key 1 still lives in protected\n        assert!(cache.get(&2).is_none());         // key 2 was evicted\n        assert_eq!(*cache.get(&3).unwrap(), 30);\n    }\n\n    #[test]\n    fn put_updates_eviction_order() {\n        // put(1,11) on an existing key promotes key 1 to protected;\n        // put(3) then evicts key 2 from prob.\n        let mut cache = LruCache::new(2);\n        cache.put(1, 10);\n        cache.put(2, 20);\n        cache.put(1, 11); // update key 1 → promoted to protected\n        cache.put(3, 30); // evicts key 2 (prob head)\n\n        assert_eq!(*cache.get(&1).unwrap(), 11);\n        assert!(cache.get(&2).is_none());\n        assert_eq!(*cache.get(&3).unwrap(), 30);\n    }\n\n    #[test]\n    fn eviction_chain() {\n        let mut cache = LruCache::new(3);\n        cache.put(1, 10);\n        cache.put(2, 20);\n        cache.put(3, 30);\n        cache.put(4, 40); // evicts 1\n        cache.put(5, 50); // evicts 2\n        cache.put(6, 60); // evicts 3\n\n        assert!(cache.get(&1).is_none());\n        assert!(cache.get(&2).is_none());\n        assert!(cache.get(&3).is_none());\n        assert_eq!(*cache.get(&4).unwrap(), 40);\n        assert_eq!(*cache.get(&5).unwrap(), 50);\n        assert_eq!(*cache.get(&6).unwrap(), 60);\n    }\n\n    #[test]\n    fn capacity_one() {\n        let mut cache = LruCache::new(1);\n        cache.put(1, 10);\n        assert_eq!(*cache.get(&1).unwrap(), 10);\n        cache.put(2, 20); // evicts 1\n        assert!(cache.get(&1).is_none());\n        assert_eq!(*cache.get(&2).unwrap(), 20);\n    }\n\n    #[test]\n    fn overwrite_at_full_capacity_does_not_evict() {\n        let mut cache = LruCache::new(2);\n        cache.put(1, 10);\n        cache.put(2, 20); // cache full\n        cache.put(1, 11); // overwrite, must NOT evict key 2\n        assert_eq!(*cache.get(&1).unwrap(), 11);\n        assert_eq!(*cache.get(&2).unwrap(), 20);\n    }\n\n    #[test]\n    fn repeated_get_keeps_item_from_eviction() {\n        // Repeated hits on key 1 graduate it to protected, shielding it from eviction.\n        let mut cache = LruCache::new(3);\n        cache.put(1, 10);\n        cache.put(2, 20);\n        cache.put(3, 30);\n        for _ in 0..10 {\n            cache.get(&1);\n        }\n        // Eviction drains prob (keys 2 and 3 are there); key 1 is protected.\n        cache.put(4, 40); // should evict key 2 (prob LRU), not key 1\n        assert_eq!(*cache.get(&1).unwrap(), 10);\n        assert!(cache.get(&2).is_none());\n    }\n\n    #[test]\n    fn get_middle_node_promotes_correctly() {\n        let mut cache = LruCache::new(3);\n        cache.put(1, 10); // prob LRU\n        cache.put(2, 20); // prob middle\n        cache.put(3, 30); // prob MRU\n        cache.get(&2);    // graduate key 2 to protected\n        cache.put(4, 40); // evicts key 1 (prob LRU)\n        assert!(cache.get(&1).is_none());\n        assert_eq!(*cache.get(&3).unwrap(), 30);\n        assert_eq!(*cache.get(&2).unwrap(), 20);\n    }\n\n    #[test]\n    fn slot_reuse_with_2q_protection() {\n        // With 2Q (capacity=2, prob_cap=1, prot_cap=1):\n        //   key 3 is promoted to protected by get(&3);\n        //   subsequent puts evict from prob (key 2, then key 4), not from protected.\n        let mut cache = LruCache::new(2);\n        cache.put(1, 10); // slot 0\n        cache.put(2, 20); // slot 1\n        cache.put(3, 30); // evicts 1 → slot 0 reused for key 3\n        cache.get(&3);    // promote slot 0 (key 3) to protected\n        cache.put(4, 40); // evicts 2 (prob) → slot 1 reused for key 4\n        cache.put(5, 50); // evicts 4 (prob) → slot 1 reused for key 5; key 3 stays protected\n        assert!(cache.get(&1).is_none());\n        assert!(cache.get(&2).is_none());\n        assert!(cache.get(&4).is_none());        // 4 was in prob and got evicted\n        assert_eq!(*cache.get(&3).unwrap(), 30); // 3 is still protected\n        assert_eq!(*cache.get(&5).unwrap(), 50);\n    }\n\n    #[test]\n    fn get_only_element() {\n        let mut cache = LruCache::new(3);\n        cache.put(1, 10);\n        // Single node: get promotes it to protected (no budget overflow since prot_cap=2).\n        assert_eq!(*cache.get(&1).unwrap(), 10);\n        // Verify list is still intact after promotion.\n        cache.put(2, 20);\n        cache.put(3, 30);\n        assert_eq!(*cache.get(&1).unwrap(), 10);\n    }\n\n    #[test]\n    fn overwrite_mru_at_capacity() {\n        let mut cache = LruCache::new(3);\n        cache.put(1, 10);\n        cache.put(2, 20);\n        cache.put(3, 30); // full; 3 is MRU (tail of prob)\n        cache.put(3, 31); // overwrite MRU → promotes key 3 to protected\n                          // prob: {key1, key2}; prot: {key3}\n        cache.put(4, 40); // evicts key 1 (prob LRU)\n        assert!(cache.get(&1).is_none());\n        assert_eq!(*cache.get(&2).unwrap(), 20);\n        assert_eq!(*cache.get(&3).unwrap(), 31);\n        assert_eq!(*cache.get(&4).unwrap(), 40);\n    }\n\n    #[test]\n    #[should_panic(expected = \"initialization failed\")]\n    fn capacity_zero_panics() {\n        LruCache::<i64, i64>::new(0);\n    }\n}\n"
}
```

### 07:46:39 chat

The overlay file has been written. Here's a summary of the three changes applied to the parent variant (cycle/0006/v001):

1. **`prob_cap` reduced** from `(capacity / 8).max(1)` to `(capacity / 16).max(1)` — this expands the protected quota from 87.5% to 93.75% of total capacity, further reducing demotion pressure on frequently-accessed Zipfian items.

2. **FNV-1a hasher added** — `FnvHasher` and `FnvBuildHasher` are defined before the `LruCache` struct. The hasher uses 8 sequential XOR+multiply steps per byte (FNV-1a), which is significantly cheaper than SipHash-1-3's two full mixing rounds for 8-byte `i64` keys.

3. **`map` field type changed** — `HashMap<K, usize>` → `HashMap<K, usize, FnvBuildHasher>`, with construction changed from `HashMap::with_capacity(capacity)` to `HashMap::with_capacity_and_hasher(capacity, FnvBuildHasher)`.

All other logic (linking, promotion, eviction, the `Cache<K, V>` trait impl, and all unit tests) is unchanged from the parent variant. The `K: Hash` bound is preserved and equality semantics are unaffected since FNV-1a is deterministic.


---

- finished: 2026-07-21T07:46:40+00:00
- duration: 132.29s
- status: ok
- input_tokens: 8
- output_tokens: 8047
- cache_read_tokens: 148685
- cache_write_tokens: 20480
- cost_usd: 0.243227
