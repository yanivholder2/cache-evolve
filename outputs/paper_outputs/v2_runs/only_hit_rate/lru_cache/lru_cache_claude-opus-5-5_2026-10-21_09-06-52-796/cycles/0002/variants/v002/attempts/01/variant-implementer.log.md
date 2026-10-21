# Agent log: variant-implementer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0002/variants/v002/attempts/01
- allow_edits: True
- started: 2026-10-21T09:14:13+00:00

## timeline

### 09:14:17 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796; cat cycles/0002/hypotheses/H02.md; echo ----; cat /home/user/cache-evolve/inputs/subjects/lru_cache/manifest.toml; echo ----; find cycles/0001/variants/v001/attempts/01/overlay -type f; cat cycles/0001/variants/v001/insight.md cycles/0001/variants/v001/eval/fitness-vector.json; ls -R /home/user/cache-evolve/inputs/subjects/lru_cache/src"
}
```

### 09:14:18 thinking



### 09:14:19 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0002/hypotheses/H02.md"
}
```

### 09:14:19 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/lru_cache/manifest.toml"
}
```

### 09:14:20 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0001/variants/v001/insight.md"
}
```

### 09:14:20 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0001/variants/v001/attempts/01/overlay"
}
```

### 09:14:21 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0001/variants/v001/attempts/01/overlay/src/lru_cache/mod.rs"
}
```

### 09:14:29 thinking



### 09:14:30 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0001/variants/v001/attempts/01/overlay/src/factory.rs"
}
```

### 09:14:30 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/lru_cache/src/test_utils/mod.rs"
}
```

### 09:14:31 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/lru_cache/src/test_utils/invariants.rs"
}
```

### 09:14:35 thinking



### 09:14:35 chat

I'm writing the v002 overlay now. It changes v001's S3-FIFO `mod.rs` to add a main-queue ghost and an adaptive `small_target`.

### 09:15:15 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0002/variants/v002/attempts/01/overlay/src/lru_cache/mod.rs",
  "content": "// S3-FIFO cache (small probation FIFO + main FIFO with lazy reinsertion + ghost queues)\n// with an ARC-style adaptive small-queue target driven by two ghosts:\n//   - small ghost hit  -> probation was too short: grow small_target\n//   - main ghost hit   -> main was too small:      shrink small_target\n// Nodes live in a Vec arena indexed by u32; evicted slots are reused in place.\n// Public surface (LruCache, Node, try_new, new, get, put, framework::Cache impl) is\n// unchanged so the JNI layer and bins keep working. No unsafe, no external crates.\n\nuse std::collections::{HashMap, VecDeque};\nuse std::fmt::Debug;\nuse std::hash::Hash;\n\npub mod lru_cache_jni;\n\nconst MAX_FREQ: u8 = 3;\n\n#[derive(Debug)]\npub struct Node<K, V> {\n\tkey: K,\n\tvalue: V,\n\tfreq: u8,\n\t#[allow(dead_code)] // read by tests / Debug; queue membership is authoritative\n\tin_main: bool,\n}\n\n#[derive(Debug)]\npub struct LruCache<K, V> {\n\tmap: HashMap<K, u32>,   // key -> arena index\n\tnodes: Vec<Node<K, V>>, // arena\n\tsmall: VecDeque<u32>,   // probation FIFO (front = oldest)\n\tmain: VecDeque<u32>,    // main FIFO (front = oldest)\n\tghost_q: VecDeque<K>,   // recently evicted-from-small keys, FIFO\n\tghost_set: HashMap<K, ()>,\n\tmain_ghost_q: VecDeque<K>, // recently evicted-from-main keys, FIFO\n\tmain_ghost_set: HashMap<K, ()>,\n\tsmall_target: usize, // adaptive, within [min_small, max_small]\n\tmin_small: usize,\n\tmax_small: usize,\n\tghost_cap: usize,\n\tmain_ghost_cap: usize,\n\tcapacity: usize,\n}\n\nimpl<K, V> LruCache<K, V>\nwhere\n\tK: Hash + Eq + Clone + Debug,\n\tV: Debug,\n{\n\tpub fn try_new(capacity: usize) -> Option<Self> {\n\t\tif capacity == 0 {\n\t\t\tlog::error!(\"LruCache.try_new: capacity must be > 0\");\n\t\t\treturn None;\n\t\t}\n\t\tlet initial_small = std::cmp::max(1, capacity / 10);\n\t\tlet min_small = std::cmp::max(1, capacity / 100);\n\t\tlet max_small = std::cmp::max(min_small, capacity / 2);\n\t\tlet small_target = initial_small.clamp(min_small, max_small);\n\t\t// Small ghost sizing is unchanged from v001 (based on the initial target).\n\t\tlet ghost_cap = std::cmp::max(1, capacity - initial_small);\n\t\tlet main_ghost_cap = std::cmp::max(1, capacity / 2);\n\t\tlog::info!(\n\t\t\t\"LruCache.try_new: capacity={capacity} small_target={small_target} \\\n\t\t\t bounds=[{min_small},{max_small}] ghost_cap={ghost_cap} main_ghost_cap={main_ghost_cap}\"\n\t\t);\n\t\tSome(Self {\n\t\t\tmap: HashMap::with_capacity(capacity),\n\t\t\tnodes: Vec::with_capacity(capacity),\n\t\t\tsmall: VecDeque::with_capacity(capacity),\n\t\t\tmain: VecDeque::with_capacity(capacity),\n\t\t\tghost_q: VecDeque::with_capacity(ghost_cap + 1),\n\t\t\tghost_set: HashMap::with_capacity(ghost_cap + 1),\n\t\t\tmain_ghost_q: VecDeque::with_capacity(main_ghost_cap + 1),\n\t\t\tmain_ghost_set: HashMap::with_capacity(main_ghost_cap + 1),\n\t\t\tsmall_target,\n\t\t\tmin_small,\n\t\t\tmax_small,\n\t\t\tghost_cap,\n\t\t\tmain_ghost_cap,\n\t\t\tcapacity,\n\t\t})\n\t}\n\n\tpub fn new(capacity: usize) -> Self {\n\t\tSelf::try_new(capacity).expect(\"LruCache::new: initialization failed\")\n\t}\n\n\t/// Returns the value for `key` if present, bumping its (saturating) access frequency.\n\tpub fn get(&mut self, key: &K) -> Option<&V> {\n\t\tlet idx = *self.map.get(key)? as usize;\n\t\tlet node = &mut self.nodes[idx];\n\t\tif node.freq < MAX_FREQ {\n\t\t\tnode.freq += 1;\n\t\t}\n\t\tSome(&node.value)\n\t}\n\n\t/// Inserts or updates `key`. New keys enter `small`, or `main` on a ghost hit.\n\t/// Ghost hits adapt `small_target` ARC-style.\n\tpub fn put(&mut self, key: K, value: V) {\n\t\tif let Some(&idx) = self.map.get(&key) {\n\t\t\tlet node = &mut self.nodes[idx as usize];\n\t\t\tnode.value = value;\n\t\t\tif node.freq < MAX_FREQ {\n\t\t\t\tnode.freq += 1;\n\t\t\t}\n\t\t\treturn;\n\t\t}\n\n\t\tlet small_ghost_len = self.ghost_q.len();\n\t\tlet main_ghost_len = self.main_ghost_q.len();\n\n\t\tlet to_main = if self.ghost_set.remove(&key).is_some() {\n\t\t\t// Evicted from probation too early: grow the probation queue.\n\t\t\tlet delta = std::cmp::max(1, main_ghost_len / std::cmp::max(1, small_ghost_len));\n\t\t\tself.small_target = std::cmp::min(self.max_small, self.small_target.saturating_add(delta));\n\t\t\ttrue\n\t\t} else if self.main_ghost_set.remove(&key).is_some() {\n\t\t\t// Main was too small: shrink the probation queue.\n\t\t\tlet delta = std::cmp::max(1, small_ghost_len / std::cmp::max(1, main_ghost_len));\n\t\t\tself.small_target = std::cmp::max(self.min_small, self.small_target.saturating_sub(delta));\n\t\t\ttrue\n\t\t} else {\n\t\t\tfalse\n\t\t};\n\t\t// Stale entries in the ghost FIFO deques are tolerated: on pop, the set\n\t\t// removal is a no-op unless the key was re-ghosted, which is benign.\n\n\t\tlet idx = if self.map.len() >= self.capacity {\n\t\t\tlet slot = self.evict_one();\n\t\t\tlet node = &mut self.nodes[slot as usize];\n\t\t\tnode.key = key.clone();\n\t\t\tnode.value = value;\n\t\t\tnode.freq = 0;\n\t\t\tnode.in_main = to_main;\n\t\t\tslot\n\t\t} else {\n\t\t\tlet slot = self.nodes.len() as u32;\n\t\t\tself.nodes.push(Node {\n\t\t\t\tkey: key.clone(),\n\t\t\t\tvalue,\n\t\t\t\tfreq: 0,\n\t\t\t\tin_main: to_main,\n\t\t\t});\n\t\t\tslot\n\t\t};\n\n\t\tif to_main {\n\t\t\tself.main.push_back(idx);\n\t\t} else {\n\t\t\tself.small.push_back(idx);\n\t\t}\n\t\tself.map.insert(key, idx);\n\t\tdebug_assert!(self.map.len() <= self.capacity);\n\t}\n\n\t/// Evicts exactly one resident entry and returns its (now free) arena slot.\n\t/// Terminates: every iteration either lowers some node's freq or frees a slot.\n\tfn evict_one(&mut self) -> u32 {\n\t\tloop {\n\t\t\tif self.small.len() >= self.small_target || self.main.is_empty() {\n\t\t\t\tlet idx = self.small.pop_front().expect(\"small must be non-empty when main is empty and cache is full\");\n\t\t\t\tlet node = &mut self.nodes[idx as usize];\n\t\t\t\tif node.freq > 0 {\n\t\t\t\t\tnode.freq = 0;\n\t\t\t\t\tnode.in_main = true;\n\t\t\t\t\tself.main.push_back(idx);\n\t\t\t\t\tcontinue;\n\t\t\t\t}\n\t\t\t\tlet evicted = node.key.clone();\n\t\t\t\tself.map.remove(&evicted);\n\t\t\t\tself.push_ghost(evicted);\n\t\t\t\treturn idx;\n\t\t\t} else {\n\t\t\t\tlet idx = self.main.pop_front().expect(\"main checked non-empty\");\n\t\t\t\tlet node = &mut self.nodes[idx as usize];\n\t\t\t\tif node.freq > 0 {\n\t\t\t\t\tnode.freq -= 1;\n\t\t\t\t\tself.main.push_back(idx);\n\t\t\t\t\tcontinue;\n\t\t\t\t}\n\t\t\t\tlet evicted = node.key.clone();\n\t\t\t\tself.map.remove(&evicted);\n\t\t\t\tself.push_main_ghost(evicted);\n\t\t\t\treturn idx;\n\t\t\t}\n\t\t}\n\t}\n\n\tfn push_ghost(&mut self, key: K) {\n\t\tself.ghost_set.insert(key.clone(), ());\n\t\tself.ghost_q.push_back(key);\n\t\twhile self.ghost_q.len() > self.ghost_cap {\n\t\t\tif let Some(old) = self.ghost_q.pop_front() {\n\t\t\t\tself.ghost_set.remove(&old);\n\t\t\t}\n\t\t}\n\t}\n\n\tfn push_main_ghost(&mut self, key: K) {\n\t\tself.main_ghost_set.insert(key.clone(), ());\n\t\tself.main_ghost_q.push_back(key);\n\t\twhile self.main_ghost_q.len() > self.main_ghost_cap {\n\t\t\tif let Some(old) = self.main_ghost_q.pop_front() {\n\t\t\t\tself.main_ghost_set.remove(&old);\n\t\t\t}\n\t\t}\n\t}\n}\n\nimpl<K, V> framework::Cache<K, V> for LruCache<K, V>\nwhere\n\tK: Hash + Eq + Clone + Debug,\n\tV: Debug,\n{\n\tfn put(&mut self, key: K, value: V) {\n\t\tLruCache::put(self, key, value);\n\t}\n\n\tfn get(&mut self, key: &K) -> Option<&V> {\n\t\tLruCache::get(self, key)\n\t}\n}\n\n#[cfg(test)]\nmod tests {\n\tuse super::*;\n\n\tfn in_main(cache: &LruCache<i64, i64>, key: i64) -> bool {\n\t\tlet idx = cache.map[&key] as usize;\n\t\tcache.nodes[idx].in_main\n\t}\n\n\t#[test]\n\tfn get_miss_returns_none() {\n\t\tlet mut cache = LruCache::<i64, i64>::new(4);\n\t\tassert!(cache.get(&42).is_none());\n\t}\n\n\t#[test]\n\tfn put_then_get() {\n\t\tlet mut cache = LruCache::new(4);\n\t\tcache.put(1, 10);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 10);\n\t}\n\n\t#[test]\n\tfn put_overwrites_existing_key() {\n\t\tlet mut cache = LruCache::new(4);\n\t\tcache.put(1, 10);\n\t\tcache.put(1, 20);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 20);\n\t}\n\n\t#[test]\n\tfn overwrite_does_not_grow_order() {\n\t\tlet mut cache = LruCache::new(2);\n\t\tcache.put(1, 10);\n\t\tcache.put(1, 20);\n\t\tcache.put(1, 30);\n\t\tcache.put(2, 40);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 30);\n\t\tassert_eq!(*cache.get(&2).unwrap(), 40);\n\t}\n\n\t#[test]\n\tfn eviction_chain() {\n\t\tlet mut cache = LruCache::new(3);\n\t\tcache.put(1, 10);\n\t\tcache.put(2, 20);\n\t\tcache.put(3, 30);\n\t\tcache.put(4, 40); // evicts 1 (one-hit wonder from small)\n\t\tcache.put(5, 50); // evicts 2\n\t\tcache.put(6, 60); // evicts 3\n\n\t\tassert!(cache.get(&1).is_none());\n\t\tassert!(cache.get(&2).is_none());\n\t\tassert!(cache.get(&3).is_none());\n\t\tassert_eq!(*cache.get(&4).unwrap(), 40);\n\t\tassert_eq!(*cache.get(&5).unwrap(), 50);\n\t\tassert_eq!(*cache.get(&6).unwrap(), 60);\n\t}\n\n\t#[test]\n\tfn capacity_one() {\n\t\tlet mut cache = LruCache::new(1);\n\t\tcache.put(1, 10);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 10);\n\t\tcache.put(2, 20); // evicts 1\n\t\tassert!(cache.get(&1).is_none());\n\t\tassert_eq!(*cache.get(&2).unwrap(), 20);\n\t\tassert_eq!(cache.map.len(), 1);\n\t}\n\n\t#[test]\n\tfn overwrite_at_full_capacity_does_not_evict() {\n\t\tlet mut cache = LruCache::new(2);\n\t\tcache.put(1, 10);\n\t\tcache.put(2, 20);\n\t\tcache.put(1, 11);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 11);\n\t\tassert_eq!(*cache.get(&2).unwrap(), 20);\n\t}\n\n\t#[test]\n\tfn get_only_element() {\n\t\tlet mut cache = LruCache::new(3);\n\t\tcache.put(1, 10);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 10);\n\t\tcache.put(2, 20);\n\t\tcache.put(3, 30);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 10);\n\t}\n\n\t#[test]\n\tfn one_hit_wonders_evicted_from_small_first() {\n\t\tlet mut cache = LruCache::new(10); // small_target = 1\n\t\tfor k in 1..=10 {\n\t\t\tcache.put(k, k * 10);\n\t\t}\n\t\tcache.get(&1); // key 1 is reused; keys 2..=10 are one-hit wonders\n\t\tcache.put(11, 110); // 1 promoted to main, 2 evicted\n\t\tassert_eq!(*cache.get(&1).unwrap(), 10);\n\t\tassert!(cache.get(&2).is_none());\n\t\tassert_eq!(*cache.get(&3).unwrap(), 30);\n\t}\n\n\t#[test]\n\tfn accessed_small_entry_promoted_to_main() {\n\t\tlet mut cache = LruCache::new(10);\n\t\tfor k in 1..=10 {\n\t\t\tcache.put(k, k);\n\t\t}\n\t\tcache.get(&2);\n\t\tassert!(!in_main(&cache, 2));\n\t\tcache.put(11, 11); // evicts 1 (freq 0)\n\t\tassert!(!in_main(&cache, 2));\n\t\tcache.put(12, 12); // 2 has freq > 0 -> promoted to main; 3 evicted\n\t\tassert!(in_main(&cache, 2));\n\t\tassert!(cache.get(&3).is_none());\n\t\tassert_eq!(*cache.get(&2).unwrap(), 2);\n\t}\n\n\t#[test]\n\tfn ghost_hit_admitted_into_main() {\n\t\tlet mut cache = LruCache::new(10);\n\t\tfor k in 1..=10 {\n\t\t\tcache.put(k, k);\n\t\t}\n\t\tcache.put(11, 11); // evicts 1 into ghost\n\t\tassert!(cache.get(&1).is_none());\n\t\tassert!(cache.ghost_set.contains_key(&1));\n\t\tlet before = cache.small_target;\n\t\tcache.put(1, 100); // ghost hit -> straight into main, small_target grows\n\t\tassert!(in_main(&cache, 1));\n\t\tassert!(!cache.ghost_set.contains_key(&1));\n\t\tassert!(cache.small_target > before);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 100);\n\t\tassert!(cache.map.len() <= 10);\n\t}\n\n\t#[test]\n\tfn main_evictions_not_ghosted() {\n\t\tlet mut cache = LruCache::new(1);\n\t\tcache.put(1, 1);\n\t\tcache.put(2, 2); // 1 -> small ghost\n\t\tcache.put(1, 1); // ghost hit -> main; 2 evicted from small -> small ghost\n\t\tassert!(in_main(&cache, 1));\n\t\tcache.put(3, 3); // small empty, main non-empty: evict 1 from main\n\t\tassert!(cache.get(&1).is_none());\n\t\tassert!(!cache.ghost_set.contains_key(&1));\n\t\tassert!(cache.main_ghost_set.contains_key(&1));\n\t\tassert_eq!(*cache.get(&3).unwrap(), 3);\n\t}\n\n\t#[test]\n\tfn main_ghost_hit_shrinks_small_target() {\n\t\tlet mut cache = LruCache::new(100);\n\t\tcache.small_target = 20;\n\t\tcache.push_main_ghost(500);\n\t\tcache.put(500, 5);\n\t\tassert!(in_main(&cache, 500));\n\t\tassert!(!cache.main_ghost_set.contains_key(&500));\n\t\tassert_eq!(cache.small_target, 19);\n\t}\n\n\t#[test]\n\tfn adaptive_target_stays_in_bounds_under_churn() {\n\t\tfor &cap in &[1usize, 2, 8, 100, 300] {\n\t\t\tlet mut cache = LruCache::new(cap);\n\t\t\tfor i in 0..20_000i64 {\n\t\t\t\t// Mix of a loop larger than the cache and a hot set.\n\t\t\t\tlet k = if i % 3 == 0 { i % 7 } else { (i * 7919) % (cap as i64 * 3 + 5) };\n\t\t\t\tif cache.get(&k).is_none() {\n\t\t\t\t\tcache.put(k, k * 10);\n\t\t\t\t}\n\t\t\t\tassert!(cache.small_target >= cache.min_small);\n\t\t\t\tassert!(cache.small_target <= cache.max_small);\n\t\t\t\tassert!(cache.map.len() <= cap);\n\t\t\t\tassert_eq!(cache.small.len() + cache.main.len(), cache.map.len());\n\t\t\t\tassert!(cache.ghost_q.len() <= cache.ghost_cap);\n\t\t\t\tassert!(cache.main_ghost_q.len() <= cache.main_ghost_cap);\n\t\t\t}\n\t\t}\n\t}\n\n\t#[test]\n\tfn capacity_bound_under_churn() {\n\t\tlet mut cache = LruCache::new(8);\n\t\tfor i in 0..10_000i64 {\n\t\t\tlet k = (i * 7919) % 37;\n\t\t\tif cache.get(&k).is_none() {\n\t\t\t\tcache.put(k, k * 10);\n\t\t\t}\n\t\t\tassert!(cache.map.len() <= 8);\n\t\t\tassert_eq!(cache.small.len() + cache.main.len(), cache.map.len());\n\t\t}\n\t\tfor k in 0..37 {\n\t\t\tif let Some(&v) = cache.get(&k) {\n\t\t\t\tassert_eq!(v, k * 10);\n\t\t\t}\n\t\t}\n\t}\n\n\t#[test]\n\t#[should_panic(expected = \"initialization failed\")]\n\tfn capacity_zero_panics() {\n\t\tlet mut _c = LruCache::<i64, i64>::new(0);\n\t}\n}\n"
}
```

### 09:15:20 thinking



### 09:15:21 tool: Edit

```json
{
  "replace_all": false,
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0002/variants/v002/attempts/01/overlay/src/lru_cache/mod.rs",
  "old_string": "\t\tlet mut _c = LruCache::<i64, i64>::new(0);",
  "new_string": "\t\tLruCache::<i64, i64>::new(0);"
}
```

### 09:15:29 thinking



### 09:15:30 chat

I wrote the v002 overlay as a single file, but it hasn't been compiled or tested. This session had no shell, so I couldn't run the four gates (build, contract, universal invariant, invariant) or the unit tests. I checked the logic by reading it through.

**File:** `overlay/src/lru_cache/mod.rs`, a copy of v001's S3-FIFO with only the H02 changes. `src/factory.rs` is unchanged; it still just calls `LruCache::new`.

**What changed, per H02:**
- **Second ghost list:** keys evicted from the main queue now go into `main_ghost_q` / `main_ghost_set`, capped at `max(1, capacity/2)` through a new `push_main_ghost`. It uses the same capped FIFO logic as the existing small ghost.
- **Adaptive small target:** `small_target` now changes at runtime. It starts at `max(1, capacity/10)` and stays between `min_small = max(1, capacity/100)` and `max_small = max(min_small, capacity/2)`. I added the `max(min_small, …)` because at capacity 1 the upper bound `capacity/2` would be 0, below the lower bound.
- **Small-ghost hit:** the target grows by `max(1, main_ghost_len / small_ghost_len)` and the key goes into main.
- **Main-ghost hit:** the target shrinks by `max(1, small_ghost_len / main_ghost_len)`, the key is removed from the main ghost, and it goes into main with `freq = 0`.
- **Miss in both:** the key goes into the small queue, as before.
- **Ghost sizes:** both ghost lengths are read before the hit key is removed, and both divisors are guarded against zero. The small ghost's size is still based on the initial target, as before.
- **Eviction:** `evict_one`'s condition is the same but now reads the moving target. It still terminates: the target never drops below 1, and an empty small queue sends eviction to main.

**Tests:**
- `main_evictions_not_ghosted` now checks the key is absent from the small ghost and present in the main ghost.
- `ghost_hit_admitted_into_main` now also checks that the target grows.
- New `main_ghost_hit_shrinks_small_target` checks the target goes from 20 to 19.
- New `adaptive_target_stays_in_bounds_under_churn` runs at capacities 1, 2, 8, 100 and 300. After every operation it checks the target bounds, `small.len() + main.len() == map.len()`, both ghost caps, and the capacity limit.

**Limitation:** removing a key from a ghost on a hit leaves its old entry in that ghost's FIFO queue, as v001 already did. The ghost lengths used to size each adjustment can therefore be a little too high. This doesn't affect correctness.


---

- finished: 2026-10-21T09:15:31+00:00
- duration: 78.31s
- status: ok
- input_tokens: 16
- output_tokens: 9936
- cache_read_tokens: 212815
- cache_write_tokens: 22924
- cost_usd: 0.424739
