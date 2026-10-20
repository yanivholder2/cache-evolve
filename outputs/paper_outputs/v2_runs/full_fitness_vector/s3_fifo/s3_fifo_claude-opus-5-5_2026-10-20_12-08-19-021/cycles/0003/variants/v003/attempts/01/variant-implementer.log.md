# Agent log: variant-implementer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0003/variants/v003/attempts/01
- allow_edits: True
- started: 2026-10-20T12:26:39+00:00

## timeline

### 12:26:43 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021; cat cycles/0003/hypotheses/H03.md; P=cycles/0002/variants/v003; find $P/attempts/01/overlay -type f; cat $P/insight.md $P/eval/fitness-vector.json; ls /home/user/cache-evolve/inputs/subjects/s3_fifo/src -R"
}
```

### 12:26:46 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0003/hypotheses/H03.md"
}
```

### 12:26:46 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0002/variants/v003/insight.md"
}
```

### 12:26:48 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0002/variants/v003/attempts/01/overlay"
}
```

### 12:26:50 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0002/variants/v003/attempts/01/overlay/src/s3_fifo/mod.rs"
}
```

### 12:27:00 thinking



### 12:27:00 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/s3_fifo/tests/invariant_gate.rs"
}
```

### 12:27:01 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0002/variants/v003/attempts/01/overlay/src/factory.rs"
}
```

### 12:27:04 thinking



### 12:27:04 chat

I've read the hypothesis and the parent's `mod.rs`. Next I'm checking the invariant suite for edge capacities, then I'll write the overlay.

### 12:27:05 tool: Grep

```json
{
  "pattern": "new\\(|capacity|cap",
  "path": "/home/user/cache-evolve/inputs/subjects/s3_fifo/src/test_utils",
  "output_mode": "content",
  "-n": "true"
}
```

### 12:27:06 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/s3_fifo/src/test_utils/mod.rs"
}
```

### 12:27:10 thinking



### 12:27:10 chat

Now I'll write the overlay with the fixed split, the freq ≥ 1 rule, and the reset on promotion.

### 12:27:52 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0003/variants/v003/attempts/01/overlay/src/s3_fifo/mod.rs",
  "content": "use std::collections::{HashMap, VecDeque};\nuse std::fmt::Debug;\nuse std::hash::{BuildHasher, Hash, Hasher};\n\npub mod s3_fifo_jni;\n\n// Simple S3-FIFO implementation derived from \"FIFO Queues are ALL You Need for\n// Cache Eviction\" by Juncheng Yang et al. The policy uses a small FIFO admission\n// queue, a main FIFO resident queue, and a ghost queue for recently evicted\n// items. Reaccesses to ghosted keys promote them into the main queue, and main\n// eviction uses a second-chance-style frequency decrement.\n//\n// Variant cycle/0003/v003 (H03): the paper's static configuration. The split is\n// fixed at 10% small / 90% main, the ghost is sized to main, and a small-queue\n// entry graduates to main after a single access while in small (freq >= 1),\n// with its frequency reset to 0 on graduation. The adaptive split of the parent\n// (cycle/0002/v003) is removed.\n//\n// Data structures (inherited from cycle/0002/v003): entries live in a slab\n// (`slots`) addressed by `u32` indices; `small` / `main` are FIFOs of slot\n// indices; `index` maps resident keys to their slot, so `get` and the residency\n// check in `put` are O(1). The ghost keeps a FIFO for order plus a hashed\n// membership map with lazy deletion.\nconst MAX_FREQ: u8 = 3;\n\n// ---------------------------------------------------------------------------\n// Inline FxHash (rustc-hash) — Cargo.toml cannot change, so no external crate.\n// ---------------------------------------------------------------------------\n\nconst FX_SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;\n\n#[derive(Debug, Default, Clone, Copy)]\npub struct FxHasher {\n\thash: u64,\n}\n\nimpl FxHasher {\n\t#[inline]\n\tfn add_to_hash(&mut self, word: u64) {\n\t\tself.hash = (self.hash.rotate_left(5) ^ word).wrapping_mul(FX_SEED);\n\t}\n}\n\nimpl Hasher for FxHasher {\n\t#[inline]\n\tfn write(&mut self, bytes: &[u8]) {\n\t\tlet mut chunks = bytes.chunks_exact(8);\n\t\tfor chunk in &mut chunks {\n\t\t\tlet mut buf = [0u8; 8];\n\t\t\tbuf.copy_from_slice(chunk);\n\t\t\tself.add_to_hash(u64::from_le_bytes(buf));\n\t\t}\n\t\tfor &b in chunks.remainder() {\n\t\t\tself.add_to_hash(b as u64);\n\t\t}\n\t}\n\n\t#[inline]\n\tfn write_u8(&mut self, i: u8) {\n\t\tself.add_to_hash(i as u64);\n\t}\n\n\t#[inline]\n\tfn write_u16(&mut self, i: u16) {\n\t\tself.add_to_hash(i as u64);\n\t}\n\n\t#[inline]\n\tfn write_u32(&mut self, i: u32) {\n\t\tself.add_to_hash(i as u64);\n\t}\n\n\t#[inline]\n\tfn write_u64(&mut self, i: u64) {\n\t\tself.add_to_hash(i);\n\t}\n\n\t#[inline]\n\tfn write_usize(&mut self, i: usize) {\n\t\tself.add_to_hash(i as u64);\n\t}\n\n\t#[inline]\n\tfn finish(&self) -> u64 {\n\t\tself.hash\n\t}\n}\n\n#[derive(Debug, Default, Clone, Copy)]\npub struct FxBuildHasher;\n\nimpl BuildHasher for FxBuildHasher {\n\ttype Hasher = FxHasher;\n\n\t#[inline]\n\tfn build_hasher(&self) -> FxHasher {\n\t\tFxHasher::default()\n\t}\n}\n\ntype FxHashMap<K, V> = HashMap<K, V, FxBuildHasher>;\n\n// ---------------------------------------------------------------------------\n\n#[derive(Debug)]\npub struct Entry<K, V> {\n\tkey: K,\n\tvalue: V,\n\tfreq: u8,\n}\n\nimpl<K, V> Entry<K, V> {\n\tfn new(key: K, value: V) -> Self {\n\t\tSelf { key, value, freq: 0 }\n\t}\n}\n\n#[derive(Debug)]\npub struct S3Fifo<K: PartialEq + Clone, V> {\n\t/// Slab of entries; `None` marks a free slot (listed in `free`).\n\tslots: Vec<Option<Entry<K, V>>>,\n\tfree: Vec<u32>,\n\t/// Resident key -> slot index.\n\tindex: FxHashMap<K, u32>,\n\tsmall: VecDeque<u32>,\n\tmain: VecDeque<u32>,\n\t/// Ghost FIFO order: (key, stamp). An element is live iff\n\t/// `ghost_set[key] == stamp`; otherwise it is a stale lazily-deleted record.\n\tghost: VecDeque<(K, u64)>,\n\t/// Ghosted key -> stamp of its live record in `ghost`.\n\tghost_set: FxHashMap<K, u64>,\n\tghost_live: usize,\n\tghost_stamp: u64,\n\tsmall_size: usize,\n\tmain_size: usize,\n\tcapacity: usize,\n\tghost_cap: usize,\n}\n\nimpl<K: PartialEq + Eq + Hash + Clone, V> S3Fifo<K, V> {\n\tpub fn try_new(capacity: usize) -> Option<Self> {\n\t\tif capacity == 0 {\n\t\t\tlog::error!(\"S3Fifo.try_new: capacity must be > 0\");\n\t\t\treturn None;\n\t\t}\n\t\tif capacity > u32::MAX as usize {\n\t\t\tlog::error!(\"S3Fifo.try_new: capacity must fit in u32\");\n\t\t\treturn None;\n\t\t}\n\t\t// Paper configuration: fixed 10% small queue, ghost sized to main.\n\t\tlet small_size = if capacity == 1 {\n\t\t\t1\n\t\t} else {\n\t\t\t(capacity / 10).max(1).min(capacity - 1)\n\t\t};\n\t\tlet main_size = capacity - small_size;\n\t\tlet index = FxHashMap::with_capacity_and_hasher(capacity, FxBuildHasher);\n\t\tSome(Self {\n\t\t\tslots: Vec::with_capacity(capacity),\n\t\t\tfree: Vec::with_capacity(capacity),\n\t\t\tindex,\n\t\t\tsmall: VecDeque::with_capacity(capacity),\n\t\t\tmain: VecDeque::with_capacity(capacity),\n\t\t\tghost: VecDeque::with_capacity(Self::ghost_phys_bound(main_size) + 1),\n\t\t\tghost_set: FxHashMap::with_capacity_and_hasher(main_size + 1, FxBuildHasher),\n\t\t\tghost_live: 0,\n\t\t\tghost_stamp: 0,\n\t\t\tsmall_size,\n\t\t\tmain_size,\n\t\t\tcapacity,\n\t\t\tghost_cap: main_size,\n\t\t})\n\t}\n\n\tpub fn new(capacity: usize) -> Self {\n\t\tSelf::try_new(capacity).expect(\"S3Fifo::new: initialization failed\")\n\t}\n\n\t#[inline]\n\tfn ghost_phys_bound(ghost_cap: usize) -> usize {\n\t\t2 * ghost_cap.max(1)\n\t}\n\n\t// ---- slab helpers -----------------------------------------------------\n\n\t#[inline]\n\tfn slot(&self, i: u32) -> &Entry<K, V> {\n\t\tself.slots[i as usize].as_ref().expect(\"S3Fifo: queue references a free slot\")\n\t}\n\n\t#[inline]\n\tfn slot_mut(&mut self, i: u32) -> &mut Entry<K, V> {\n\t\tself.slots[i as usize].as_mut().expect(\"S3Fifo: queue references a free slot\")\n\t}\n\n\t/// Store a new resident entry and register it in `index`.\n\t#[inline]\n\tfn alloc(&mut self, entry: Entry<K, V>) -> u32 {\n\t\tlet key = entry.key.clone();\n\t\tlet i = match self.free.pop() {\n\t\t\tSome(i) => {\n\t\t\t\tself.slots[i as usize] = Some(entry);\n\t\t\t\ti\n\t\t\t}\n\t\t\tNone => {\n\t\t\t\tlet i = self.slots.len() as u32;\n\t\t\t\tself.slots.push(Some(entry));\n\t\t\t\ti\n\t\t\t}\n\t\t};\n\t\tself.index.insert(key, i);\n\t\ti\n\t}\n\n\t/// Remove a resident entry from the slab and `index`; return it.\n\t#[inline]\n\tfn release(&mut self, i: u32) -> Entry<K, V> {\n\t\tlet entry = self.slots[i as usize].take().expect(\"S3Fifo: double release of slot\");\n\t\tself.index.remove(&entry.key);\n\t\tself.free.push(i);\n\t\tentry\n\t}\n\n\t// ---- ghost helpers ----------------------------------------------------\n\n\t/// O(1) ghost membership test + removal (lazy: the FIFO record stays and\n\t/// becomes stale).\n\t#[inline]\n\tfn ghost_take(&mut self, key: &K) -> bool {\n\t\tif self.ghost_set.remove(key).is_some() {\n\t\t\tself.ghost_live -= 1;\n\t\t\ttrue\n\t\t} else {\n\t\t\tfalse\n\t\t}\n\t}\n\n\t/// Drop the oldest *live* ghost record, discarding stale records on the way.\n\tfn ghost_pop_oldest_live(&mut self) {\n\t\twhile let Some((k, stamp)) = self.ghost.pop_back() {\n\t\t\tif self.ghost_set.get(&k) == Some(&stamp) {\n\t\t\t\tself.ghost_set.remove(&k);\n\t\t\t\tself.ghost_live -= 1;\n\t\t\t\treturn;\n\t\t\t}\n\t\t}\n\t}\n\n\tfn ghost_push(&mut self, key: K) {\n\t\tif self.ghost_live >= self.ghost_cap {\n\t\t\tself.ghost_pop_oldest_live();\n\t\t}\n\t\tself.ghost_stamp = self.ghost_stamp.wrapping_add(1);\n\t\tlet stamp = self.ghost_stamp;\n\t\tself.ghost_set.insert(key.clone(), stamp);\n\t\tself.ghost.push_front((key, stamp));\n\t\tself.ghost_live += 1;\n\t\tif self.ghost.len() > Self::ghost_phys_bound(self.ghost_cap) {\n\t\t\tlet set = &self.ghost_set;\n\t\t\tself.ghost.retain(|(k, s)| set.get(k) == Some(s));\n\t\t}\n\t}\n\n\t// ---- public API -------------------------------------------------------\n\n\tpub fn get(&mut self, key: &K) -> Option<&V> {\n\t\tlet &i = self.index.get(key)?;\n\t\tlet entry = self.slots[i as usize].as_mut()?;\n\t\tentry.freq = (entry.freq.saturating_add(1)).min(MAX_FREQ);\n\t\tSome(&entry.value)\n\t}\n\n\tpub fn put(&mut self, key: K, value: V) {\n\t\tif let Some(&i) = self.index.get(&key) {\n\t\t\tself.slot_mut(i).value = value;\n\t\t\treturn;\n\t\t}\n\n\t\tif self.ghost_take(&key) {\n\t\t\t// Ghost hit -> insert directly into main.\n\t\t\tif self.main.len() >= self.main_size {\n\t\t\t\tself.evict_main();\n\t\t\t}\n\t\t\tlet i = self.alloc(Entry::new(key, value));\n\t\t\tself.main.push_front(i);\n\t\t\treturn;\n\t\t}\n\n\t\tif self.small.len() >= self.small_size {\n\t\t\tself.evict_small();\n\t\t}\n\t\tlet i = self.alloc(Entry::new(key, value));\n\t\tself.small.push_front(i);\n\t}\n\n\tfn evict_main(&mut self) {\n\t\twhile let Some(i) = self.main.pop_back() {\n\t\t\tlet entry = self.slot_mut(i);\n\t\t\tif entry.freq > 0 {\n\t\t\t\tentry.freq -= 1;\n\t\t\t\tself.main.push_front(i);\n\t\t\t} else {\n\t\t\t\tself.release(i);\n\t\t\t\tbreak;\n\t\t\t}\n\t\t}\n\t}\n\n\tfn evict_small(&mut self) {\n\t\tif let Some(i) = self.small.pop_back() {\n\t\t\t// Paper rule: one access while in small is enough to graduate.\n\t\t\t// (Guard: with capacity 1 there is no main queue to graduate into.)\n\t\t\tif self.slot(i).freq >= 1 && self.main_size > 0 {\n\t\t\t\tif self.main.len() >= self.main_size {\n\t\t\t\t\tself.evict_main();\n\t\t\t\t}\n\t\t\t\tself.slot_mut(i).freq = 0;\n\t\t\t\tself.main.push_front(i);\n\t\t\t} else {\n\t\t\t\tlet entry = self.release(i);\n\t\t\t\tself.ghost_push(entry.key);\n\t\t\t}\n\t\t}\n\t}\n\n\t// ---- test helpers -----------------------------------------------------\n\n\t#[cfg(test)]\n\tfn in_main(&self, key: &K) -> bool {\n\t\tself.main.iter().any(|&i| &self.slot(i).key == key)\n\t}\n\n\t#[cfg(test)]\n\tfn push_main_back_for_test(&mut self, entry: Entry<K, V>) {\n\t\tlet i = self.alloc(entry);\n\t\tself.main.push_back(i);\n\t}\n\n\t#[cfg(test)]\n\tfn push_small_front_for_test(&mut self, entry: Entry<K, V>) {\n\t\tlet i = self.alloc(entry);\n\t\tself.small.push_front(i);\n\t}\n}\n\nimpl<K: PartialEq + Eq + std::hash::Hash + Clone + Debug, V: Debug> framework::Cache<K, V> for S3Fifo<K, V> {\n\tfn put(&mut self, key: K, value: V) {\n\t\tS3Fifo::put(self, key, value);\n\t}\n\n\tfn get(&mut self, key: &K) -> Option<&V> {\n\t\tS3Fifo::get(self, key)\n\t}\n}\n\n#[cfg(test)]\nmod tests {\n\tuse super::*;\n\n\t#[test]\n\tfn get_miss_returns_none() {\n\t\tlet mut cache = S3Fifo::<i64, i64>::new(4);\n\t\tassert!(cache.get(&42).is_none());\n\t}\n\n\t#[test]\n\tfn put_then_get_finds_value() {\n\t\tlet mut cache = S3Fifo::new(4);\n\t\tcache.put(1, 10);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 10);\n\t}\n\n\t#[test]\n\tfn put_existing_updates_value() {\n\t\tlet mut cache = S3Fifo::new(4);\n\t\tcache.put(1, 10);\n\t\tcache.put(1, 20);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 20);\n\t}\n\n\t#[test]\n\tfn small_eviction_promotes_to_main_after_multiple_hits() {\n\t\tlet mut cache = S3Fifo::new(4);\n\t\tcache.put(1, 10);\n\t\tcache.get(&1);\n\t\tcache.get(&1);\n\t\tcache.put(2, 20);\n\t\tassert!(cache.in_main(&1));\n\t}\n\n\t#[test]\n\tfn small_eviction_promotes_after_single_hit_and_resets_freq() {\n\t\tlet mut cache = S3Fifo::new(4);\n\t\tcache.put(1, 10);\n\t\tcache.get(&1);\n\t\tcache.put(2, 20);\n\t\tassert!(cache.in_main(&1));\n\t\tlet &i = cache.index.get(&1).unwrap();\n\t\tassert_eq!(cache.slot(i).freq, 0);\n\t}\n\n\t#[test]\n\tfn ghost_hit_promotes_back_to_main() {\n\t\tlet mut cache = S3Fifo::new(4);\n\t\tcache.put(1, 10);\n\t\tcache.put(2, 20);\n\t\tcache.put(3, 30);\n\t\tcache.put(4, 40);\n\t\tcache.put(1, 10);\n\t\tassert!(cache.in_main(&1));\n\t}\n\n\t#[test]\n\tfn main_eviction_uses_frequency_second_chance() {\n\t\tlet mut cache = S3Fifo::new(5);\n\t\tcache.push_main_back_for_test(Entry { freq: 0, ..Entry::new(1, 10) });\n\t\tcache.push_main_back_for_test(Entry { freq: 2, ..Entry::new(2, 20) });\n\t\tcache.push_main_back_for_test(Entry { freq: 0, ..Entry::new(3, 30) });\n\t\tcache.push_main_back_for_test(Entry { freq: 1, ..Entry::new(4, 40) });\n\t\tcache.push_small_front_for_test(Entry { freq: 2, ..Entry::new(5, 50) });\n\t\tcache.evict_small();\n\t\tassert!(cache.in_main(&4));\n\t\tassert!(!cache.in_main(&3));\n\t\tassert!(cache.get(&3).is_none());\n\t}\n\n\t#[test]\n\tfn capacity_bound_never_exceeded() {\n\t\tlet mut cache = S3Fifo::new(4);\n\t\tfor key in 0..20 {\n\t\t\tcache.put(key, key);\n\t\t}\n\t\tassert!(cache.small.len() + cache.main.len() <= 4);\n\t\tassert_eq!(cache.index.len(), cache.small.len() + cache.main.len());\n\t}\n\n\t#[test]\n\tfn adaptive_split_keeps_bounds() {\n\t\tlet cap = 64;\n\t\tlet mut cache = S3Fifo::new(cap);\n\t\tfor round in 0..50i64 {\n\t\t\tfor key in 0..200i64 {\n\t\t\t\tlet k = (key * 7 + round) % 150;\n\t\t\t\tif cache.get(&k).is_none() {\n\t\t\t\t\tcache.put(k, k);\n\t\t\t\t}\n\t\t\t\tassert!(cache.small.len() + cache.main.len() <= cap);\n\t\t\t\tassert_eq!(cache.small_size + cache.main_size, cap);\n\t\t\t\tassert!(cache.small_size >= 1 && cache.main_size >= 1);\n\t\t\t\tassert!(cache.ghost_live <= cache.ghost_cap);\n\t\t\t\tassert_eq!(cache.ghost_live, cache.ghost_set.len());\n\t\t\t\tassert!(cache.ghost.len() <= S3Fifo::<i64, i64>::ghost_phys_bound(cache.ghost_cap));\n\t\t\t}\n\t\t}\n\t}\n\n\t#[test]\n\tfn fixed_split_stays_at_ten_percent() {\n\t\tlet cap = 100;\n\t\tlet mut cache = S3Fifo::new(cap);\n\t\tfor round in 0..50i64 {\n\t\t\tfor key in 0..300i64 {\n\t\t\t\tlet k = (key * 7 + round) % 250;\n\t\t\t\tif cache.get(&k).is_none() {\n\t\t\t\t\tcache.put(k, k);\n\t\t\t\t}\n\t\t\t}\n\t\t}\n\t\tassert_eq!(cache.small_size, cap / 10);\n\t\tassert_eq!(cache.main_size, cap - cap / 10);\n\t\tassert_eq!(cache.ghost_cap, cache.main_size);\n\t\tassert!(cache.small.len() + cache.main.len() <= cap);\n\t}\n\n\t#[test]\n\tfn index_tracks_residency() {\n\t\tlet cap = 64;\n\t\tlet mut cache = S3Fifo::new(cap);\n\t\tfor round in 0..50i64 {\n\t\t\tfor key in 0..200i64 {\n\t\t\t\tlet k = (key * 7 + round) % 150;\n\t\t\t\tif cache.get(&k).is_none() {\n\t\t\t\t\tcache.put(k, k);\n\t\t\t\t}\n\t\t\t\tassert_eq!(cache.index.len(), cache.small.len() + cache.main.len());\n\t\t\t\tfor &i in cache.small.iter().chain(cache.main.iter()) {\n\t\t\t\t\tlet e = cache.slot(i);\n\t\t\t\t\tassert_eq!(cache.index.get(&e.key), Some(&i));\n\t\t\t\t\tassert!(!cache.ghost_set.contains_key(&e.key));\n\t\t\t\t}\n\t\t\t}\n\t\t}\n\t}\n\n\t#[test]\n\tfn repeated_get_clamps_frequency() {\n\t\tlet mut cache = S3Fifo::new(4);\n\t\tcache.put(1, 10);\n\t\tfor _ in 0..10 {\n\t\t\tassert_eq!(*cache.get(&1).unwrap(), 10);\n\t\t}\n\t}\n}\n"
}
```

### 12:28:02 thinking



### 12:28:02 chat

I wrote the v003 overlay at `overlay/src/s3_fifo/mod.rs`, but I haven't compiled it or run any gates. This session only lets me read and write files, so the harness will be the first to build it.

All three changes are applied to the parent's code, and the hashed slab, `index` and stamped ghost are unchanged:

1. **Fixed split.** The small queue is set once at 10% of capacity (`(capacity / 10).max(1).min(capacity - 1)`, or 1 when capacity is 1). Main gets the rest, and the ghost is sized to main. I deleted `adaptive()` and every resize branch, including the fix-up evictions in `put` and the `grow_small` block. A ghost hit now just evicts from main if main is full and inserts the key at the front of main. With adaptation gone, the old `evict_small_inner(adapt)` wrapper became just `evict_small()`.
2. **Paper graduation rule.** An entry leaving small now graduates after a single access (`freq >= 1`), and its `freq` is reset to 0 before it goes to the front of main.
3. **Everything else is the same:** `MAX_FREQ = 3`, second-chance eviction in main, and only small-queue victims go to the ghost.

**One change beyond the hypothesis:** promotion also requires `main_size > 0`. At capacity 1 there is no main queue, and promoting into it would put two entries in a one-slot cache. The old code had the same latent overflow, but the looser graduation rule makes it much more likely to fire. The gates run at capacity 8, where this check never matters.

**Tests:** I traced the existing in-module tests by hand against the new code, and they should still pass. I added two:
- `fixed_split_stays_at_ten_percent`: at capacity 100, the small size stays at `cap / 10` after a workload loop, with main and ghost sized to match.
- `small_eviction_promotes_after_single_hit_and_resets_freq`: a single access is enough to graduate, and the promoted entry's `freq` is 0.


---

- finished: 2026-10-20T12:28:03+00:00
- duration: 83.65s
- status: ok
- input_tokens: 18
- output_tokens: 9500
- cache_read_tokens: 236073
- cache_write_tokens: 23892
- cost_usd: 0.428423
