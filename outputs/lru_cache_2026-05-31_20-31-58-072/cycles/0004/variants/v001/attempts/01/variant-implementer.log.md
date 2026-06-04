# Agent log: variant-implementer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-7)
- workspace: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0004/variants/v001/attempts/01
- allow_edits: True
- started: 2026-05-31T18:02:53+00:00

## timeline

### 18:02:56 thinking



### 18:02:57 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0004/hypotheses/H01.md"
}
```

### 18:02:58 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/manifest.toml"
}
```

### 18:02:58 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0003/variants/v001/insight.md"
}
```

### 18:02:59 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0003/variants/v001/eval/fitness-vector.json"
}
```

### 18:03:02 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0003/variants/v001/attempts/01/overlay"
}
```

### 18:03:07 tool: Bash

```json
{
  "command": "find /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0003/variants/v001/attempts/01/overlay -type f",
  "description": "List parent overlay files"
}
```

### 18:03:07 tool: Bash

```json
{
  "command": "find /home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src -type f",
  "description": "List subject source files"
}
```

### 18:03:10 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0003/variants/v001/attempts/01/overlay/**/*"
}
```

### 18:03:10 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src/**/*"
}
```

### 18:03:13 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0003/variants/v001/attempts/01/overlay/src/lru_cache/mod.rs"
}
```

### 18:03:15 thinking



### 18:03:17 chat

I'll skip task tracking for this single-file overlay edit and focus on the work itself. Let me read the subject source and test infrastructure.

### 18:03:18 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src/lru_cache/mod.rs"
}
```

### 18:03:19 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src/test_utils/invariants.rs"
}
```

### 18:03:19 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src/test_utils/mod.rs"
}
```

### 18:03:22 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src/lib.rs"
}
```

### 18:03:25 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src/lru_cache/lru_cache_jni.rs"
}
```

### 18:04:50 thinking



### 18:04:51 chat

Now I have full context. The hypothesis adds a first-seen sketch + scan-taint bit to the parent CLOCK-Pro-lite, with a pre-pass in eviction. Let me write the overlay.

### 18:05:59 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0004/variants/v001/attempts/01/overlay/src/lru_cache/mod.rs",
  "content": "// CLOCK-Pro-lite cache + H01 scan-taint filter.\n//\n// Inherits the parent v001 (cycle 3) skeleton:\n//   * Per-slot (state, referenced) pair with Cold/Hot states.\n//   * `get` hit promotes Cold->Hot and sets the ref bit; Hot just sets ref.\n//   * `put` of a brand-new key whose key is in the ghost ring inserts as Hot;\n//     otherwise as Cold. Ghost ring is full `capacity` long.\n//   * Two-pass eviction sweep: pass 1 prefers Cold ref-clear; pass 2 demotes\n//     Hot ref-set then evicts the first Hot ref-clear; fallback evicts the\n//     slot at hand (now Cold ref-clear).\n//\n// H01 additions:\n//   * A `seen: Vec<bool>` first-seen sketch of length `4 * capacity` (>=4),\n//     indexed by `hash(key) % seen.len()` using `DefaultHasher`. Every `get`\n//     and every `put` marks the slot, regardless of resident/ghost outcome.\n//   * A `scan_taint: bool` on each Entry, set only when admitting a brand-new\n//     Cold key (not from the ghost ring, not when the cache is still below\n//     capacity) whose fingerprint had not been seen prior to this access.\n//     Resident updates and ghost-promoted Hot admissions leave the taint\n//     untouched at insert.\n//   * A new pre-pass in `evict_one` sweeps the ring once (bounded at\n//     `capacity` steps) for Cold ref-clear slots whose `scan_taint` is true,\n//     evicting the first one found as a Cold eviction (ghost-pushed). If\n//     none is found, the existing pass 1 / pass 2 / fallback chain runs\n//     unchanged.\n//   * On a `get` hit, the taint clears at the same point the ref bit is set\n//     so a touched scan insert is no longer treated as scan-only.\n//\n// The JNI ABI in `lru_cache_jni.rs` is unchanged.\n\nuse std::collections::hash_map::DefaultHasher;\nuse std::collections::{HashMap, HashSet};\nuse std::fmt::Debug;\nuse std::hash::{Hash, Hasher};\n\npub mod lru_cache_jni;\n\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\nenum SlotState {\n\tCold,\n\tHot,\n}\n\n#[derive(Debug)]\nstruct Entry<K, V> {\n\tkey: K,\n\tvalue: V,\n\tstate: SlotState,\n\treferenced: bool,\n\t// H01: true when this slot was admitted as a brand-new Cold key whose\n\t// fingerprint had never been seen before. Cleared on any subsequent `get`\n\t// hit; cleared implicitly when the slot is evicted.\n\tscan_taint: bool,\n}\n\n#[derive(Debug)]\npub struct LruCache<K, V> {\n\tmap: HashMap<K, usize>,          // key -> slot index\n\tslots: Vec<Option<Entry<K, V>>>, // fixed-length ring (Option lets eviction leave a hole)\n\thand: usize,                     // CLOCK hand, modulo capacity\n\tlen: usize,                      // current resident count\n\tcapacity: usize,\n\tghost: Vec<Option<K>>,           // recent-cold-evictions ring\n\tghost_head: usize,               // ghost ring write head\n\tghost_set: HashSet<K>,           // O(1) membership mirror of `ghost`\n\tseen: Vec<bool>,                 // H01: first-seen sketch over fingerprints\n}\n\nimpl<K, V> LruCache<K, V>\nwhere\n\tK: Hash + Eq + Clone + Debug,\n\tV: Debug,\n{\n\tpub fn try_new(capacity: usize) -> Option<Self> {\n\t\tif capacity == 0 {\n\t\t\tlog::error!(\"LruCache.try_new: capacity must be > 0\");\n\t\t\treturn None;\n\t\t}\n\t\tlog::info!(\"LruCache.try_new: capacity={capacity}\");\n\t\tlet mut slots = Vec::with_capacity(capacity);\n\t\tfor _ in 0..capacity {\n\t\t\tslots.push(None);\n\t\t}\n\t\t// Full-capacity ghost ring inherited from parent v001 (cycle 3).\n\t\tlet ghost_len = capacity;\n\t\tlet mut ghost = Vec::with_capacity(ghost_len);\n\t\tfor _ in 0..ghost_len {\n\t\t\tghost.push(None);\n\t\t}\n\t\t// H01 sketch: 4 * capacity bits, floored at 4 so the modulo is always\n\t\t// defined even at capacity=1. Sized to comfortably exceed the working\n\t\t// set; collisions only make the taint slightly more permissive (rarer\n\t\t// taints, not spurious ones).\n\t\tlet seen_len = capacity.saturating_mul(4).max(4);\n\t\tlet seen = vec![false; seen_len];\n\t\tSome(Self {\n\t\t\tmap: HashMap::with_capacity(capacity),\n\t\t\tslots,\n\t\t\thand: 0,\n\t\t\tlen: 0,\n\t\t\tcapacity,\n\t\t\tghost,\n\t\t\tghost_head: 0,\n\t\t\tghost_set: HashSet::with_capacity(ghost_len),\n\t\t\tseen,\n\t\t})\n\t}\n\n\tpub fn new(capacity: usize) -> Self {\n\t\tSelf::try_new(capacity).expect(\"LruCache::new: initialization failed\")\n\t}\n\n\t/// Hash `key` with `DefaultHasher` and fold to a sketch index.\n\tfn fingerprint(&self, key: &K) -> usize {\n\t\tlet mut hasher = DefaultHasher::new();\n\t\tkey.hash(&mut hasher);\n\t\t(hasher.finish() as usize) % self.seen.len()\n\t}\n\n\t/// Returns the value for `key` if resident. A Cold hit is promoted to Hot\n\t/// (the second-touch promotion the parent cites as scan-resistant); the\n\t/// reference bit is set in either case. The first-seen sketch is updated\n\t/// regardless of hit/miss. A hit also clears `scan_taint` — a touched\n\t/// scan insert is no longer scan-only.\n\tpub fn get(&mut self, key: &K) -> Option<&V> {\n\t\tlet fp = self.fingerprint(key);\n\t\t// Mark seen unconditionally: per H01, seen[fp] tracks whether we've\n\t\t// ever observed this fingerprint, independent of hit/miss/ghost.\n\t\tself.seen[fp] = true;\n\t\tlet &idx = self.map.get(key)?;\n\t\tlet entry = self\n\t\t\t.slots[idx]\n\t\t\t.as_mut()\n\t\t\t.expect(\"map index points to occupied slot\");\n\t\tif entry.state == SlotState::Cold {\n\t\t\tentry.state = SlotState::Hot;\n\t\t}\n\t\tentry.referenced = true;\n\t\tentry.scan_taint = false;\n\t\tSome(&entry.value)\n\t}\n\n\t/// Inserts or updates `key`. Brand-new keys land in an empty slot if one\n\t/// exists, otherwise the CLOCK-Pro-lite sweep (with the H01 pre-pass)\n\t/// selects a victim.\n\tpub fn put(&mut self, key: K, value: V) {\n\t\tlet fp = self.fingerprint(&key);\n\t\t// Capture seen[fp] *before* marking, so the taint decision below sees\n\t\t// the prior bit. Then mark the fingerprint as seen — this happens for\n\t\t// every put outcome (resident update, ghost promotion, brand-new).\n\t\tlet prior_seen = self.seen[fp];\n\t\tself.seen[fp] = true;\n\n\t\t// Key already resident: update value in place, set the ref bit.\n\t\t// Taint is left untouched (H01: \"resident updates ... must not be\n\t\t// tainted\" — i.e. we do not newly mark them; we also do not clear\n\t\t// here, per the spec which limits clearing to `get` hits).\n\t\tif let Some(&idx) = self.map.get(&key) {\n\t\t\tlet entry = self\n\t\t\t\t.slots[idx]\n\t\t\t\t.as_mut()\n\t\t\t\t.expect(\"map index points to occupied slot\");\n\t\t\tentry.value = value;\n\t\t\tentry.referenced = true;\n\t\t\treturn;\n\t\t}\n\n\t\t// Brand-new key. Ghost promotion takes precedence over scan-taint:\n\t\t// a key that was recently Cold-evicted and is now re-referenced is\n\t\t// treated as Hot and is explicitly excluded from tainting.\n\t\tlet from_ghost = self.ghost_set.contains(&key);\n\t\tif from_ghost {\n\t\t\tself.ghost_remove(&key);\n\t\t}\n\t\tlet initial_state = if from_ghost {\n\t\t\tSlotState::Hot\n\t\t} else {\n\t\t\tSlotState::Cold\n\t\t};\n\n\t\tif self.len < self.capacity {\n\t\t\t// Below capacity: per H01, do not taint. Scan forward from `hand`\n\t\t\t// for the first empty slot (terminates because len < capacity).\n\t\t\tloop {\n\t\t\t\tif self.slots[self.hand].is_none() {\n\t\t\t\t\tlet idx = self.hand;\n\t\t\t\t\tself.slots[idx] = Some(Entry {\n\t\t\t\t\t\tkey: key.clone(),\n\t\t\t\t\t\tvalue,\n\t\t\t\t\t\tstate: initial_state,\n\t\t\t\t\t\treferenced: false,\n\t\t\t\t\t\tscan_taint: false,\n\t\t\t\t\t});\n\t\t\t\t\tself.map.insert(key, idx);\n\t\t\t\t\tself.len += 1;\n\t\t\t\t\tself.hand = (self.hand + 1) % self.capacity;\n\t\t\t\t\treturn;\n\t\t\t\t}\n\t\t\t\tself.hand = (self.hand + 1) % self.capacity;\n\t\t\t}\n\t\t} else {\n\t\t\t// At capacity: evict one slot via pre-pass + two-pass sweep, then\n\t\t\t// refill. Taint only fires on brand-new non-ghost admissions\n\t\t\t// whose fingerprint hadn't been seen before this put.\n\t\t\tlet idx = self.evict_one();\n\t\t\tlet scan_taint = !from_ghost && !prior_seen;\n\t\t\tself.slots[idx] = Some(Entry {\n\t\t\t\tkey: key.clone(),\n\t\t\t\tvalue,\n\t\t\t\tstate: initial_state,\n\t\t\t\treferenced: false,\n\t\t\t\tscan_taint,\n\t\t\t});\n\t\t\tself.map.insert(key, idx);\n\t\t\tself.len += 1;\n\t\t\tdebug_assert!(self.map.len() <= self.capacity);\n\t\t}\n\t}\n\n\t/// Selects a victim per H01 pre-pass + CLOCK-Pro-lite policy and empties\n\t/// its slot.\n\t///\n\t/// Pre-pass (H01): sweep the ring once starting at `hand`, evicting the\n\t/// first Cold ref-clear slot whose `scan_taint` is true. Treated as a\n\t/// Cold eviction for ghost purposes. The sweep is read-only on the slots\n\t/// it visits (no ref-bit clearing, no demotion), so failing the pre-pass\n\t/// leaves the ring untouched and pass 1 starts from `hand` unchanged.\n\t///\n\t/// Pass 1 only touches Cold slots: ref-set -> clear and advance, ref-clear\n\t/// -> evict (and push to ghost). Hot slots are skipped untouched.\n\t///\n\t/// Pass 2: Hot ref-set slots are cleared and demoted to Cold; the first\n\t/// Hot ref-clear slot is evicted (Hot evictions skip the ghost). Cold\n\t/// slots in pass 2 are now all ref-clear (pass 1 cleared them) and are\n\t/// skipped in favour of a Hot victim.\n\t///\n\t/// Fallback: if pass 2 also finds nothing (every Hot slot was ref-set\n\t/// when pass 2 started, so all got demoted), evict the slot at the\n\t/// current hand position — by that point it is Cold ref-clear.\n\t///\n\t/// Returns the index of the now-empty slot. Map, ghost ring, and `len`\n\t/// have been updated. Total work is O(capacity) across all phases.\n\tfn evict_one(&mut self) -> usize {\n\t\t// H01 pre-pass: bounded O(capacity) scan for tainted Cold ref-clear\n\t\t// victims. We index by offset off `hand` rather than mutating `hand`\n\t\t// during the scan, so a no-hit pre-pass leaves pass 1's starting\n\t\t// position identical to the parent's behaviour.\n\t\tfor offset in 0..self.capacity {\n\t\t\tlet i = (self.hand + offset) % self.capacity;\n\t\t\tlet tainted = self\n\t\t\t\t.slots[i]\n\t\t\t\t.as_ref()\n\t\t\t\t.map(|e| e.state == SlotState::Cold && !e.referenced && e.scan_taint)\n\t\t\t\t.unwrap_or(false);\n\t\t\tif tainted {\n\t\t\t\tself.evict_at(i, /*was_cold=*/ true);\n\t\t\t\tself.hand = (i + 1) % self.capacity;\n\t\t\t\treturn i;\n\t\t\t}\n\t\t}\n\n\t\t// Pass 1: prefer cold-unreferenced victims.\n\t\tfor _ in 0..self.capacity {\n\t\t\tlet entry = self\n\t\t\t\t.slots[self.hand]\n\t\t\t\t.as_mut()\n\t\t\t\t.expect(\"slot must be occupied at full capacity\");\n\t\t\tmatch entry.state {\n\t\t\t\tSlotState::Cold => {\n\t\t\t\t\tif entry.referenced {\n\t\t\t\t\t\tentry.referenced = false;\n\t\t\t\t\t\tself.hand = (self.hand + 1) % self.capacity;\n\t\t\t\t\t} else {\n\t\t\t\t\t\tlet idx = self.hand;\n\t\t\t\t\t\tself.evict_at(idx, /*was_cold=*/ true);\n\t\t\t\t\t\tself.hand = (idx + 1) % self.capacity;\n\t\t\t\t\t\treturn idx;\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t\tSlotState::Hot => {\n\t\t\t\t\tself.hand = (self.hand + 1) % self.capacity;\n\t\t\t\t}\n\t\t\t}\n\t\t}\n\n\t\t// Pass 2: demote ref-set Hot slots and evict the first ref-clear Hot.\n\t\tfor _ in 0..self.capacity {\n\t\t\tlet entry = self\n\t\t\t\t.slots[self.hand]\n\t\t\t\t.as_mut()\n\t\t\t\t.expect(\"slot must be occupied at full capacity\");\n\t\t\tmatch (entry.state, entry.referenced) {\n\t\t\t\t(SlotState::Hot, true) => {\n\t\t\t\t\tentry.referenced = false;\n\t\t\t\t\tentry.state = SlotState::Cold;\n\t\t\t\t\tself.hand = (self.hand + 1) % self.capacity;\n\t\t\t\t}\n\t\t\t\t(SlotState::Hot, false) => {\n\t\t\t\t\tlet idx = self.hand;\n\t\t\t\t\tself.evict_at(idx, /*was_cold=*/ false);\n\t\t\t\t\tself.hand = (idx + 1) % self.capacity;\n\t\t\t\t\treturn idx;\n\t\t\t\t}\n\t\t\t\t(SlotState::Cold, _) => {\n\t\t\t\t\t// Pass 1 already cleared every Cold ref bit; defer to the\n\t\t\t\t\t// fallback rather than competing with the Hot pool.\n\t\t\t\t\tself.hand = (self.hand + 1) % self.capacity;\n\t\t\t\t}\n\t\t\t}\n\t\t}\n\n\t\t// Fallback (rare): pass 2 demoted every Hot slot to Cold without\n\t\t// finding a ref-clear Hot victim. The slot at `hand` is now Cold\n\t\t// ref-clear; evict it and treat it as a cold eviction for ghost\n\t\t// purposes.\n\t\tlet idx = self.hand;\n\t\tself.evict_at(idx, /*was_cold=*/ true);\n\t\tself.hand = (idx + 1) % self.capacity;\n\t\tidx\n\t}\n\n\t/// Removes the entry at `idx` from the cache. If `was_cold`, push the\n\t/// evicted key onto the ghost ring; Hot evictions do not enter the ghost.\n\tfn evict_at(&mut self, idx: usize, was_cold: bool) {\n\t\tlet entry = self\n\t\t\t.slots[idx]\n\t\t\t.take()\n\t\t\t.expect(\"evicted slot must be occupied\");\n\t\tself.map.remove(&entry.key);\n\t\tdebug_assert!(self.len > 0);\n\t\tself.len -= 1;\n\t\tif was_cold {\n\t\t\tself.push_ghost(entry.key);\n\t\t}\n\t}\n\n\t/// Writes `key` into the ghost ring at `ghost_head`, displacing any prior\n\t/// occupant. Keeps `ghost_set` in lockstep with the ring contents.\n\tfn push_ghost(&mut self, key: K) {\n\t\tlet ring_len = self.ghost.len();\n\t\tif let Some(old) = self.ghost[self.ghost_head].take() {\n\t\t\tself.ghost_set.remove(&old);\n\t\t}\n\t\t// A re-inserted key is removed from the ghost via `ghost_remove` in\n\t\t// `put`, so a freshly-evicted Cold key never collides with an existing\n\t\t// ring entry — but `HashSet::insert` is idempotent regardless.\n\t\tself.ghost_set.insert(key.clone());\n\t\tself.ghost[self.ghost_head] = Some(key);\n\t\tself.ghost_head = (self.ghost_head + 1) % ring_len;\n\t}\n\n\t/// Removes `key` from both the ghost set and its ring slot. The ring is\n\t/// `capacity` slots long, so the linear scan is O(capacity) worst case.\n\tfn ghost_remove(&mut self, key: &K) {\n\t\tif !self.ghost_set.remove(key) {\n\t\t\treturn;\n\t\t}\n\t\tfor slot in self.ghost.iter_mut() {\n\t\t\tif slot.as_ref().map_or(false, |k| k == key) {\n\t\t\t\t*slot = None;\n\t\t\t\treturn;\n\t\t\t}\n\t\t}\n\t}\n}\n\nimpl<K, V> framework::Cache<K, V> for LruCache<K, V>\nwhere\n\tK: Hash + Eq + Clone + Debug,\n\tV: Debug,\n{\n\tfn put(&mut self, key: K, value: V) {\n\t\tLruCache::put(self, key, value);\n\t}\n\n\tfn get(&mut self, key: &K) -> Option<&V> {\n\t\tLruCache::get(self, key)\n\t}\n}\n\n#[cfg(test)]\nmod tests {\n\t// CLOCK-Pro-lite + scan-taint is not LRU, so strict-LRU eviction-order\n\t// tests would fail; only structural / contract-style checks are kept\n\t// here. The four gates (build / contract / universal-invariant /\n\t// invariant) do not depend on these inline tests.\n\n\tuse super::*;\n\n\t#[test]\n\tfn get_miss_returns_none() {\n\t\tlet mut cache = LruCache::<i64, i64>::new(4);\n\t\tassert!(cache.get(&42).is_none());\n\t}\n\n\t#[test]\n\tfn put_then_get_round_trips() {\n\t\tlet mut cache = LruCache::new(4);\n\t\tcache.put(1, 10);\n\t\tassert_eq!(*cache.get(&1).unwrap(), 10);\n\t}\n\n\t#[test]\n\tfn capacity_bound_under_overfill() {\n\t\tlet mut cache: LruCache<i64, i64> = LruCache::new(8);\n\t\tfor k in 0..100i64 {\n\t\t\tif cache.get(&k).is_none() {\n\t\t\t\tcache.put(k, k);\n\t\t\t}\n\t\t}\n\t\tlet resident = (0..100i64).filter(|k| cache.get(k).is_some()).count();\n\t\tassert!(resident <= 8, \"residents {resident} exceeds capacity 8\");\n\t}\n\n\t#[test]\n\tfn never_put_keys_read_none() {\n\t\tlet mut cache: LruCache<i64, i64> = LruCache::new(8);\n\t\tfor k in 0..16i64 {\n\t\t\tif cache.get(&k).is_none() {\n\t\t\t\tcache.put(k, k);\n\t\t\t}\n\t\t}\n\t\tfor phantom in [1000i64, 2000, -1, -50] {\n\t\t\tassert!(cache.get(&phantom).is_none());\n\t\t}\n\t}\n\n\t#[test]\n\tfn resident_keys_read_back_their_value() {\n\t\tlet mut cache: LruCache<i64, i64> = LruCache::new(8);\n\t\tfor k in 0..50i64 {\n\t\t\tif cache.get(&k).is_none() {\n\t\t\t\tcache.put(k, k * 10);\n\t\t\t}\n\t\t}\n\t\tfor k in 0..50i64 {\n\t\t\tif let Some(&v) = cache.get(&k) {\n\t\t\t\tassert_eq!(v, k * 10);\n\t\t\t}\n\t\t}\n\t}\n\n\t#[test]\n\tfn ghost_promotion_recovers_evicted_key_as_hot() {\n\t\t// capacity=2, ghost length = 2 (parent v001 sizing).\n\t\tlet mut cache: LruCache<i64, i64> = LruCache::new(2);\n\t\tcache.put(1, 10);\n\t\tcache.put(2, 20);\n\t\tcache.put(3, 30); // evicts a Cold slot; key joins ghost ring\n\t\tlet evicted: Vec<i64> = [1i64, 2]\n\t\t\t.into_iter()\n\t\t\t.filter(|k| cache.get(k).is_none())\n\t\t\t.collect();\n\t\tassert!(\n\t\t\t!evicted.is_empty(),\n\t\t\t\"expected at least one of {{1,2}} to be evicted by capacity=2\"\n\t\t);\n\t\tlet k = evicted[0];\n\t\tcache.put(k, k * 100);\n\t\tassert_eq!(*cache.get(&k).unwrap(), k * 100);\n\t}\n\n\t#[test]\n\t#[should_panic(expected = \"initialization failed\")]\n\tfn capacity_zero_panics() {\n\t\tLruCache::<i64, i64>::new(0);\n\t}\n}\n"
}
```

### 18:06:09 chat

I've authored the H01 overlay at `overlay/src/lru_cache/mod.rs` (the only file that needs to change — JNI ABI and tests stay untouched). Key changes against the parent CLOCK-Pro-lite (cycle 3 v001):

- Added `seen: Vec<bool>` of length `max(4, 4 * capacity)`, indexed by `DefaultHasher` % len. Marked unconditionally on every `get` and `put`.
- Added `scan_taint: bool` to `Entry`. Set only when a brand-new key is admitted at capacity, `from_ghost == false`, and `seen[fp]` was false **before** this access marked it. Resident updates, ghost-promoted Hot admissions, and below-capacity fills never taint.
- Cleared `scan_taint` on every `get` hit at the same point `referenced` is set (a touched scan insert is no longer scan-only).
- Added a bounded O(capacity) pre-pass to `evict_one`: it sweeps offsets `0..capacity` off `hand` looking for a Cold ref-clear slot with `scan_taint = true`; the first one is evicted as a Cold eviction (ghost-pushed). The pre-pass mutates nothing on the slots it visits, so when it finds no tainted victim, pass 1 / pass 2 / fallback run with the exact same starting state as the parent.

Invariants preserved: ghost ring sized to `capacity`, Cold→Hot second-touch promotion on `get`, pass-2 Hot demotion behaviour, fallback eviction at hand, and the protected `lru_cache_jni.rs` ABI.


---

- finished: 2026-05-31T18:06:10+00:00
- duration: 196.72s
- status: ok
- input_tokens: 14
- output_tokens: 17113
- cache_read_tokens: 285404
- cache_write_tokens: 37592
- cost_usd: 0.806517
