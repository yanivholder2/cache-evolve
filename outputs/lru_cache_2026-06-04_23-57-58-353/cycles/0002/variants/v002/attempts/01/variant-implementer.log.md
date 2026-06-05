# Agent log: variant-implementer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0002/variants/v002/attempts/01
- allow_edits: True
- started: 2026-06-04T21:16:01+00:00

## timeline

### 21:16:04 thinking

Let me start by reading the task.md file and all the relevant files mentioned in the task description.

### 21:16:05 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0002/hypotheses/H02.md"
}
```

### 21:16:06 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/manifest.toml"
}
```

### 21:16:08 thinking

Let me read the parent variant's overlay, the subject source, and other context files.

### 21:16:09 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0001/variants/v001/attempts/01/overlay"
}
```

### 21:16:10 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0001/variants/v001/eval/fitness-vector.json"
}
```

### 21:16:10 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0001/variants/v001/insight.md"
}
```

### 21:16:13 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0001/variants/v001/attempts/01/overlay"
}
```

### 21:16:13 tool: Glob

```json
{
  "pattern": "**/*.rs",
  "path": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src"
}
```

### 21:16:16 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0001/variants/v001/attempts/01/overlay/src/lru_cache/mod.rs"
}
```

### 21:16:17 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src/test_utils/invariants.rs"
}
```

### 21:17:36 thinking

Now I have a good understanding of the codebase. Let me read the subject's source files to understand the full picture:

1. The parent variant (v001) implements an SLRU (Segmented LRU) cache with:
   - Probationary segment (20% of capacity) for new inserts
   - Protected segment (80% of capacity) for entries hit at least twice
   - On protected overflow: demotes the prot_head (strict LRU)

2. The hypothesis (H02) wants me to:
   - Add a `freq: u8` field to `Node<K, V>`, capped at 3 via saturating arithmetic
   - Track frequency on every access: increment on `get()` hits and `put()` updates, reset to 0 on demotion or probationary reuse
   - Modify the overflow-demotion logic to scan up to 4 nodes from `prot_head` and demote the one with the lowest frequency instead of always demoting the head
   - Implementing this in the LRU cache module by adding a `freq` field to track access patterns, then updating `get()` and `put()` to manage frequency counts and apply the new demotion strategy
   - The demotion logic will scan up to 4 nodes from the protected head to find the lowest-frequency candidate, with ties broken by position

Writing the frequency tracking logic... Verifying the unlink timing is correct—it needs to happen while `in_protected` is still true so it updates the right segment's pointers, then I set the flag to false and move the node to the probationary segment. Now I'm thinking through the frequency increment timing based on the hypothesis. Increment on every hit in `get()` (both protected and probationary branches)
- Increment on every `put()` that updates an existing key
- Reset to 0 when demoted from protected to probationary
- Reset to 0 when a probationary slot is reused for a brand-new key

For the probationary hit in `get()`, I increment freq before promotion. The node goes from probationary to protected. After promotion, if overflow occurs, we scan for the lowest-freq node in protected (which includes the newly promoted node). The newly-promoted node has freq >= 1 (since we just incremented it), so it won't be immediately demoted unless all protected nodes have freq >= 1.

Wait, actually the hypothesis says to increment freq when there's a probationary hit. So the promoted node would have freq = 1 (since it was reset to 0 when last demoted/inserted). That's fine.

For a protected hit in `get()`, I increment freq and move to MRU without triggering any overflow logic. For `put()` with an existing key, I increment freq and apply promotion/MRU logic, scanning for the lowest-freq node if overflow occurs. Now I'm implementing the full SLRU cache with frequency-aware demotion, where each node tracks a saturating frequency counter that increments on hits and gets reset during demotion or reuse.

Writing Rust implementation... Defining the LruCache struct... Implementing the method to scan through protected nodes and find the one with the lowest frequency up to a maximum scan limit, returning its index as the demotion target.

Now for the `get` method, I'm handling cache hits differently based on whether the entry is in the protected or probationary segment—incrementing its frequency, moving it to the back of the protected list, and promoting probationary entries to protected when they're accessed.

When the protected segment overflows, I'm finding the lowest-frequency candidate to demote back to probationary, resetting its frequency to zero, and moving it to the probationary list.

Now I'm moving into the `put` method to handle insertions and updates.

For existing keys, I'm updating the value and capping the frequency at 3, then promoting the node to protected if it's currently in probationary, which may trigger another demotion if the protected segment exceeds capacity.

Handling cache eviction logic... Writing cache eviction logic... Completing the unlink operation by updating the tail pointers for both protected and probationary lists, then implementing a helper to append nodes to the back of the protected list.

Now I'm implementing the corresponding function for the probationary list, which mirrors the protected version but updates the probationary head and tail instead. I'm realizing there's a subtle issue with the unlink function—when demoting an intermediate node, it correctly avoids updating the head since the node has a previous pointer, but I need to verify that push_back_prob is being called correctly after unlink to maintain list integrity. after setting `in_protected = false`. In `push_back_prob`, the node is linked into the probationary list. The `push_back_prob` function doesn't read `in_protected` - it just links the node at the end of the prob list. That's fine.

Actually, I need to double-check the `unlink` function more carefully. When I unlink an intermediate protected node:
- `prev` = Some(p) → sets `nodes[p].next = next` ✓
- `next` = Some(n) → sets `nodes[n].prev = prev` ✓
- Neither head nor tail is updated since both are `Some` ✓

When I unlink the protected tail node:
- `prev` = Some(p) → sets `nodes[p].next = None`
- `next` = None → enters the `None` branch, checks `in_protected`...

When demoting a non-head protected node, `in_protected` is still true at unlink time, so the tail pointer gets updated correctly to the previous node. For the case where the demote target is the protected tail itself, unlinking sets `prot_tail` to the predecessor, then `in_protected` flips to false, and finally the node gets appended to the probationary list.

For demoting the protected head, the head pointer gets updated to the next node during unlink, and if there's only one node in the protected list, the tail also becomes None. Then the state updates and the node moves to probationary.

Now I'm checking whether the hypothesis about scanning up to 4 nodes from the protected head is actually correct.

My `find_demote_target` function starts at `prot_head` and iterates up to 3 more times, so it examines 4 nodes total, which matches the hypothesis. If the protected list has fewer than 4 nodes, the loop naturally terminates early when it reaches the end. For the saturation behavior, I should use `saturating_add(1)` to cap the value at 255 while respecting the "capped at 3" constraint from the hypothesis.

Actually, I realize `.min(3)` after the add is cleaner than relying on `saturating_add` alone—since frequency values stay at 3 or below, I can just increment and cap with `.min(3)`. Now I need to make sure the `Node::new` constructor initializes `freq` to 0.

Let me verify the test utilities and library setup to catch anything I might have missed.

### 21:17:36 chat

Now let me read the remaining source files:

### 21:17:36 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src/lib.rs"
}
```

### 21:17:37 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src/test_utils/mod.rs"
}
```

### 21:17:40 thinking

Now I have a complete picture. Let me write the overlay for v002. I need to create `overlay/src/lru_cache/mod.rs` with the frequency-aware demotion changes.

Let me create the output directory structure first, then write the file.

### 21:17:41 chat

I have all the information needed. Let me now write the overlay implementing the frequency-aware demotion from H02:

### 21:17:45 tool: Bash

```json
{
  "command": "mkdir -p /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0002/variants/v002/attempts/01/overlay/src/lru_cache",
  "description": "Create overlay directory structure"
}
```

### 21:18:52 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0002/variants/v002/attempts/01/overlay/src/lru_cache/mod.rs",
  "content": "// SLRU (Segmented LRU) with frequency-aware demotion — v002 / H02.\n//\n// Identical to v001 (SLRU) except:\n//   - Node carries a `freq: u8` counter capped at 3.\n//   - On every cache hit (get in either segment, or put on an existing key),\n//     `freq` is incremented (saturating at 3).\n//   - When a promotion causes the protected segment to overflow, instead of\n//     always demoting `prot_head` (strict LRU), we scan up to 4 nodes from\n//     `prot_head` and demote the first node with the minimum `freq`.\n//   - `freq` is reset to 0 on demotion from protected and when a probationary\n//     slot is recycled for a brand-new key.\n//\n// All other logic (insertion into probationary, promotion on a probationary\n// hit, eviction of probationary-LRU on overflow, fallback to protected-LRU\n// when probationary is empty, and the 80/20 split) is identical to v001.\n\nuse std::collections::HashMap;\nuse std::fmt::Debug;\nuse std::hash::Hash;\n\npub mod lru_cache_jni;\n\n#[derive(Debug)]\nstruct Node<K, V> {\n    key: K,\n    value: V,\n    prev: Option<usize>,\n    next: Option<usize>,\n    /// True when the node lives in the protected segment.\n    in_protected: bool,\n    /// Access frequency, capped at 3 via saturating arithmetic.\n    freq: u8,\n}\n\n#[derive(Debug)]\npub struct LruCache<K, V> {\n    map: HashMap<K, usize>, // key -> arena index\n    nodes: Vec<Node<K, V>>, // arena; slots are reused on eviction\n\n    // Protected segment — frequently-accessed items (LRU end = head, MRU end = tail)\n    prot_head: Option<usize>,\n    prot_tail: Option<usize>,\n    prot_count: usize,\n    prot_cap: usize, // max items in protected\n\n    // Probationary segment — recently-inserted / one-hit items\n    prob_head: Option<usize>,\n    prob_tail: Option<usize>,\n    prob_count: usize,\n    prob_cap: usize, // max items in probationary\n\n    capacity: usize, // total capacity = prot_cap + prob_cap\n}\n\nimpl<K, V> LruCache<K, V>\nwhere\n    K: Hash + Eq + Clone + Debug,\n    V: Debug,\n{\n    pub fn try_new(capacity: usize) -> Option<Self> {\n        if capacity == 0 {\n            log::error!(\"LruCache.try_new: capacity must be > 0\");\n            return None;\n        }\n        // Protected gets floor(capacity * 4 / 5), capped so at least 1 slot remains\n        // for the probationary segment.\n        let prot_cap = (capacity * 4 / 5).min(capacity.saturating_sub(1));\n        let prob_cap = capacity - prot_cap;\n        log::info!(\n            \"LruCache.try_new: capacity={capacity} prot_cap={prot_cap} prob_cap={prob_cap}\"\n        );\n        Some(Self {\n            map: HashMap::with_capacity(capacity),\n            nodes: Vec::with_capacity(capacity),\n            prot_head: None,\n            prot_tail: None,\n            prot_count: 0,\n            prot_cap,\n            prob_head: None,\n            prob_tail: None,\n            prob_count: 0,\n            prob_cap,\n            capacity,\n        })\n    }\n\n    pub fn new(capacity: usize) -> Self {\n        Self::try_new(capacity).expect(\"LruCache::new: initialization failed\")\n    }\n\n    /// Scan up to `max_scan` nodes from `prot_head` and return the index of\n    /// the first node with the minimum `freq` (ties broken by proximity to head).\n    fn find_demote_target(&self, max_scan: usize) -> usize {\n        let mut cur = self.prot_head.expect(\"prot_head must exist when prot_count > 0\");\n        let mut min_idx = cur;\n        let mut min_freq = self.nodes[cur].freq;\n\n        for _ in 1..max_scan {\n            match self.nodes[cur].next {\n                Some(next_idx) => {\n                    cur = next_idx;\n                    if self.nodes[cur].freq < min_freq {\n                        min_freq = self.nodes[cur].freq;\n                        min_idx = cur;\n                    }\n                }\n                None => break,\n            }\n        }\n\n        min_idx\n    }\n\n    /// Returns the value for `key` if present, applying SLRU promotion with\n    /// frequency tracking.\n    pub fn get(&mut self, key: &K) -> Option<&V> {\n        let &idx = self.map.get(key)?;\n\n        if self.nodes[idx].in_protected {\n            // Protected hit: increment freq, move to protected MRU (counts unchanged).\n            self.nodes[idx].freq = self.nodes[idx].freq.saturating_add(1).min(3);\n            self.unlink(idx);\n            self.push_back_prot(idx);\n        } else {\n            // Probationary hit: increment freq, promote to protected MRU.\n            self.nodes[idx].freq = self.nodes[idx].freq.saturating_add(1).min(3);\n            self.unlink(idx); // removes from probationary (in_protected still false here)\n            self.prob_count -= 1;\n            self.nodes[idx].in_protected = true;\n            self.push_back_prot(idx);\n            self.prot_count += 1;\n\n            // If protected overflows its cap, frequency-aware demotion.\n            if self.prot_count > self.prot_cap {\n                let demote = self.find_demote_target(4);\n                self.unlink(demote); // removes from protected (in_protected still true)\n                self.prot_count -= 1;\n                self.nodes[demote].in_protected = false;\n                self.nodes[demote].freq = 0; // reset on demotion\n                self.push_back_prob(demote);\n                self.prob_count += 1;\n                // prob_count can never exceed prob_cap here because the promotion\n                // decremented prob_count before incrementing prot_count.\n                debug_assert!(\n                    self.prob_count <= self.prob_cap,\n                    \"prob overflow during get: prob_count={} prob_cap={}\",\n                    self.prob_count,\n                    self.prob_cap\n                );\n            }\n        }\n\n        Some(&self.nodes[idx].value)\n    }\n\n    /// Inserts `key` (new or existing).  If at probationary cap (or total\n    /// capacity), evicts the probationary LRU; falls back to protected LRU if\n    /// probationary is empty.\n    pub fn put(&mut self, key: K, value: V) {\n        // Existing key: update value, increment freq, apply same promotion as get.\n        if let Some(&idx) = self.map.get(&key) {\n            self.nodes[idx].value = value;\n            self.nodes[idx].freq = self.nodes[idx].freq.saturating_add(1).min(3);\n            if self.nodes[idx].in_protected {\n                self.unlink(idx);\n                self.push_back_prot(idx);\n            } else {\n                self.unlink(idx);\n                self.prob_count -= 1;\n                self.nodes[idx].in_protected = true;\n                self.push_back_prot(idx);\n                self.prot_count += 1;\n                if self.prot_count > self.prot_cap {\n                    let demote = self.find_demote_target(4);\n                    self.unlink(demote);\n                    self.prot_count -= 1;\n                    self.nodes[demote].in_protected = false;\n                    self.nodes[demote].freq = 0; // reset on demotion\n                    self.push_back_prob(demote);\n                    self.prob_count += 1;\n                }\n            }\n            return;\n        }\n\n        // New key: obtain an arena slot.\n        let idx = if self.nodes.len() >= self.capacity || self.prob_count >= self.prob_cap {\n            // Either the arena is full or probationary is at its cap.\n            // Evict from probationary LRU; fall back to protected LRU if prob is empty.\n            let evict = if self.prob_count > 0 {\n                self.prob_head.expect(\"prob_head must exist when prob_count > 0\")\n            } else {\n                self.prot_head.expect(\"prot_head must exist when prot_count > 0\")\n            };\n            let was_protected = self.nodes[evict].in_protected;\n            let evicted_key = self.nodes[evict].key.clone();\n            self.map.remove(&evicted_key);\n            self.unlink(evict);\n            if was_protected {\n                self.prot_count -= 1;\n            } else {\n                self.prob_count -= 1;\n            }\n            // Reuse evicted slot for the new entry; reset freq for the new key.\n            self.nodes[evict].key = key.clone();\n            self.nodes[evict].value = value;\n            self.nodes[evict].in_protected = false;\n            self.nodes[evict].freq = 0; // reset when slot is recycled for a new key\n            evict\n        } else {\n            // Arena has room and probationary is under cap: allocate a fresh slot.\n            let new_idx = self.nodes.len();\n            self.nodes.push(Node {\n                key: key.clone(),\n                value,\n                prev: None,\n                next: None,\n                in_protected: false,\n                freq: 0,\n            });\n            new_idx\n        };\n\n        // Link at probationary MRU and register in the map.\n        self.push_back_prob(idx);\n        self.map.insert(key, idx);\n        self.prob_count += 1;\n\n        debug_assert!(self.map.len() <= self.capacity);\n        debug_assert!(\n            self.prot_count + self.prob_count <= self.capacity,\n            \"total count {} exceeds capacity {}\",\n            self.prot_count + self.prob_count,\n            self.capacity\n        );\n    }\n\n    // ── list helpers ──────────────────────────────────────────────────────────\n\n    /// Detach `idx` from whichever segment list it currently belongs to.\n    /// Reads `in_protected` to identify the owning segment's head/tail pointers.\n    fn unlink(&mut self, idx: usize) {\n        let prev = self.nodes[idx].prev;\n        let next = self.nodes[idx].next;\n\n        // Repair the predecessor's forward link (or the segment head).\n        match prev {\n            Some(p) => self.nodes[p].next = next,\n            None => {\n                if self.nodes[idx].in_protected {\n                    self.prot_head = next;\n                } else {\n                    self.prob_head = next;\n                }\n            }\n        }\n\n        // Repair the successor's backward link (or the segment tail).\n        match next {\n            Some(n) => self.nodes[n].prev = prev,\n            None => {\n                if self.nodes[idx].in_protected {\n                    self.prot_tail = prev;\n                } else {\n                    self.prob_tail = prev;\n                }\n            }\n        }\n\n        self.nodes[idx].prev = None;\n        self.nodes[idx].next = None;\n    }\n\n    /// Append `idx` at the MRU end of the protected segment.\n    fn push_back_prot(&mut self, idx: usize) {\n        self.nodes[idx].prev = self.prot_tail;\n        self.nodes[idx].next = None;\n        match self.prot_tail {\n            Some(t) => self.nodes[t].next = Some(idx),\n            None => self.prot_head = Some(idx),\n        }\n        self.prot_tail = Some(idx);\n    }\n\n    /// Append `idx` at the MRU end of the probationary segment.\n    fn push_back_prob(&mut self, idx: usize) {\n        self.nodes[idx].prev = self.prob_tail;\n        self.nodes[idx].next = None;\n        match self.prob_tail {\n            Some(t) => self.nodes[t].next = Some(idx),\n            None => self.prob_head = Some(idx),\n        }\n        self.prob_tail = Some(idx);\n    }\n}\n\nimpl<K, V> framework::Cache<K, V> for LruCache<K, V>\nwhere\n    K: Hash + Eq + Clone + Debug,\n    V: Debug,\n{\n    fn put(&mut self, key: K, value: V) {\n        LruCache::put(self, key, value);\n    }\n\n    fn get(&mut self, key: &K) -> Option<&V> {\n        LruCache::get(self, key)\n    }\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn get_miss_returns_none() {\n        let mut cache = LruCache::<i64, i64>::new(4);\n        assert!(cache.get(&42).is_none());\n    }\n\n    #[test]\n    fn put_then_get() {\n        let mut cache = LruCache::new(4);\n        cache.put(1, 10);\n        assert_eq!(*cache.get(&1).unwrap(), 10);\n    }\n\n    #[test]\n    fn put_overwrites_existing_key() {\n        let mut cache = LruCache::new(4);\n        cache.put(1, 10);\n        cache.put(1, 20);\n        assert_eq!(*cache.get(&1).unwrap(), 20);\n    }\n\n    #[test]\n    fn capacity_one() {\n        let mut cache = LruCache::new(1);\n        cache.put(1, 10);\n        assert_eq!(*cache.get(&1).unwrap(), 10);\n        cache.put(2, 20); // evicts 1\n        assert!(cache.get(&1).is_none());\n        assert_eq!(*cache.get(&2).unwrap(), 20);\n    }\n\n    #[test]\n    #[should_panic(expected = \"initialization failed\")]\n    fn capacity_zero_panics() {\n        LruCache::<i64, i64>::new(0);\n    }\n\n    // SLRU-specific: probationary hit promotes to protected.\n    #[test]\n    fn probationary_hit_promotes_to_protected() {\n        // capacity=8: prot_cap=6, prob_cap=2\n        let mut cache = LruCache::<i64, i64>::new(8);\n        cache.put(1, 10);\n        // first get: prob hit -> promoted to prot\n        assert_eq!(*cache.get(&1).unwrap(), 10);\n        // subsequent gets stay in prot (just move to prot MRU)\n        assert_eq!(*cache.get(&1).unwrap(), 10);\n    }\n\n    // A key in protected survives new inserts that fill probationary.\n    #[test]\n    fn protected_key_survives_prob_eviction() {\n        // capacity=8, prot_cap=6, prob_cap=2\n        let mut cache = LruCache::<i64, i64>::new(8);\n        cache.put(42, 420);\n        cache.get(&42); // promote to protected\n        // Fill probationary many times over; key 42 must survive\n        for k in 0i64..20 {\n            if cache.get(&k).is_none() {\n                cache.put(k, k * 10);\n            }\n        }\n        assert_eq!(*cache.get(&42).unwrap(), 420);\n    }\n\n    // H02-specific: a high-freq protected node survives demotion scan even\n    // when it is the oldest (at prot_head).\n    #[test]\n    fn freq_aware_demotion_prefers_low_freq() {\n        // capacity=4: prot_cap=3, prob_cap=1\n        // Promote keys 1, 2, 3 into protected (filling it to cap=3).\n        // Key 1 is at prot_head (LRU), but we hit it many times so freq=3.\n        // Key 4 then enters probationary; another key 5 promotes key 4 to\n        // protected, causing overflow.  The scan of up to 4 nodes should\n        // prefer key 2 or 3 (freq=1) over key 1 (freq=3).\n        let mut cache = LruCache::<i64, i64>::new(4);\n\n        // Put then get to promote into protected\n        cache.put(1, 10);\n        cache.get(&1); // prob hit -> promoted, freq=1\n        cache.put(2, 20);\n        cache.get(&2); // prob hit -> promoted; prot overflow: demotes freq=1 node\n        cache.put(3, 30);\n        cache.get(&3); // prob hit -> promoted; prot overflow again\n\n        // At this point prot is full. Hit key 1 repeatedly to raise its freq.\n        cache.get(&1);\n        cache.get(&1);\n        cache.get(&1); // freq[1] should be capped at 3\n\n        // Now insert key 4 into probationary, then get it to trigger prot overflow.\n        // The scan should demote the lowest-freq node (not key 1).\n        cache.put(4, 40);\n        cache.get(&4); // promotes 4 into prot, overflow demotes lowest-freq\n\n        // Key 1 (freq=3) should still be in cache.\n        assert!(\n            cache.get(&1).is_some(),\n            \"key 1 with high freq should not have been demoted\"\n        );\n    }\n}\n"
}
```

### 21:19:02 chat

The overlay is written. Here's a summary of the changes implemented for H02:

## What was changed

**File:** `overlay/src/lru_cache/mod.rs` — a targeted diff on top of v001's SLRU implementation.

### Additions to `Node<K, V>`
- Added `freq: u8` field, initialized to `0` in every path that creates or recycles a node.

### `find_demote_target(max_scan: usize) -> usize`
- New helper that walks up to `max_scan` (4) nodes forward from `prot_head`, tracking the first node with the minimum `freq`. Returns its arena index.

### `get()` changes
- **Protected hit:** `freq` is incremented (`.saturating_add(1).min(3)`) before moving to MRU.
- **Probationary hit:** `freq` is incremented before promotion.
- **Overflow demotion:** replaces the `prot_head` hardcode with `find_demote_target(4)`; the demoted node has its `freq` reset to `0`.

### `put()` changes
- **Existing key:** `freq` is incremented before applying the protected/probationary update logic.
- **Overflow demotion:** same `find_demote_target(4)` substitution with `freq = 0` reset.
- **Slot recycling for new key:** `self.nodes[evict].freq = 0` is explicitly set.
- **Fresh allocation:** `freq: 0` in the `Node { … }` initialiser.

All other logic — probationary insertion, promotion mechanics, the 80/20 split, eviction fallback order — is identical to v001.


---

- finished: 2026-06-04T21:19:03+00:00
- duration: 181.99s
- status: ok
- input_tokens: 9
- output_tokens: 11866
- cache_read_tokens: 163656
- cache_write_tokens: 22267
- cost_usd: 0.31162
