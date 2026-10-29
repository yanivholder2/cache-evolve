# Agent log: variant-implementer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0002/variants/v003/attempts/01
- allow_edits: True
- started: 2026-10-28T13:00:50+00:00

## timeline

### 13:00:54 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355; cat cycles/0002/hypotheses/H03.md; P=cycles/0001/variants/v001; find $P/attempts/01/overlay -type f; cat $P/insight.md $P/eval/fitness-vector.json; cat /home/user/cache-evolve/inputs/subjects/tiny_lfu/manifest.toml; find /home/user/cache-evolve/inputs/subjects/tiny_lfu/src -type f | xargs wc -l"
}
```

### 13:00:56 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0002/hypotheses/H03.md"
}
```

### 13:00:56 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/tiny_lfu/manifest.toml"
}
```

### 13:00:56 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0001/variants/v001/insight.md"
}
```

### 13:00:58 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0001/variants/v001/attempts/01/overlay"
}
```

### 13:00:58 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/inputs/subjects/tiny_lfu/src"
}
```

### 13:01:00 thinking



### 13:01:00 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0001/variants/v001/attempts/01/overlay/src/tiny_lfu/mod.rs"
}
```

### 13:01:02 thinking



### 13:01:03 tool: Grep

```json
{
  "pattern": "window_cap|main_cap|win\\.|\\.main\\b|nodes|capacity",
  "path": "/home/user/cache-evolve/inputs/subjects/tiny_lfu",
  "output_mode": "content",
  "glob": "!src/tiny_lfu/mod.rs"
}
```

### 13:01:04 thinking



### 13:01:04 chat

I'm reading the subject's invariant helpers next. Then I'll write the overlay for the adaptive window.

### 13:01:05 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/tiny_lfu/src/test_utils/mod.rs"
}
```

### 13:01:06 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/tiny_lfu/src/test_utils/invariants.rs"
}
```

### 13:01:30 thinking



### 13:02:14 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0002/variants/v003/attempts/01/overlay/src/tiny_lfu/mod.rs",
  "content": "use std::collections::hash_map::DefaultHasher;\nuse std::collections::HashMap;\nuse std::fmt::Debug;\nuse std::hash::{Hash, Hasher};\n\npub mod tiny_lfu_jni;\n\nconst SKETCH_DEPTH: usize = 4;\nconst SKETCH_WIDTH_FACTOR: usize = 4;\nconst MIN_SKETCH_WIDTH: usize = 16;\nconst MAX_COUNT: u32 = u32::MAX;\n/// Initial window size as a percentage of total capacity (W-TinyLFU admission window).\n/// The window is resized afterwards by the hill climber (see `climb`).\nconst WINDOW_PERCENT: usize = 1;\n/// Initial hill-climber step as a percentage of capacity.\nconst INITIAL_STEP_PERCENT: isize = 5;\n/// Upper bound on the window as a percentage of capacity.\nconst MAX_WINDOW_PERCENT: isize = 80;\n/// Per-climb multiplicative decay of the step magnitude.\nconst STEP_DECAY: f64 = 0.98;\n/// Hit-rate drop (absolute) that triggers a direction reversal.\nconst REVERSE_TOLERANCE: f64 = 0.001;\n/// Minimum sampling period in `get` calls.\nconst MIN_SAMPLE_PERIOD: usize = 64;\n\n/// Which recency list a node currently lives in.\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\nenum Region {\n\tWindow,\n\tMain,\n}\n\n#[derive(Debug)]\npub struct Node<K, V> {\n\tkey: K,\n\tvalue: V,\n\tprev: Option<usize>,\n\tnext: Option<usize>,\n\tregion: Region,\n}\n\n/// Head/tail/len of one intrusive recency list over the shared `nodes` arena.\n/// Head is LRU, tail is MRU.\n#[derive(Debug, Default, Clone, Copy)]\nstruct List {\n\thead: Option<usize>,\n\ttail: Option<usize>,\n\tlen: usize,\n}\n\n#[derive(Debug)]\npub struct TinyLfuCache<K, V> {\n\tmap: HashMap<K, usize>,\n\tnodes: Vec<Node<K, V>>,\n\twin: List,\n\tmain: List,\n\twindow_cap: usize,\n\tmain_cap: usize,\n\tcapacity: usize,\n\tsketch: CountMinSketch,\n\t// Hill-climber state.\n\tsample_hits: u64,\n\tsample_count: u64,\n\tprev_hit_rate: f64,\n\tstep: isize,\n\tsample_period: u64,\n}\n\nimpl<K, V> TinyLfuCache<K, V>\nwhere\n\tK: Hash + Eq + Clone + Debug,\n\tV: Debug,\n{\n\tpub fn try_new(capacity: usize) -> Option<Self> {\n\t\tif capacity == 0 {\n\t\t\tlog::error!(\"TinyLfuCache.try_new: capacity must be > 0\");\n\t\t\treturn None;\n\t\t}\n\t\tlet width = (capacity * SKETCH_WIDTH_FACTOR)\n\t\t\t.max(MIN_SKETCH_WIDTH)\n\t\t\t.next_power_of_two();\n\t\tlet window_cap = (capacity * WINDOW_PERCENT / 100).max(1).min(capacity);\n\t\tlet main_cap = capacity - window_cap;\n\t\tlet step = (capacity as isize * INITIAL_STEP_PERCENT / 100).max(1);\n\t\tlet sample_period = capacity.max(MIN_SAMPLE_PERIOD) as u64;\n\t\tlog::info!(\n\t\t\t\"TinyLfuCache.try_new: capacity={capacity} window_cap={window_cap} main_cap={main_cap} sketch_width={width} step={step} sample_period={sample_period}\"\n\t\t);\n\t\tSome(Self {\n\t\t\tmap: HashMap::with_capacity(capacity),\n\t\t\tnodes: Vec::with_capacity(capacity),\n\t\t\twin: List::default(),\n\t\t\tmain: List::default(),\n\t\t\twindow_cap,\n\t\t\tmain_cap,\n\t\t\tcapacity,\n\t\t\tsketch: CountMinSketch::new(width),\n\t\t\tsample_hits: 0,\n\t\t\tsample_count: 0,\n\t\t\tprev_hit_rate: 0.0,\n\t\t\tstep,\n\t\t\tsample_period,\n\t\t})\n\t}\n\n\tpub fn new(capacity: usize) -> Self {\n\t\tSelf::try_new(capacity).expect(\"TinyLfuCache::new: initialization failed\")\n\t}\n\n\t/// Heap bytes held by the frequency sketch for this cache: `depth * width * size_of::<u32>()`.\n\t///\n\t/// This is the FIXED metadata overhead TinyLfu carries beyond the per-entry nodes — it is\n\t/// sized at construction from capacity (see `try_new`) and does not grow with population.\n\t/// The metadata-size fitness bin reports it so variant edits to the sketch geometry are visible.\n\tpub fn sketch_heap_bytes(&self) -> usize {\n\t\tself.sketch.heap_bytes()\n\t}\n\n\tpub fn get(&mut self, key: &K) -> Option<&V> {\n\t\tself.sketch.increment(key);\n\t\tlet hit = self.map.get(key).copied();\n\n\t\tself.sample_count += 1;\n\t\tif hit.is_some() {\n\t\t\tself.sample_hits += 1;\n\t\t}\n\t\tif self.sample_count >= self.sample_period {\n\t\t\tself.climb();\n\t\t\tself.sample_hits = 0;\n\t\t\tself.sample_count = 0;\n\t\t}\n\n\t\tlet idx = hit?;\n\t\tself.promote(idx);\n\t\tSome(&self.nodes[idx].value)\n\t}\n\n\t/// Hill-climbing step: compare this sample's hit rate to the previous one, reverse the\n\t/// direction on a drop, decay the step, and retarget `window_cap`/`main_cap`. Nodes are\n\t/// not moved here; the lists rebalance lazily in `put`/`evict_for_insert`.\n\tfn climb(&mut self) {\n\t\tif self.capacity < 2 || self.sample_count == 0 {\n\t\t\treturn;\n\t\t}\n\t\tlet hr = self.sample_hits as f64 / self.sample_count as f64;\n\t\tif hr < self.prev_hit_rate - REVERSE_TOLERANCE {\n\t\t\tself.step = -self.step;\n\t\t}\n\t\tlet sign: isize = if self.step < 0 { -1 } else { 1 };\n\t\tlet mag = ((self.step.unsigned_abs() as f64) * STEP_DECAY) as isize;\n\t\tself.step = sign * mag.max(1);\n\n\t\tlet cap = self.capacity as isize;\n\t\tlet upper = (cap * MAX_WINDOW_PERCENT / 100).min(cap - 1).max(1);\n\t\tlet target = (self.window_cap as isize + self.step).clamp(1, upper);\n\t\tself.window_cap = target as usize;\n\t\tself.main_cap = self.capacity - self.window_cap;\n\t\tself.prev_hit_rate = hr;\n\t}\n\n\tpub fn put(&mut self, key: K, value: V) {\n\t\tself.sketch.increment(&key);\n\t\tif let Some(&idx) = self.map.get(&key) {\n\t\t\tself.nodes[idx].value = value;\n\t\t\tself.promote(idx);\n\t\t\treturn;\n\t\t}\n\n\t\tif self.nodes.len() < self.capacity {\n\t\t\t// Not full: new node goes to window MRU; spill window LRU to main while the window\n\t\t\t// is over its (possibly retargeted) cap and main has room.\n\t\t\tlet idx = self.nodes.len();\n\t\t\tself.nodes.push(Node {\n\t\t\t\tkey: key.clone(),\n\t\t\t\tvalue,\n\t\t\t\tprev: None,\n\t\t\t\tnext: None,\n\t\t\t\tregion: Region::Window,\n\t\t\t});\n\t\t\tself.push_back(idx, Region::Window);\n\t\t\tself.map.insert(key, idx);\n\t\t\twhile self.win.len > self.window_cap && self.main.len < self.main_cap {\n\t\t\t\tlet cand = self.win.head.expect(\"TinyLfuCache: overflowing window is empty\");\n\t\t\t\tself.unlink(cand);\n\t\t\t\tself.push_back(cand, Region::Main);\n\t\t\t}\n\t\t\treturn;\n\t\t}\n\n\t\t// Full: free one slot (lazy rebalance + admission step), then reuse that slot for the\n\t\t// newcomer at window MRU.\n\t\tlet freed = self.evict_for_insert();\n\t\tself.nodes[freed].key = key.clone();\n\t\tself.nodes[freed].value = value;\n\t\tself.push_back(freed, Region::Window);\n\t\tself.map.insert(key, freed);\n\t}\n\n\t/// Called when the cache is full. Evicts exactly one key and returns its (unlinked) slot.\n\t///\n\t/// - `win.len < window_cap` (main over its cap): evict main LRU directly; the window grows\n\t///   by one through the newcomer insert.\n\t/// - `win.len > window_cap` (main under its cap): spill the window LRU into main (main has\n\t///   room), then run the usual contest with the new window LRU as candidate. Net effect: the\n\t///   window shrinks by one per insert until it reaches its cap.\n\t/// - `win.len == window_cap`: v001's contest unchanged.\n\tfn evict_for_insert(&mut self) -> usize {\n\t\tif self.win.len < self.window_cap {\n\t\t\tif let Some(v) = self.main.head {\n\t\t\t\tself.evict(v);\n\t\t\t\treturn v;\n\t\t\t}\n\t\t}\n\n\t\tif self.win.len > self.window_cap && self.main.len < self.main_cap {\n\t\t\t// win.len > window_cap >= 1, so window keeps at least one node after this spill.\n\t\t\tlet spill = self.win.head.expect(\"TinyLfuCache: overflowing window is empty\");\n\t\t\tself.unlink(spill);\n\t\t\tself.push_back(spill, Region::Main);\n\t\t}\n\n\t\tlet cand = match self.win.head {\n\t\t\tSome(c) => c,\n\t\t\t// Defensive: window empty; evict main LRU.\n\t\t\tNone => {\n\t\t\t\tlet v = self.main.head.expect(\"TinyLfuCache: full cache has no victim\");\n\t\t\t\tself.evict(v);\n\t\t\t\treturn v;\n\t\t\t}\n\t\t};\n\n\t\tlet victim = match self.main.head {\n\t\t\tSome(v) => v,\n\t\t\tNone => {\n\t\t\t\t// main_cap == 0 (capacity 1) or main empty: no contest, candidate leaves.\n\t\t\t\tself.evict(cand);\n\t\t\t\treturn cand;\n\t\t\t}\n\t\t};\n\n\t\tlet cand_freq = self.sketch.estimate(&self.nodes[cand].key);\n\t\tlet victim_freq = self.sketch.estimate(&self.nodes[victim].key);\n\t\tif cand_freq > victim_freq {\n\t\t\tself.evict(victim);\n\t\t\tself.unlink(cand);\n\t\t\tself.push_back(cand, Region::Main);\n\t\t\tvictim\n\t\t} else {\n\t\t\tself.evict(cand);\n\t\t\tcand\n\t\t}\n\t}\n\n\t/// Removes `idx` from its list and from the map. The slot stays in the arena for reuse.\n\tfn evict(&mut self, idx: usize) {\n\t\tself.unlink(idx);\n\t\tlet k = self.nodes[idx].key.clone();\n\t\tself.map.remove(&k);\n\t}\n\n\t/// Moves `idx` to the MRU end of whichever list it currently lives in.\n\tfn promote(&mut self, idx: usize) {\n\t\tlet region = self.nodes[idx].region;\n\t\tself.unlink(idx);\n\t\tself.push_back(idx, region);\n\t}\n\n\tfn list_mut(&mut self, region: Region) -> &mut List {\n\t\tmatch region {\n\t\t\tRegion::Window => &mut self.win,\n\t\t\tRegion::Main => &mut self.main,\n\t\t}\n\t}\n\n\tfn unlink(&mut self, idx: usize) {\n\t\tlet region = self.nodes[idx].region;\n\t\tlet prev = self.nodes[idx].prev;\n\t\tlet next = self.nodes[idx].next;\n\t\tmatch prev {\n\t\t\tSome(p) => self.nodes[p].next = next,\n\t\t\tNone => self.list_mut(region).head = next,\n\t\t}\n\t\tmatch next {\n\t\t\tSome(n) => self.nodes[n].prev = prev,\n\t\t\tNone => self.list_mut(region).tail = prev,\n\t\t}\n\t\tself.list_mut(region).len -= 1;\n\t\tself.nodes[idx].prev = None;\n\t\tself.nodes[idx].next = None;\n\t}\n\n\tfn push_back(&mut self, idx: usize, region: Region) {\n\t\tlet tail = self.list_mut(region).tail;\n\t\tself.nodes[idx].region = region;\n\t\tself.nodes[idx].prev = tail;\n\t\tself.nodes[idx].next = None;\n\t\tmatch tail {\n\t\t\tSome(t) => self.nodes[t].next = Some(idx),\n\t\t\tNone => self.list_mut(region).head = Some(idx),\n\t\t}\n\t\tlet list = self.list_mut(region);\n\t\tlist.tail = Some(idx);\n\t\tlist.len += 1;\n\t}\n}\n\n#[derive(Debug)]\nstruct CountMinSketch {\n\twidth: usize,\n\tdepth: usize,\n\tcounts: Vec<Vec<u32>>,\n}\n\nimpl CountMinSketch {\n\tfn new(width: usize) -> Self {\n\t\tlet depth = SKETCH_DEPTH;\n\t\tlet counts = vec![vec![0; width]; depth];\n\t\tSelf { width, depth, counts }\n\t}\n\n\t/// Heap bytes backing the counter table: `depth * width * size_of::<u32>()`.\n\tfn heap_bytes(&self) -> usize {\n\t\tself.depth * self.width * std::mem::size_of::<u32>()\n\t}\n\n\tfn increment<K: Hash>(&mut self, key: &K) {\n\t\tfor row in 0..self.depth {\n\t\t\tlet pos = self.position(key, row as u64);\n\t\t\tlet counter = &mut self.counts[row][pos];\n\t\t\tif *counter < MAX_COUNT {\n\t\t\t\t*counter += 1;\n\t\t\t}\n\t\t}\n\t}\n\n\tfn estimate<K: Hash>(&self, key: &K) -> u32 {\n\t\tself\n\t\t\t.counts\n\t\t\t.iter()\n\t\t\t.enumerate()\n\t\t\t.map(|(row, _)| self.counts[row][self.position(key, row as u64)])\n\t\t\t.min()\n\t\t\t.unwrap_or(0)\n\t}\n\n\tfn position<K: Hash>(&self, key: &K, seed: u64) -> usize {\n\t\tlet mut hasher = DefaultHasher::new();\n\t\thasher.write_u64(seed);\n\t\tkey.hash(&mut hasher);\n\t\t(hasher.finish() as usize) & (self.width - 1)\n\t}\n}\n\nimpl<K, V> framework::Cache<K, V> for TinyLfuCache<K, V>\nwhere\n\tK: Hash + Eq + Clone + Debug,\n\tV: Debug,\n{\n\tfn put(&mut self, key: K, value: V) {\n\t\tTinyLfuCache::put(self, key, value);\n\t}\n\n\tfn get(&mut self, key: &K) -> Option<&V> {\n\t\tTinyLfuCache::get(self, key)\n\t}\n}\n\n#[cfg(test)]\nmod tests {\n\tuse super::*;\n\n\t#[test]\n\tfn get_miss_returns_none() {\n\t\tlet mut cache = TinyLfuCache::<i64, i64>::new(4);\n\t\tassert!(cache.get(&42).is_none());\n\t}\n\n\t#[test]\n\tfn put_then_get() {\n\t\tlet mut cache = TinyLfuCache::new(4);\n\t\tcache.put(1, 10);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 10);\n\t}\n\n\t#[test]\n\tfn eviction_prefers_hot_items() {\n\t\tlet mut cache = TinyLfuCache::new(2);\n\t\tcache.put(1, 10);\n\t\tcache.put(2, 20);\n\t\tcache.get(&1); // make key 1 hotter than key 2\n\t\tcache.get(&3);\n\t\tcache.get(&3);\n\t\tcache.put(3, 30);\n\n\t\tassert_eq!(*cache.get(&1).unwrap(), 10);\n\t\tassert_eq!(*cache.get(&3).unwrap(), 30);\n\t\tassert!(cache.get(&2).is_none());\n\t}\n\n\t#[test]\n\tfn newcomer_enters_window_and_cold_candidate_loses_contest() {\n\t\t// capacity 2: window=1, main=1. After puts 1,2: main=[1], window=[2].\n\t\tlet mut cache = TinyLfuCache::new(2);\n\t\tcache.put(1, 10);\n\t\tcache.put(2, 20);\n\t\tcache.get(&1);\n\t\tcache.put(3, 30);\n\n\t\t// Newcomer 3 is admitted unconditionally into the window; the window evictee (2)\n\t\t// loses the contest against the hotter main victim (1).\n\t\tassert!(cache.get(&2).is_none());\n\t\tassert_eq!(*cache.get(&3).unwrap(), 30);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 10);\n\t}\n\n\t#[test]\n\tfn hot_candidate_wins_contest() {\n\t\tlet mut cache = TinyLfuCache::new(2);\n\t\tcache.put(1, 10);\n\t\tcache.put(2, 20); // main=[1], window=[2]\n\t\tcache.get(&2);\n\t\tcache.get(&2); // 2 hotter than 1\n\t\tcache.put(3, 30); // candidate 2 beats victim 1\n\n\t\tassert!(cache.get(&1).is_none());\n\t\tassert_eq!(*cache.get(&2).unwrap(), 20);\n\t\tassert_eq!(*cache.get(&3).unwrap(), 30);\n\t}\n\n\t#[test]\n\tfn capacity_one_keeps_most_recent() {\n\t\tlet mut cache = TinyLfuCache::new(1);\n\t\tcache.put(1, 10);\n\t\tcache.put(2, 20);\n\t\tassert!(cache.get(&1).is_none());\n\t\tassert_eq!(*cache.get(&2).unwrap(), 20);\n\t}\n\n\t#[test]\n\tfn update_promotes_existing_item_and_admits_hot_arrivals() {\n\t\tlet mut cache = TinyLfuCache::new(2);\n\t\tcache.put(1, 10);\n\t\tcache.put(2, 20);\n\t\tcache.put(1, 11);\n\t\tcache.get(&3);\n\t\tcache.get(&3);\n\t\tcache.put(3, 30);\n\n\t\tassert_eq!(*cache.get(&1).unwrap(), 11);\n\t\tassert_eq!(*cache.get(&3).unwrap(), 30);\n\t\tassert!(cache.get(&2).is_none());\n\t}\n\n\tfn check_structure(cache: &TinyLfuCache<i64, i64>) {\n\t\tassert!(cache.win.len + cache.main.len <= cache.capacity);\n\t\tassert_eq!(cache.win.len + cache.main.len, cache.map.len());\n\t\tassert!(cache.window_cap >= 1);\n\t\tassert_eq!(cache.window_cap + cache.main_cap, cache.capacity);\n\t\tif cache.capacity >= 2 {\n\t\t\tassert!(cache.window_cap <= cache.capacity - 1);\n\t\t}\n\t}\n\n\t#[test]\n\tfn adaptive_window_keeps_structure_sound() {\n\t\tfor &cap in &[1usize, 2, 3, 8, 100] {\n\t\t\tlet mut cache = TinyLfuCache::<i64, i64>::new(cap);\n\t\t\t// Phase-shifting workload so the climber moves in both directions.\n\t\t\tlet mut x: u64 = 0x9E37_79B9_7F4A_7C15;\n\t\t\tfor i in 0..20_000i64 {\n\t\t\t\tx ^= x << 13;\n\t\t\t\tx ^= x >> 7;\n\t\t\t\tx ^= x << 17;\n\t\t\t\tlet key = if (i / 2_000) % 2 == 0 {\n\t\t\t\t\t(x % (cap as u64 * 3 + 1)) as i64\n\t\t\t\t} else {\n\t\t\t\t\ti % (cap as i64 * 2 + 5)\n\t\t\t\t};\n\t\t\t\tif cache.get(&key).is_none() {\n\t\t\t\t\tcache.put(key, key * 10);\n\t\t\t\t}\n\t\t\t\tcheck_structure(&cache);\n\t\t\t}\n\t\t\tfor (k, &idx) in &cache.map {\n\t\t\t\tassert_eq!(cache.nodes[idx].key, *k);\n\t\t\t\tassert_eq!(cache.nodes[idx].value, *k * 10);\n\t\t\t}\n\t\t}\n\t}\n}\n"
}
```

### 13:02:17 thinking



### 13:02:17 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0001/variants/v001/attempts/01/overlay/src/factory.rs"
}
```

### 13:02:25 thinking



### 13:02:26 chat

I've written the v003 overlay, but it hasn't been compiled or run against any gate. This session only had file read/write tools, so the gates are the first real check. It is a single file, `overlay/src/tiny_lfu/mod.rs`, built on v001. v001 had no `src/factory.rs` overlay, so the stock constructor is used.

**What changed (H03):** the window is no longer fixed at 1%; it resizes itself based on hit rate.
- **Counting:** `get` tracks hits and calls. Every `max(capacity, 64)` calls, a new `climb()` method compares this period's hit rate with the last one.
- **Steering:** if the hit rate dropped by more than 0.001, the step reverses direction. Each round the step shrinks by 0.98, with a floor of 1. It starts at +5% of capacity, so the window grows first.
- **Limits:** the window stays between 1 and the smaller of 80% of capacity and capacity − 1, and the main region gets the rest. With capacity 1 the climber does nothing.
- **Lazy rebalancing:** nodes don't move when the caps change.
  - While the cache isn't full, `put` moves the oldest window entries into main while the window is over its cap and main has room.
  - When full and the window is under its cap, the oldest main entry is evicted with no contest.
  - When the window equals its cap, the contest runs exactly as in v001.
- **Kept from v001:** one main LRU, the u32 sketch with no aging, and the strict `>` contest.

**One departure from the spec:** for a full cache with the window over its cap, the spec says to run the normal contest. But that contest frees one slot and the newcomer takes it back, so both regions stay the same size and the window never shrinks. So I first move the oldest window entry into main, which has room in that case, and then run the contest on the next oldest. The window shrinks by one per insert until it reaches its cap. Without this, the climber's shrink decisions would have no effect.

I also added a unit test that runs a shifting workload at capacities 1, 2, 3, 8 and 100. It checks list sizes against capacity and the map, the cap bounds, and that every stored value is correct. Note that the invariant gate uses capacity 8, so it will run the climber: its sample period is 64 calls, and the gate makes 200+ `get`s.


---

- finished: 2026-10-28T13:02:26+00:00
- duration: 96.34s
- status: ok
- input_tokens: 20
- output_tokens: 11465
- cache_read_tokens: 277945
- cache_write_tokens: 25015
- cost_usd: 0.485089
