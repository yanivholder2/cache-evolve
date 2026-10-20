# Agent log: variant-implementer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0009/variants/v002/attempts/01
- allow_edits: True
- started: 2026-10-20T14:41:45+00:00

## timeline

### 14:41:50 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021; cat cycles/0009/hypotheses/H02.md; P=cycles/0008/variants/v001; find $P/attempts/01/overlay -type f; cat $P/insight.md $P/eval/fitness-vector.json; ls /home/user/cache-evolve/inputs/subjects/s3_fifo/src -R"
}
```

### 14:41:53 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0009/hypotheses/H02.md"
}
```

### 14:41:53 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0008/variants/v001/insight.md"
}
```

### 14:41:56 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0008/variants/v001/attempts/01/overlay"
}
```

### 14:41:56 tool: Glob

```json
{
  "pattern": "src/**/*",
  "path": "/home/user/cache-evolve/inputs/subjects/s3_fifo"
}
```

### 14:41:58 thinking



### 14:41:59 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0008/variants/v001/attempts/01/overlay/src/s3_fifo/mod.rs"
}
```

### 14:42:03 thinking



### 14:42:04 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0008/variants/v001/attempts/01/overlay/src/factory.rs"
}
```

### 14:42:04 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/s3_fifo/manifest.toml"
}
```

### 14:42:07 thinking



### 14:42:07 chat

I've read the hypothesis and the parent's `mod.rs`. Next I'm checking whether the invariant gate's test utilities touch `index`, and after that I'll write the new table.

### 14:42:07 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/s3_fifo/tests/invariant_gate.rs"
}
```

### 14:42:08 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/s3_fifo/src/test_utils/mod.rs"
}
```

### 14:42:10 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/s3_fifo/src/test_utils/invariants.rs"
}
```

### 14:42:30 thinking



### 14:42:30 chat

Writing the overlay now: a custom linear-probe `ResidentIndex` replaces `index`. The ghost map, policy logic and call counts stay the same, and I'm adding an oracle stress test.

### 14:44:09 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0009/variants/v002/attempts/01/overlay/src/s3_fifo/mod.rs",
  "content": "use std::collections::{HashMap, VecDeque};\nuse std::fmt::Debug;\nuse std::hash::{BuildHasher, Hash, Hasher};\nuse std::marker::PhantomData;\nuse std::mem::MaybeUninit;\n\npub mod s3_fifo_jni;\n\n// Simple S3-FIFO implementation derived from \"FIFO Queues are ALL You Need for\n// Cache Eviction\" by Juncheng Yang et al. The policy uses a small FIFO admission\n// queue, a main FIFO resident queue, and a ghost queue for recently evicted\n// items. Reaccesses to ghosted keys promote them into the main queue, and main\n// eviction uses a second-chance-style frequency decrement.\n//\n// Adaptive split (inherited from cycle/0001/v003): the small/main split adapts\n// at runtime (ARC-style). A ghost hit moves one slot from small to main; a\n// small-queue graduate arriving while main is full moves one slot from main back\n// to small. `small_size + main_size` always equals `capacity`; the ghost bound\n// stays fixed at the initial main size.\n//\n// Slab layout (inherited from cycle/0002/v003): entries live in a slab\n// (`slots`) addressed by `u32` indices; `small` / `main` are FIFOs of slot\n// indices; `index` maps resident keys to their slot.\n//\n// Ghost ring (inherited from cycle/0003/v002): the ghost is a fixed-size ring\n// of 64-bit key fingerprints (0 = empty slot) plus a fingerprint -> ring-position\n// map.\n//\n// put-path trim (inherited from cycle/0004/v003): one fingerprint per miss,\n// `alloc_with_key`, compare-based ring wrap.\n//\n// Split floor (inherited from cycle/0005/v001): the lower clamp on\n// `small_size` is `capacity / 64`.\n//\n// Hot/cold split (inherited from cycle/0006/v002): the key lives in a parallel\n// `keys: Vec<Option<K>>` array indexed by the same `u32` slot id.\n//\n// Freq out of Entry (inherited from cycle/0007/v001): `Entry` is `{ value }`\n// (8 bytes for u64/u64), so the slab is a dense `[V]`.\n//\n// Packed freq (inherited from cycle/0008/v001): the 2-bit frequency is packed\n// into the top bits of the `u32` stored in `index` as `(freq << FREQ_SHIFT) |\n// slot`.\n//\n// Variant cycle/0009/v002 (H02): policy-neutral layout refactor. The resident\n// `index` is no longer a hashbrown map but a custom power-of-two\n// open-addressing linear-probe table of `(K, u32)` buckets (`ResidentIndex`),\n// load factor <= 0.5, empty marked by value word `u32::MAX`, backward-shift\n// deletion (no tombstones). The ghost map stays an `FxHashMap`.\nconst MAX_FREQ: u32 = 3;\n\n/// Packed index value layout: low 30 bits = slot id, top 2 bits = freq.\nconst FREQ_SHIFT: u32 = 30;\nconst SLOT_MASK: u32 = (1 << FREQ_SHIFT) - 1;\nconst FREQ_ONE: u32 = 1 << FREQ_SHIFT;\n\n/// Divisor of the adaptive split's lower clamp: `small_size` never shrinks\n/// below `(capacity / SMALL_FLOOR_DIV).max(1)` on a ghost hit.\nconst SMALL_FLOOR_DIV: usize = 64;\n\n// ---------------------------------------------------------------------------\n// Inline FxHash (rustc-hash) — Cargo.toml cannot change, so no external crate.\n// ---------------------------------------------------------------------------\n\nconst FX_SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;\n\n#[derive(Debug, Default, Clone, Copy)]\npub struct FxHasher {\n\thash: u64,\n}\n\nimpl FxHasher {\n\t#[inline]\n\tfn add_to_hash(&mut self, word: u64) {\n\t\tself.hash = (self.hash.rotate_left(5) ^ word).wrapping_mul(FX_SEED);\n\t}\n}\n\nimpl Hasher for FxHasher {\n\t#[inline]\n\tfn write(&mut self, bytes: &[u8]) {\n\t\tlet mut chunks = bytes.chunks_exact(8);\n\t\tfor chunk in &mut chunks {\n\t\t\tlet mut buf = [0u8; 8];\n\t\t\tbuf.copy_from_slice(chunk);\n\t\t\tself.add_to_hash(u64::from_le_bytes(buf));\n\t\t}\n\t\tfor &b in chunks.remainder() {\n\t\t\tself.add_to_hash(b as u64);\n\t\t}\n\t}\n\n\t#[inline]\n\tfn write_u8(&mut self, i: u8) {\n\t\tself.add_to_hash(i as u64);\n\t}\n\n\t#[inline]\n\tfn write_u16(&mut self, i: u16) {\n\t\tself.add_to_hash(i as u64);\n\t}\n\n\t#[inline]\n\tfn write_u32(&mut self, i: u32) {\n\t\tself.add_to_hash(i as u64);\n\t}\n\n\t#[inline]\n\tfn write_u64(&mut self, i: u64) {\n\t\tself.add_to_hash(i);\n\t}\n\n\t#[inline]\n\tfn write_usize(&mut self, i: usize) {\n\t\tself.add_to_hash(i as u64);\n\t}\n\n\t#[inline]\n\tfn finish(&self) -> u64 {\n\t\tself.hash\n\t}\n}\n\n#[derive(Debug, Default, Clone, Copy)]\npub struct FxBuildHasher;\n\nimpl BuildHasher for FxBuildHasher {\n\ttype Hasher = FxHasher;\n\n\t#[inline]\n\tfn build_hasher(&self) -> FxHasher {\n\t\tFxHasher::default()\n\t}\n}\n\ntype FxHashMap<K, V> = HashMap<K, V, FxBuildHasher>;\n\n/// Plain FxHash of a key.\n#[inline]\nfn fx_hash<K: Hash + ?Sized>(key: &K) -> u64 {\n\tlet mut h = FxBuildHasher.build_hasher();\n\tkey.hash(&mut h);\n\th.finish()\n}\n\n/// Ghost fingerprint of a key: FxHash of the key, forced odd so it is never 0\n/// (0 marks an empty ring slot).\n#[inline]\nfn fp<K: Hash + ?Sized>(key: &K) -> u64 {\n\tfx_hash(key) | 1\n}\n\n// ---------------------------------------------------------------------------\n// ResidentIndex: power-of-two open-addressing linear-probe table.\n// ---------------------------------------------------------------------------\n\n/// Value word marking an empty bucket. A live packed word is\n/// `(freq << 30) | slot` with `slot < capacity <= SLOT_MASK`, so it never\n/// equals `u32::MAX`.\nconst EMPTY: u32 = u32::MAX;\n\n/// Resident key -> packed `u32` map. Buckets are `(key, word)`; the key is\n/// initialised iff `word != EMPTY`. Load factor <= 0.5 (callers keep at most\n/// `capacity` entries in `(2 * capacity).next_power_of_two()` buckets), so\n/// every probe sequence terminates on an `EMPTY` bucket.\nstruct ResidentIndex<K> {\n\tbuckets: Vec<(MaybeUninit<K>, u32)>,\n\tmask: usize,\n\tshift: u32,\n\tlen: usize,\n}\n\nimpl<K> Debug for ResidentIndex<K> {\n\tfn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n\t\tf.debug_struct(\"ResidentIndex\")\n\t\t\t.field(\"buckets\", &self.buckets.len())\n\t\t\t.field(\"len\", &self.len)\n\t\t\t.finish()\n\t}\n}\n\nimpl<K> Drop for ResidentIndex<K> {\n\tfn drop(&mut self) {\n\t\tif std::mem::needs_drop::<K>() {\n\t\t\tfor b in self.buckets.iter_mut() {\n\t\t\t\tif b.1 != EMPTY {\n\t\t\t\t\tb.1 = EMPTY;\n\t\t\t\t\t// SAFETY: word != EMPTY means the key is initialised.\n\t\t\t\t\tunsafe { b.0.assume_init_drop() };\n\t\t\t\t}\n\t\t\t}\n\t\t}\n\t}\n}\n\nimpl<K: Hash + Eq> ResidentIndex<K> {\n\tfn with_capacity(capacity: usize) -> Self {\n\t\tlet n = (2 * capacity.max(1)).next_power_of_two();\n\t\tlet mut buckets = Vec::with_capacity(n);\n\t\tbuckets.resize_with(n, || (MaybeUninit::uninit(), EMPTY));\n\t\tSelf { buckets, mask: n - 1, shift: 64 - n.trailing_zeros(), len: 0 }\n\t}\n\n\t/// Home bucket: high bits of a Fibonacci multiply of the FxHash.\n\t#[inline]\n\tfn home(&self, key: &K) -> usize {\n\t\t(fx_hash(key).wrapping_mul(FX_SEED) >> self.shift) as usize\n\t}\n\n\t/// Bucket position holding `key`, if present.\n\t#[inline]\n\tfn find(&self, key: &K) -> Option<usize> {\n\t\tlet mut i = self.home(key);\n\t\tloop {\n\t\t\tdebug_assert!(i < self.buckets.len());\n\t\t\t// SAFETY: `home` < n and `& mask` keeps `i` < n.\n\t\t\tlet b = unsafe { self.buckets.get_unchecked(i) };\n\t\t\tif b.1 == EMPTY {\n\t\t\t\treturn None;\n\t\t\t}\n\t\t\t// SAFETY: word != EMPTY means the key is initialised.\n\t\t\tif unsafe { b.0.assume_init_ref() } == key {\n\t\t\t\treturn Some(i);\n\t\t\t}\n\t\t\ti = (i + 1) & self.mask;\n\t\t}\n\t}\n\n\t#[inline]\n\tfn get(&self, key: &K) -> Option<u32> {\n\t\tlet i = self.find(key)?;\n\t\t// SAFETY: `find` returns an in-bounds position.\n\t\tSome(unsafe { self.buckets.get_unchecked(i) }.1)\n\t}\n\n\t#[inline]\n\tfn get_mut(&mut self, key: &K) -> Option<&mut u32> {\n\t\tlet i = self.find(key)?;\n\t\t// SAFETY: `find` returns an in-bounds position.\n\t\tSome(&mut unsafe { self.buckets.get_unchecked_mut(i) }.1)\n\t}\n\n\t/// Insert an absent key. Callers guarantee absence.\n\t#[inline]\n\tfn insert(&mut self, key: K, word: u32) {\n\t\tdebug_assert!(self.find(&key).is_none(), \"ResidentIndex: duplicate insert\");\n\t\tdebug_assert!(word != EMPTY);\n\t\tdebug_assert!(self.len < self.buckets.len() / 2 + 1);\n\t\tlet mut i = self.home(&key);\n\t\tloop {\n\t\t\t// SAFETY: `i` < n (see `find`).\n\t\t\tlet b = unsafe { self.buckets.get_unchecked_mut(i) };\n\t\t\tif b.1 == EMPTY {\n\t\t\t\tb.0.write(key);\n\t\t\t\tb.1 = word;\n\t\t\t\tself.len += 1;\n\t\t\t\treturn;\n\t\t\t}\n\t\t\ti = (i + 1) & self.mask;\n\t\t}\n\t}\n\n\t/// Remove `key` with backward-shift deletion; returns its word.\n\t#[inline]\n\tfn remove(&mut self, key: &K) -> Option<u32> {\n\t\tlet mut hole = self.find(key)?;\n\t\tlet mask = self.mask;\n\t\tlet word = self.buckets[hole].1;\n\t\tself.buckets[hole].1 = EMPTY;\n\t\t// SAFETY: the bucket was occupied; it is now marked EMPTY so the key\n\t\t// is read out exactly once.\n\t\tdrop(unsafe { self.buckets[hole].0.assume_init_read() });\n\t\tself.len -= 1;\n\t\tlet mut j = hole;\n\t\tloop {\n\t\t\tj = (j + 1) & mask;\n\t\t\tif self.buckets[j].1 == EMPTY {\n\t\t\t\tbreak;\n\t\t\t}\n\t\t\t// SAFETY: occupied bucket.\n\t\t\tlet h = self.home(unsafe { self.buckets[j].0.assume_init_ref() });\n\t\t\t// Move j back into the hole iff its home is cyclically at or\n\t\t\t// before the hole, i.e. dist(home -> j) >= dist(hole -> j).\n\t\t\tif (j.wrapping_sub(h) & mask) >= (j.wrapping_sub(hole) & mask) {\n\t\t\t\tlet w = self.buckets[j].1;\n\t\t\t\tself.buckets[j].1 = EMPTY;\n\t\t\t\t// SAFETY: bucket j was occupied and is now marked EMPTY; the\n\t\t\t\t// hole bucket is EMPTY (uninitialised) and receives the key.\n\t\t\t\tlet k = unsafe { self.buckets[j].0.assume_init_read() };\n\t\t\t\tself.buckets[hole].0.write(k);\n\t\t\t\tself.buckets[hole].1 = w;\n\t\t\t\thole = j;\n\t\t\t}\n\t\t}\n\t\tSome(word)\n\t}\n\n\t#[inline]\n\tfn len(&self) -> usize {\n\t\tself.len\n\t}\n}\n\n// ---------------------------------------------------------------------------\n\n/// Key-free, freq-free slab entry. The key lives in the parallel\n/// `S3Fifo::keys` array and the frequency in the packed `S3Fifo::index` value.\n#[derive(Debug)]\npub struct Entry<K, V> {\n\tvalue: V,\n\t_k: PhantomData<K>,\n}\n\nimpl<K, V> Entry<K, V> {\n\t#[inline]\n\tfn new(value: V) -> Self {\n\t\tSelf { value, _k: PhantomData }\n\t}\n}\n\n/// Test-only instrumentation of the adaptive split.\n#[cfg(test)]\n#[derive(Debug, Default, Clone, Copy)]\nstruct SplitStats {\n\t/// Ghost hits that shrank small by one slot.\n\tshrinks: u64,\n\t/// Ghost hits on which the floor guard blocked a shrink.\n\tfloor_blocks: u64,\n\t/// Graduates-while-main-full that grew small by one slot.\n\tgrows: u64,\n\t/// Smallest `small_size` observed.\n\tmin_small: usize,\n}\n\n#[derive(Debug)]\npub struct S3Fifo<K: PartialEq + Clone, V> {\n\t/// Slab of entries. A free slot (listed in `free`) keeps its stale value\n\t/// until the next alloc overwrites it; liveness is tracked by `keys`.\n\tslots: Vec<Entry<K, V>>,\n\t/// Parallel to `slots`: the resident key of each slot, `None` if free.\n\tkeys: Vec<Option<K>>,\n\tfree: Vec<u32>,\n\t/// Resident key -> packed `(freq << FREQ_SHIFT) | slot`.\n\tindex: ResidentIndex<K>,\n\tsmall: VecDeque<u32>,\n\tmain: VecDeque<u32>,\n\t/// Ghost ring of key fingerprints; 0 = empty slot. Fixed length.\n\tghost_ring: Vec<u64>,\n\t/// Next ring write position (wraps).\n\tghost_head: usize,\n\t/// Live fingerprint -> ring position.\n\tghost_map: FxHashMap<u64, u32>,\n\tghost_live: usize,\n\tsmall_size: usize,\n\tmain_size: usize,\n\tcapacity: usize,\n\tghost_cap: usize,\n\t/// Test-only: override of `SMALL_FLOOR_DIV`.\n\t#[cfg(test)]\n\tfloor_div: usize,\n\t#[cfg(test)]\n\tstats: SplitStats,\n}\n\nimpl<K: PartialEq + Eq + Hash + Clone, V> S3Fifo<K, V> {\n\tpub fn try_new(capacity: usize) -> Option<Self> {\n\t\tif capacity == 0 {\n\t\t\tlog::error!(\"S3Fifo.try_new: capacity must be > 0\");\n\t\t\treturn None;\n\t\t}\n\t\t// `capacity <= SLOT_MASK` keeps every slot id < SLOT_MASK, so a live\n\t\t// packed word never equals the index's `EMPTY` marker.\n\t\tif capacity > SLOT_MASK as usize {\n\t\t\tlog::error!(\"S3Fifo.try_new: capacity must fit in 30-bit slot id\");\n\t\t\treturn None;\n\t\t}\n\t\tlet small_size = if capacity == 1 {\n\t\t\t1\n\t\t} else {\n\t\t\t(capacity / 4).max(1).min(capacity - 1)\n\t\t};\n\t\tlet main_size = capacity - small_size;\n\t\tlet index = ResidentIndex::with_capacity(capacity);\n\t\t// The ring needs at least one slot so the wrap is defined. Ghost sizing\n\t\t// stays pinned to the *initial* main size, independent of the split floor.\n\t\tlet ring_len = main_size.max(1);\n\t\tSome(Self {\n\t\t\tslots: Vec::with_capacity(capacity),\n\t\t\tkeys: Vec::with_capacity(capacity),\n\t\t\tfree: Vec::with_capacity(capacity),\n\t\t\tindex,\n\t\t\tsmall: VecDeque::with_capacity(capacity),\n\t\t\tmain: VecDeque::with_capacity(capacity),\n\t\t\tghost_ring: vec![0u64; ring_len],\n\t\t\tghost_head: 0,\n\t\t\tghost_map: FxHashMap::with_capacity_and_hasher(ring_len + 1, FxBuildHasher),\n\t\t\tghost_live: 0,\n\t\t\tsmall_size,\n\t\t\tmain_size,\n\t\t\tcapacity,\n\t\t\tghost_cap: main_size,\n\t\t\t#[cfg(test)]\n\t\t\tfloor_div: SMALL_FLOOR_DIV,\n\t\t\t#[cfg(test)]\n\t\t\tstats: SplitStats { min_small: small_size, ..SplitStats::default() },\n\t\t})\n\t}\n\n\tpub fn new(capacity: usize) -> Self {\n\t\tSelf::try_new(capacity).expect(\"S3Fifo::new: initialization failed\")\n\t}\n\n\t/// Current `(small_size, main_size)` split.\n\t#[inline]\n\tpub fn split(&self) -> (usize, usize) {\n\t\t(self.small_size, self.main_size)\n\t}\n\n\t#[inline]\n\tfn adaptive(&self) -> bool {\n\t\tself.capacity >= 4\n\t}\n\n\t/// Lower clamp on `small_size` for ghost-hit shrinking.\n\t#[inline]\n\tfn small_floor(&self) -> usize {\n\t\t#[cfg(test)]\n\t\tlet div = self.floor_div;\n\t\t#[cfg(not(test))]\n\t\tlet div = SMALL_FLOOR_DIV;\n\t\t(self.capacity / div).max(1)\n\t}\n\n\t// ---- slab helpers -----------------------------------------------------\n\n\t#[cfg(test)]\n\t#[inline]\n\tfn slot(&self, i: u32) -> &Entry<K, V> {\n\t\t&self.slots[i as usize]\n\t}\n\n\t#[inline]\n\tfn slot_mut(&mut self, i: u32) -> &mut Entry<K, V> {\n\t\t&mut self.slots[i as usize]\n\t}\n\n\t/// Mutable packed index entry of resident slot `i` (one probe).\n\t#[inline]\n\tfn packed_mut(&mut self, i: u32) -> &mut u32 {\n\t\tlet k = self.keys[i as usize].as_ref().expect(\"S3Fifo: queue references a free slot\");\n\t\tself.index.get_mut(k).expect(\"S3Fifo: resident key missing from index\")\n\t}\n\n\t/// Store a new resident entry (freq 0) and register it in `index`.\n\t/// Exactly one key clone and one `index` insert.\n\t#[inline]\n\tfn alloc_with_key(&mut self, key: K, value: V) -> u32 {\n\t\tmatch self.free.pop() {\n\t\t\tSome(i) => {\n\t\t\t\tself.index.insert(key.clone(), i);\n\t\t\t\tself.keys[i as usize] = Some(key);\n\t\t\t\tself.slots[i as usize] = Entry::new(value);\n\t\t\t\ti\n\t\t\t}\n\t\t\tNone => {\n\t\t\t\tlet i = self.slots.len() as u32;\n\t\t\t\tself.index.insert(key.clone(), i);\n\t\t\t\tself.keys.push(Some(key));\n\t\t\t\tself.slots.push(Entry::new(value));\n\t\t\t\ti\n\t\t\t}\n\t\t}\n\t}\n\n\t/// Store an entry with a preset frequency (test helpers only).\n\t#[cfg(test)]\n\tfn alloc(&mut self, key: K, value: V, freq: u8) -> u32 {\n\t\tlet i = self.alloc_with_key(key, value);\n\t\tlet e = self.packed_mut(i);\n\t\t*e = ((freq as u32) << FREQ_SHIFT) | (*e & SLOT_MASK);\n\t\ti\n\t}\n\n\t/// Remove a resident slot from `keys` and `index`; return its key. The\n\t/// value stays in `slots[i]` until the next alloc overwrites it; the packed\n\t/// freq leaves with the index entry.\n\t#[inline]\n\tfn release(&mut self, i: u32) -> K {\n\t\tlet k = self.keys[i as usize].take().expect(\"S3Fifo: double release of slot\");\n\t\tself.index.remove(&k);\n\t\tself.free.push(i);\n\t\tk\n\t}\n\n\t// ---- ghost helpers ----------------------------------------------------\n\n\t/// O(1) ghost membership test + removal by precomputed fingerprint.\n\t#[inline]\n\tfn ghost_take_fp(&mut self, f: u64) -> bool {\n\t\tif let Some(p) = self.ghost_map.remove(&f) {\n\t\t\tself.ghost_ring[p as usize] = 0;\n\t\t\tself.ghost_live -= 1;\n\t\t\ttrue\n\t\t} else {\n\t\t\tfalse\n\t\t}\n\t}\n\n\t/// Record `key` in the ghost ring, overwriting (evicting) the oldest slot.\n\t#[inline]\n\tfn ghost_push(&mut self, key: &K) {\n\t\tlet f = fp(key);\n\t\tlet h = self.ghost_head;\n\t\tlet old = self.ghost_ring[h];\n\t\tif old != 0 {\n\t\t\tif self.ghost_map.get(&old) == Some(&(h as u32)) {\n\t\t\t\tself.ghost_map.remove(&old);\n\t\t\t}\n\t\t\tself.ghost_live -= 1;\n\t\t}\n\t\tif let Some(p) = self.ghost_map.insert(f, h as u32) {\n\t\t\t// Same fingerprint already live elsewhere: clear its old slot so it\n\t\t\t// is not counted twice.\n\t\t\tif p as usize != h {\n\t\t\t\tself.ghost_ring[p as usize] = 0;\n\t\t\t\tself.ghost_live -= 1;\n\t\t\t}\n\t\t}\n\t\tself.ghost_ring[h] = f;\n\t\tself.ghost_live += 1;\n\t\tlet n = h + 1;\n\t\tself.ghost_head = if n == self.ghost_ring.len() { 0 } else { n };\n\t}\n\n\t// ---- public API -------------------------------------------------------\n\n\t#[inline]\n\tpub fn get(&mut self, key: &K) -> Option<&V> {\n\t\tlet e = self.index.get_mut(key)?;\n\t\tif (*e >> FREQ_SHIFT) < MAX_FREQ {\n\t\t\t*e += FREQ_ONE;\n\t\t}\n\t\tlet i = (*e & SLOT_MASK) as usize;\n\t\tSome(&self.slots[i].value)\n\t}\n\n\t#[inline]\n\tpub fn put(&mut self, key: K, value: V) {\n\t\tif let Some(e) = self.index.get(&key) {\n\t\t\tself.slot_mut(e & SLOT_MASK).value = value;\n\t\t\treturn;\n\t\t}\n\n\t\tlet f = fp(&key);\n\t\tif self.ghost_take_fp(f) {\n\t\t\t// Ghost hit -> grow main by one slot (shrink small).\n\t\t\tif self.adaptive() && self.small_size > self.small_floor() {\n\t\t\t\tself.small_size -= 1;\n\t\t\t\tself.main_size += 1;\n\t\t\t\t#[cfg(test)]\n\t\t\t\t{\n\t\t\t\t\tself.stats.shrinks += 1;\n\t\t\t\t\tself.stats.min_small = self.stats.min_small.min(self.small_size);\n\t\t\t\t}\n\t\t\t\tif self.small.len() > self.small_size {\n\t\t\t\t\t// No adaptation on this fix-up eviction, so the ghost-driven\n\t\t\t\t\t// growth is not immediately undone by the graduate rule.\n\t\t\t\t\tself.evict_small_inner(false);\n\t\t\t\t}\n\t\t\t\tif self.main.len() > self.main_size {\n\t\t\t\t\tself.evict_main();\n\t\t\t\t}\n\t\t\t} else {\n\t\t\t\t#[cfg(test)]\n\t\t\t\t{\n\t\t\t\t\tif self.adaptive() {\n\t\t\t\t\t\tself.stats.floor_blocks += 1;\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t\tif self.main.len() >= self.main_size {\n\t\t\t\tself.evict_main();\n\t\t\t}\n\t\t\tlet i = self.alloc_with_key(key, value);\n\t\t\tself.main.push_front(i);\n\t\t\treturn;\n\t\t}\n\n\t\tif self.small.len() >= self.small_size {\n\t\t\tself.evict_small();\n\t\t}\n\t\tlet i = self.alloc_with_key(key, value);\n\t\tself.small.push_front(i);\n\t}\n\n\t#[inline]\n\tfn evict_main(&mut self) {\n\t\twhile let Some(i) = self.main.pop_back() {\n\t\t\tlet e = self.packed_mut(i);\n\t\t\tif (*e >> FREQ_SHIFT) > 0 {\n\t\t\t\t*e -= FREQ_ONE;\n\t\t\t\tself.main.push_front(i);\n\t\t\t} else {\n\t\t\t\tself.release(i);\n\t\t\t\tbreak;\n\t\t\t}\n\t\t}\n\t}\n\n\t#[inline]\n\tfn evict_small(&mut self) {\n\t\tself.evict_small_inner(true);\n\t}\n\n\t#[inline]\n\tfn evict_small_inner(&mut self, adapt: bool) {\n\t\tif let Some(i) = self.small.pop_back() {\n\t\t\tif (*self.packed_mut(i) >> FREQ_SHIFT) > 1 {\n\t\t\t\tlet main_full = self.main.len() >= self.main_size;\n\t\t\t\tlet grow_small =\n\t\t\t\t\tadapt && self.adaptive() && main_full && self.small_size < self.capacity / 2;\n\t\t\t\tif main_full {\n\t\t\t\t\tself.evict_main();\n\t\t\t\t}\n\t\t\t\tself.main.push_front(i);\n\t\t\t\tif grow_small {\n\t\t\t\t\t// Main is crowded with reused small-queue keys -> give one\n\t\t\t\t\t// slot back to small.\n\t\t\t\t\tself.small_size += 1;\n\t\t\t\t\tself.main_size -= 1;\n\t\t\t\t\t#[cfg(test)]\n\t\t\t\t\t{\n\t\t\t\t\t\tself.stats.grows += 1;\n\t\t\t\t\t}\n\t\t\t\t\tif self.main.len() > self.main_size {\n\t\t\t\t\t\tself.evict_main();\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t} else {\n\t\t\t\tlet key = self.release(i);\n\t\t\t\tself.ghost_push(&key);\n\t\t\t}\n\t\t}\n\t}\n\n\t// ---- test helpers -----------------------------------------------------\n\n\t#[cfg(test)]\n\tfn key_of(&self, i: u32) -> &K {\n\t\tself.keys[i as usize].as_ref().expect(\"S3Fifo: queue references a free slot\")\n\t}\n\n\t#[cfg(test)]\n\tfn in_main(&self, key: &K) -> bool {\n\t\tself.main.iter().any(|&i| self.key_of(i) == key)\n\t}\n\n\t#[cfg(test)]\n\tfn push_main_back_for_test(&mut self, key: K, value: V, freq: u8) {\n\t\tlet i = self.alloc(key, value, freq);\n\t\tself.main.push_back(i);\n\t}\n\n\t#[cfg(test)]\n\tfn push_small_front_for_test(&mut self, key: K, value: V, freq: u8) {\n\t\tlet i = self.alloc(key, value, freq);\n\t\tself.small.push_front(i);\n\t}\n}\n\nimpl<K: PartialEq + Eq + std::hash::Hash + Clone + Debug, V: Debug> framework::Cache<K, V> for S3Fifo<K, V> {\n\t#[inline]\n\tfn put(&mut self, key: K, value: V) {\n\t\tS3Fifo::put(self, key, value);\n\t}\n\n\t#[inline]\n\tfn get(&mut self, key: &K) -> Option<&V> {\n\t\tS3Fifo::get(self, key)\n\t}\n}\n\n#[cfg(test)]\nmod tests {\n\tuse super::*;\n\n\t#[test]\n\tfn entry_is_8_bytes() {\n\t\tassert_eq!(std::mem::size_of::<Entry<u64, u64>>(), 8);\n\t\tassert_eq!(std::mem::size_of::<Entry<i64, i64>>(), 8);\n\t\t// Index bucket stays narrow: (K, packed u32) for u64 keys.\n\t\tassert_eq!(std::mem::size_of::<(u64, u32)>(), 16);\n\t}\n\n\t#[test]\n\tfn packed_layout_fits_max_freq() {\n\t\tassert_eq!(SLOT_MASK, (1u32 << 30) - 1);\n\t\tassert!(MAX_FREQ <= (u32::MAX >> FREQ_SHIFT));\n\t\tassert!(S3Fifo::<i64, i64>::try_new(0).is_none());\n\t\t// Largest live word never collides with EMPTY.\n\t\tassert_ne!((MAX_FREQ << FREQ_SHIFT) | (SLOT_MASK - 1), EMPTY);\n\t\tassert_eq!(std::mem::size_of::<(MaybeUninit<u64>, u32)>(), 16);\n\t}\n\n\t#[test]\n\tfn resident_index_sizing() {\n\t\tlet idx = ResidentIndex::<i64>::with_capacity(2048);\n\t\tassert_eq!(idx.buckets.len(), 4096);\n\t\tassert_eq!(idx.mask, 4095);\n\t\tassert_eq!(idx.shift, 52);\n\t\tlet one = ResidentIndex::<i64>::with_capacity(1);\n\t\tassert_eq!(one.buckets.len(), 2);\n\t\tlet odd = ResidentIndex::<i64>::with_capacity(3);\n\t\tassert_eq!(odd.buckets.len(), 8);\n\t}\n\n\t/// xorshift64* for the stress test.\n\tfn xs(state: &mut u64) -> u64 {\n\t\tlet mut x = *state;\n\t\tx ^= x >> 12;\n\t\tx ^= x << 25;\n\t\tx ^= x >> 27;\n\t\t*state = x;\n\t\tx.wrapping_mul(0x2545_F491_4F6C_DD1D)\n\t}\n\n\tfn stress(cap: usize, key_space: u64, ops: usize, seed: u64) {\n\t\tlet mut idx = ResidentIndex::<i64>::with_capacity(cap);\n\t\tlet mut oracle: std::collections::HashMap<i64, u32> = std::collections::HashMap::new();\n\t\tlet mut st = seed | 1;\n\t\tfor step in 0..ops {\n\t\t\tlet r = xs(&mut st);\n\t\t\tlet k = (r % key_space) as i64;\n\t\t\tmatch (r >> 40) % 3 {\n\t\t\t\t0 => {\n\t\t\t\t\tif !oracle.contains_key(&k) && oracle.len() < cap {\n\t\t\t\t\t\tlet w = (step as u32) & SLOT_MASK;\n\t\t\t\t\t\tidx.insert(k, w);\n\t\t\t\t\t\toracle.insert(k, w);\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t\t1 => {\n\t\t\t\t\tassert_eq!(idx.remove(&k), oracle.remove(&k));\n\t\t\t\t}\n\t\t\t\t_ => {\n\t\t\t\t\tif let Some(w) = idx.get_mut(&k) {\n\t\t\t\t\t\t*w = (*w + 1) & SLOT_MASK;\n\t\t\t\t\t}\n\t\t\t\t\tif let Some(w) = oracle.get_mut(&k) {\n\t\t\t\t\t\t*w = (*w + 1) & SLOT_MASK;\n\t\t\t\t\t}\n\t\t\t\t\tassert_eq!(idx.get(&k), oracle.get(&k).copied());\n\t\t\t\t}\n\t\t\t}\n\t\t\tassert_eq!(idx.len(), oracle.len());\n\t\t}\n\t\tfor k in 0..key_space as i64 {\n\t\t\tassert_eq!(idx.get(&k), oracle.get(&k).copied(), \"key {k}\");\n\t\t}\n\t\tlet occupied = idx.buckets.iter().filter(|b| b.1 != EMPTY).count();\n\t\tassert_eq!(occupied, oracle.len());\n\t}\n\n\t#[test]\n\tfn resident_index_matches_hashmap_oracle() {\n\t\t// Tiny tables force probe chains to wrap around the bucket array.\n\t\tfor seed in 1..40u64 {\n\t\t\tstress(1, 4, 2_000, seed);\n\t\t\tstress(2, 8, 2_000, seed);\n\t\t\tstress(3, 16, 5_000, seed);\n\t\t\tstress(8, 64, 5_000, seed);\n\t\t}\n\t\tstress(64, 200, 100_000, 0xDEAD_BEEF);\n\t\tstress(2048, 10_000, 300_000, 0x1234_5678);\n\t\t// Keys whose home is the last bucket wrap to the front.\n\t\tstress(4, 1_000, 50_000, 0xABCD);\n\t}\n\n\t#[test]\n\tfn resident_index_wraparound_explicit() {\n\t\tlet mut idx = ResidentIndex::<i64>::with_capacity(4); // 8 buckets\n\t\tlet last = idx.mask;\n\t\tlet homed: Vec<i64> = (0..10_000i64).filter(|k| idx.home(k) == last).take(3).collect();\n\t\tassert_eq!(homed.len(), 3);\n\t\tfor (n, &k) in homed.iter().enumerate() {\n\t\t\tidx.insert(k, n as u32);\n\t\t}\n\t\t// Second and third wrapped into buckets 0 and 1.\n\t\tassert_ne!(idx.buckets[0].1, EMPTY);\n\t\tassert_ne!(idx.buckets[1].1, EMPTY);\n\t\tassert_eq!(idx.remove(&homed[0]), Some(0));\n\t\tassert_eq!(idx.get(&homed[1]), Some(1));\n\t\tassert_eq!(idx.get(&homed[2]), Some(2));\n\t\tassert_eq!(idx.remove(&homed[1]), Some(1));\n\t\tassert_eq!(idx.get(&homed[2]), Some(2));\n\t\tassert_eq!(idx.buckets[last].1, 2);\n\t\tassert_eq!(idx.len(), 1);\n\t}\n\n\t#[test]\n\tfn resident_index_drops_keys() {\n\t\tuse std::rc::Rc;\n\t\tlet probe = Rc::new(());\n\t\t{\n\t\t\tlet mut idx = ResidentIndex::<(i64, Rc<()>)>::with_capacity(8);\n\t\t\tfor k in 0..8i64 {\n\t\t\t\tidx.insert((k, probe.clone()), k as u32);\n\t\t\t}\n\t\t\tassert_eq!(Rc::strong_count(&probe), 9);\n\t\t\tidx.remove(&(3, probe.clone()));\n\t\t\tassert_eq!(Rc::strong_count(&probe), 8);\n\t\t}\n\t\tassert_eq!(Rc::strong_count(&probe), 1);\n\t}\n\n\t#[test]\n\tfn get_miss_returns_none() {\n\t\tlet mut cache = S3Fifo::<i64, i64>::new(4);\n\t\tassert!(cache.get(&42).is_none());\n\t}\n\n\t#[test]\n\tfn put_then_get_finds_value() {\n\t\tlet mut cache = S3Fifo::new(4);\n\t\tcache.put(1, 10);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 10);\n\t}\n\n\t#[test]\n\tfn put_existing_updates_value() {\n\t\tlet mut cache = S3Fifo::new(4);\n\t\tcache.put(1, 10);\n\t\tcache.put(1, 20);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 20);\n\t}\n\n\t#[test]\n\tfn small_eviction_promotes_to_main_after_multiple_hits() {\n\t\tlet mut cache = S3Fifo::new(4);\n\t\tcache.put(1, 10);\n\t\tcache.get(&1);\n\t\tcache.get(&1);\n\t\tcache.put(2, 20);\n\t\tassert!(cache.in_main(&1));\n\t}\n\n\t#[test]\n\tfn ghost_hit_promotes_back_to_main() {\n\t\tlet mut cache = S3Fifo::new(4);\n\t\tcache.put(1, 10);\n\t\tcache.put(2, 20);\n\t\tcache.put(3, 30);\n\t\tcache.put(4, 40);\n\t\tcache.put(1, 10);\n\t\tassert!(cache.in_main(&1));\n\t}\n\n\t#[test]\n\tfn main_eviction_uses_frequency_second_chance() {\n\t\tlet mut cache = S3Fifo::new(5);\n\t\tcache.push_main_back_for_test(1, 10, 0);\n\t\tcache.push_main_back_for_test(2, 20, 2);\n\t\tcache.push_main_back_for_test(3, 30, 0);\n\t\tcache.push_main_back_for_test(4, 40, 1);\n\t\tcache.push_small_front_for_test(5, 50, 2);\n\t\tcache.evict_small();\n\t\tassert!(cache.in_main(&4));\n\t\tassert!(!cache.in_main(&3));\n\t\tassert!(cache.get(&3).is_none());\n\t}\n\n\t#[test]\n\tfn capacity_bound_never_exceeded() {\n\t\tlet mut cache = S3Fifo::new(4);\n\t\tfor key in 0..20 {\n\t\t\tcache.put(key, key);\n\t\t}\n\t\tassert!(cache.small.len() + cache.main.len() <= 4);\n\t\tassert_eq!(cache.index.len(), cache.small.len() + cache.main.len());\n\t}\n\n\t#[test]\n\tfn adaptive_split_keeps_bounds() {\n\t\tlet cap = 64;\n\t\tlet mut cache = S3Fifo::new(cap);\n\t\tfor round in 0..50i64 {\n\t\t\tfor key in 0..200i64 {\n\t\t\t\tlet k = (key * 7 + round) % 150;\n\t\t\t\tif cache.get(&k).is_none() {\n\t\t\t\t\tcache.put(k, k);\n\t\t\t\t}\n\t\t\t\tassert!(cache.small.len() + cache.main.len() <= cap);\n\t\t\t\tassert_eq!(cache.small_size + cache.main_size, cap);\n\t\t\t\tassert!(cache.small_size >= 1 && cache.main_size >= 1);\n\t\t\t\tassert!(cache.small_size >= cache.small_floor());\n\t\t\t\tassert!(cache.ghost_live <= cache.ghost_cap);\n\t\t\t\tassert_eq!(cache.ghost_live, cache.ghost_map.len());\n\t\t\t\tassert_eq!(cache.slots.len(), cache.keys.len());\n\t\t\t\tassert!(cache.slots.len() <= cap);\n\t\t\t}\n\t\t}\n\t}\n\n\t#[test]\n\tfn small_floor_is_capacity_over_64() {\n\t\tlet cache = S3Fifo::<i64, i64>::new(2048);\n\t\tassert_eq!(cache.small_floor(), 32);\n\t\tassert_eq!(cache.ghost_cap, 1536);\n\t\tassert_eq!(cache.ghost_ring.len(), 1536);\n\t\tassert_eq!(cache.split(), (512, 1536));\n\t\tlet tiny = S3Fifo::<i64, i64>::new(4);\n\t\tassert_eq!(tiny.small_floor(), 1);\n\t}\n\n\t#[test]\n\tfn index_tracks_residency() {\n\t\tlet cap = 64;\n\t\tlet mut cache = S3Fifo::new(cap);\n\t\tfor round in 0..50i64 {\n\t\t\tfor key in 0..200i64 {\n\t\t\t\tlet k = (key * 7 + round) % 150;\n\t\t\t\tif cache.get(&k).is_none() {\n\t\t\t\t\tcache.put(k, k);\n\t\t\t\t}\n\t\t\t\tassert_eq!(cache.index.len(), cache.small.len() + cache.main.len());\n\t\t\t\tfor &i in cache.small.iter().chain(cache.main.iter()) {\n\t\t\t\t\tlet key = cache.key_of(i);\n\t\t\t\t\tlet e = cache.index.get(key).unwrap();\n\t\t\t\t\tassert_eq!(e & SLOT_MASK, i);\n\t\t\t\t\tassert!((e >> FREQ_SHIFT) <= MAX_FREQ);\n\t\t\t\t\tassert_eq!(cache.slot(i).value, *key);\n\t\t\t\t\tassert!(!cache.ghost_map.contains_key(&fp(key)));\n\t\t\t\t}\n\t\t\t\tlet live = cache.keys.iter().filter(|k| k.is_some()).count();\n\t\t\t\tassert_eq!(live, cache.index.len());\n\t\t\t\tassert_eq!(live + cache.free.len(), cache.keys.len());\n\t\t\t}\n\t\t}\n\t}\n\n\t#[test]\n\tfn ghost_ring_wraps() {\n\t\tlet mut cache = S3Fifo::<i64, i64>::new(8);\n\t\tfor key in 0..1000i64 {\n\t\t\tcache.put(key, key);\n\t\t\tassert!(cache.ghost_head < cache.ghost_ring.len());\n\t\t}\n\t}\n\n\t#[test]\n\tfn repeated_get_clamps_frequency() {\n\t\tlet mut cache = S3Fifo::new(4);\n\t\tcache.put(1, 10);\n\t\tfor _ in 0..10 {\n\t\t\tassert_eq!(*cache.get(&1).unwrap(), 10);\n\t\t}\n\t\tassert_eq!(cache.index.get(&1).unwrap() >> FREQ_SHIFT, MAX_FREQ);\n\t}\n\n\t// ---- split instrumentation (inherited from cycle/0005/v001) ------------\n\n\t/// Deterministic Zipf-like key stream (inverse-CDF over `n` ranks, skew\n\t/// `alpha`, xorshift64* PRNG), ranks scrambled so key order != popularity.\n\tstruct Zipf {\n\t\tcdf: Vec<f64>,\n\t\tstate: u64,\n\t}\n\n\timpl Zipf {\n\t\tfn new(n: usize, alpha: f64, seed: u64) -> Self {\n\t\t\tlet mut cdf = Vec::with_capacity(n);\n\t\t\tlet mut acc = 0.0;\n\t\t\tfor r in 1..=n {\n\t\t\t\tacc += 1.0 / (r as f64).powf(alpha);\n\t\t\t\tcdf.push(acc);\n\t\t\t}\n\t\t\tfor c in cdf.iter_mut() {\n\t\t\t\t*c /= acc;\n\t\t\t}\n\t\t\tSelf { cdf, state: seed | 1 }\n\t\t}\n\n\t\tfn next_u64(&mut self) -> u64 {\n\t\t\tlet mut x = self.state;\n\t\t\tx ^= x >> 12;\n\t\t\tx ^= x << 25;\n\t\t\tx ^= x >> 27;\n\t\t\tself.state = x;\n\t\t\tx.wrapping_mul(0x2545_F491_4F6C_DD1D)\n\t\t}\n\n\t\tfn next_key(&mut self) -> i64 {\n\t\t\tlet u = (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64;\n\t\t\tlet rank = self.cdf.partition_point(|&c| c < u).min(self.cdf.len() - 1);\n\t\t\t((rank as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 1) as i64\n\t\t}\n\t}\n\n\tstruct Run {\n\t\thit_rate: f64,\n\t\tsettled_small: usize,\n\t\tmin_small: usize,\n\t\tstats: SplitStats,\n\t\tghost_hits: u64,\n\t}\n\n\tfn replay(floor_div: usize, n: usize, alpha: f64, ops: usize) -> Run {\n\t\tlet cap = 2048;\n\t\tlet mut cache = S3Fifo::<i64, i64>::new(cap);\n\t\tcache.floor_div = floor_div;\n\t\tlet mut z = Zipf::new(n, alpha, 0xC0FF_EE12_3456_789B);\n\t\tlet mut hits = 0u64;\n\t\tfor _ in 0..ops {\n\t\t\tlet k = z.next_key();\n\t\t\tif cache.get(&k).is_some() {\n\t\t\t\thits += 1;\n\t\t\t} else {\n\t\t\t\tcache.put(k, k);\n\t\t\t}\n\t\t\tassert_eq!(cache.small_size + cache.main_size, cap);\n\t\t\tassert!(cache.small_size >= cache.small_floor());\n\t\t\tassert!(cache.ghost_live <= cache.ghost_cap);\n\t\t}\n\t\tRun {\n\t\t\thit_rate: hits as f64 / ops as f64,\n\t\t\tsettled_small: cache.small_size,\n\t\t\tmin_small: cache.stats.min_small,\n\t\t\tstats: cache.stats,\n\t\t\tghost_hits: cache.stats.shrinks + cache.stats.floor_blocks,\n\t\t}\n\t}\n\n\t/// Run with `cargo test --features test-utils --lib h01_split_report -- --nocapture`.\n\t#[test]\n\tfn h01_split_report() {\n\t\tfor &(n, alpha) in &[(20_000usize, 0.8f64), (20_000, 0.99), (100_000, 0.99)] {\n\t\t\tfor &div in &[16usize, 64] {\n\t\t\t\tlet r = replay(div, n, alpha, 400_000);\n\t\t\t\tprintln!(\n\t\t\t\t\t\"H01 n={n} alpha={alpha} floor=cap/{div}={floor}: hit_rate={hr:.4} \\\n\t\t\t\t\t settled_small={s} min_small={m} shrinks={sh} floor_blocks={fb} \\\n\t\t\t\t\t ({pct:.1}% of ghost hits) grows={g}\",\n\t\t\t\t\tfloor = (2048 / div).max(1),\n\t\t\t\t\thr = r.hit_rate,\n\t\t\t\t\ts = r.settled_small,\n\t\t\t\t\tm = r.min_small,\n\t\t\t\t\tsh = r.stats.shrinks,\n\t\t\t\t\tfb = r.stats.floor_blocks,\n\t\t\t\t\tpct = if r.ghost_hits == 0 {\n\t\t\t\t\t\t0.0\n\t\t\t\t\t} else {\n\t\t\t\t\t\t100.0 * r.stats.floor_blocks as f64 / r.ghost_hits as f64\n\t\t\t\t\t},\n\t\t\t\t\tg = r.stats.grows,\n\t\t\t\t);\n\t\t\t\tassert!(r.settled_small >= (2048 / div).max(1));\n\t\t\t}\n\t\t}\n\t}\n}\n"
}
```

### 14:44:23 thinking



### 14:44:23 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/s3_fifo/src/lib.rs"
}
```

### 14:44:27 thinking



### 14:44:28 tool: Edit

```json
{
  "replace_all": false,
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0009/variants/v002/attempts/01/overlay/src/s3_fifo/mod.rs",
  "old_string": "\t#[inline]\n\tfn len(&self) -> usize {",
  "new_string": "\t#[inline]\n\t#[allow(dead_code)]\n\tfn len(&self) -> usize {"
}
```

### 14:44:37 thinking



### 14:44:38 chat

I've written the v002 overlay, but I haven't compiled it or run any tests. This session only had file read/write tools (no shell), so none of the four gates or the new unit tests have been run. The build gate will be the first real check.

**What changed:** one file, `overlay/src/s3_fifo/mod.rs`, which is the parent's file plus the H02 change. The factory and every other file come from the parent and subject unchanged.

- **New `ResidentIndex<K>` replaces the hashbrown `index`.**
  - It has `n = (2·capacity).next_power_of_two()` buckets, so the table is at most half full.
  - Each bucket is a key slot plus a `u32` word, and the word `u32::MAX` marks an empty bucket.
  - A key's starting bucket is the top bits of the existing `FxHasher` hash multiplied by `FX_SEED`.
  - Lookups scan forward one bucket at a time and stop at the first empty one.
  - `remove` fills the gap by moving later entries back (backward-shift deletion), so there are no tombstones.
  - A custom `Drop` frees any keys still in the table.
- **Capacity limit tightened.** `try_new` now requires `capacity <= SLOT_MASK`, so a live value word can never equal the empty marker.
- **Call sites rewired.** `get`, `put`'s resident check, `packed_mut`, `alloc_with_key` and `release` now use the new table. Each still does one lookup per hit, one insert per allocation and one remove per release, and `get` keeps the same lookup, freq bump, value load sequence.
- **Hit rate should stay at 0.7309.** The ghost map is still an `FxHashMap`, ghost fingerprints are computed exactly as before, and no eviction decision depends on the index's internal order.
- **Tests:**
  - Updated the ones that read `index[&1]` or dereferenced `index.get(..)`.
  - Added a randomized stress test against a `std` `HashMap`, including tiny tables whose probes wrap past the end of the array.
  - Added an explicit wrap-around case that also checks entries move back correctly on delete.
  - Added a check that keys are dropped exactly once, and a check of table sizing.
  - Left `entry_is_8_bytes` unchanged.

I added `#[allow(dead_code)]` on `ResidentIndex::len`, because only the tests call it and it would otherwise produce a warning.


---

- finished: 2026-10-20T14:44:38+00:00
- duration: 172.71s
- status: ok
- input_tokens: 22
- output_tokens: 21020
- cache_read_tokens: 374282
- cache_write_tokens: 42294
- cost_usd: 0.833696
