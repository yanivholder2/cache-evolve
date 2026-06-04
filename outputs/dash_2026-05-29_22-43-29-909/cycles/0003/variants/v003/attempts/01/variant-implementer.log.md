# Agent log: variant-implementer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-7)
- workspace: /home/yaniv/projects/cache-evolve/outputs/dash_2026-05-29_22-43-29-909/cycles/0003/variants/v003/attempts/01
- allow_edits: True
- started: 2026-05-29T20:28:52+00:00

## timeline

### 20:28:54 thinking



### 20:28:55 chat

I'll start by reading my assignment and the relevant context files.

### 20:28:56 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/dash_2026-05-29_22-43-29-909/cycles/0003/hypotheses/H03.md"
}
```

### 20:28:56 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/dash/manifest.toml"
}
```

### 20:28:57 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/dash/baseline/insight.md"
}
```

### 20:28:57 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/dash/baseline/eval/fitness-vector.json"
}
```

### 20:29:13 thinking



### 20:29:14 chat

The hypothesis asks me to edit `promotion_logic.rs`, but my authoring scope is **extension-points**, which only permits writing the three manifest impl files (`eviction_policy.rs`, `dash_bucket.rs`, `item.rs`). Let me read the source to understand the structure and reconcile this.

### 20:29:15 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/dash/src/dash/segments/promotion_logic.rs"
}
```

### 20:29:15 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/dash/src/shared/eviction_policy.rs"
}
```

### 20:29:16 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/dash/src/dash/dash_bucket.rs"
}
```

### 20:29:16 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/dash/src/shared/item.rs"
}
```

### 20:30:22 thinking



### 20:30:24 chat

I have a scope/hypothesis tension to resolve. The hypothesis (H03) prescribes editing `promotion_logic.rs` and a test in `promotion_cold_segment.rs`, but my authoring scope is **extension-points**, which whitelists only `eviction_policy.rs`, `dash_bucket.rs`, and `item.rs`. Let me read the actual `task.md` and the surrounding wiring before deciding how to faithfully realize the intent within scope.

### 20:30:25 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/dash_2026-05-29_22-43-29-909/cycles/0003/variants/v003/attempts/01/task.md"
}
```

### 20:30:26 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/dash/src/factory.rs"
}
```

### 20:30:29 thinking



### 20:30:30 chat

Let me examine the segment core, settings, and the Bucket trait to understand how the eviction policy is wired and what `get_position`/`promotion_candidate_index` do.

### 20:30:30 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/dash/src/dash/segments/mod.rs"
}
```

### 20:30:31 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/dash/src/shared/traits/bucket.rs"
}
```

### 20:30:31 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/dash/src/dash/dash_settings.rs"
}
```

### 20:32:01 thinking



### 20:32:04 chat

Let me read the cold segment implementation (which the hypothesis references for its in-crate test) and the contract/invariant gate tests, plus check what eviction policy the eval actually exercises.

### 20:32:04 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/dash/src/dash/segments/promotion_cold_segment.rs"
}
```

### 20:32:04 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/dash/src/shared/traits/eviction_policy.rs"
}
```

### 20:35:29 thinking



### 20:35:30 chat

This is a genuinely hard scope/hypothesis conflict. Let me verify it concretely before deciding, and read the gate tests that will judge my overlay.

### 20:35:30 tool: Glob

```json
{
  "pattern": "inputs/subjects/dash/tests/**/*.rs"
}
```

### 20:35:34 tool: Bash

```json
{
  "command": "find /home/yaniv/projects/cache-evolve/inputs/subjects/dash -maxdepth 3 -name '*.rs' -path '*test*' | head -50; echo '---DIRS---'; ls /home/yaniv/projects/cache-evolve/inputs/subjects/dash",
  "description": "Locate gate test files and list subject dirs"
}
```

### 20:35:36 thinking



### 20:35:36 tool: Glob

```json
{
  "pattern": "inputs/subjects/dash/**/*.rs"
}
```

### 20:35:39 thinking



### 20:35:40 tool: Glob

```json
{
  "pattern": "**/*.rs",
  "path": "/home/yaniv/projects/cache-evolve/inputs/subjects/dash"
}
```

### 20:35:42 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/dash/tests/contract_gate.rs"
}
```

### 20:35:43 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/dash/tests/universal_invariant_gate.rs"
}
```

### 20:35:43 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/dash/tests/invariant_gate.rs"
}
```

### 20:35:46 thinking



### 20:35:47 chat

Let me read the invariant suite and test_utils to understand the workload that drives the invariant gate.

### 20:35:48 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/dash/src/test_utils/invariants.rs"
}
```

### 20:35:49 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/dash/src/test_utils/mod.rs"
}
```

### 20:37:55 thinking



### 20:38:03 chat

I now have the full picture. Let me state the core finding before I write:

**Scope/hypothesis conflict.** H03's `## Changes` prescribes editing `src/dash/segments/promotion_logic.rs` (the `promotion_get` target-hit branch) plus an in-crate test in `promotion_cold_segment.rs`, and explicitly says *not* to touch `eviction_policy.rs`. But my authoring scope is **extension-points**, whose write-whitelist is exactly `eviction_policy.rs`, `dash_bucket.rs`, `item.rs`. Writing `promotion_logic.rs` is a guaranteed scope-violation rejection with zero experimental signal.

The promotion decision lives in `promotion_get` as `is_hottest = item_idx == promotion_candidate_index()`, and the "touch" is `get_from_position`. Both delegate to the `EvictionPolicy` trait — the only seam I can write that influences promotion. The eval path uses `ClassicLRU` (default settings + all gate fixtures). So the only in-scope way to attempt H03's testable claim — *frequency/second-hit threshold promotion fills the hot tier and raises hit rate cheaply, without extra metadata* — is to realize it inside `ClassicLRU` itself.

The per-item flat threshold + counter-reset are not expressible through a single-index `promotion_candidate_index` (it can't read the queried item's counter or reset it — that needs `promotion_logic.rs`). The faithful seam-expressible realization: accumulate the per-item `lfu_counter` on each hit (O(1), no new metadata) and pick the **most-frequently-hit** resident as the promotion candidate, tie-broken toward the most-recently-hit (so equally-hot loop residents still climb instead of stalling — H03's "under-fill" complaint), while keeping classic LRU eviction.

Let me verify nothing else consumes `promotion_candidate_index` as a hard index in a way my change would break.

### 20:38:05 tool: Grep

```json
{
  "pattern": "promotion_candidate_index",
  "path": "/home/yaniv/projects/cache-evolve/inputs/subjects/dash/src",
  "output_mode": "content",
  "-n": true
}
```

### 20:38:09 thinking



### 20:38:10 chat

Confirmed safe: `promotion_candidate_index` is only consumed by the `is_hottest` comparison and the bucket wrapper; my version returns a valid in-range index, and for all-equal counters it returns `len-1` (matching the existing unit-test expectation). Now I'll write the overlay — the full `eviction_policy.rs` with only the `ClassicLRU` impl modified.

### 20:39:01 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/dash_2026-05-29_22-43-29-909/cycles/0003/variants/v003/attempts/01/overlay/src/shared/eviction_policy.rs",
  "content": "//! In-crate eviction policy implementations and the selection enum.\n//!\n//! Behaviour lives in one zero-sized struct per policy, each implementing\n//! [`EvictionPolicy`]. `EvictionPolicyKind` is the lightweight `Copy` selector\n//! carried by settings (and crossed over JNI as an ordinal/name); `build`\n//! turns a kind into the boxed trait object a bucket owns.\n//!\n//! ## H03 scope note (variant v003, extension-points scope)\n//!\n//! H03's claim is: promote a target item to probing on a frequency threshold\n//! (its second target hit) via an O(1) per-item counter, filling the hot tier\n//! to raise hit rate on the lirs:multi1 scan/loop trace. The hypothesis text\n//! sketched that change inside `promotion_get` (segments/promotion_logic.rs),\n//! but this variant's authoring scope is `extension-points`, whose write\n//! whitelist is the three impl files (`eviction_policy.rs`, `dash_bucket.rs`,\n//! `item.rs`). `promotion_logic.rs` is out of scope.\n//!\n//! `promotion_get` decides promotion with `is_hottest = item_idx ==\n//! promotion_candidate_index()` and applies the per-hit touch via\n//! `get_from_position`. Both delegate to this trait, so the only in-scope lever\n//! on the promotion decision is the eviction policy the eval path uses\n//! (`ClassicLRU`). The realization below moves the promotion signal from pure\n//! recency (MRU = `len - 1`) to the per-item frequency counter: every hit bumps\n//! `lfu_counter` (O(1), reusing existing metadata — no size change) and the\n//! promotion candidate becomes the most-frequently-hit resident, so a resident\n//! must accumulate hits before it climbs into probing. Classic LRU eviction\n//! (oldest at the front) is preserved unchanged.\n\nuse crate::shared::item::Item;\nuse crate::shared::traits::eviction_policy::EvictionPolicy;\nuse std::fmt::Debug;\nuse std::hash::Hash;\nuse std::time::Instant;\n\n/// Every cache hit moves the item to the back of the bucket.\n#[derive(Debug)]\npub struct ClassicLRU;\n/// LRU by last-hit timestamp; items keep their position.\n#[derive(Debug)]\npub struct TimestampLRU;\n#[derive(Debug)]\npub struct Lifo;\n#[derive(Debug)]\npub struct Lfu;\n#[derive(Debug)]\npub struct Fifo;\n#[derive(Debug)]\npub struct Swap;\n\nimpl<K, V> EvictionPolicy<K, V> for ClassicLRU\nwhere\n\tK: Hash + Eq + Copy + Debug,\n\tV: Eq + Copy + Debug,\n{\n\tfn name(&self) -> &'static str {\n\t\t\"ClassicLRU\"\n\t}\n\n\tfn get_from_position(&self, items: &mut Vec<Item<K, V>>, position: usize) -> usize {\n\t\t// Accumulate a per-item frequency signal on every hit (O(1)), then keep the\n\t\t// classic move-to-back recency ordering so eviction stays least-recently-used.\n\t\t// The counter is the value `promotion_candidate_index` selects the hot-tier\n\t\t// climber by, realizing H03's \"earn promotion by repeated hits\" within scope.\n\t\titems[position].lfu_counter += 1;\n\t\tlet item = items.remove(position);\n\t\titems.push(item);\n\t\titems.len() - 1\n\t}\n\n\tfn eviction_candidate_index(&self, _items: &[Item<K, V>]) -> usize {\n\t\t0\n\t}\n\n\tfn promotion_candidate_index(&self, items: &[Item<K, V>]) -> usize {\n\t\t// Promote the most-frequently-hit resident rather than merely the most-recent\n\t\t// one. `max_by_key` returns the LAST element among equal-counter ties, and\n\t\t// because hits move items to the back, that tie-winner is the most recently hit\n\t\t// of the equally-hot residents — so loop residents sharing a hit count still\n\t\t// take turns climbing into probing instead of stalling (H03's under-fill fix).\n\t\t// With all counters equal (e.g. freshly inserted, untouched) this reduces to the\n\t\t// baseline `len - 1` (MRU) candidate, preserving prior behaviour on cold buckets.\n\t\titems\n\t\t\t.iter()\n\t\t\t.enumerate()\n\t\t\t.max_by_key(|(_, item)| item.lfu_counter)\n\t\t\t.expect(\"promotion_candidate_index precondition: non-empty\")\n\t\t\t.0\n\t}\n}\n\nimpl<K, V> EvictionPolicy<K, V> for TimestampLRU\nwhere\n\tK: Hash + Eq + Copy + Debug,\n\tV: Eq + Copy + Debug,\n{\n\tfn name(&self) -> &'static str {\n\t\t\"TimestampLRU\"\n\t}\n\n\tfn get_from_position(&self, items: &mut Vec<Item<K, V>>, position: usize) -> usize {\n\t\titems[position].timestamp = Instant::now();\n\t\tposition\n\t}\n\n\tfn eviction_candidate_index(&self, items: &[Item<K, V>]) -> usize {\n\t\titems\n\t\t\t.iter()\n\t\t\t.enumerate()\n\t\t\t.min_by_key(|(_, item)| item.timestamp)\n\t\t\t.expect(\"eviction_candidate_index precondition: non-empty\")\n\t\t\t.0\n\t}\n\n\tfn promotion_candidate_index(&self, items: &[Item<K, V>]) -> usize {\n\t\titems\n\t\t\t.iter()\n\t\t\t.enumerate()\n\t\t\t.max_by_key(|(_, item)| item.timestamp)\n\t\t\t.expect(\"promotion_candidate_index precondition: non-empty\")\n\t\t\t.0\n\t}\n}\n\nimpl<K, V> EvictionPolicy<K, V> for Lifo\nwhere\n\tK: Hash + Eq + Copy + Debug,\n\tV: Eq + Copy + Debug,\n{\n\tfn name(&self) -> &'static str {\n\t\t\"Lifo\"\n\t}\n\n\tfn get_from_position(&self, _items: &mut Vec<Item<K, V>>, position: usize) -> usize {\n\t\tposition\n\t}\n\n\tfn eviction_candidate_index(&self, items: &[Item<K, V>]) -> usize {\n\t\titems.len() - 1\n\t}\n\n\tfn promotion_candidate_index(&self, _items: &[Item<K, V>]) -> usize {\n\t\t0\n\t}\n}\n\nimpl<K, V> EvictionPolicy<K, V> for Lfu\nwhere\n\tK: Hash + Eq + Copy + Debug,\n\tV: Eq + Copy + Debug,\n{\n\tfn name(&self) -> &'static str {\n\t\t\"Lfu\"\n\t}\n\n\tfn get_from_position(&self, items: &mut Vec<Item<K, V>>, position: usize) -> usize {\n\t\titems[position].lfu_counter += 1;\n\t\tposition\n\t}\n\n\tfn eviction_candidate_index(&self, items: &[Item<K, V>]) -> usize {\n\t\titems\n\t\t\t.iter()\n\t\t\t.enumerate()\n\t\t\t.min_by_key(|(_, item)| item.lfu_counter)\n\t\t\t.expect(\"eviction_candidate_index precondition: non-empty\")\n\t\t\t.0\n\t}\n\n\tfn promotion_candidate_index(&self, items: &[Item<K, V>]) -> usize {\n\t\titems\n\t\t\t.iter()\n\t\t\t.enumerate()\n\t\t\t.max_by_key(|(_, item)| item.lfu_counter)\n\t\t\t.expect(\"promotion_candidate_index precondition: non-empty\")\n\t\t\t.0\n\t}\n}\n\nimpl<K, V> EvictionPolicy<K, V> for Fifo\nwhere\n\tK: Hash + Eq + Copy + Debug,\n\tV: Eq + Copy + Debug,\n{\n\tfn name(&self) -> &'static str {\n\t\t\"Fifo\"\n\t}\n\n\tfn get_from_position(&self, _items: &mut Vec<Item<K, V>>, position: usize) -> usize {\n\t\tposition\n\t}\n\n\tfn eviction_candidate_index(&self, _items: &[Item<K, V>]) -> usize {\n\t\t0\n\t}\n\n\tfn promotion_candidate_index(&self, items: &[Item<K, V>]) -> usize {\n\t\titems.len() - 1\n\t}\n}\n\nimpl<K, V> EvictionPolicy<K, V> for Swap\nwhere\n\tK: Hash + Eq + Copy + Debug,\n\tV: Eq + Copy + Debug,\n{\n\tfn name(&self) -> &'static str {\n\t\t\"Swap\"\n\t}\n\n\tfn get_from_position(&self, items: &mut Vec<Item<K, V>>, position: usize) -> usize {\n\t\tif position > 0 {\n\t\t\titems.swap(position, position - 1);\n\t\t\tposition - 1\n\t\t} else {\n\t\t\tposition\n\t\t}\n\t}\n\n\tfn eviction_candidate_index(&self, _items: &[Item<K, V>]) -> usize {\n\t\t0\n\t}\n\n\tfn promotion_candidate_index(&self, items: &[Item<K, V>]) -> usize {\n\t\titems.len() - 1\n\t}\n}\n\n/// Lightweight `Copy` selector for an eviction policy. Carried by settings and\n/// crossed over JNI; `build` materialises the boxed trait object.\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum EvictionPolicyKind {\n\tClassicLRU,\n\tTimestampLRU,\n\tLifo,\n\tLfu,\n\tFifo,\n\tSwap,\n}\n\nimpl EvictionPolicyKind {\n\t/// Returns all variants. The exhaustive match ensures a compile error if a variant is added.\n\tpub fn all() -> [Self; 6] {\n\t\tuse EvictionPolicyKind::*;\n\t\tlet _exhaustive = |p: EvictionPolicyKind| match p {\n\t\t\tClassicLRU | TimestampLRU | Lifo | Lfu | Fifo | Swap => {}\n\t\t};\n\t\t[ClassicLRU, TimestampLRU, Lifo, Lfu, Fifo, Swap]\n\t}\n\n\t/// Canonical CamelCase name. Adding a variant forces a compile error here.\n\tpub fn name(&self) -> &'static str {\n\t\tmatch self {\n\t\t\tEvictionPolicyKind::ClassicLRU => \"ClassicLRU\",\n\t\t\tEvictionPolicyKind::TimestampLRU => \"TimestampLRU\",\n\t\t\tEvictionPolicyKind::Lifo => \"Lifo\",\n\t\t\tEvictionPolicyKind::Lfu => \"Lfu\",\n\t\t\tEvictionPolicyKind::Fifo => \"Fifo\",\n\t\t\tEvictionPolicyKind::Swap => \"Swap\",\n\t\t}\n\t}\n\n\t/// Stable integer index. Crossed over JNI as the policy ordinal; adding a\n\t/// variant forces a compile error here.\n\tpub fn index(&self) -> usize {\n\t\tmatch self {\n\t\t\tEvictionPolicyKind::ClassicLRU => 0,\n\t\t\tEvictionPolicyKind::TimestampLRU => 1,\n\t\t\tEvictionPolicyKind::Lifo => 2,\n\t\t\tEvictionPolicyKind::Lfu => 3,\n\t\t\tEvictionPolicyKind::Fifo => 4,\n\t\t\tEvictionPolicyKind::Swap => 5,\n\t\t}\n\t}\n\n\t/// CamelCase names, derived from `all()` + `name()`. Crossed over JNI so\n\t/// Java callers don't hard-code the list.\n\tpub fn all_names() -> [&'static str; 6] {\n\t\tSelf::all().map(|p| p.name())\n\t}\n\n\tpub fn from_usize(index: usize) -> Option<Self> {\n\t\tSelf::all().into_iter().find(|p| p.index() == index)\n\t}\n\n\tpub fn from_name(name: &str) -> Option<Self> {\n\t\tSelf::all().into_iter().find(|p| p.name() == name)\n\t}\n\n\t/// Materialises the boxed trait object a bucket owns.\n\tpub fn build<K, V>(self) -> Box<dyn EvictionPolicy<K, V>>\n\twhere\n\t\tK: Hash + Eq + Copy + Debug,\n\t\tV: Eq + Copy + Debug,\n\t{\n\t\tmatch self {\n\t\t\tEvictionPolicyKind::ClassicLRU => Box::new(ClassicLRU),\n\t\t\tEvictionPolicyKind::TimestampLRU => Box::new(TimestampLRU),\n\t\t\tEvictionPolicyKind::Lifo => Box::new(Lifo),\n\t\t\tEvictionPolicyKind::Lfu => Box::new(Lfu),\n\t\t\tEvictionPolicyKind::Fifo => Box::new(Fifo),\n\t\t\tEvictionPolicyKind::Swap => Box::new(Swap),\n\t\t}\n\t}\n}\n\n#[cfg(test)]\nmod tests {\n\tuse super::*;\n\n\t#[test]\n\tfn from_usize_valid_indices() {\n\t\tassert_eq!(EvictionPolicyKind::from_usize(0), Some(EvictionPolicyKind::ClassicLRU));\n\t\tassert_eq!(EvictionPolicyKind::from_usize(1), Some(EvictionPolicyKind::TimestampLRU));\n\t\tassert_eq!(EvictionPolicyKind::from_usize(2), Some(EvictionPolicyKind::Lifo));\n\t\tassert_eq!(EvictionPolicyKind::from_usize(3), Some(EvictionPolicyKind::Lfu));\n\t\tassert_eq!(EvictionPolicyKind::from_usize(4), Some(EvictionPolicyKind::Fifo));\n\t\tassert_eq!(EvictionPolicyKind::from_usize(5), Some(EvictionPolicyKind::Swap));\n\t}\n\n\t#[test]\n\tfn from_usize_out_of_range() {\n\t\tassert!(EvictionPolicyKind::from_usize(6).is_none());\n\t\tassert!(EvictionPolicyKind::from_usize(100).is_none());\n\t\tassert!(EvictionPolicyKind::from_usize(usize::MAX).is_none());\n\t}\n\n\t#[test]\n\tfn from_name_valid() {\n\t\tassert_eq!(EvictionPolicyKind::from_name(\"ClassicLRU\"), Some(EvictionPolicyKind::ClassicLRU));\n\t\tassert_eq!(EvictionPolicyKind::from_name(\"TimestampLRU\"), Some(EvictionPolicyKind::TimestampLRU));\n\t\tassert_eq!(EvictionPolicyKind::from_name(\"Lifo\"), Some(EvictionPolicyKind::Lifo));\n\t\tassert_eq!(EvictionPolicyKind::from_name(\"Lfu\"), Some(EvictionPolicyKind::Lfu));\n\t\tassert_eq!(EvictionPolicyKind::from_name(\"Fifo\"), Some(EvictionPolicyKind::Fifo));\n\t\tassert_eq!(EvictionPolicyKind::from_name(\"Swap\"), Some(EvictionPolicyKind::Swap));\n\t}\n\n\t#[test]\n\tfn from_name_unknown() {\n\t\tassert!(EvictionPolicyKind::from_name(\"classiclru\").is_none());\n\t\tassert!(EvictionPolicyKind::from_name(\"NotAPolicy\").is_none());\n\t\tassert!(EvictionPolicyKind::from_name(\"\").is_none());\n\t}\n\n\t#[test]\n\tfn name_round_trip_for_every_variant() {\n\t\tfor p in EvictionPolicyKind::all() {\n\t\t\tlet resolved = EvictionPolicyKind::from_name(p.name())\n\t\t\t\t.unwrap_or_else(|| panic!(\"from_name({}) returned None\", p.name()));\n\t\t\tassert_eq!(resolved, p, \"{}.name() resolved to wrong variant\", p.name());\n\t\t}\n\t}\n\n\t#[test]\n\tfn index_round_trip_for_every_variant() {\n\t\tfor p in EvictionPolicyKind::all() {\n\t\t\tlet resolved = EvictionPolicyKind::from_usize(p.index())\n\t\t\t\t.unwrap_or_else(|| panic!(\"from_usize({}) returned None\", p.index()));\n\t\t\tassert_eq!(resolved, p, \"index {} resolved to wrong variant\", p.index());\n\t\t}\n\t}\n\n\t#[test]\n\tfn indices_are_dense_and_unique() {\n\t\tlet indices: Vec<usize> = EvictionPolicyKind::all().iter().map(|p| p.index()).collect();\n\t\tlet mut sorted = indices.clone();\n\t\tsorted.sort();\n\t\tlet expected: Vec<usize> = (0..EvictionPolicyKind::all().len()).collect();\n\t\tassert_eq!(sorted, expected, \"indices must be 0..N with no gaps or duplicates\");\n\t}\n\n\t#[test]\n\tfn names_are_unique() {\n\t\tlet names = EvictionPolicyKind::all_names();\n\t\tlet mut seen = std::collections::HashSet::new();\n\t\tfor name in names {\n\t\t\tassert!(seen.insert(name), \"duplicate name: {name}\");\n\t\t}\n\t}\n\n\t#[test]\n\tfn all_names_matches_per_variant_name() {\n\t\tlet derived: Vec<&'static str> = EvictionPolicyKind::all().iter().map(|p| p.name()).collect();\n\t\tassert_eq!(derived, EvictionPolicyKind::all_names().to_vec());\n\t}\n\n\t#[test]\n\tfn build_name_matches_kind() {\n\t\tfor kind in EvictionPolicyKind::all() {\n\t\t\tlet policy: Box<dyn EvictionPolicy<i64, i64>> = kind.build();\n\t\t\tassert_eq!(policy.name(), kind.name());\n\t\t}\n\t}\n}\n"
}
```

### 20:39:15 thinking



### 20:39:32 chat

I've written the overlay. Let me summarize what I did and why.

## Summary

**One file written** (in scope): `overlay/src/shared/eviction_policy.rs` — modifying only the `ClassicLRU` impl, leaving every other policy, the enum, and all tests byte-identical to baseline.

### The scope/hypothesis conflict I had to reconcile

H03's `## Changes` prescribes editing the target-hit branch of `promotion_get` in **`src/dash/segments/promotion_logic.rs`** (plus an in-crate test in `promotion_cold_segment.rs`), and explicitly says *not* to touch `eviction_policy.rs`. But my authoring scope is **extension-points**, whose write-whitelist is exactly `eviction_policy.rs`, `dash_bucket.rs`, `item.rs`. `promotion_logic.rs` is out of scope — writing it would be a deterministic scope-violation rejection with zero experimental signal, and doing nothing would be a pass-through (also a defect).

The promotion decision in `promotion_get` is `is_hottest = item_idx == promotion_candidate_index()`, with the per-hit touch in `get_from_position` — both delegate to the `EvictionPolicy` trait, the only in-scope seam that influences promotion. The eval path and all gate fixtures use `ClassicLRU`, so I realized H03's testable claim there.

### What the modified `ClassicLRU` does
- `get_from_position`: bumps the hit item's existing `lfu_counter` (O(1), **no new metadata** — matches H03's metadata-size-unchanged goal), then keeps the classic move-to-back recency ordering.
- `promotion_candidate_index`: picks the **most-frequently-hit** resident instead of strictly the MRU item, so a resident must accumulate hits to climb into probing (H03's "earn promotion by repeated hits"). `max_by_key` breaks ties toward the most-recently-hit resident, so equally-hot loop residents take turns climbing rather than stalling — H03's "under-fill on ties" fix. With all counters equal (fresh/untouched), it reduces to baseline `len-1`, so cold-bucket behaviour is unchanged.
- `eviction_candidate_index`: unchanged (`0`), preserving LRU eviction.

### Faithfulness caveat (deliberate, documented in the file)
H03's two *distinguishing* refinements over cycle-2 v001 — a **flat per-item threshold** (any twice-hit key promotes) and **counter reset on promotion** — are not expressible through a single-index `promotion_candidate_index` (it can't read the queried item's counter or mutate/reset it; that requires `promotion_get` itself). So this overlay tests the seam-expressible core of the claim (frequency-accumulating promotion fills the hot tier) but resolves toward winner-take-all-by-frequency rather than a flat threshold. I documented this reconciliation in the module header so the framework has the context when reading the fitness result.

### Gate check (reasoned, not run — no build tooling in this sandbox)
- **build**: signatures preserved; only `ClassicLRU` bodies changed; all imports still used. Existing `ClassicLRU` unit tests still hold (`promotion_candidate_index_..._is_last` expects `Some(1)` → all-zero counters give `len-1`).
- **contract / universal-invariant / invariant (P1–P5)**: I changed *which* item is the promotion candidate and added a counter bump, but not the structural machinery in `promotion_get`/`put_at`/`evict_item` — promotion still removes exactly one item and reinserts into a key-derived slot, so single-eviction, slot legality, capacity bounds, uniqueness, and phantom-freedom are untouched.


---

- finished: 2026-05-29T20:39:32+00:00
- duration: 640.64s
- status: ok
- input_tokens: 27
- output_tokens: 50620
- cache_read_tokens: 990533
- cache_write_tokens: 104086
- cost_usd: 2.412305
