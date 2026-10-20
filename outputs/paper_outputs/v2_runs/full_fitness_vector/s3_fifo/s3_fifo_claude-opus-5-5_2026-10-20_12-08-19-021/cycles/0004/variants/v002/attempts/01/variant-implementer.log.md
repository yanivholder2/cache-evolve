# Agent log: variant-implementer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0004/variants/v002/attempts/01
- allow_edits: True
- started: 2026-10-20T12:32:56+00:00

## timeline

### 12:33:01 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles; cat 0004/hypotheses/H02.md; echo ----; P=0003/variants/v002; find $P/attempts/01/overlay -type f; cat $P/insight.md $P/eval/fitness-vector.json; echo ----; cat /home/user/cache-evolve/inputs/subjects/s3_fifo/manifest.toml; ls -R /home/user/cache-evolve/inputs/subjects/s3_fifo/src"
}
```

### 12:33:04 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0004/hypotheses/H02.md"
}
```

### 12:33:05 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0003/variants/v002/insight.md"
}
```

### 12:33:05 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/s3_fifo/manifest.toml"
}
```

### 12:33:07 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0003/variants/v002/attempts/01/overlay"
}
```

### 12:33:15 thinking



### 12:33:15 tool: Grep

```json
{
  "pattern": "adaptive",
  "path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0003/variants/v002/attempts/01/overlay",
  "output_mode": "files_with_matches"
}
```

### 12:33:19 thinking



### 12:33:20 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0003/variants/v002/attempts/01/overlay"
}
```

### 12:33:20 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/s3_fifo/src/lib.rs"
}
```

### 12:33:22 chat

I'm finding the files in the parent overlay. Only Read, Write and Edit work in this session, so I'm guessing paths from the subject's layout.

### 12:33:22 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/s3_fifo/src/s3_fifo/mod.rs"
}
```

### 12:33:23 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/s3_fifo/src/factory.rs"
}
```

### 12:33:25 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0003/variants/v002/attempts/01/overlay/src/s3_fifo/mod.rs"
}
```

### 12:33:29 thinking



### 12:33:30 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0003/variants/v002/attempts/01/overlay/src/factory.rs"
}
```

### 12:33:30 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0003/variants/v002/attempts/01/overlay/src/logging.rs"
}
```

### 12:33:37 thinking



### 12:34:17 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0004/variants/v002/attempts/01/overlay/src/s3_fifo/mod.rs",
  "content": "use std::collections::{HashMap, VecDeque};\nuse std::fmt::Debug;\nuse std::hash::{BuildHasher, Hash, Hasher};\n\npub mod s3_fifo_jni;\n\n// Simple S3-FIFO implementation derived from \"FIFO Queues are ALL You Need for\n// Cache Eviction\" by Juncheng Yang et al. The policy uses a small FIFO admission\n// queue, a main FIFO resident queue, and a ghost queue for recently evicted\n// items. Reaccesses to ghosted keys promote them into the main queue, and main\n// eviction uses a second-chance-style frequency decrement.\n//\n// Static split (cycle/0004/v002, H02): the small/main split is fixed at\n// construction to 10/90 (`small_size = capacity / 10`, clamped to\n// [1, capacity - 1]) and never changes. The adaptive ARC-style resizing of the\n// parent (cycle/0003/v002) is removed. Small-queue graduation still requires\n// `freq > 1`; the ghost is sized to the (fixed) main size.\n//\n// Slab layout (inherited from cycle/0002/v003): entries live in a slab\n// (`slots`) addressed by `u32` indices; `small` / `main` are FIFOs of slot\n// indices; `index` maps resident keys to their slot.\n//\n// Ghost (inherited from cycle/0003/v002): a fixed-size ring of 64-bit key\n// fingerprints (0 = empty slot) plus a fingerprint -> ring-position map. Keys\n// are never cloned into the ghost, there are no stamps, and there is no\n// compaction pass: overwriting the head slot evicts the oldest record.\nconst MAX_FREQ: u8 = 3;\n\n// ---------------------------------------------------------------------------\n// Inline FxHash (rustc-hash) — Cargo.toml cannot change, so no external crate.\n// ---------------------------------------------------------------------------\n\nconst FX_SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;\n\n#[derive(Debug, Default, Clone, Copy)]\npub struct FxHasher {\n\thash: u64,\n}\n\nimpl FxHasher {\n\t#[inline]\n\tfn add_to_hash(&mut self, word: u64) {\n\t\tself.hash = (self.hash.rotate_left(5) ^ word).wrapping_mul(FX_SEED);\n\t}\n}\n\nimpl Hasher for FxHasher {\n\t#[inline]\n\tfn write(&mut self, bytes: &[u8]) {\n\t\tlet mut chunks = bytes.chunks_exact(8);\n\t\tfor chunk in &mut chunks {\n\t\t\tlet mut buf = [0u8; 8];\n\t\t\tbuf.copy_from_slice(chunk);\n\t\t\tself.add_to_hash(u64::from_le_bytes(buf));\n\t\t}\n\t\tfor &b in chunks.remainder() {\n\t\t\tself.add_to_hash(b as u64);\n\t\t}\n\t}\n\n\t#[inline]\n\tfn write_u8(&mut self, i: u8) {\n\t\tself.add_to_hash(i as u64);\n\t}\n\n\t#[inline]\n\tfn write_u16(&mut self, i: u16) {\n\t\tself.add_to_hash(i as u64);\n\t}\n\n\t#[inline]\n\tfn write_u32(&mut self, i: u32) {\n\t\tself.add_to_hash(i as u64);\n\t}\n\n\t#[inline]\n\tfn write_u64(&mut self, i: u64) {\n\t\tself.add_to_hash(i);\n\t}\n\n\t#[inline]\n\tfn write_usize(&mut self, i: usize) {\n\t\tself.add_to_hash(i as u64);\n\t}\n\n\t#[inline]\n\tfn finish(&self) -> u64 {\n\t\tself.hash\n\t}\n}\n\n#[derive(Debug, Default, Clone, Copy)]\npub struct FxBuildHasher;\n\nimpl BuildHasher for FxBuildHasher {\n\ttype Hasher = FxHasher;\n\n\t#[inline]\n\tfn build_hasher(&self) -> FxHasher {\n\t\tFxHasher::default()\n\t}\n}\n\ntype FxHashMap<K, V> = HashMap<K, V, FxBuildHasher>;\n\n/// Ghost fingerprint of a key: FxHash of the key, forced odd so it is never 0\n/// (0 marks an empty ring slot).\n#[inline]\nfn fp<K: Hash + ?Sized>(key: &K) -> u64 {\n\tlet mut h = FxBuildHasher.build_hasher();\n\tkey.hash(&mut h);\n\th.finish() | 1\n}\n\n// ---------------------------------------------------------------------------\n\n#[derive(Debug)]\npub struct Entry<K, V> {\n\tkey: K,\n\tvalue: V,\n\tfreq: u8,\n}\n\nimpl<K, V> Entry<K, V> {\n\tfn new(key: K, value: V) -> Self {\n\t\tSelf { key, value, freq: 0 }\n\t}\n}\n\n#[derive(Debug)]\npub struct S3Fifo<K: PartialEq + Clone, V> {\n\t/// Slab of entries; `None` marks a free slot (listed in `free`).\n\tslots: Vec<Option<Entry<K, V>>>,\n\tfree: Vec<u32>,\n\t/// Resident key -> slot index.\n\tindex: FxHashMap<K, u32>,\n\tsmall: VecDeque<u32>,\n\tmain: VecDeque<u32>,\n\t/// Ghost ring of key fingerprints; 0 = empty slot. Fixed length.\n\tghost_ring: Vec<u64>,\n\t/// Next ring write position (wraps).\n\tghost_head: usize,\n\t/// Live fingerprint -> ring position.\n\tghost_map: FxHashMap<u64, u32>,\n\tghost_live: usize,\n\tsmall_size: usize,\n\tmain_size: usize,\n\t#[allow(dead_code)]\n\tghost_cap: usize,\n}\n\nimpl<K: PartialEq + Eq + Hash + Clone, V> S3Fifo<K, V> {\n\tpub fn try_new(capacity: usize) -> Option<Self> {\n\t\tif capacity == 0 {\n\t\t\tlog::error!(\"S3Fifo.try_new: capacity must be > 0\");\n\t\t\treturn None;\n\t\t}\n\t\tif capacity > u32::MAX as usize {\n\t\t\tlog::error!(\"S3Fifo.try_new: capacity must fit in u32\");\n\t\t\treturn None;\n\t\t}\n\t\tlet small_size = if capacity == 1 {\n\t\t\t1\n\t\t} else {\n\t\t\t(capacity / 10).max(1).min(capacity - 1)\n\t\t};\n\t\tlet main_size = capacity - small_size;\n\t\tlet index = FxHashMap::with_capacity_and_hasher(capacity, FxBuildHasher);\n\t\t// The ring needs at least one slot so the modulo is defined; with\n\t\t// ghost_cap == 0 (capacity 1) this gives a one-record ghost.\n\t\tlet ring_len = main_size.max(1);\n\t\tSome(Self {\n\t\t\tslots: Vec::with_capacity(capacity),\n\t\t\tfree: Vec::with_capacity(capacity),\n\t\t\tindex,\n\t\t\tsmall: VecDeque::with_capacity(small_size),\n\t\t\tmain: VecDeque::with_capacity(main_size.max(1)),\n\t\t\tghost_ring: vec![0u64; ring_len],\n\t\t\tghost_head: 0,\n\t\t\tghost_map: FxHashMap::with_capacity_and_hasher(ring_len + 1, FxBuildHasher),\n\t\t\tghost_live: 0,\n\t\t\tsmall_size,\n\t\t\tmain_size,\n\t\t\tghost_cap: main_size,\n\t\t})\n\t}\n\n\tpub fn new(capacity: usize) -> Self {\n\t\tSelf::try_new(capacity).expect(\"S3Fifo::new: initialization failed\")\n\t}\n\n\t// ---- slab helpers -----------------------------------------------------\n\n\t#[inline]\n\tfn slot(&self, i: u32) -> &Entry<K, V> {\n\t\tself.slots[i as usize].as_ref().expect(\"S3Fifo: queue references a free slot\")\n\t}\n\n\t#[inline]\n\tfn slot_mut(&mut self, i: u32) -> &mut Entry<K, V> {\n\t\tself.slots[i as usize].as_mut().expect(\"S3Fifo: queue references a free slot\")\n\t}\n\n\t/// Store a new resident entry and register it in `index`.\n\t#[inline]\n\tfn alloc(&mut self, entry: Entry<K, V>) -> u32 {\n\t\tlet key = entry.key.clone();\n\t\tlet i = match self.free.pop() {\n\t\t\tSome(i) => {\n\t\t\t\tself.slots[i as usize] = Some(entry);\n\t\t\t\ti\n\t\t\t}\n\t\t\tNone => {\n\t\t\t\tlet i = self.slots.len() as u32;\n\t\t\t\tself.slots.push(Some(entry));\n\t\t\t\ti\n\t\t\t}\n\t\t};\n\t\tself.index.insert(key, i);\n\t\ti\n\t}\n\n\t/// Remove a resident entry from the slab and `index`; return it.\n\t#[inline]\n\tfn release(&mut self, i: u32) -> Entry<K, V> {\n\t\tlet entry = self.slots[i as usize].take().expect(\"S3Fifo: double release of slot\");\n\t\tself.index.remove(&entry.key);\n\t\tself.free.push(i);\n\t\tentry\n\t}\n\n\t// ---- ghost helpers ----------------------------------------------------\n\n\t/// O(1) ghost membership test + removal.\n\t#[inline]\n\tfn ghost_take(&mut self, key: &K) -> bool {\n\t\tlet f = fp(key);\n\t\tif let Some(p) = self.ghost_map.remove(&f) {\n\t\t\tself.ghost_ring[p as usize] = 0;\n\t\t\tself.ghost_live -= 1;\n\t\t\ttrue\n\t\t} else {\n\t\t\tfalse\n\t\t}\n\t}\n\n\t/// Record `key` in the ghost ring, overwriting (evicting) the oldest slot.\n\t#[inline]\n\tfn ghost_push(&mut self, key: &K) {\n\t\tlet f = fp(key);\n\t\tlet h = self.ghost_head;\n\t\tlet old = self.ghost_ring[h];\n\t\tif old != 0 {\n\t\t\tif self.ghost_map.get(&old) == Some(&(h as u32)) {\n\t\t\t\tself.ghost_map.remove(&old);\n\t\t\t}\n\t\t\tself.ghost_live -= 1;\n\t\t}\n\t\tif let Some(p) = self.ghost_map.insert(f, h as u32) {\n\t\t\t// Same fingerprint already live elsewhere: clear its old slot so it\n\t\t\t// is not counted twice.\n\t\t\tif p as usize != h {\n\t\t\t\tself.ghost_ring[p as usize] = 0;\n\t\t\t\tself.ghost_live -= 1;\n\t\t\t}\n\t\t}\n\t\tself.ghost_ring[h] = f;\n\t\tself.ghost_live += 1;\n\t\tself.ghost_head = (h + 1) % self.ghost_ring.len();\n\t}\n\n\t// ---- public API -------------------------------------------------------\n\n\t#[inline]\n\tpub fn get(&mut self, key: &K) -> Option<&V> {\n\t\tlet i = *self.index.get(key)? as usize;\n\t\tmatch &mut self.slots[i] {\n\t\t\tSome(entry) => {\n\t\t\t\tentry.freq = entry.freq.saturating_add(1).min(MAX_FREQ);\n\t\t\t\tSome(&entry.value)\n\t\t\t}\n\t\t\tNone => None,\n\t\t}\n\t}\n\n\t#[inline]\n\tpub fn put(&mut self, key: K, value: V) {\n\t\tif let Some(&i) = self.index.get(&key) {\n\t\t\tself.slot_mut(i).value = value;\n\t\t\treturn;\n\t\t}\n\n\t\tif self.ghost_take(&key) {\n\t\t\t// Ghost hit -> insert straight into main (split is fixed).\n\t\t\tif self.main.len() >= self.main_size {\n\t\t\t\tself.evict_main();\n\t\t\t}\n\t\t\tlet i = self.alloc(Entry::new(key, value));\n\t\t\tself.main.push_front(i);\n\t\t\treturn;\n\t\t}\n\n\t\tif self.small.len() >= self.small_size {\n\t\t\tself.evict_small();\n\t\t}\n\t\tlet i = self.alloc(Entry::new(key, value));\n\t\tself.small.push_front(i);\n\t}\n\n\tfn evict_main(&mut self) {\n\t\twhile let Some(i) = self.main.pop_back() {\n\t\t\tlet entry = self.slot_mut(i);\n\t\t\tif entry.freq > 0 {\n\t\t\t\tentry.freq -= 1;\n\t\t\t\tself.main.push_front(i);\n\t\t\t} else {\n\t\t\t\tself.release(i);\n\t\t\t\tbreak;\n\t\t\t}\n\t\t}\n\t}\n\n\tfn evict_small(&mut self) {\n\t\tif let Some(i) = self.small.pop_back() {\n\t\t\tif self.slot(i).freq > 1 {\n\t\t\t\tif self.main.len() >= self.main_size {\n\t\t\t\t\tself.evict_main();\n\t\t\t\t}\n\t\t\t\tself.main.push_front(i);\n\t\t\t} else {\n\t\t\t\tlet entry = self.release(i);\n\t\t\t\tself.ghost_push(&entry.key);\n\t\t\t}\n\t\t}\n\t}\n\n\t// ---- test helpers -----------------------------------------------------\n\n\t#[cfg(test)]\n\tfn in_main(&self, key: &K) -> bool {\n\t\tself.main.iter().any(|&i| &self.slot(i).key == key)\n\t}\n\n\t#[cfg(test)]\n\tfn push_main_back_for_test(&mut self, entry: Entry<K, V>) {\n\t\tlet i = self.alloc(entry);\n\t\tself.main.push_back(i);\n\t}\n\n\t#[cfg(test)]\n\tfn push_small_front_for_test(&mut self, entry: Entry<K, V>) {\n\t\tlet i = self.alloc(entry);\n\t\tself.small.push_front(i);\n\t}\n}\n\nimpl<K: PartialEq + Eq + std::hash::Hash + Clone + Debug, V: Debug> framework::Cache<K, V> for S3Fifo<K, V> {\n\t#[inline]\n\tfn put(&mut self, key: K, value: V) {\n\t\tS3Fifo::put(self, key, value);\n\t}\n\n\t#[inline]\n\tfn get(&mut self, key: &K) -> Option<&V> {\n\t\tS3Fifo::get(self, key)\n\t}\n}\n\n#[cfg(test)]\nmod tests {\n\tuse super::*;\n\n\t#[test]\n\tfn get_miss_returns_none() {\n\t\tlet mut cache = S3Fifo::<i64, i64>::new(4);\n\t\tassert!(cache.get(&42).is_none());\n\t}\n\n\t#[test]\n\tfn put_then_get_finds_value() {\n\t\tlet mut cache = S3Fifo::new(4);\n\t\tcache.put(1, 10);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 10);\n\t}\n\n\t#[test]\n\tfn put_existing_updates_value() {\n\t\tlet mut cache = S3Fifo::new(4);\n\t\tcache.put(1, 10);\n\t\tcache.put(1, 20);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 20);\n\t}\n\n\t#[test]\n\tfn small_eviction_promotes_to_main_after_multiple_hits() {\n\t\tlet mut cache = S3Fifo::new(4);\n\t\tcache.put(1, 10);\n\t\tcache.get(&1);\n\t\tcache.get(&1);\n\t\tcache.put(2, 20);\n\t\tassert!(cache.in_main(&1));\n\t}\n\n\t#[test]\n\tfn ghost_hit_promotes_back_to_main() {\n\t\tlet mut cache = S3Fifo::new(4);\n\t\tcache.put(1, 10);\n\t\tcache.put(2, 20);\n\t\tcache.put(3, 30);\n\t\tcache.put(4, 40);\n\t\tcache.put(1, 10);\n\t\tassert!(cache.in_main(&1));\n\t}\n\n\t#[test]\n\tfn main_eviction_uses_frequency_second_chance() {\n\t\tlet mut cache = S3Fifo::new(5);\n\t\tcache.push_main_back_for_test(Entry { freq: 0, ..Entry::new(1, 10) });\n\t\tcache.push_main_back_for_test(Entry { freq: 2, ..Entry::new(2, 20) });\n\t\tcache.push_main_back_for_test(Entry { freq: 0, ..Entry::new(3, 30) });\n\t\tcache.push_main_back_for_test(Entry { freq: 1, ..Entry::new(4, 40) });\n\t\tcache.push_small_front_for_test(Entry { freq: 2, ..Entry::new(5, 50) });\n\t\tcache.evict_small();\n\t\tassert!(cache.in_main(&4));\n\t\tassert!(!cache.in_main(&3));\n\t\tassert!(cache.get(&3).is_none());\n\t}\n\n\t#[test]\n\tfn capacity_bound_never_exceeded() {\n\t\tlet mut cache = S3Fifo::new(4);\n\t\tfor key in 0..20 {\n\t\t\tcache.put(key, key);\n\t\t}\n\t\tassert!(cache.small.len() + cache.main.len() <= 4);\n\t\tassert_eq!(cache.index.len(), cache.small.len() + cache.main.len());\n\t}\n\n\t#[test]\n\tfn static_split_keeps_bounds() {\n\t\tlet cap = 64;\n\t\tlet mut cache = S3Fifo::new(cap);\n\t\tlet (small0, main0) = (cache.small_size, cache.main_size);\n\t\tassert_eq!(small0, 6);\n\t\tfor round in 0..50i64 {\n\t\t\tfor key in 0..200i64 {\n\t\t\t\tlet k = (key * 7 + round) % 150;\n\t\t\t\tif cache.get(&k).is_none() {\n\t\t\t\t\tcache.put(k, k);\n\t\t\t\t}\n\t\t\t\tassert!(cache.small.len() + cache.main.len() <= cap);\n\t\t\t\tassert!(cache.small.len() <= cache.small_size);\n\t\t\t\tassert!(cache.main.len() <= cache.main_size);\n\t\t\t\tassert_eq!(cache.small_size + cache.main_size, cap);\n\t\t\t\tassert_eq!((cache.small_size, cache.main_size), (small0, main0));\n\t\t\t\tassert!(cache.ghost_live <= cache.ghost_cap);\n\t\t\t\tassert_eq!(cache.ghost_live, cache.ghost_map.len());\n\t\t\t}\n\t\t}\n\t}\n\n\t#[test]\n\tfn index_tracks_residency() {\n\t\tlet cap = 64;\n\t\tlet mut cache = S3Fifo::new(cap);\n\t\tfor round in 0..50i64 {\n\t\t\tfor key in 0..200i64 {\n\t\t\t\tlet k = (key * 7 + round) % 150;\n\t\t\t\tif cache.get(&k).is_none() {\n\t\t\t\t\tcache.put(k, k);\n\t\t\t\t}\n\t\t\t\tassert_eq!(cache.index.len(), cache.small.len() + cache.main.len());\n\t\t\t\tfor &i in cache.small.iter().chain(cache.main.iter()) {\n\t\t\t\t\tlet e = cache.slot(i);\n\t\t\t\t\tassert_eq!(cache.index.get(&e.key), Some(&i));\n\t\t\t\t\tassert!(!cache.ghost_map.contains_key(&fp(&e.key)));\n\t\t\t\t}\n\t\t\t}\n\t\t}\n\t}\n\n\t#[test]\n\tfn repeated_get_clamps_frequency() {\n\t\tlet mut cache = S3Fifo::new(4);\n\t\tcache.put(1, 10);\n\t\tfor _ in 0..10 {\n\t\t\tassert_eq!(*cache.get(&1).unwrap(), 10);\n\t\t}\n\t}\n}\n"
}
```

### 12:34:28 thinking



### 12:34:29 tool: Edit

```json
{
  "replace_all": false,
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0004/variants/v002/attempts/01/overlay/src/s3_fifo/mod.rs",
  "old_string": "\t\t\tsmall: VecDeque::with_capacity(small_size),\n\t\t\tmain: VecDeque::with_capacity(main_size.max(1)),",
  "new_string": "\t\t\tsmall: VecDeque::with_capacity(capacity),\n\t\t\tmain: VecDeque::with_capacity(capacity),"
}
```

### 12:34:37 thinking



### 12:34:37 chat

I've written the H02 variant: the small/main split is now fixed at 10/90 and the adaptive resizing is removed. I haven't compiled it or run the gates, because only Read, Write and Edit worked in this session.

**Where it is:** one file, `overlay/src/s3_fifo/mod.rs`, built from the parent's `src/s3_fifo/mod.rs`. As far as I could check, that is the only file the parent overlaid. I couldn't list the directory, so I tested a few likely paths and found nothing else.

**What changed:**
1. **Construction:** `small_size` is now `if capacity == 1 { 1 } else { (capacity / 10).max(1).min(capacity - 1) }`, and `main_size = capacity - small_size`. `ring_len` and `ghost_cap` are set from `main_size` as before.
2. **Ghost hit in `put`:** the whole resize block and its fix-up evictions are gone. What's left is evict from main if it's full, allocate, then `main.push_front`.
3. **Small-queue eviction:** `evict_small_inner`, the `adapt` parameter and the `grow_small` logic are removed and folded into one `evict_small`. An entry with `freq > 1` still graduates to main, evicting from main first if main is full. Anything else still goes to the ghost.
4. **Clean-up:** I removed `adaptive()` and the `capacity` field, which nothing used any more. I marked `ghost_cap` with `#[allow(dead_code)]` because only the tests read it. The header comment now describes the fixed split.

As instructed, `get`, `evict_main`, `MAX_FREQ`, `fp`, `ghost_push` and `ghost_take` are untouched.

**Tests:** the parent's `adaptive_split_keeps_bounds` is now `static_split_keeps_bounds`. It checks that at capacity 64 the split is 6/58 and never changes. It also checks that each queue stays within its size limit, and that the ghost count matches the ghost map. I traced the other existing tests by hand and they should still pass.

**Existing quirk:** at capacity 1, `main_size` is 0. A ghost hit or a graduating entry can then leave 2 items resident in a cache of capacity 1. The parent and the original subject behave the same way, so I didn't change it. It only matters if a gate tests capacity 1.


---

- finished: 2026-10-20T12:34:38+00:00
- duration: 102.00s
- status: ok
- input_tokens: 22
- output_tokens: 10086
- cache_read_tokens: 302861
- cache_write_tokens: 27999
- cost_usd: 0.486372
