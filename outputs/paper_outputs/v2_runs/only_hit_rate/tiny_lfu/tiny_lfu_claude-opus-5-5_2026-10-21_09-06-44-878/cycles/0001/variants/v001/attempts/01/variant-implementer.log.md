# Agent log: variant-implementer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0001/variants/v001/attempts/01
- allow_edits: True
- started: 2026-10-21T09:08:06+00:00

## timeline

### 09:08:11 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve; cat outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0001/hypotheses/H01.md; cat inputs/subjects/tiny_lfu/manifest.toml; ls -R inputs/subjects/tiny_lfu/baseline inputs/subjects/tiny_lfu/src; cat inputs/subjects/tiny_lfu/baseline/insight.md inputs/subjects/tiny_lfu/baseline/eval/fitness-vector.json",
  "description": "Read hypothesis, manifest, baseline"
}
```

### 09:08:13 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0001/hypotheses/H01.md"
}
```

### 09:08:13 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/tiny_lfu/manifest.toml"
}
```

### 09:08:13 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/tiny_lfu/baseline/insight.md"
}
```

### 09:08:14 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/inputs/subjects/tiny_lfu"
}
```

### 09:08:16 thinking



### 09:08:16 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/tiny_lfu/src/tiny_lfu/mod.rs"
}
```

### 09:08:16 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/tiny_lfu/baseline/overlay"
}
```

### 09:08:18 thinking



### 09:08:19 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/tiny_lfu/tests/invariant_gate.rs"
}
```

### 09:08:19 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/tiny_lfu/src/test_utils/mod.rs"
}
```

### 09:08:19 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/tiny_lfu/src/factory.rs"
}
```

### 09:08:20 chat

Reading the invariant suite to see what structural checks my W-TinyLFU rewrite must satisfy.

### 09:08:21 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/tiny_lfu/src/test_utils/invariants.rs"
}
```

### 09:08:42 thinking



### 09:08:42 chat

Writing the W-TinyLFU version of `src/tiny_lfu/mod.rs` as the overlay.

### 09:09:24 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0001/variants/v001/attempts/01/overlay/src/tiny_lfu/mod.rs",
  "content": "use std::collections::hash_map::DefaultHasher;\nuse std::collections::HashMap;\nuse std::fmt::Debug;\nuse std::hash::{Hash, Hasher};\n\npub mod tiny_lfu_jni;\n\nconst SKETCH_DEPTH: usize = 4;\nconst SKETCH_WIDTH_FACTOR: usize = 4;\nconst MIN_SKETCH_WIDTH: usize = 16;\nconst MAX_COUNT: u32 = u32::MAX;\n\n/// Segment tags (also used as list selectors for the intrusive lists).\nconst WINDOW: u8 = 0;\nconst PROBATION: u8 = 1;\nconst PROTECTED: u8 = 2;\n\n#[derive(Debug)]\npub struct Node<K, V> {\n\tkey: K,\n\tvalue: V,\n\tprev: Option<usize>,\n\tnext: Option<usize>,\n\tseg: u8,\n}\n\n/// W-TinyLFU: a small admission-window LRU feeding a segmented (probation /\n/// protected) main region. The TinyLFU admission contest runs only when a\n/// window evictee meets the main region's victim.\n#[derive(Debug)]\npub struct TinyLfuCache<K, V> {\n\tmap: HashMap<K, usize>,\n\tnodes: Vec<Node<K, V>>,\n\twin_head: Option<usize>,\n\twin_tail: Option<usize>,\n\twin_len: usize,\n\tprob_head: Option<usize>,\n\tprob_tail: Option<usize>,\n\tprob_len: usize,\n\tprot_head: Option<usize>,\n\tprot_tail: Option<usize>,\n\tprot_len: usize,\n\tcapacity: usize,\n\twindow_cap: usize,\n\tmain_cap: usize,\n\tprotected_cap: usize,\n\tsketch: CountMinSketch,\n}\n\nimpl<K, V> TinyLfuCache<K, V>\nwhere\n\tK: Hash + Eq + Clone + Debug,\n\tV: Debug,\n{\n\tpub fn try_new(capacity: usize) -> Option<Self> {\n\t\tif capacity == 0 {\n\t\t\tlog::error!(\"TinyLfuCache.try_new: capacity must be > 0\");\n\t\t\treturn None;\n\t\t}\n\t\tlet width = (capacity * SKETCH_WIDTH_FACTOR)\n\t\t\t.max(MIN_SKETCH_WIDTH)\n\t\t\t.next_power_of_two();\n\t\tlet (window_cap, main_cap) = if capacity == 1 {\n\t\t\t(1, 0)\n\t\t} else {\n\t\t\tlet w = (capacity / 100).max(1);\n\t\t\t(w, capacity - w)\n\t\t};\n\t\tlet protected_cap = main_cap * 80 / 100;\n\t\tlog::info!(\n\t\t\t\"TinyLfuCache.try_new: capacity={capacity} window={window_cap} main={main_cap} \\\n\t\t\t protected={protected_cap} sketch_width={width}\"\n\t\t);\n\t\tSome(Self {\n\t\t\tmap: HashMap::with_capacity(capacity),\n\t\t\tnodes: Vec::with_capacity(capacity),\n\t\t\twin_head: None,\n\t\t\twin_tail: None,\n\t\t\twin_len: 0,\n\t\t\tprob_head: None,\n\t\t\tprob_tail: None,\n\t\t\tprob_len: 0,\n\t\t\tprot_head: None,\n\t\t\tprot_tail: None,\n\t\t\tprot_len: 0,\n\t\t\tcapacity,\n\t\t\twindow_cap,\n\t\t\tmain_cap,\n\t\t\tprotected_cap,\n\t\t\tsketch: CountMinSketch::new(width),\n\t\t})\n\t}\n\n\tpub fn new(capacity: usize) -> Self {\n\t\tSelf::try_new(capacity).expect(\"TinyLfuCache::new: initialization failed\")\n\t}\n\n\t/// Heap bytes held by the frequency sketch for this cache: `depth * width * size_of::<u32>()`.\n\t///\n\t/// This is the FIXED metadata overhead TinyLfu carries beyond the per-entry nodes — it is\n\t/// sized at construction from capacity (see `try_new`) and does not grow with population.\n\t/// The metadata-size fitness bin reports it so variant edits to the sketch geometry are visible.\n\tpub fn sketch_heap_bytes(&self) -> usize {\n\t\tself.sketch.heap_bytes()\n\t}\n\n\tpub fn get(&mut self, key: &K) -> Option<&V> {\n\t\tself.sketch.increment(key);\n\t\tlet &idx = self.map.get(key)?;\n\t\tself.on_hit(idx);\n\t\tSome(&self.nodes[idx].value)\n\t}\n\n\tpub fn put(&mut self, key: K, value: V) {\n\t\tself.sketch.increment(&key);\n\t\tif let Some(&idx) = self.map.get(&key) {\n\t\t\tself.nodes[idx].value = value;\n\t\t\tself.on_hit(idx);\n\t\t\treturn;\n\t\t}\n\n\t\tif self.nodes.len() < self.capacity {\n\t\t\tlet idx = self.nodes.len();\n\t\t\tself.nodes.push(Node {\n\t\t\t\tkey: key.clone(),\n\t\t\t\tvalue,\n\t\t\t\tprev: None,\n\t\t\t\tnext: None,\n\t\t\t\tseg: WINDOW,\n\t\t\t});\n\t\t\tself.push_back(WINDOW, idx);\n\t\t\tself.map.insert(key, idx);\n\t\t\tif self.win_len > self.window_cap {\n\t\t\t\t// Not full: the main region necessarily has room.\n\t\t\t\tlet cand = self.win_head.expect(\"TinyLfuCache: window overflow with empty window\");\n\t\t\t\tself.unlink(WINDOW, cand);\n\t\t\t\tif self.prob_len + self.prot_len < self.main_cap {\n\t\t\t\t\tself.push_back(PROBATION, cand);\n\t\t\t\t} else {\n\t\t\t\t\t// Defensive: should be unreachable while not full.\n\t\t\t\t\tlet freed = self.contest(cand);\n\t\t\t\t\tself.drop_slot_unreachable(freed);\n\t\t\t\t}\n\t\t\t}\n\t\t\treturn;\n\t\t}\n\n\t\t// Full: decide which slot is freed before writing the new key.\n\t\tlet cand = self.win_head.expect(\"TinyLfuCache: full cache has empty window\");\n\t\tself.unlink(WINDOW, cand);\n\t\tlet freed = if self.prob_len + self.prot_len < self.main_cap {\n\t\t\t// Unreachable when full (window would be over cap); fall back to\n\t\t\t// evicting the main victim so the arena never grows.\n\t\t\tself.push_back(PROBATION, cand);\n\t\t\tlet victim = self\n\t\t\t\t.prob_head\n\t\t\t\t.or(self.prot_head)\n\t\t\t\t.expect(\"TinyLfuCache: no victim\");\n\t\t\tlet seg = self.nodes[victim].seg;\n\t\t\tself.unlink(seg, victim);\n\t\t\tlet vk = self.nodes[victim].key.clone();\n\t\t\tself.map.remove(&vk);\n\t\t\tvictim\n\t\t} else {\n\t\t\tself.contest(cand)\n\t\t};\n\t\tself.nodes[freed].key = key.clone();\n\t\tself.nodes[freed].value = value;\n\t\tself.push_back(WINDOW, freed);\n\t\tself.map.insert(key, freed);\n\t}\n\n\t/// Admission contest between the (already unlinked) window candidate and the\n\t/// main region's victim (probation LRU, else protected LRU). Returns the index\n\t/// of the freed slot, already unlinked and removed from `map`.\n\tfn contest(&mut self, cand: usize) -> usize {\n\t\tlet victim = self.prob_head.or(self.prot_head);\n\t\tmatch victim {\n\t\t\tSome(v) => {\n\t\t\t\tlet cand_freq = self.sketch.estimate(&self.nodes[cand].key);\n\t\t\t\tlet victim_freq = self.sketch.estimate(&self.nodes[v].key);\n\t\t\t\tif cand_freq > victim_freq {\n\t\t\t\t\tlet seg = self.nodes[v].seg;\n\t\t\t\t\tself.unlink(seg, v);\n\t\t\t\t\tlet vk = self.nodes[v].key.clone();\n\t\t\t\t\tself.map.remove(&vk);\n\t\t\t\t\tself.push_back(PROBATION, cand);\n\t\t\t\t\tv\n\t\t\t\t} else {\n\t\t\t\t\tlet ck = self.nodes[cand].key.clone();\n\t\t\t\t\tself.map.remove(&ck);\n\t\t\t\t\tcand\n\t\t\t\t}\n\t\t\t}\n\t\t\tNone => {\n\t\t\t\tlet ck = self.nodes[cand].key.clone();\n\t\t\t\tself.map.remove(&ck);\n\t\t\t\tcand\n\t\t\t}\n\t\t}\n\t}\n\n\t/// Remove an unlinked, unmapped slot from the arena by swapping with the last\n\t/// slot. Only used on a defensive path that should never run.\n\tfn drop_slot_unreachable(&mut self, freed: usize) {\n\t\tlet last = self.nodes.len() - 1;\n\t\tif freed != last {\n\t\t\tlet seg = self.nodes[last].seg;\n\t\t\tself.unlink(seg, last);\n\t\t\tself.nodes.swap(freed, last);\n\t\t\tself.nodes.pop();\n\t\t\tlet k = self.nodes[freed].key.clone();\n\t\t\tself.map.insert(k, freed);\n\t\t\tself.push_back(seg, freed);\n\t\t} else {\n\t\t\tself.nodes.pop();\n\t\t}\n\t}\n\n\tfn on_hit(&mut self, idx: usize) {\n\t\tmatch self.nodes[idx].seg {\n\t\t\tWINDOW => {\n\t\t\t\tself.unlink(WINDOW, idx);\n\t\t\t\tself.push_back(WINDOW, idx);\n\t\t\t}\n\t\t\tPROBATION => {\n\t\t\t\tself.unlink(PROBATION, idx);\n\t\t\t\tself.push_back(PROTECTED, idx);\n\t\t\t\tif self.prot_len > self.protected_cap {\n\t\t\t\t\tlet demote = self.prot_head.expect(\"TinyLfuCache: protected empty\");\n\t\t\t\t\tself.unlink(PROTECTED, demote);\n\t\t\t\t\tself.push_back(PROBATION, demote);\n\t\t\t\t}\n\t\t\t}\n\t\t\t_ => {\n\t\t\t\tself.unlink(PROTECTED, idx);\n\t\t\t\tself.push_back(PROTECTED, idx);\n\t\t\t}\n\t\t}\n\t}\n\n\tfn list_mut(&mut self, list: u8) -> (&mut Option<usize>, &mut Option<usize>, &mut usize) {\n\t\tmatch list {\n\t\t\tWINDOW => (&mut self.win_head, &mut self.win_tail, &mut self.win_len),\n\t\t\tPROBATION => (&mut self.prob_head, &mut self.prob_tail, &mut self.prob_len),\n\t\t\t_ => (&mut self.prot_head, &mut self.prot_tail, &mut self.prot_len),\n\t\t}\n\t}\n\n\tfn unlink(&mut self, list: u8, idx: usize) {\n\t\tlet prev = self.nodes[idx].prev;\n\t\tlet next = self.nodes[idx].next;\n\t\tif let Some(p) = prev {\n\t\t\tself.nodes[p].next = next;\n\t\t}\n\t\tif let Some(n) = next {\n\t\t\tself.nodes[n].prev = prev;\n\t\t}\n\t\tself.nodes[idx].prev = None;\n\t\tself.nodes[idx].next = None;\n\t\tlet (head, tail, len) = self.list_mut(list);\n\t\tif prev.is_none() {\n\t\t\t*head = next;\n\t\t}\n\t\tif next.is_none() {\n\t\t\t*tail = prev;\n\t\t}\n\t\t*len -= 1;\n\t}\n\n\tfn push_back(&mut self, list: u8, idx: usize) {\n\t\tlet old_tail = {\n\t\t\tlet (_, tail, _) = self.list_mut(list);\n\t\t\t*tail\n\t\t};\n\t\tself.nodes[idx].prev = old_tail;\n\t\tself.nodes[idx].next = None;\n\t\tself.nodes[idx].seg = list;\n\t\tif let Some(t) = old_tail {\n\t\t\tself.nodes[t].next = Some(idx);\n\t\t}\n\t\tlet (head, tail, len) = self.list_mut(list);\n\t\tif old_tail.is_none() {\n\t\t\t*head = Some(idx);\n\t\t}\n\t\t*tail = Some(idx);\n\t\t*len += 1;\n\t}\n}\n\n#[derive(Debug)]\nstruct CountMinSketch {\n\twidth: usize,\n\tdepth: usize,\n\tcounts: Vec<Vec<u32>>,\n}\n\nimpl CountMinSketch {\n\tfn new(width: usize) -> Self {\n\t\tlet depth = SKETCH_DEPTH;\n\t\tlet counts = vec![vec![0; width]; depth];\n\t\tSelf { width, depth, counts }\n\t}\n\n\t/// Heap bytes backing the counter table: `depth * width * size_of::<u32>()`.\n\tfn heap_bytes(&self) -> usize {\n\t\tself.depth * self.width * std::mem::size_of::<u32>()\n\t}\n\n\tfn increment<K: Hash>(&mut self, key: &K) {\n\t\tfor row in 0..self.depth {\n\t\t\tlet pos = self.position(key, row as u64);\n\t\t\tlet counter = &mut self.counts[row][pos];\n\t\t\tif *counter < MAX_COUNT {\n\t\t\t\t*counter += 1;\n\t\t\t}\n\t\t}\n\t}\n\n\tfn estimate<K: Hash>(&self, key: &K) -> u32 {\n\t\tself\n\t\t\t.counts\n\t\t\t.iter()\n\t\t\t.enumerate()\n\t\t\t.map(|(row, _)| self.counts[row][self.position(key, row as u64)])\n\t\t\t.min()\n\t\t\t.unwrap_or(0)\n\t}\n\n\tfn position<K: Hash>(&self, key: &K, seed: u64) -> usize {\n\t\tlet mut hasher = DefaultHasher::new();\n\t\thasher.write_u64(seed);\n\t\tkey.hash(&mut hasher);\n\t\t(hasher.finish() as usize) & (self.width - 1)\n\t}\n}\n\nimpl<K, V> framework::Cache<K, V> for TinyLfuCache<K, V>\nwhere\n\tK: Hash + Eq + Clone + Debug,\n\tV: Debug,\n{\n\tfn put(&mut self, key: K, value: V) {\n\t\tTinyLfuCache::put(self, key, value);\n\t}\n\n\tfn get(&mut self, key: &K) -> Option<&V> {\n\t\tTinyLfuCache::get(self, key)\n\t}\n}\n\n#[cfg(test)]\nmod tests {\n\tuse super::*;\n\n\tfn check_structure(cache: &TinyLfuCache<i64, i64>) {\n\t\tassert_eq!(cache.map.len(), cache.nodes.len());\n\t\tassert!(cache.nodes.len() <= cache.capacity);\n\t\tassert_eq!(cache.win_len + cache.prob_len + cache.prot_len, cache.nodes.len());\n\t\tassert!(cache.win_len <= cache.window_cap);\n\t\tassert!(cache.prot_len <= cache.protected_cap);\n\t\tfor (list, head, len) in [\n\t\t\t(WINDOW, cache.win_head, cache.win_len),\n\t\t\t(PROBATION, cache.prob_head, cache.prob_len),\n\t\t\t(PROTECTED, cache.prot_head, cache.prot_len),\n\t\t] {\n\t\t\tlet mut n = 0;\n\t\t\tlet mut cur = head;\n\t\t\twhile let Some(i) = cur {\n\t\t\t\tassert_eq!(cache.nodes[i].seg, list);\n\t\t\t\tassert_eq!(cache.map[&cache.nodes[i].key], i);\n\t\t\t\tn += 1;\n\t\t\t\tcur = cache.nodes[i].next;\n\t\t\t}\n\t\t\tassert_eq!(n, len);\n\t\t}\n\t}\n\n\t#[test]\n\tfn get_miss_returns_none() {\n\t\tlet mut cache = TinyLfuCache::<i64, i64>::new(4);\n\t\tassert!(cache.get(&42).is_none());\n\t}\n\n\t#[test]\n\tfn put_then_get() {\n\t\tlet mut cache = TinyLfuCache::new(4);\n\t\tcache.put(1, 10);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 10);\n\t}\n\n\t#[test]\n\tfn eviction_prefers_hot_items() {\n\t\t// capacity 2: window=1, main=1.\n\t\tlet mut cache = TinyLfuCache::new(2);\n\t\tcache.put(1, 10);\n\t\tcache.put(2, 20); // window overflow: 1 -> probation (main has room)\n\t\tcache.get(&1); // make key 1 hotter than key 2\n\t\tcache.get(&3);\n\t\tcache.get(&3);\n\t\tcache.put(3, 30); // contest: window evictee 2 vs probation victim 1 -> 2 loses\n\t\tcheck_structure(&cache);\n\n\t\tassert_eq!(*cache.get(&1).unwrap(), 10);\n\t\tassert_eq!(*cache.get(&3).unwrap(), 30);\n\t\tassert!(cache.get(&2).is_none());\n\t}\n\n\t#[test]\n\tfn new_item_is_resident_and_contest_runs_at_window_overflow() {\n\t\tlet mut cache = TinyLfuCache::new(2);\n\t\tcache.put(1, 10);\n\t\tcache.put(2, 20);\n\t\tcache.get(&1);\n\t\tcache.put(3, 30); // low-frequency newcomer is still admitted to the window\n\t\tcheck_structure(&cache);\n\n\t\tassert!(cache.get(&3).is_some());\n\t\tassert!(cache.get(&1).is_some());\n\t\tassert!(cache.get(&2).is_none()); // window evictee lost the contest\n\t}\n\n\t#[test]\n\tfn window_evictee_beats_cold_main_victim() {\n\t\tlet mut cache = TinyLfuCache::new(2);\n\t\tcache.put(1, 10);\n\t\tcache.put(2, 20); // 1 -> probation\n\t\tcache.get(&2);\n\t\tcache.get(&2);\n\t\tcache.put(3, 30); // evictee 2 (hot) beats victim 1 (cold)\n\t\tcheck_structure(&cache);\n\n\t\tassert!(cache.get(&1).is_none());\n\t\tassert_eq!(*cache.get(&2).unwrap(), 20);\n\t\tassert_eq!(*cache.get(&3).unwrap(), 30);\n\t}\n\n\t#[test]\n\tfn capacity_one_drops_window_evictee() {\n\t\tlet mut cache = TinyLfuCache::new(1);\n\t\tcache.put(1, 10);\n\t\tcache.put(2, 20);\n\t\tcheck_structure(&cache);\n\t\tassert!(cache.get(&1).is_none());\n\t\tassert_eq!(*cache.get(&2).unwrap(), 20);\n\t}\n\n\t#[test]\n\tfn update_promotes_existing_item_and_admits_hot_arrivals() {\n\t\tlet mut cache = TinyLfuCache::new(2);\n\t\tcache.put(1, 10);\n\t\tcache.put(2, 20);\n\t\tcache.put(1, 11);\n\t\tcache.get(&3);\n\t\tcache.get(&3);\n\t\tcache.put(3, 30);\n\t\tcheck_structure(&cache);\n\n\t\tassert_eq!(*cache.get(&1).unwrap(), 11);\n\t\tassert_eq!(*cache.get(&3).unwrap(), 30);\n\t\tassert!(cache.get(&2).is_none());\n\t}\n\n\t#[test]\n\tfn structure_holds_under_mixed_workload() {\n\t\tfor cap in [1usize, 2, 3, 8, 150, 300] {\n\t\t\tlet mut cache = TinyLfuCache::new(cap);\n\t\t\tlet mut x: u64 = 0x9e3779b97f4a7c15;\n\t\t\tfor _ in 0..5000 {\n\t\t\t\tx ^= x << 13;\n\t\t\t\tx ^= x >> 7;\n\t\t\t\tx ^= x << 17;\n\t\t\t\tlet k = (x % (cap as u64 * 3 + 5)) as i64;\n\t\t\t\tif cache.get(&k).is_none() {\n\t\t\t\t\tcache.put(k, k * 10);\n\t\t\t\t\tassert_eq!(*cache.get(&k).unwrap(), k * 10);\n\t\t\t\t}\n\t\t\t\tcheck_structure(&cache);\n\t\t\t}\n\t\t}\n\t}\n}\n"
}
```

### 09:09:28 thinking



### 09:09:28 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/tiny_lfu/Cargo.toml"
}
```

### 09:09:30 thinking



### 09:09:30 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/tiny_lfu/src/tiny_lfu/tiny_lfu_jni.rs"
}
```

### 09:09:40 thinking



### 09:09:42 chat

I wrote the W-TinyLFU variant v001 as one overlay file, but I couldn't build it or run any gates or tests. This session had no shell, so compile errors or gate failures would only show up in the harness run.

**What changed:** only `overlay/src/tiny_lfu/mod.rs`. The public API (`try_new`, `new`, `get`, `put`, `sketch_heap_bytes` and the `framework::Cache` impl) keeps its signatures.

- **Three lists instead of one:** each node gets a `seg` tag (window, probation or protected). The window, probation and protected lists each have their own head, tail and length in the same arena. `unlink` and `push_back` now take the list to update as an argument.
- **Sizing:** `window_cap = max(1, cap/100)`, `main_cap = cap - window_cap`, `protected_cap = main_cap*80/100`. A capacity of 1 gets a window of 1 and no main region. The sketch width formula is unchanged.
- **Hits (`get`, and `put` on an existing key after updating the value):**
  - A window hit moves the key to the window's most-recent end.
  - A probation hit promotes the key to protected. If protected is then over its limit, its least-recent key drops back to probation.
  - A protected hit moves the key to protected's most-recent end.
- **New keys:** a new key always goes into the window, so it is resident right after `put`.
  - **Cache not yet full:** the arena grows by one slot. If the window overflows, its oldest key moves to probation; the main region always has room at this point.
  - **Cache full:** the window's oldest key competes with the main region's victim (probation's oldest, or protected's oldest if probation is empty). The candidate wins only with a strictly higher frequency estimate; whichever loses is evicted. The new key goes into the freed slot, so the arena never grows past capacity and no free list is needed.
  - **Capacity 1:** the old key is simply dropped.
- **Unit tests:**
  - `low_frequency_new_item_is_rejected` is replaced by `new_item_is_resident_and_contest_runs_at_window_overflow`.
  - `eviction_prefers_hot_items` and `update_promotes_existing_item_and_admits_hot_arrivals` keep their assertions; I traced both by hand and they should still pass under the new rules, but haven't run them.
  - New tests cover a window candidate beating a cold victim, the capacity-1 drop, and a randomised workload at several capacities. That workload checks after every operation that the map and arena match, nothing exceeds capacity, and every key is in exactly one list.

Two code paths can't be reached: one for a full cache whose main region still has room, and one for a contest while the cache isn't full. I added fallbacks for them that keep the arena size and the map consistent anyway.


---

- finished: 2026-10-21T09:09:43+00:00
- duration: 96.59s
- status: ok
- input_tokens: 20
- output_tokens: 11461
- cache_read_tokens: 273582
- cache_write_tokens: 25193
- cost_usd: 0.48556
