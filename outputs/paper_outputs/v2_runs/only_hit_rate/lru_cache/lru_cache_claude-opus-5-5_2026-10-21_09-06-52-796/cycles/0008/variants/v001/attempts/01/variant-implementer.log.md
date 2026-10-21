# Agent log: variant-implementer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0008/variants/v001/attempts/01
- allow_edits: True
- started: 2026-10-21T09:48:36+00:00

## timeline

### 09:48:40 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796; cat cycles/0008/hypotheses/H01.md; echo ----; find cycles/0007/variants/v002/attempts/01/overlay -type f; cat cycles/0007/variants/v002/insight.md cycles/0007/variants/v002/eval/fitness-vector.json; echo ---; cat /home/user/cache-evolve/inputs/subjects/lru_cache/manifest.toml; ls -R /home/user/cache-evolve/inputs/subjects/lru_cache/src",
  "description": "Read hypothesis, parent overlay, manifest"
}
```

### 09:48:42 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0008/hypotheses/H01.md"
}
```

### 09:48:42 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0007/variants/v002/insight.md"
}
```

### 09:48:43 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/lru_cache/manifest.toml"
}
```

### 09:48:44 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0007/variants/v002/attempts/01/overlay"
}
```

### 09:48:45 chat

Changes are all in `src/lru_cache/mod.rs`; I'll read the parent's copy.

### 09:48:46 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0007/variants/v002/attempts/01/overlay/src/lru_cache/mod.rs"
}
```

### 09:48:48 thinking



### 09:48:48 chat

I can't list directories, so I'll check whether the parent also overlaid `factory.rs`.

### 09:48:48 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0007/variants/v002/attempts/01/overlay/src/factory.rs"
}
```

### 09:48:52 thinking



### 09:48:52 chat

Only `mod.rs` was overlaid. Next I'll write the new version with the three edits: set `MAX_FREQ` to 7, keep surplus small-queue hits as credit on promotion, and add the new test.

### 09:49:35 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0008/variants/v001/attempts/01/overlay/src/lru_cache/mod.rs",
  "content": "// S3-FIFO cache (small probation FIFO + main FIFO with lazy reinsertion + ghost queue).\n// Nodes live in a Vec arena indexed by u32; evicted slots are reused in place.\n// Public surface (LruCache, Node, try_new, new, get, put, framework::Cache impl) is\n// unchanged so the JNI layer and bins keep working. No unsafe, no external crates.\n\nuse std::collections::{HashMap, VecDeque};\nuse std::fmt::Debug;\nuse std::hash::Hash;\n\npub mod lru_cache_jni;\n\n// H01 (cycle 8): raised from 3 so carried-over small-queue credits can matter.\nconst MAX_FREQ: u8 = 7;\n\n#[derive(Debug)]\npub struct Node<K, V> {\n\tkey: K,\n\tvalue: V,\n\tfreq: u8,\n\t#[allow(dead_code)] // read by tests / Debug; queue membership is authoritative\n\tin_main: bool,\n}\n\n#[derive(Debug)]\npub struct LruCache<K, V> {\n\tmap: HashMap<K, u32>,   // key -> arena index\n\tnodes: Vec<Node<K, V>>, // arena\n\tsmall: VecDeque<u32>,   // probation FIFO (front = oldest)\n\tmain: VecDeque<u32>,    // main FIFO (front = oldest)\n\tghost_q: VecDeque<K>,   // recently evicted-from-small keys, FIFO\n\tghost_set: HashMap<K, ()>,\n\tsmall_target: usize,\n\tghost_cap: usize,\n\tcapacity: usize,\n}\n\nimpl<K, V> LruCache<K, V>\nwhere\n\tK: Hash + Eq + Clone + Debug,\n\tV: Debug,\n{\n\tpub fn try_new(capacity: usize) -> Option<Self> {\n\t\tif capacity == 0 {\n\t\t\tlog::error!(\"LruCache.try_new: capacity must be > 0\");\n\t\t\treturn None;\n\t\t}\n\t\t// H02 (cycle 7): capacity / 40 probation queue (was capacity / 20 in cycle 6, capacity / 10 before).\n\t\tlet small_target = std::cmp::max(1, capacity / 40);\n\t\t// H02 (cycle 7): 3 * capacity / 20 ghost window (was capacity / 8), keeping small + ghost ~constant.\n\t\tlet ghost_cap = std::cmp::max(1, capacity.saturating_mul(3) / 20);\n\t\tlog::info!(\"LruCache.try_new: capacity={capacity} small_target={small_target} (capacity / 40) ghost_cap={ghost_cap} (3 * capacity / 20)\");\n\t\tSome(Self {\n\t\t\tmap: HashMap::with_capacity(capacity),\n\t\t\tnodes: Vec::with_capacity(capacity),\n\t\t\tsmall: VecDeque::with_capacity(capacity),\n\t\t\tmain: VecDeque::with_capacity(capacity),\n\t\t\tghost_q: VecDeque::with_capacity(ghost_cap + 1),\n\t\t\tghost_set: HashMap::with_capacity(ghost_cap + 1),\n\t\t\tsmall_target,\n\t\t\tghost_cap,\n\t\t\tcapacity,\n\t\t})\n\t}\n\n\tpub fn new(capacity: usize) -> Self {\n\t\tSelf::try_new(capacity).expect(\"LruCache::new: initialization failed\")\n\t}\n\n\t/// Returns the value for `key` if present, bumping its (saturating) access frequency.\n\tpub fn get(&mut self, key: &K) -> Option<&V> {\n\t\tlet idx = *self.map.get(key)? as usize;\n\t\tlet node = &mut self.nodes[idx];\n\t\tif node.freq < MAX_FREQ {\n\t\t\tnode.freq += 1;\n\t\t}\n\t\tSome(&node.value)\n\t}\n\n\t/// Inserts or updates `key`. New keys enter `small`, or `main` on a ghost hit.\n\tpub fn put(&mut self, key: K, value: V) {\n\t\tif let Some(&idx) = self.map.get(&key) {\n\t\t\tlet node = &mut self.nodes[idx as usize];\n\t\t\tnode.value = value;\n\t\t\tif node.freq < MAX_FREQ {\n\t\t\t\tnode.freq += 1;\n\t\t\t}\n\t\t\treturn;\n\t\t}\n\n\t\tlet ghost_hit = self.ghost_set.remove(&key).is_some();\n\n\t\tlet idx = if self.map.len() >= self.capacity {\n\t\t\tlet slot = self.evict_one();\n\t\t\tlet node = &mut self.nodes[slot as usize];\n\t\t\tnode.key = key.clone();\n\t\t\tnode.value = value;\n\t\t\tnode.freq = 0; // ghost arrivals get no credit (H01, cycle 8)\n\t\t\tnode.in_main = ghost_hit;\n\t\t\tslot\n\t\t} else {\n\t\t\tlet slot = self.nodes.len() as u32;\n\t\t\tself.nodes.push(Node {\n\t\t\t\tkey: key.clone(),\n\t\t\t\tvalue,\n\t\t\t\tfreq: 0,\n\t\t\t\tin_main: ghost_hit,\n\t\t\t});\n\t\t\tslot\n\t\t};\n\n\t\tif ghost_hit {\n\t\t\tself.main.push_back(idx);\n\t\t} else {\n\t\t\tself.small.push_back(idx);\n\t\t}\n\t\tself.map.insert(key, idx);\n\t\tdebug_assert!(self.map.len() <= self.capacity);\n\t}\n\n\t/// Evicts exactly one resident entry and returns its (now free) arena slot.\n\t/// Terminates: every iteration either lowers some node's freq by >= 1 or frees a slot.\n\tfn evict_one(&mut self) -> u32 {\n\t\tloop {\n\t\t\tif self.small.len() >= self.small_target || self.main.is_empty() {\n\t\t\t\tlet idx = self.small.pop_front().expect(\"small must be non-empty when main is empty and cache is full\");\n\t\t\t\tlet node = &mut self.nodes[idx as usize];\n\t\t\t\tif node.freq > 0 {\n\t\t\t\t\t// H01 (cycle 8): the first resident hit pays for admission to main;\n\t\t\t\t\t// every extra hit seen in small carries over as credit.\n\t\t\t\t\tnode.freq -= 1;\n\t\t\t\t\tnode.in_main = true;\n\t\t\t\t\tself.main.push_back(idx);\n\t\t\t\t\tcontinue;\n\t\t\t\t}\n\t\t\t\tlet evicted = node.key.clone();\n\t\t\t\tself.map.remove(&evicted);\n\t\t\t\tself.push_ghost(evicted);\n\t\t\t\treturn idx;\n\t\t\t} else {\n\t\t\t\tlet idx = self.main.pop_front().expect(\"main checked non-empty\");\n\t\t\t\tlet node = &mut self.nodes[idx as usize];\n\t\t\t\tif node.freq > 0 {\n\t\t\t\t\tnode.freq -= 1;\n\t\t\t\t\tself.main.push_back(idx);\n\t\t\t\t\tcontinue;\n\t\t\t\t}\n\t\t\t\tlet evicted = node.key.clone();\n\t\t\t\tself.map.remove(&evicted);\n\t\t\t\treturn idx;\n\t\t\t}\n\t\t}\n\t}\n\n\tfn push_ghost(&mut self, key: K) {\n\t\tself.ghost_set.insert(key.clone(), ());\n\t\tself.ghost_q.push_back(key);\n\t\twhile self.ghost_q.len() > self.ghost_cap {\n\t\t\tif let Some(old) = self.ghost_q.pop_front() {\n\t\t\t\tself.ghost_set.remove(&old);\n\t\t\t}\n\t\t}\n\t}\n}\n\nimpl<K, V> framework::Cache<K, V> for LruCache<K, V>\nwhere\n\tK: Hash + Eq + Clone + Debug,\n\tV: Debug,\n{\n\tfn put(&mut self, key: K, value: V) {\n\t\tLruCache::put(self, key, value);\n\t}\n\n\tfn get(&mut self, key: &K) -> Option<&V> {\n\t\tLruCache::get(self, key)\n\t}\n}\n\n#[cfg(test)]\nmod tests {\n\tuse super::*;\n\n\tfn in_main(cache: &LruCache<i64, i64>, key: i64) -> bool {\n\t\tlet idx = cache.map[&key] as usize;\n\t\tcache.nodes[idx].in_main\n\t}\n\n\tfn freq_of(cache: &LruCache<i64, i64>, key: i64) -> u8 {\n\t\tlet idx = cache.map[&key] as usize;\n\t\tcache.nodes[idx].freq\n\t}\n\n\t#[test]\n\tfn get_miss_returns_none() {\n\t\tlet mut cache = LruCache::<i64, i64>::new(4);\n\t\tassert!(cache.get(&42).is_none());\n\t}\n\n\t#[test]\n\tfn put_then_get() {\n\t\tlet mut cache = LruCache::new(4);\n\t\tcache.put(1, 10);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 10);\n\t}\n\n\t#[test]\n\tfn put_overwrites_existing_key() {\n\t\tlet mut cache = LruCache::new(4);\n\t\tcache.put(1, 10);\n\t\tcache.put(1, 20);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 20);\n\t}\n\n\t#[test]\n\tfn overwrite_does_not_grow_order() {\n\t\tlet mut cache = LruCache::new(2);\n\t\tcache.put(1, 10);\n\t\tcache.put(1, 20);\n\t\tcache.put(1, 30);\n\t\tcache.put(2, 40);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 30);\n\t\tassert_eq!(*cache.get(&2).unwrap(), 40);\n\t}\n\n\t#[test]\n\tfn eviction_chain() {\n\t\tlet mut cache = LruCache::new(3);\n\t\tcache.put(1, 10);\n\t\tcache.put(2, 20);\n\t\tcache.put(3, 30);\n\t\tcache.put(4, 40); // evicts 1 (one-hit wonder from small)\n\t\tcache.put(5, 50); // evicts 2\n\t\tcache.put(6, 60); // evicts 3\n\n\t\tassert!(cache.get(&1).is_none());\n\t\tassert!(cache.get(&2).is_none());\n\t\tassert!(cache.get(&3).is_none());\n\t\tassert_eq!(*cache.get(&4).unwrap(), 40);\n\t\tassert_eq!(*cache.get(&5).unwrap(), 50);\n\t\tassert_eq!(*cache.get(&6).unwrap(), 60);\n\t}\n\n\t#[test]\n\tfn capacity_one() {\n\t\tlet mut cache = LruCache::new(1);\n\t\tcache.put(1, 10);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 10);\n\t\tcache.put(2, 20); // evicts 1\n\t\tassert!(cache.get(&1).is_none());\n\t\tassert_eq!(*cache.get(&2).unwrap(), 20);\n\t\tassert_eq!(cache.map.len(), 1);\n\t}\n\n\t#[test]\n\tfn overwrite_at_full_capacity_does_not_evict() {\n\t\tlet mut cache = LruCache::new(2);\n\t\tcache.put(1, 10);\n\t\tcache.put(2, 20);\n\t\tcache.put(1, 11);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 11);\n\t\tassert_eq!(*cache.get(&2).unwrap(), 20);\n\t}\n\n\t#[test]\n\tfn get_only_element() {\n\t\tlet mut cache = LruCache::new(3);\n\t\tcache.put(1, 10);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 10);\n\t\tcache.put(2, 20);\n\t\tcache.put(3, 30);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 10);\n\t}\n\n\t#[test]\n\tfn one_hit_wonders_evicted_from_small_first() {\n\t\tlet mut cache = LruCache::new(10); // small_target = 1\n\t\tfor k in 1..=10 {\n\t\t\tcache.put(k, k * 10);\n\t\t}\n\t\tcache.get(&1); // key 1 is reused; keys 2..=10 are one-hit wonders\n\t\tcache.put(11, 110); // 1 promoted to main, 2 evicted\n\t\tassert_eq!(*cache.get(&1).unwrap(), 10);\n\t\tassert!(cache.get(&2).is_none());\n\t\tassert_eq!(*cache.get(&3).unwrap(), 30);\n\t}\n\n\t#[test]\n\tfn accessed_small_entry_promoted_to_main() {\n\t\tlet mut cache = LruCache::new(10);\n\t\tfor k in 1..=10 {\n\t\t\tcache.put(k, k);\n\t\t}\n\t\tcache.get(&2);\n\t\tassert!(!in_main(&cache, 2));\n\t\tcache.put(11, 11); // evicts 1 (freq 0)\n\t\tassert!(!in_main(&cache, 2));\n\t\tcache.put(12, 12); // 2 has freq > 0 -> promoted to main; 3 evicted\n\t\tassert!(in_main(&cache, 2));\n\t\tassert!(cache.get(&3).is_none());\n\t\tassert_eq!(*cache.get(&2).unwrap(), 2);\n\t}\n\n\t#[test]\n\tfn small_hit_credits_carry_into_main() {\n\t\tlet mut cache = LruCache::new(10); // small_target = 1\n\t\tfor k in 1..=10 {\n\t\t\tcache.put(k, k);\n\t\t}\n\t\t// Key 1: three resident hits in small -> carries 2 credits into main.\n\t\tcache.get(&1);\n\t\tcache.get(&1);\n\t\tcache.get(&1);\n\t\tassert_eq!(freq_of(&cache, 1), 3);\n\t\tcache.put(11, 11); // 1 promoted (freq 3 -> 2), 2 evicted\n\t\tassert!(in_main(&cache, 1));\n\t\tassert_eq!(freq_of(&cache, 1), 2);\n\t\tassert!(cache.get(&2).is_none());\n\n\t\t// Key 3: a single resident hit -> arrives in main with freq 0.\n\t\tcache.get(&3);\n\t\tcache.put(12, 12); // 3 promoted (freq 1 -> 0), 4 evicted\n\t\tassert!(in_main(&cache, 3));\n\t\tassert_eq!(freq_of(&cache, 3), 0);\n\t\tassert!(cache.get(&4).is_none());\n\t}\n\n\t#[test]\n\tfn freq_saturates_at_max_freq() {\n\t\tlet mut cache = LruCache::new(4);\n\t\tcache.put(1, 1);\n\t\tfor _ in 0..20 {\n\t\t\tcache.get(&1);\n\t\t}\n\t\tassert_eq!(freq_of(&cache, 1), MAX_FREQ);\n\t\tassert_eq!(MAX_FREQ, 7);\n\t}\n\n\t#[test]\n\tfn ghost_hit_admitted_into_main() {\n\t\tlet mut cache = LruCache::new(16); // small_target = 1, ghost_cap = 48 / 20 = 2\n\t\tassert_eq!(cache.ghost_cap, 2);\n\t\tassert_eq!(cache.small_target, 1);\n\t\tfor k in 1..=16 {\n\t\t\tcache.put(k, k);\n\t\t}\n\t\tcache.put(17, 17); // evicts 1 into ghost\n\t\tassert!(cache.get(&1).is_none());\n\t\tassert!(cache.ghost_set.contains_key(&1));\n\t\tcache.put(1, 100); // ghost hit -> straight into main\n\t\tassert!(in_main(&cache, 1));\n\t\tassert_eq!(freq_of(&cache, 1), 0); // ghost arrivals get no credit\n\t\tassert!(!cache.ghost_set.contains_key(&1));\n\t\tassert_eq!(*cache.get(&1).unwrap(), 100);\n\t\tassert!(cache.map.len() <= 16);\n\t}\n\n\t#[test]\n\tfn ghost_window_expires_after_ghost_cap_plus_one_evictions() {\n\t\tlet mut cache = LruCache::new(16); // small_target = 1, ghost_cap = 2\n\t\tlet ghost_cap = cache.ghost_cap as i64;\n\t\tassert_eq!(ghost_cap, 2);\n\t\tassert_eq!(cache.small_target, 1);\n\t\tfor k in 1..=16 {\n\t\t\tcache.put(k, k);\n\t\t}\n\t\t// ghost_cap + 1 small-queue evictions: keys 1..=ghost_cap+1.\n\t\tfor k in 17..=(17 + ghost_cap) {\n\t\t\tcache.put(k, k);\n\t\t\tassert!(cache.ghost_q.len() <= cache.ghost_cap);\n\t\t}\n\t\tassert!(cache.get(&1).is_none());\n\t\tassert!(!cache.ghost_set.contains_key(&1));\n\t\tassert!(cache.ghost_set.contains_key(&2));\n\t\tcache.put(1, 100); // no longer a ghost -> lands in small\n\t\tassert!(!in_main(&cache, 1));\n\t\tassert_eq!(*cache.get(&1).unwrap(), 100);\n\t\tassert!(cache.map.len() <= 16);\n\t\tassert_eq!(cache.small.len() + cache.main.len(), cache.map.len());\n\t}\n\n\t#[test]\n\tfn main_evictions_not_ghosted() {\n\t\tlet mut cache = LruCache::new(1);\n\t\tcache.put(1, 1);\n\t\tcache.put(2, 2); // 1 -> ghost\n\t\tcache.put(1, 1); // ghost hit -> main; 2 evicted from small -> ghost\n\t\tassert!(in_main(&cache, 1));\n\t\tcache.put(3, 3); // small empty, main non-empty: evict 1 from main\n\t\tassert!(cache.get(&1).is_none());\n\t\tassert!(!cache.ghost_set.contains_key(&1));\n\t\tassert_eq!(*cache.get(&3).unwrap(), 3);\n\t}\n\n\t#[test]\n\tfn capacity_bound_under_churn() {\n\t\tlet mut cache = LruCache::new(8);\n\t\tfor i in 0..10_000i64 {\n\t\t\tlet k = (i * 7919) % 37;\n\t\t\tif cache.get(&k).is_none() {\n\t\t\t\tcache.put(k, k * 10);\n\t\t\t}\n\t\t\tassert!(cache.map.len() <= 8);\n\t\t\tassert_eq!(cache.small.len() + cache.main.len(), cache.map.len());\n\t\t\tassert!(cache.ghost_q.len() <= cache.ghost_cap);\n\t\t}\n\t\tfor k in 0..37 {\n\t\t\tif let Some(&v) = cache.get(&k) {\n\t\t\t\tassert_eq!(v, k * 10);\n\t\t\t}\n\t\t}\n\t}\n\n\t#[test]\n\tfn split_ratios_at_2048() {\n\t\tlet cache = LruCache::<i64, i64>::new(2048);\n\t\tassert_eq!(cache.small_target, 51);\n\t\tassert_eq!(cache.ghost_cap, 307);\n\t\t// Combined admission window unchanged vs cycle 6 (102 + 256).\n\t\tassert_eq!(cache.small_target + cache.ghost_cap, 358);\n\t}\n\n\t#[test]\n\tfn capacity_bound_under_churn_400() {\n\t\tlet mut cache = LruCache::new(400); // small_target = 10, ghost_cap = 60\n\t\tassert_eq!(cache.small_target, 10);\n\t\tassert_eq!(cache.ghost_cap, 60);\n\t\tlet mut state: u64 = 0x9E37_79B9_7F4A_7C15;\n\t\tfor _ in 0..50_000 {\n\t\t\tstate = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);\n\t\t\tlet r = (state >> 33) as i64;\n\t\t\t// Skewed mix: hot set of 300 keys plus a 2000-key cold tail.\n\t\t\tlet k = if r % 3 == 0 { (r >> 2) % 2000 + 1000 } else { (r >> 2) % 300 };\n\t\t\tif cache.get(&k).is_none() {\n\t\t\t\tcache.put(k, k * 10);\n\t\t\t}\n\t\t\tassert!(cache.map.len() <= 400);\n\t\t\tassert_eq!(cache.small.len() + cache.main.len(), cache.map.len());\n\t\t\tassert!(cache.ghost_q.len() <= cache.ghost_cap);\n\t\t}\n\t\tfor k in 0..3000i64 {\n\t\t\tif let Some(&v) = cache.get(&k) {\n\t\t\t\tassert_eq!(v, k * 10);\n\t\t\t}\n\t\t}\n\t}\n\n\t#[test]\n\t#[should_panic(expected = \"initialization failed\")]\n\tfn capacity_zero_panics() {\n\t\tLruCache::<i64, i64>::new(0);\n\t}\n}\n"
}
```

### 09:49:41 thinking



### 09:49:41 chat

I wrote variant v001, but it hasn't been compiled or tested. This session only had file read/write tools, so none of the four gates or the in-file tests were run. The new tests were checked by tracing the code by hand.

The overlay is one file, `overlay/src/lru_cache/mod.rs`. It is the parent's (cycle 7 v002) copy of that file with only the H01 changes applied:

1. **Higher cap:** `MAX_FREQ` goes from 3 to 7.
2. **Credits carry into `main`:** when a key with hits leaves `small`, its count now drops by 1 (`node.freq -= 1`) instead of resetting to 0. The first hit pays for entry to `main`, and each extra hit stays as credit. So a key hit three times in `small` arrives with `freq = 2`, and a key hit once arrives with `freq = 0`, as before.
3. **Everything else unchanged:** keys re-admitted from the ghost list still start at `freq = 0`. `get`, `put`, how `main` evicts, the ghost list, and the queue sizes (`capacity / 40` and `3 * capacity / 20`) are the parent's. The loop still ends because each pass either lowers a count by 1 or frees a slot.

**Tests added:**
- `small_hit_credits_carry_into_main`: the test the hypothesis asked for. A key hit 3 times reaches `main` with `freq == 2`; a key hit once reaches it with `freq == 0`.
- `freq_saturates_at_max_freq`: checks the count stops at 7.
- A new check in `ghost_hit_admitted_into_main` that a key re-admitted from the ghost list starts at 0.

The parent's other tests are kept. I couldn't list the parent's `overlay/` directory, so I only confirmed it has no `src/factory.rs`. If it had other files, this overlay doesn't include them.


---

- finished: 2026-10-21T09:49:42+00:00
- duration: 65.93s
- status: ok
- input_tokens: 14
- output_tokens: 8806
- cache_read_tokens: 169298
- cache_write_tokens: 21952
- cost_usd: 0.385652
