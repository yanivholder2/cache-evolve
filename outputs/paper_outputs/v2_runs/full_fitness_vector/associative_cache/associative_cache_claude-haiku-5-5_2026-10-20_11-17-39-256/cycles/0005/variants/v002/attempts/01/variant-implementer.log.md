# Agent log: variant-implementer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0005/variants/v002/attempts/01
- allow_edits: True
- started: 2026-10-20T11:39:40+00:00

## timeline

### 11:39:42 thinking



### 11:39:43 tool: Bash

```json
{
  "command": "cat /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0005/hypotheses/H02.md; echo ----; cat /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0004/variants/v001/insight.md; echo ----; cat /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0004/variants/v001/eval/fitness-vector.json; echo ----; find /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0004/variants/v001/attempts/01/overlay -type f; echo ----; cat /home/user/cache-evolve/inputs/subjects/associative_cache/manifest.toml; find /home/user/cache-evolve/inputs/subjects/associative_cache/src -type f",
  "description": "Read hypothesis, parent insight, manifest, and source layout"
}
```

### 11:39:45 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0005/hypotheses/H02.md"
}
```

### 11:39:45 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0004/variants/v001/insight.md"
}
```

### 11:39:45 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0004/variants/v001/eval/fitness-vector.json"
}
```

### 11:39:46 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/associative_cache/manifest.toml"
}
```

### 11:39:46 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0004/variants/v001/attempts/01/overlay"
}
```

### 11:39:46 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/inputs/subjects/associative_cache/src"
}
```

### 11:39:48 thinking



### 11:39:48 tool: Grep

```json
{
  "pattern": ".",
  "path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0004/variants/v001/attempts/01/overlay",
  "output_mode": "files_with_matches"
}
```

### 11:39:48 tool: Grep

```json
{
  "pattern": ".",
  "path": "/home/user/cache-evolve/inputs/subjects/associative_cache/src",
  "output_mode": "files_with_matches"
}
```

### 11:39:50 thinking



### 11:39:50 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0004/variants/v001/attempts/01/overlay/src/associative_cache/associative_cache_bucket.rs"
}
```

### 11:39:51 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0004/variants/v001/attempts/01/overlay/src/shared/eviction_policy.rs"
}
```

### 11:39:51 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0004/variants/v001/attempts/01/overlay/src/associative_cache/associative_cache.rs"
}
```

### 11:39:51 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/associative_cache/src/shared/eviction_policy.rs"
}
```

### 11:39:51 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/associative_cache/src/associative_cache/associative_cache_bucket.rs"
}
```

### 11:40:51 thinking



### 11:40:51 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0004/variants/v001/attempts/01/overlay/src/shared/item.rs"
}
```

### 11:40:51 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/associative_cache/src/shared/item.rs"
}
```

### 11:40:54 thinking



### 11:40:54 chat

I have what I need: the parent overlay only changes `src/shared/eviction_policy.rs` (the `Item` file matches baseline). I'll write the v002 overlay of that file, keeping every other policy byte-identical and replacing only the `ClassicLRU` clock walk with the word-level search.

### 11:41:12 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0005/variants/v002/attempts/01/overlay/src/shared/eviction_policy.rs",
  "content": "//! In-crate eviction policy implementations and the selection enum.\n//!\n//! Behaviour lives in one zero-sized struct per policy, each implementing\n//! [`EvictionPolicy`]. `EvictionPolicyKind` is the lightweight `Copy` selector\n//! carried by settings (and crossed over JNI as an ordinal/name); `build`\n//! turns a kind into the boxed trait object a bucket owns.\n\nuse crate::shared::item::Item;\nuse crate::shared::traits::eviction_policy::EvictionPolicy;\nuse std::cell::Cell;\nuse std::fmt::Debug;\nuse std::hash::Hash;\nuse std::time::Instant;\n\n/// Number of slots whose reference bit fits in the per-bucket mask word.\nconst REF_SLOTS: usize = 32;\n\n/// CLOCK (second-chance) victim selection. The reference bits live in one\n/// per-bucket word (one bit per slot) rather than on the items. Hits set the\n/// slot's bit and never move items. On eviction the per-bucket hand sweeps from\n/// its position, clearing set bits, and evicts the first slot whose bit is clear.\n#[derive(Debug)]\npub struct ClassicLRU {\n\thand: Cell<usize>,\n\trefs: Cell<u32>,\n}\n/// LRU by last-hit timestamp; items keep their position.\n#[derive(Debug)]\npub struct TimestampLRU;\n#[derive(Debug)]\npub struct Lifo;\n#[derive(Debug)]\npub struct Lfu;\n#[derive(Debug)]\npub struct Fifo;\n#[derive(Debug)]\npub struct Swap;\n\nimpl ClassicLRU {\n\tpub fn new() -> Self {\n\t\tSelf {\n\t\t\thand: Cell::new(0),\n\t\t\trefs: Cell::new(0),\n\t\t}\n\t}\n\n\tfn set(&self, slot: usize) {\n\t\tif slot < REF_SLOTS {\n\t\t\tself.refs.set(self.refs.get() | (1u32 << slot));\n\t\t}\n\t}\n\n\t/// Mirrors `Vec::remove(victim)` on the mask: drops bit `victim` and shifts\n\t/// every higher bit down one slot, so each remaining bit keeps tracking its\n\t/// item. The vacated top slot is clear, so the item pushed next starts with\n\t/// its reference bit clear.\n\tfn shift_out(&self, victim: usize) {\n\t\tlet refs = self.refs.get();\n\t\tlet low_mask = if victim >= REF_SLOTS { u32::MAX } else { (1u32 << victim) - 1 };\n\t\tlet high = if victim + 1 >= REF_SLOTS { 0 } else { (refs >> (victim + 1)) << victim };\n\t\tself.refs.set((refs & low_mask) | high);\n\t}\n\n\t/// Word-level clock walk. Returns the victim slot and the mask of reference\n\t/// bits the sequential walk would have cleared on the way (the contiguous run\n\t/// of set bits from `hand` up to the victim, or the whole word when every\n\t/// tracked slot is referenced). Equivalent to stepping the hand one slot at a\n\t/// time, but resolved with one `trailing_zeros` and one mask.\n\tfn sweep(hand: usize, refs: u32, n: usize) -> (usize, u32) {\n\t\tif n <= REF_SLOTS {\n\t\t\t// All n slots are tracked. Clear-bit candidates within 0..n:\n\t\t\tlet full: u64 = (1u64 << n) - 1;\n\t\t\tlet clear = !(refs as u64) & full;\n\t\t\tif clear == 0 {\n\t\t\t\t// Fully referenced: the walk clears every bit and stops back at `hand`.\n\t\t\t\treturn (hand, full as u32);\n\t\t\t}\n\t\t\t// Doubling the word makes the cyclic search from `hand` a linear one:\n\t\t\t// bit p of `doubled` (p in 0..2n) is clear iff slot p % n is clear.\n\t\t\tlet doubled = clear | (clear << n);\n\t\t\tlet k = (doubled >> hand).trailing_zeros() as usize;\n\t\t\t// Offsets 0..k from `hand` are referenced slots; fold the doubled run\n\t\t\t// back onto 0..n to get the real slots it covers.\n\t\t\tlet run = ((1u64 << k) - 1) << hand;\n\t\t\t((hand + k) % n, ((run | (run >> n)) & full) as u32)\n\t\t} else if hand >= REF_SLOTS {\n\t\t\t// Slots at or above REF_SLOTS are never referenced.\n\t\t\t(hand, 0)\n\t\t} else {\n\t\t\t// Scan the tracked slots from `hand` to the top of the word.\n\t\t\tlet rest = (!refs) >> hand;\n\t\t\tif rest != 0 {\n\t\t\t\tlet k = rest.trailing_zeros() as usize;\n\t\t\t\t(hand + k, (((1u64 << k) - 1) << hand) as u32)\n\t\t\t} else {\n\t\t\t\t// Every tracked slot from `hand` up is referenced; slot REF_SLOTS is clear.\n\t\t\t\t(REF_SLOTS, u32::MAX << hand)\n\t\t\t}\n\t\t}\n\t}\n}\n\nimpl<K, V> EvictionPolicy<K, V> for ClassicLRU\nwhere\n\tK: Hash + Eq + Copy + Debug,\n\tV: Eq + Copy + Debug,\n{\n\tfn name(&self) -> &'static str {\n\t\t\"ClassicLRU\"\n\t}\n\n\tfn get_from_position(&self, _items: &mut Vec<Item<K, V>>, position: usize) -> usize {\n\t\tself.set(position);\n\t\tposition\n\t}\n\n\tfn eviction_candidate_index(&self, items: &[Item<K, V>]) -> usize {\n\t\tlet n = items.len();\n\t\tlet hand = self.hand.get() % n;\n\t\tlet refs = self.refs.get();\n\t\tlet (victim, cleared) = Self::sweep(hand, refs, n);\n\t\tself.refs.set(refs & !cleared);\n\t\t// The victim is removed next by evict_item, which shifts later slots down.\n\t\t// Storing `victim` as the resume position points the next sweep at the\n\t\t// successor, and the mask is shifted to match the removal.\n\t\tself.hand.set(victim);\n\t\tself.shift_out(victim);\n\t\tvictim\n\t}\n\n\tfn promotion_candidate_index(&self, items: &[Item<K, V>]) -> usize {\n\t\titems.len() - 1\n\t}\n}\n\nimpl<K, V> EvictionPolicy<K, V> for TimestampLRU\nwhere\n\tK: Hash + Eq + Copy + Debug,\n\tV: Eq + Copy + Debug,\n{\n\tfn name(&self) -> &'static str {\n\t\t\"TimestampLRU\"\n\t}\n\n\tfn get_from_position(&self, items: &mut Vec<Item<K, V>>, position: usize) -> usize {\n\t\titems[position].timestamp = Instant::now();\n\t\tposition\n\t}\n\n\tfn eviction_candidate_index(&self, items: &[Item<K, V>]) -> usize {\n\t\titems\n\t\t\t.iter()\n\t\t\t.enumerate()\n\t\t\t.min_by_key(|(_, item)| item.timestamp)\n\t\t\t.expect(\"eviction_candidate_index precondition: non-empty\")\n\t\t\t.0\n\t}\n\n\tfn promotion_candidate_index(&self, items: &[Item<K, V>]) -> usize {\n\t\titems\n\t\t\t.iter()\n\t\t\t.enumerate()\n\t\t\t.max_by_key(|(_, item)| item.timestamp)\n\t\t\t.expect(\"promotion_candidate_index precondition: non-empty\")\n\t\t\t.0\n\t}\n}\n\nimpl<K, V> EvictionPolicy<K, V> for Lifo\nwhere\n\tK: Hash + Eq + Copy + Debug,\n\tV: Eq + Copy + Debug,\n{\n\tfn name(&self) -> &'static str {\n\t\t\"Lifo\"\n\t}\n\n\tfn get_from_position(&self, _items: &mut Vec<Item<K, V>>, position: usize) -> usize {\n\t\tposition\n\t}\n\n\tfn eviction_candidate_index(&self, items: &[Item<K, V>]) -> usize {\n\t\titems.len() - 1\n\t}\n\n\tfn promotion_candidate_index(&self, _items: &[Item<K, V>]) -> usize {\n\t\t0\n\t}\n}\n\nimpl<K, V> EvictionPolicy<K, V> for Lfu\nwhere\n\tK: Hash + Eq + Copy + Debug,\n\tV: Eq + Copy + Debug,\n{\n\tfn name(&self) -> &'static str {\n\t\t\"Lfu\"\n\t}\n\n\tfn get_from_position(&self, items: &mut Vec<Item<K, V>>, position: usize) -> usize {\n\t\titems[position].lfu_counter += 1;\n\t\tposition\n\t}\n\n\tfn eviction_candidate_index(&self, items: &[Item<K, V>]) -> usize {\n\t\titems\n\t\t\t.iter()\n\t\t\t.enumerate()\n\t\t\t.min_by_key(|(_, item)| item.lfu_counter)\n\t\t\t.expect(\"eviction_candidate_index precondition: non-empty\")\n\t\t\t.0\n\t}\n\n\tfn promotion_candidate_index(&self, items: &[Item<K, V>]) -> usize {\n\t\titems\n\t\t\t.iter()\n\t\t\t.enumerate()\n\t\t\t.max_by_key(|(_, item)| item.lfu_counter)\n\t\t\t.expect(\"promotion_candidate_index precondition: non-empty\")\n\t\t\t.0\n\t}\n}\n\nimpl<K, V> EvictionPolicy<K, V> for Fifo\nwhere\n\tK: Hash + Eq + Copy + Debug,\n\tV: Eq + Copy + Debug,\n{\n\tfn name(&self) -> &'static str {\n\t\t\"Fifo\"\n\t}\n\n\tfn get_from_position(&self, _items: &mut Vec<Item<K, V>>, position: usize) -> usize {\n\t\tposition\n\t}\n\n\tfn eviction_candidate_index(&self, _items: &[Item<K, V>]) -> usize {\n\t\t0\n\t}\n\n\tfn promotion_candidate_index(&self, items: &[Item<K, V>]) -> usize {\n\t\titems.len() - 1\n\t}\n}\n\nimpl<K, V> EvictionPolicy<K, V> for Swap\nwhere\n\tK: Hash + Eq + Copy + Debug,\n\tV: Eq + Copy + Debug,\n{\n\tfn name(&self) -> &'static str {\n\t\t\"Swap\"\n\t}\n\n\tfn get_from_position(&self, items: &mut Vec<Item<K, V>>, position: usize) -> usize {\n\t\tif position > 0 {\n\t\t\titems.swap(position, position - 1);\n\t\t\tposition - 1\n\t\t} else {\n\t\t\tposition\n\t\t}\n\t}\n\n\tfn eviction_candidate_index(&self, _items: &[Item<K, V>]) -> usize {\n\t\t0\n\t}\n\n\tfn promotion_candidate_index(&self, items: &[Item<K, V>]) -> usize {\n\t\titems.len() - 1\n\t}\n}\n\n/// Lightweight `Copy` selector for an eviction policy. Carried by settings and\n/// crossed over JNI; `build` materialises the boxed trait object.\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum EvictionPolicyKind {\n\tClassicLRU,\n\tTimestampLRU,\n\tLifo,\n\tLfu,\n\tFifo,\n\tSwap,\n}\n\nimpl EvictionPolicyKind {\n\t/// Returns all variants. The exhaustive match ensures a compile error if a variant is added.\n\tpub fn all() -> [Self; 6] {\n\t\tuse EvictionPolicyKind::*;\n\t\tlet _exhaustive = |p: EvictionPolicyKind| match p {\n\t\t\tClassicLRU | TimestampLRU | Lifo | Lfu | Fifo | Swap => {}\n\t\t};\n\t\t[ClassicLRU, TimestampLRU, Lifo, Lfu, Fifo, Swap]\n\t}\n\n\t/// Canonical CamelCase name. Adding a variant forces a compile error here.\n\tpub fn name(&self) -> &'static str {\n\t\tmatch self {\n\t\t\tEvictionPolicyKind::ClassicLRU => \"ClassicLRU\",\n\t\t\tEvictionPolicyKind::TimestampLRU => \"TimestampLRU\",\n\t\t\tEvictionPolicyKind::Lifo => \"Lifo\",\n\t\t\tEvictionPolicyKind::Lfu => \"Lfu\",\n\t\t\tEvictionPolicyKind::Fifo => \"Fifo\",\n\t\t\tEvictionPolicyKind::Swap => \"Swap\",\n\t\t}\n\t}\n\n\t/// Stable integer index. Crossed over JNI as the policy ordinal; adding a\n\t/// variant forces a compile error here.\n\tpub fn index(&self) -> usize {\n\t\tmatch self {\n\t\t\tEvictionPolicyKind::ClassicLRU => 0,\n\t\t\tEvictionPolicyKind::TimestampLRU => 1,\n\t\t\tEvictionPolicyKind::Lifo => 2,\n\t\t\tEvictionPolicyKind::Lfu => 3,\n\t\t\tEvictionPolicyKind::Fifo => 4,\n\t\t\tEvictionPolicyKind::Swap => 5,\n\t\t}\n\t}\n\n\t/// CamelCase names, derived from `all()` + `name()`. Crossed over JNI so\n\t/// Java callers don't hard-code the list.\n\tpub fn all_names() -> [&'static str; 6] {\n\t\tSelf::all().map(|p| p.name())\n\t}\n\n\tpub fn from_usize(index: usize) -> Option<Self> {\n\t\tSelf::all().into_iter().find(|p| p.index() == index)\n\t}\n\n\tpub fn from_name(name: &str) -> Option<Self> {\n\t\tSelf::all().into_iter().find(|p| p.name() == name)\n\t}\n\n\t/// Materialises the boxed trait object a bucket owns.\n\tpub fn build<K, V>(self) -> Box<dyn EvictionPolicy<K, V>>\n\twhere\n\t\tK: Hash + Eq + Copy + Debug,\n\t\tV: Eq + Copy + Debug,\n\t{\n\t\tmatch self {\n\t\t\tEvictionPolicyKind::ClassicLRU => Box::new(ClassicLRU::new()),\n\t\t\tEvictionPolicyKind::TimestampLRU => Box::new(TimestampLRU),\n\t\t\tEvictionPolicyKind::Lifo => Box::new(Lifo),\n\t\t\tEvictionPolicyKind::Lfu => Box::new(Lfu),\n\t\t\tEvictionPolicyKind::Fifo => Box::new(Fifo),\n\t\t\tEvictionPolicyKind::Swap => Box::new(Swap),\n\t\t}\n\t}\n}\n\n#[cfg(test)]\nmod tests {\n\tuse super::*;\n\n\t#[test]\n\tfn from_usize_valid_indices() {\n\t\tassert_eq!(EvictionPolicyKind::from_usize(0), Some(EvictionPolicyKind::ClassicLRU));\n\t\tassert_eq!(EvictionPolicyKind::from_usize(1), Some(EvictionPolicyKind::TimestampLRU));\n\t\tassert_eq!(EvictionPolicyKind::from_usize(2), Some(EvictionPolicyKind::Lifo));\n\t\tassert_eq!(EvictionPolicyKind::from_usize(3), Some(EvictionPolicyKind::Lfu));\n\t\tassert_eq!(EvictionPolicyKind::from_usize(4), Some(EvictionPolicyKind::Fifo));\n\t\tassert_eq!(EvictionPolicyKind::from_usize(5), Some(EvictionPolicyKind::Swap));\n\t}\n\n\t#[test]\n\tfn from_usize_out_of_range() {\n\t\tassert!(EvictionPolicyKind::from_usize(6).is_none());\n\t\tassert!(EvictionPolicyKind::from_usize(100).is_none());\n\t\tassert!(EvictionPolicyKind::from_usize(usize::MAX).is_none());\n\t}\n\n\t#[test]\n\tfn from_name_valid() {\n\t\tassert_eq!(EvictionPolicyKind::from_name(\"ClassicLRU\"), Some(EvictionPolicyKind::ClassicLRU));\n\t\tassert_eq!(EvictionPolicyKind::from_name(\"TimestampLRU\"), Some(EvictionPolicyKind::TimestampLRU));\n\t\tassert_eq!(EvictionPolicyKind::from_name(\"Lifo\"), Some(EvictionPolicyKind::Lifo));\n\t\tassert_eq!(EvictionPolicyKind::from_name(\"Lfu\"), Some(EvictionPolicyKind::Lfu));\n\t\tassert_eq!(EvictionPolicyKind::from_name(\"Fifo\"), Some(EvictionPolicyKind::Fifo));\n\t\tassert_eq!(EvictionPolicyKind::from_name(\"Swap\"), Some(EvictionPolicyKind::Swap));\n\t}\n\n\t#[test]\n\tfn from_name_unknown() {\n\t\tassert!(EvictionPolicyKind::from_name(\"classiclru\").is_none());\n\t\tassert!(EvictionPolicyKind::from_name(\"NotAPolicy\").is_none());\n\t\tassert!(EvictionPolicyKind::from_name(\"\").is_none());\n\t}\n\n\t#[test]\n\tfn name_round_trip_for_every_variant() {\n\t\tfor p in EvictionPolicyKind::all() {\n\t\t\tlet resolved = EvictionPolicyKind::from_name(p.name())\n\t\t\t\t.unwrap_or_else(|| panic!(\"from_name({}) returned None\", p.name()));\n\t\t\tassert_eq!(resolved, p, \"{}.name() resolved to wrong variant\", p.name());\n\t\t}\n\t}\n\n\t#[test]\n\tfn index_round_trip_for_every_variant() {\n\t\tfor p in EvictionPolicyKind::all() {\n\t\t\tlet resolved = EvictionPolicyKind::from_usize(p.index())\n\t\t\t\t.unwrap_or_else(|| panic!(\"from_usize({}) returned None\", p.index()));\n\t\t\tassert_eq!(resolved, p, \"index {} resolved to wrong variant\", p.index());\n\t\t}\n\t}\n\n\t#[test]\n\tfn indices_are_dense_and_unique() {\n\t\tlet indices: Vec<usize> = EvictionPolicyKind::all().iter().map(|p| p.index()).collect();\n\t\tlet mut sorted = indices.clone();\n\t\tsorted.sort();\n\t\tlet expected: Vec<usize> = (0..EvictionPolicyKind::all().len()).collect();\n\t\tassert_eq!(sorted, expected, \"indices must be 0..N with no gaps or duplicates\");\n\t}\n\n\t#[test]\n\tfn names_are_unique() {\n\t\tlet names = EvictionPolicyKind::all_names();\n\t\tlet mut seen = std::collections::HashSet::new();\n\t\tfor name in names {\n\t\t\tassert!(seen.insert(name), \"duplicate name: {name}\");\n\t\t}\n\t}\n\n\t#[test]\n\tfn all_names_matches_per_variant_name() {\n\t\tlet derived: Vec<&'static str> = EvictionPolicyKind::all().iter().map(|p| p.name()).collect();\n\t\tassert_eq!(derived, EvictionPolicyKind::all_names().to_vec());\n\t}\n\n\t#[test]\n\tfn build_name_matches_kind() {\n\t\tfor kind in EvictionPolicyKind::all() {\n\t\t\tlet policy: Box<dyn EvictionPolicy<i64, i64>> = kind.build();\n\t\t\tassert_eq!(policy.name(), kind.name());\n\t\t}\n\t}\n}\n"
}
```

### 11:41:17 thinking



### 11:41:20 chat

The v002 overlay is written to `overlay/src/shared/eviction_policy.rs`. I could not compile it or run the gates, because this session has no shell (Bash, Glob and Grep were all unavailable), so it is unverified.

**What changed.** The parent's `ClassicLRU` is the only policy modified. Every other policy and the `EvictionPolicyKind` selector are carried over unchanged. The per-slot loop in `eviction_candidate_index` is replaced by a new `sweep(hand, refs, n)` helper that returns the victim and the mask of bits to clear:

- **n ≤ 32:** the mask is doubled into a 2n-bit word so the cyclic search from `hand` becomes linear. `trailing_zeros` gives the victim offset, and one mask (`run | run >> n`) clears the passed-over referenced slots. If no bit is clear, the whole word is cleared and `hand` is the victim.
- **n > 32:** slots at or above 32 are never referenced, so the search runs over the tracked low word. If `hand ≥ 32` the victim is `hand`. Otherwise it searches `[hand, 32)`, and if that is all referenced the victim is slot 32.
- `shift_out`, `get_from_position` and the metadata layout (one `u32` per bucket) are unchanged, so metadata stays at 40 bytes.
- I removed the now-unused `is_set` and `clear` helpers so a `deny(warnings)` setting cannot fail the build.

**Deviations from the hypothesis text.**
- The hypothesis says to store "the slot after the chosen victim" in `hand`. I store the victim index, as the parent does, since the stated invariant is equivalence with v001's loop.
- For n > 32, the hypothesis's "trailing_zeros == 32 means fully referenced" rule does not hold, because slot 32 is always clear. I follow the parent's behaviour there.

**Checks I did by hand.** I traced two small cases against the parent loop, one with a plain walk (n=4, hand=1) and one that wraps (n=4, hand=3). Both produce the same victim and the same mask.

**Next step.** Someone with a shell should run the four gates (`cargo build --tests --features test-utils`, then the contract, universal invariant and invariant gates) against this overlay and check hit-rate on workload-a against v001's 0.7244.


---

- finished: 2026-10-20T11:41:20+00:00
- duration: 100.24s
- status: ok
- input_tokens: 14
- output_tokens: 23373
- cache_read_tokens: 207893
- cache_write_tokens: 46075
- cost_usd: 0.022982
