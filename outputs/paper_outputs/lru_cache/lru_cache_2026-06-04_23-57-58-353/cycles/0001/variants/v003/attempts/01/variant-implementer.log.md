# Agent log: variant-implementer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0001/variants/v003/attempts/01
- allow_edits: True
- started: 2026-06-04T21:00:39+00:00

## timeline

### 21:00:43 thinking

Let me start by reading the task.md file and then the relevant files mentioned in the task.

### 21:00:43 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0001/hypotheses/H03.md"
}
```

### 21:00:44 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/manifest.toml"
}
```

### 21:00:47 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src"
}
```

### 21:00:48 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/baseline/insight.md"
}
```

### 21:00:50 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src"
}
```

### 21:00:50 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/baseline/eval/fitness-vector.json"
}
```

### 21:00:54 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src/lru_cache/mod.rs"
}
```

### 21:00:54 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src/lib.rs"
}
```

### 21:01:01 thinking

Now I understand the full picture. I need to implement the hypothesis H03 which adds a small-window victim-selection (WLFU-like) to the LRU eviction:

1. Add `freq: u8` field to `Node<K, V>`
2. On `get` hit: increment freq (saturating, max 3)
3. On `put` existing key: increment freq after promoting
4. On `put` new key: set `freq: 0`
5. On eviction (arena full): scan window of 8 from LRU end, evict lowest-freq node


6. I should verify the invariants to ensure the frequency tracking and eviction logic maintain consistency throughout the cache operations.

### 21:01:01 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src/test_utils/invariants.rs"
}
```

### 21:01:01 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src/test_utils/mod.rs"
}
```

### 21:01:05 thinking

Now I have a complete picture. Let me also check the baseline overlay to see if there's anything there:

### 21:01:05 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/baseline/overlay"
}
```

### 21:01:14 thinking

The baseline overlay is empty, so I'm working directly from the source. Let me now create the modified `src/lru_cache/mod.rs` with the H03 changes:

1. Add `freq: u8` to `Node<K, V>`
2. On `get` hit: increment freq after `unlink + push_back`
3. On `put` existing key: increment freq after updating and promoting
4. On `put` new key: `freq: 0` in the new `Node`
5. On eviction: scan window of 8 from LRU end, pick minimum-freq victim


6. For the eviction logic, instead of always evicting the head node, I'll scan the first 8 entries from the LRU end to find the victim with the lowest frequency, then unlink it, update its key/value/freq, push it back to the end, and reinsert it into the map.

Writing the implementation...

### 21:01:14 chat

Now I have all the context I need. Let me create the overlay with the H03 changes — adding a `freq: u8` field to `Node` and a window-scan victim-selection at eviction time.

### 21:01:15 tool: Bash

```json
{
  "command": "mkdir -p /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0001/variants/v003/attempts/01/overlay/src/lru_cache",
  "description": "Create overlay directory structure"
}
```

### 21:02:22 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0001/variants/v003/attempts/01/overlay/src/lru_cache/mod.rs",
  "content": "// O(1) LRU cache using HashMap + arena-based doubly-linked list.\n// Nodes are stored in a Vec (arena) indexed by usize. On eviction the slot is reused in-place,\n// so no free-list management is needed. No unsafe, no external crates.\n//\n// H03: per-node saturating frequency counter + small-window victim selection.\n// At eviction time the first WINDOW nodes from the LRU end are scanned and the\n// one with the lowest frequency counter is chosen as the victim instead of\n// always evicting the strict LRU head.  This prevents frequently-accessed\n// entries from being displaced during cold-start bursts.\n\nuse std::collections::HashMap;\nuse std::fmt::Debug;\nuse std::hash::Hash;\n\npub mod lru_cache_jni;\n\n#[derive(Debug)]\nstruct Node<K, V> {\n\tkey: K,\n\tvalue: V,\n\tprev: Option<usize>,\n\tnext: Option<usize>,\n\t/// Saturating access-frequency counter; capped at 3. Reset to 0 on\n\t/// construction and whenever the slot is reused for a new key.\n\tfreq: u8,\n}\n\n#[derive(Debug)]\npub struct LruCache<K, V> {\n\tmap: HashMap<K, usize>, // key -> node index\n\tnodes: Vec<Node<K, V>>, // arena\n\thead: Option<usize>,    // LRU (front)\n\ttail: Option<usize>,    // MRU (back)\n\tcapacity: usize,\n}\n\n/// Number of LRU-end candidates examined during victim selection.\nconst WINDOW: usize = 8;\n\nimpl<K, V> LruCache<K, V>\nwhere\n\tK: Hash + Eq + Clone + Debug,\n\tV: Debug,\n{\n\tpub fn try_new(capacity: usize) -> Option<Self> {\n\t\tif capacity == 0 {\n\t\t\tlog::error!(\"LruCache.try_new: capacity must be > 0\");\n\t\t\treturn None;\n\t\t}\n\t\tlog::info!(\"LruCache.try_new: capacity={capacity}\");\n\t\tSome(Self {\n\t\t\tmap: HashMap::with_capacity(capacity),\n\t\t\tnodes: Vec::with_capacity(capacity),\n\t\t\thead: None,\n\t\t\ttail: None,\n\t\t\tcapacity,\n\t\t})\n\t}\n\n\tpub fn new(capacity: usize) -> Self {\n\t\tSelf::try_new(capacity).expect(\"LruCache::new: initialization failed\")\n\t}\n\n\t/// Returns the value for `key` if present, promoting it to most-recently-used.\n\tpub fn get(&mut self, key: &K) -> Option<&V> {\n\t\t// Look up the arena index for this key; return None on miss\n\t\tlet &idx = self.map.get(key)?;\n\t\t// Promote to MRU: detach from current position, re-attach at tail\n\t\tself.unlink(idx);\n\t\tself.push_back(idx);\n\t\t// Increment saturating frequency counter (cap 3)\n\t\tself.nodes[idx].freq = self.nodes[idx].freq.saturating_add(1).min(3);\n\t\tSome(&self.nodes[idx].value)\n\t}\n\n\t/// Inserts or updates `key`. If at capacity, evicts the least-frequently-used\n\t/// entry within a window of WINDOW candidates from the LRU end.\n\tpub fn put(&mut self, key: K, value: V) {\n\t\t// Key already exists: update value in-place and promote to MRU\n\t\tif let Some(&idx) = self.map.get(&key) {\n\t\t\tself.nodes[idx].value = value;\n\t\t\tself.unlink(idx);\n\t\t\tself.push_back(idx);\n\t\t\t// Increment saturating frequency counter (cap 3)\n\t\t\tself.nodes[idx].freq = self.nodes[idx].freq.saturating_add(1).min(3);\n\t\t\treturn;\n\t\t}\n\t\t// New key — need an arena slot\n\t\tlet idx = if self.nodes.len() >= self.capacity {\n\t\t\t// Arena full: scan up to WINDOW nodes from the LRU end and evict\n\t\t\t// the one with the lowest frequency counter.\n\t\t\tdebug_assert!(self.head.is_some(), \"head must be Some when arena is full\");\n\t\t\tlet mut victim = self.head.expect(\"non-empty list has a head\");\n\t\t\tlet mut cursor = victim;\n\t\t\tfor _ in 1..WINDOW {\n\t\t\t\tmatch self.nodes[cursor].next {\n\t\t\t\t\tSome(next_idx) => {\n\t\t\t\t\t\tif self.nodes[next_idx].freq < self.nodes[victim].freq {\n\t\t\t\t\t\t\tvictim = next_idx;\n\t\t\t\t\t\t}\n\t\t\t\t\t\tcursor = next_idx;\n\t\t\t\t\t}\n\t\t\t\t\tNone => break,\n\t\t\t\t}\n\t\t\t}\n\t\t\t// victim is the lowest-frequency node among the first WINDOW LRU-end entries\n\t\t\tlet evicted_key = self.nodes[victim].key.clone();\n\t\t\tself.map.remove(&evicted_key);\n\t\t\tself.unlink(victim);\n\t\t\tself.nodes[victim].key = key.clone();\n\t\t\tself.nodes[victim].value = value;\n\t\t\tself.nodes[victim].freq = 0;\n\t\t\tvictim\n\t\t} else {\n\t\t\t// Arena has room: allocate a new slot\n\t\t\tlet idx = self.nodes.len();\n\t\t\tself.nodes.push(Node {\n\t\t\t\tkey: key.clone(),\n\t\t\t\tvalue,\n\t\t\t\tprev: None,\n\t\t\t\tnext: None,\n\t\t\t\tfreq: 0,\n\t\t\t});\n\t\t\tidx\n\t\t};\n\t\t// Link the new/reused slot at the tail (MRU) and register in the map\n\t\tself.push_back(idx);\n\t\tself.map.insert(key, idx);\n\t\tdebug_assert!(self.map.len() <= self.capacity);\n\t}\n\n\t/// Remove node from its current position in the list.\n\tfn unlink(&mut self, idx: usize) {\n\t\tlet prev = self.nodes[idx].prev;\n\t\tlet next = self.nodes[idx].next;\n\t\tmatch prev {\n\t\t\tSome(p) => self.nodes[p].next = next,\n\t\t\tNone => self.head = next,\n\t\t}\n\t\tmatch next {\n\t\t\tSome(n) => self.nodes[n].prev = prev,\n\t\t\tNone => self.tail = prev,\n\t\t}\n\t\tself.nodes[idx].prev = None;\n\t\tself.nodes[idx].next = None;\n\t}\n\n\t/// Append node to the tail (MRU end) of the list.\n\tfn push_back(&mut self, idx: usize) {\n\t\tself.nodes[idx].prev = self.tail;\n\t\tself.nodes[idx].next = None;\n\t\tmatch self.tail {\n\t\t\tSome(t) => self.nodes[t].next = Some(idx),\n\t\t\tNone => self.head = Some(idx),\n\t\t}\n\t\tself.tail = Some(idx);\n\t}\n}\n\nimpl<K, V> framework::Cache<K, V> for LruCache<K, V>\nwhere\n\tK: Hash + Eq + Clone + Debug,\n\tV: Debug,\n{\n\tfn put(&mut self, key: K, value: V) {\n\t\tLruCache::put(self, key, value);\n\t}\n\n\tfn get(&mut self, key: &K) -> Option<&V> {\n\t\tLruCache::get(self, key)\n\t}\n}\n\n#[cfg(test)]\nmod tests {\n\tuse super::*;\n\n\t#[test]\n\tfn get_miss_returns_none() {\n\t\tlet mut cache = LruCache::<i64, i64>::new(4);\n\t\tassert!(cache.get(&42).is_none());\n\t}\n\n\t#[test]\n\tfn put_then_get() {\n\t\tlet mut cache = LruCache::new(4);\n\t\tcache.put(1, 10);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 10);\n\t}\n\n\t#[test]\n\tfn put_overwrites_existing_key() {\n\t\tlet mut cache = LruCache::new(4);\n\t\tcache.put(1, 10);\n\t\tcache.put(1, 20);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 20);\n\t}\n\n\t#[test]\n\tfn overwrite_does_not_grow_order() {\n\t\tlet mut cache = LruCache::new(2);\n\t\tcache.put(1, 10);\n\t\tcache.put(1, 20);\n\t\tcache.put(1, 30);\n\t\t// Should still have room for a second key without eviction\n\t\tcache.put(2, 40);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 30);\n\t\tassert_eq!(*cache.get(&2).unwrap(), 40);\n\t}\n\n\t#[test]\n\tfn eviction_removes_lru_when_all_freqs_equal() {\n\t\t// With equal frequencies the head (LRU) is still selected as victim\n\t\tlet mut cache = LruCache::new(2);\n\t\tcache.put(1, 10);\n\t\tcache.put(2, 20);\n\t\tcache.put(3, 30); // evicts key 1 (both have freq 0, head wins)\n\n\t\tassert!(cache.get(&1).is_none());\n\t\tassert_eq!(*cache.get(&2).unwrap(), 20);\n\t\tassert_eq!(*cache.get(&3).unwrap(), 30);\n\t}\n\n\t#[test]\n\tfn access_updates_lru_order() {\n\t\tlet mut cache = LruCache::new(2);\n\t\tcache.put(1, 10);\n\t\tcache.put(2, 20);\n\t\tcache.get(&1); // makes key 1 most recent; also bumps freq to 1\n\t\tcache.put(3, 30); // key 2 is at LRU end with freq 0; key 1 has freq 1 but is MRU\n\t\t\t\t\t\t  // window scan: only key 2 is at the LRU end (key 1 is tail), evicts key 2\n\n\t\tassert_eq!(*cache.get(&1).unwrap(), 10);\n\t\tassert!(cache.get(&2).is_none());\n\t\tassert_eq!(*cache.get(&3).unwrap(), 30);\n\t}\n\n\t#[test]\n\tfn put_updates_lru_order() {\n\t\tlet mut cache = LruCache::new(2);\n\t\tcache.put(1, 10);\n\t\tcache.put(2, 20);\n\t\tcache.put(1, 11); // update key 1 makes it MRU, bumps freq to 1\n\t\tcache.put(3, 30); // key 2 is LRU with freq 0; evicts key 2\n\n\t\tassert_eq!(*cache.get(&1).unwrap(), 11);\n\t\tassert!(cache.get(&2).is_none());\n\t\tassert_eq!(*cache.get(&3).unwrap(), 30);\n\t}\n\n\t#[test]\n\tfn eviction_chain() {\n\t\tlet mut cache = LruCache::new(3);\n\t\tcache.put(1, 10);\n\t\tcache.put(2, 20);\n\t\tcache.put(3, 30);\n\t\tcache.put(4, 40); // all freq 0; head (key 1) is evicted\n\t\tcache.put(5, 50); // all freq 0; head (key 2) is evicted\n\t\tcache.put(6, 60); // all freq 0; head (key 3) is evicted\n\n\t\tassert!(cache.get(&1).is_none());\n\t\tassert!(cache.get(&2).is_none());\n\t\tassert!(cache.get(&3).is_none());\n\t\tassert_eq!(*cache.get(&4).unwrap(), 40);\n\t\tassert_eq!(*cache.get(&5).unwrap(), 50);\n\t\tassert_eq!(*cache.get(&6).unwrap(), 60);\n\t}\n\n\t#[test]\n\tfn capacity_one() {\n\t\tlet mut cache = LruCache::new(1);\n\t\tcache.put(1, 10);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 10);\n\t\tcache.put(2, 20); // evicts 1\n\t\tassert!(cache.get(&1).is_none());\n\t\tassert_eq!(*cache.get(&2).unwrap(), 20);\n\t}\n\n\t#[test]\n\tfn overwrite_at_full_capacity_does_not_evict() {\n\t\tlet mut cache = LruCache::new(2);\n\t\tcache.put(1, 10);\n\t\tcache.put(2, 20); // cache full\n\t\tcache.put(1, 11); // overwrite, must NOT evict key 2\n\t\tassert_eq!(*cache.get(&1).unwrap(), 11);\n\t\tassert_eq!(*cache.get(&2).unwrap(), 20);\n\t}\n\n\t#[test]\n\tfn repeated_get_preserves_position_as_mru() {\n\t\t// Key 1 gets freq 3 (saturated) and is MRU; key 2 is LRU with freq 0.\n\t\t// Inserting key 4 must evict key 2 (lowest freq at LRU end), not key 1.\n\t\tlet mut cache = LruCache::new(3);\n\t\tcache.put(1, 10);\n\t\tcache.put(2, 20);\n\t\tcache.put(3, 30);\n\t\tfor _ in 0..10 {\n\t\t\tcache.get(&1); // saturates freq at 3; keeps promoting 1 to MRU\n\t\t}\n\t\t// Order at this point: 2 (LRU, freq 0), 3 (freq 0), 1 (MRU, freq 3)\n\t\tcache.put(4, 40); // window scans 2 and 3 (and 1 if WINDOW large enough);\n\t\t\t\t\t\t  // key 2 has freq 0 and is at head — evict key 2\n\t\tassert_eq!(*cache.get(&1).unwrap(), 10);\n\t\tassert!(cache.get(&2).is_none());\n\t}\n\n\t#[test]\n\tfn get_middle_node_promotes_correctly() {\n\t\tlet mut cache = LruCache::new(3);\n\t\tcache.put(1, 10); // LRU\n\t\tcache.put(2, 20); // middle\n\t\tcache.put(3, 30); // MRU\n\t\tcache.get(&2); // unlink middle, push to MRU; bumps freq to 1\n\t\t\t\t\t\t // Order: 1 (LRU, freq 0), 3 (freq 0), 2 (MRU, freq 1)\n\t\tcache.put(4, 40); // window: 1 has freq 0, 3 has freq 0 — head (1) wins ties; evicts 1\n\t\tassert!(cache.get(&1).is_none());\n\t\tassert_eq!(*cache.get(&3).unwrap(), 30);\n\t\tassert_eq!(*cache.get(&2).unwrap(), 20);\n\t}\n\n\t#[test]\n\tfn slot_reuse_after_promotion_and_re_eviction() {\n\t\tlet mut cache = LruCache::new(2);\n\t\tcache.put(1, 10); // slot 0, freq 0\n\t\tcache.put(2, 20); // slot 1, freq 0\n\t\tcache.put(3, 30); // evicts 1 (head, freq 0) -> slot 0 reused for key 3, freq 0\n\t\tcache.get(&3);    // promote slot 0 (key 3) to MRU; freq -> 1\n\t\tcache.put(4, 40); // key 2 is head (freq 0), key 3 is tail (freq 1); evicts key 2 (slot 1)\n\t\tcache.put(5, 50); // key 3 is head (freq 1), key 4 is tail (freq 0);\n\t\t\t\t\t\t  // window: key 3 freq=1, key 4 freq=0 — evicts key 4 (lowest freq)\n\t\tassert!(cache.get(&4).is_none());\n\t\tassert_eq!(*cache.get(&3).unwrap(), 30);\n\t\tassert_eq!(*cache.get(&5).unwrap(), 50);\n\t}\n\n\t#[test]\n\tfn get_only_element() {\n\t\tlet mut cache = LruCache::new(3);\n\t\tcache.put(1, 10);\n\t\t// List has one node; get unlinks then re-pushes it\n\t\tassert_eq!(*cache.get(&1).unwrap(), 10);\n\t\t// Verify list is still intact\n\t\tcache.put(2, 20);\n\t\tcache.put(3, 30);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 10);\n\t}\n\n\t#[test]\n\tfn overwrite_mru_at_capacity() {\n\t\tlet mut cache = LruCache::new(3);\n\t\tcache.put(1, 10);\n\t\tcache.put(2, 20);\n\t\tcache.put(3, 30); // full; 3 is MRU (tail), freq 0\n\t\tcache.put(3, 31); // overwrite MRU; freq bumped to 1\n\t\t\t\t\t\t  // Eviction should take 1 (LRU, freq 0) — lowest freq in window\n\t\tcache.put(4, 40); // evicts 1\n\t\tassert!(cache.get(&1).is_none());\n\t\tassert_eq!(*cache.get(&2).unwrap(), 20);\n\t\tassert_eq!(*cache.get(&3).unwrap(), 31);\n\t\tassert_eq!(*cache.get(&4).unwrap(), 40);\n\t}\n\n\t#[test]\n\t#[should_panic(expected = \"initialization failed\")]\n\tfn capacity_zero_panics() {\n\t\tLruCache::<i64, i64>::new(0);\n\t}\n\n\t#[test]\n\tfn freq_protected_entry_survives_cold_burst() {\n\t\t// Hot key 1 is accessed repeatedly; cold keys 2..=9 flood the cache.\n\t\t// With capacity 4 and WINDOW 8, key 1 (freq=3) should survive all evictions\n\t\t// as long as there are lower-freq candidates in the window.\n\t\tlet mut cache = LruCache::new(4);\n\t\tcache.put(1, 10);\n\t\tfor _ in 0..5 {\n\t\t\tcache.get(&1); // freq saturates at 3\n\t\t}\n\t\t// Fill remaining 3 slots\n\t\tcache.put(2, 20);\n\t\tcache.put(3, 30);\n\t\tcache.put(4, 40);\n\t\t// Now evict with cold keys; key 1 has freq=3, others have freq=0\n\t\tcache.put(5, 50); // evicts lowest-freq in window; key 1 protected\n\t\tcache.put(6, 60);\n\t\tcache.put(7, 70);\n\t\t// Key 1 should still be resident\n\t\tassert_eq!(*cache.get(&1).unwrap(), 10);\n\t}\n}\n"
}
```

### 21:02:25 thinking

Let me verify the file looks correct by reading it back:

### 21:02:25 chat

Let me verify the overlay file is correct and consistent with what the hypothesis requires:

### 21:02:26 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0001/variants/v003/attempts/01/overlay/src/lru_cache/mod.rs"
}
```

### 21:02:58 thinking

The overlay looks good. Let me trace through the key logic to make sure it's correct:

1. **`freq: u8` field** — added to `Node<K, V>` ✓
2. **`get` hit** — after `unlink + push_back`, increments freq with `saturating_add(1).min(3)` ✓
3. **`put` existing key** — after updating value + unlink + push_back, increments freq ✓
4. **`put` new key (arena not full)** — `freq: 0` in new Node ✓


5. **`put` new key (arena full, eviction)** — window scan from head, picks min-freq victim, reuses slot with `freq: 0` ✓

Now checking the invariant gates: when the arena fills and we need to evict, exactly one victim gets removed per insertion, keeping the map size at or below capacity. The victim's value gets overwritten before the new key is pushed back, and the map correctly maps the new key to the reused slot while leaving all other values intact.

For the edge case where capacity is 1, the window scan loop immediately breaks since there's only one node, so the head becomes the victim—that's correct. Now I'm tracing through the `slot_reuse_after_promotion_and_re_eviction` test to verify the slot reuse behavior works as expected across multiple puts and evictions. Tracing through the H03 cache behavior with specific slot assignments and frequency tracking as operations proceed. Verifying the final cache state: k3 remains at slot 0 with frequency 1, k4 got evicted, and k5 is now at slot 1 with frequency 0. I'm checking my test assertions against this expected behavior, but I also need to reconcile this with the baseline test which expects different eviction results.

The key insight is that H03's frequency-based eviction prioritizes k3 over k4 because k3 has a higher access frequency, so the test correctly reflects this behavioral difference. Now I'm validating whether the proptest invariants still hold: the capacity bound is maintained since we evict exactly one slot per insertion, and value fidelity is preserved because each key is only inserted once with a consistent value. After the insertion, the map length returns to capacity, so the assertion holds. I'm satisfied the implementation is correct and complete.

### 21:03:06 chat

The overlay is complete. Here's a summary of exactly what was implemented for H03:

**File written:** `overlay/src/lru_cache/mod.rs`

**Changes applied (faithful to H03 `## Changes`):**

1. **`freq: u8` added to `Node<K, V>`** — saturating counter, capped at 3, reset to 0 on construction.

2. **`get` hit** — after the existing `unlink + push_back`, increments `freq` with `saturating_add(1).min(3)`.

3. **`put` existing key** — after updating value and promoting, applies the same saturating increment to `freq`.

4. **`put` new key (arena has room)** — `Node { ..., freq: 0 }` on construction.

5. **Eviction path (arena full)** — replaces the direct `head_idx` grab with a WINDOW=8 scan from the LRU end, tracking the candidate with the minimum `freq`. The lowest-frequency node in the window becomes the victim; on tie the strict LRU head (the first node) wins by default since the scan only updates `victim` on strict `<`.

**Invariants preserved:**
- **P1 capacity_bound**: exactly one node is evicted per new-key insert; `map.len() ≤ capacity` always holds.
- **P2 value_fidelity**: victim slot is fully overwritten (key, value, freq=0) before being re-linked; no other slot's value is touched.
- **P3 no_phantom_items**: only `map.remove` + `map.insert` alter the keyspace; never-put keys can never appear.

The unit tests are updated to reflect H03's different eviction ordering (frequency-aware victim selection means a hot entry at the LRU end survives over cold neighbours), including the `slot_reuse_after_promotion_and_re_eviction` test which now correctly expects key 4 (freq=0) to be evicted rather than key 3 (freq=1).


---

- finished: 2026-06-04T21:03:06+00:00
- duration: 147.15s
- status: ok
- input_tokens: 11
- output_tokens: 8923
- cache_read_tokens: 203412
- cache_write_tokens: 22211
- cost_usd: 0.279088
