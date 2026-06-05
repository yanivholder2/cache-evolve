# Agent log: variant-implementer

- adapter: ClaudeSdkAdapter(model=claude-haiku-4-5-20251001)
- workspace: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0008/variants/v001/attempts/01
- allow_edits: True
- started: 2026-06-05T19:28:19+00:00

## timeline

### 19:28:23 thinking

Let me start by understanding the task. I'm a variant-implementer for the cache-evolve research loop. I need to:

1. Read the hypothesis file to understand what changes I need to make
2. Read the subject manifest to understand the extension points
3. Look at the parent variant's work to understand context
4. Implement the hypothesis changes as an overlay

Let me start by reading the key files.

First, I'll read the hypothesis file to understand what I need to implement.

### 19:28:23 chat

I'll help you implement this variant. Let me start by reading the key files to understand the assignment.

### 19:28:24 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0008/hypotheses/H01.md"
}
```

### 19:28:24 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/manifest.toml"
}
```

### 19:28:24 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007/variants/v001/attempts/01/overlay"
}
```

### 19:28:26 thinking

The overlay directory is a directory, so I need to check what files are in it. Let me use Bash to see what's in there.

### 19:28:26 chat

Let me check what files are in the parent variant's overlay:

### 19:28:26 tool: Bash

```json
{
  "command": "find /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007/variants/v001/attempts/01/overlay -type f"
}
```

### 19:28:28 thinking

I don't have direct bash access. Let me use Glob to find files in the parent variant overlay.

### 19:28:28 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007/variants/v001/attempts/01/overlay/**/*"
}
```

### 19:28:30 thinking

Good, so the parent variant modified two files:
1. src/shared/item.rs
2. src/dash/segments/dash_segment.rs

Now let me read the parent variant's insight and fitness vector to understand what was done.

### 19:28:30 chat

Let me read the parent variant's context:

### 19:28:30 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007/variants/v001/insight.md"
}
```

### 19:28:30 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007/variants/v001/eval/fitness-vector.json"
}
```

### 19:28:32 thinking

So the parent variant achieved a hit-rate of 0.684. The insight mentions that it attempted target-first eviction priority but it didn't improve the hit-rate. The takeaway suggests the bottleneck is upstream in how items arrive and navigate the promotion pathway.

Now I need to read the parent variant's overlay files to understand what was implemented, and then the hypothesis for my variant tells me I need to implement occupancy-coordinated demotion.

Let me read the parent variant's implementation:

### 19:28:33 chat

Now let me read the parent variant's overlay files to understand what was done:

### 19:28:33 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007/variants/v001/attempts/01/overlay/src/dash/segments/dash_segment.rs"
}
```

### 19:28:33 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007/variants/v001/attempts/01/overlay/src/shared/item.rs"
}
```

### 19:28:38 thinking

Now I understand the context. The parent variant (v001 in cycle 0007) implemented:
1. A `promotion_age` field in Item to track access cycles since promotion
2. A target-first eviction priority in `promote_to_probing()` that proactively evicts from target when probing is full
3. An eligibility delay gate that blocks promotions when `promotion_age < 1`

The parent achieved a hit rate of 0.684 but the insight suggests the approach didn't help much.

Now my task (H01 for this cycle) is to modify the demotion logic in `demote_from_normal_bucket()` to implement occupancy-aware eviction destination selection:

1. Before demoting from probing to target, check target tier occupancy
2. If target occupancy < 70%, demote to target with back insertion (existing behavior)
3. If target occupancy >= 70%, demote directly to stash with back insertion instead
4. Keep the existing demotion-recovery back-insertion strategy intact

Let me read the subject source code to understand the structure better.

### 19:28:38 chat

Now let me read the subject source code to understand the implementation better:

### 19:28:38 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/src/dash/segments/dash_segment.rs"
}
```

### 19:28:38 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/src/shared/item.rs"
}
```

### 19:28:44 thinking

Good! Now I understand what I need to do. The baseline (subject) code doesn't have the `promotion_age` field or the other changes from the parent variant. I need to:

1. First copy the parent variant's implementation which has the `promotion_age` field and the H01 target-first eviction priority and H02 eligibility delay gate
2. Then modify the `demote_from_normal_bucket()` function to implement occupancy-aware eviction

The key change is: when an item is being demoted from probing to target, I need to:
- Check the target tier occupancy
- If target occupancy < 70%, demote to target with back insertion (existing behavior)
- If target occupancy >= 70%, demote directly to stash with back insertion instead

Looking at the parent variant's code, I can see the `demote_from_normal_bucket()` function starting at line 147. I need to modify this to add occupancy checking.

I need to understand how to check occupancy. Looking at the core, I can see `core.buckets[target_idx].size()` is used to check if a bucket has residents. I need to calculate:
- Current number of items in target tier
- Total capacity of target tier
- Determine if occupancy >= 70%

Let me check if there are methods to get size and capacity information from the bucket.

### 19:28:44 chat

Let me check the SegmentCore and bucket structures to understand how to get occupancy information:

### 19:28:45 tool: Grep

```json
{
  "pattern": "struct SegmentCore|fn size|fn capacity",
  "type": "rust",
  "path": "/home/user/cache-evolve/inputs/subjects/dash/src",
  "head_limit": 50
}
```

### 19:28:46 thinking

Let me look at the SegmentCore structure and how occupancy is calculated.

### 19:28:47 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/src/dash/segments/mod.rs",
  "limit": 200
}
```

### 19:28:56 thinking

Good! I can see that individual buckets have `size()` and `get_max_size()` methods. Now let me understand the structure better. I need to calculate the total occupancy of the target tier. Since there are multiple target buckets (one per segment entry), I need to sum them up.

Actually, looking at the code, I see that `self.buckets` are all the normal buckets, and `segment_size` is `num_of_normal_buckets`. Each target index has a corresponding bucket. So to calculate target tier occupancy, I would need to sum the sizes of all target buckets, but that's not straightforward because target buckets are determined by the hash of the key.

Wait, let me re-read the code. Looking at the parent variant's code:
- `target_idx = get_index(&item.key, core.segment_size)` - this gets the target bucket index
- `probing_idx = (target_idx + 1) % core.segment_size` - this gets the probing bucket

So each key hashes to a specific target bucket and probing bucket. The "target tier" is not all buckets, it's all the target slots across all keys.

Actually, I think I'm overcomplicating this. Looking at the hypothesis again, it says "check target tier occupancy". The target tier consists of all the target buckets. But wait, not all buckets are always acting as targets - some might be probing buckets for other keys.

Actually, re-reading the DASH paper and the code structure:
- The cache has `segment_size` (num_of_normal_buckets) normal buckets
- Each normal bucket can hold a key in either its target role (for that key's hash) or as a probing bucket (for another key's hash)
- The target "tier" isn't a fixed set of buckets; it's the concept of being in your true target bucket

Hmm, this is getting complex. 

Looking at what the parent variant does with target occupancy on line 122, it checks `core.buckets[target_idx].size() > 0` - so it's examining occupancy of a specific bucket rather than some aggregate. For this hypothesis, I should check if the particular target bucket where this item belongs is saturated by calculating `core.buckets[true_target].size() / core.buckets[true_target].get_max_size()`.

When that ratio exceeds 0.70, I'll demote directly to stash instead of the target bucket. Otherwise, the item goes to its target bucket as normal. Let me code this up with the parent variant's changes integrated alongside this occupancy-aware logic.

### 19:28:56 chat

Now I have a good understanding. Let me create the overlay directory and implement the variant. Based on the hypothesis, I need to:

1. Copy the parent variant's implementation (which includes `promotion_age` and other changes)
2. Modify `demote_from_normal_bucket()` to check target tier occupancy before deciding where to demote

Let me create the modified `dash_segment.rs`:

### 19:29:57 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0008/variants/v001/attempts/01/overlay/src/dash/segments/dash_segment.rs",
  "content": "use super::{Segment, SegmentCore, Slot};\nuse crate::dash::dash_settings::DashSettings;\nuse crate::shared::item::Item;\nuse crate::shared::traits::bucket::InsertPosition;\nuse crate::shared::utils::get_index;\nuse log::{debug, info};\nuse std::fmt::Debug;\nuse std::hash::Hash;\n\n/// The dash Subject's baseline segment.\n///\n/// Its `get` realises the **promotion** design: target and probing form a tier,\n/// and items earn their way into the probing bucket only by proving they're the\n/// hottest in their target. A stash→target insert lands at the LRU end\n/// (`InsertPosition::Front`), so an item must climb before it can promote again.\n/// Demotion cascades down the chain `probing -> target -> stash -> out`, with a\n/// one-more-chance step between adjacent tiers.\n///\n/// The front-insert promotion is only the *baseline* design; this whole type is\n/// the segment extension point and the framework is free to evolve where items\n/// land on a hit and how they migrate between slots.\n#[derive(Debug)]\npub struct DashSegment<K, V>\nwhere\n\tK: Hash + Eq + Copy + Debug + 'static,\n\tV: Eq + Copy + Debug + 'static,\n{\n\tcore: SegmentCore<K, V>,\n}\n\nimpl<K, V> DashSegment<K, V>\nwhere\n\tK: Hash + Eq + Copy + Debug + 'static,\n\tV: Eq + Copy + Debug + 'static,\n{\n\t/// Creates a new DashSegment with fallible initialization.\n\tpub fn try_new(settings: &DashSettings) -> Option<Self> {\n\t\tif let Some(core) = SegmentCore::try_new(settings) {\n\t\t\tinfo!(\"DashSegment.try_new\");\n\t\t\tSome(Self { core })\n\t\t} else {\n\t\t\tlog::error!(\"DashSegment.try_new: core initialization failed\");\n\t\t\tNone\n\t\t}\n\t}\n\n\t/// Creates a new DashSegment.\n\t/// # Panics\n\t/// Panics if initialization fails.\n\tpub fn new(settings: &DashSettings) -> Self {\n\t\tSelf::try_new(settings).expect(\"DashSegment::new: invalid settings\")\n\t}\n}\n\nimpl<K, V> Segment<K, V> for DashSegment<K, V>\nwhere\n\tK: Hash + Eq + Copy + Debug + 'static,\n\tV: Eq + Copy + Debug + 'static,\n{\n\tfn core(&self) -> &SegmentCore<K, V> {\n\t\t&self.core\n\t}\n\tfn core_mut(&mut self) -> &mut SegmentCore<K, V> {\n\t\t&mut self.core\n\t}\n\n\tfn get(&mut self, key: &K) -> Option<&Item<K, V>> {\n\t\tpromotion_get(&mut self.core, key, InsertPosition::Front)\n\t}\n}\n\n// ── Promotion logic ────────────────────────────────────────────────────────\n// Promotion treats target and probing as a tier: items earn their way into the\n// probing bucket only by proving they're the hottest in their target. Demotion\n// cascades down the chain `probing -> target -> stash -> out`, with a\n// one-more-chance step between adjacent tiers.\n//\n// **H02 Extension**: Track promotion eligibility via promotion_age. Newly promoted\n// items (promotion_age = 0) cannot advance to the next tier until they've proven\n// fitness at their current tier (promotion_age >= 1). Demoted items have promotion_age = 1,\n// allowing immediate re-entry.\n//\n// **H01 Extension**: In promote_to_probing, implement target-first eviction priority.\n// Before inserting into the probing bucket (and potentially evicting a probing resident),\n// check if the target bucket is non-empty and probing is full. If so, proactively evict\n// the LRU item from target and demote it to stash. This protects the hot probing tier\n// by shifting eviction burden to the intermediate target tier.\n//\n// **Current cycle H01 Extension**: In demote_from_normal_bucket, implement occupancy-aware\n// destination selection. When demoting from probing to target, check target tier occupancy.\n// If target occupancy < 70%, demote to target (existing behavior). If >= 70%, bypass\n// congested target and demote directly to stash.\n\n/// Moves `item` into its target bucket. Any evicted item flows through the tier cascade.\n/// Sets promotion_age = 0 to mark the item as newly promoted to target.\nfn promote_to_target<K, V>(core: &mut SegmentCore<K, V>, mut item: Item<K, V>, insert_pos: InsertPosition)\nwhere\n\tK: Hash + Eq + Copy + Debug + 'static,\n\tV: Eq + Copy + Debug + 'static,\n{\n\titem.promotion_age = 0;\n\tlet target_idx = get_index(&item.key, core.segment_size);\n\tlet Some(evicted) = core.put_at(Slot::Target, insert_pos, item) else {\n\t\treturn;\n\t};\n\tdemote_from_normal_bucket(core, evicted, target_idx, insert_pos);\n}\n\n/// Moves `item` into its probing bucket. Any evicted item flows through the tier cascade.\n/// Sets promotion_age = 0 to mark the item as newly promoted to probing.\n///\n/// **H01 Enhancement**: Before inserting, check if probing is full and target has residents.\n/// If both conditions hold, proactively evict the LRU item from target and demote it to stash.\n/// This shifts eviction pressure away from the hot probing tier to the intermediate target tier.\nfn promote_to_probing<K, V>(core: &mut SegmentCore<K, V>, mut item: Item<K, V>, insert_pos: InsertPosition)\nwhere\n\tK: Hash + Eq + Copy + Debug + 'static,\n\tV: Eq + Copy + Debug + 'static,\n{\n\titem.promotion_age = 0;\n\tlet target_idx = get_index(&item.key, core.segment_size);\n\tlet probing_idx = (target_idx + 1) % core.segment_size;\n\n\t// H01: Target-first eviction priority. If probing is full and target has residents,\n\t// evict the LRU item from target before inserting into probing. This protects\n\t// the hot probing tier and shifts eviction burden to the intermediate target tier.\n\tif core.buckets[probing_idx].is_full() && core.buckets[target_idx].size() > 0 {\n\t\tif let Some(target_eviction_idx) = core.buckets[target_idx].eviction_candidate_index() {\n\t\t\tdebug!(\"promote_to_probing: target={target_idx} has residents and probing is full, evicting LRU from target\");\n\t\t\tlet mut evicted_from_target = core.buckets[target_idx].remove_at(target_eviction_idx);\n\t\t\tevicted_from_target.promotion_age = 1;\n\t\t\tdebug!(\n\t\t\t\t\"promote_to_probing: key={:?} evicted from target={target_idx} -> stash (MRU, promotion_age=1)\",\n\t\t\t\tevicted_from_target.key\n\t\t\t);\n\t\t\tcore.put_at(Slot::Stash, InsertPosition::Back, evicted_from_target);\n\t\t}\n\t}\n\n\tlet Some(evicted) = core.put_at(Slot::Probing, insert_pos, item) else {\n\t\treturn;\n\t};\n\tdemote_from_normal_bucket(core, evicted, probing_idx, insert_pos);\n}\n\n/// One-more-chance demotion with occupancy-aware destination selection.\n///\n/// If the item was a foreign resident at `source_idx` (placed there as someone else's\n/// probing slot), apply occupancy-aware demotion:\n/// - If target tier occupancy < 70%, retry in its own true target before stash\n/// - If target tier occupancy >= 70%, bypass target and go straight to stash\n///\n/// If it was already in its target, go straight to stash. Cascade evictions land in stash.\n///\n/// Sets promotion_age = 1 for demoted items to preserve their re-entry opportunity,\n/// differentiating them from newly promoted items (promotion_age = 0).\nfn demote_from_normal_bucket<K, V>(\n\tcore: &mut SegmentCore<K, V>,\n\tmut item: Item<K, V>,\n\tsource_idx: usize,\n\tinsert_pos: InsertPosition,\n) where\n\tK: Hash + Eq + Copy + Debug + 'static,\n\tV: Eq + Copy + Debug + 'static,\n{\n\titem.promotion_age = 1;\n\tlet true_target = get_index(&item.key, core.segment_size);\n\tif true_target == source_idx {\n\t\t// Demoting from target to stash: use Back insertion to keep item MRU\n\t\tdebug!(\n\t\t\t\"demote_from_normal_bucket: key={:?} target={source_idx} -> stash (MRU, promotion_age=1)\",\n\t\t\titem.key\n\t\t);\n\t\tcore.put_at(Slot::Stash, InsertPosition::Back, item);\n\t} else {\n\t\t// Demoting from probing to target: check target tier occupancy\n\t\tlet target_size = core.buckets[true_target].size() as f64;\n\t\tlet target_capacity = core.buckets[true_target].get_max_size() as f64;\n\t\tlet target_occupancy = if target_capacity > 0.0 {\n\t\t\ttarget_size / target_capacity\n\t\t} else {\n\t\t\t0.0\n\t\t};\n\n\t\tif target_occupancy < 0.70 {\n\t\t\t// Target not saturated: retry in target (existing behavior with Back insertion)\n\t\t\tdebug!(\n\t\t\t\t\"demote_from_normal_bucket: key={:?} probing={source_idx} -> target={true_target} (occupancy={:.2}%, MRU, promotion_age=1)\",\n\t\t\t\titem.key, target_occupancy * 100.0\n\t\t\t);\n\t\t\tif let Some(mut cascade) = core.put_at(Slot::Target, InsertPosition::Back, item) {\n\t\t\t\t// Cascade to stash: also use Back insertion and preserve promotion_age = 1\n\t\t\t\tdebug!(\n\t\t\t\t\t\"demote_from_normal_bucket: cascade key={:?} target={true_target} -> stash (MRU, promotion_age=1)\",\n\t\t\t\t\tcascade.key\n\t\t\t\t);\n\t\t\t\tcore.put_at(Slot::Stash, InsertPosition::Back, cascade);\n\t\t\t}\n\t\t} else {\n\t\t\t// Target saturated (>= 70%): bypass target and go directly to stash\n\t\t\tdebug!(\n\t\t\t\t\"demote_from_normal_bucket: key={:?} probing={source_idx} -> stash (bypassing target, occupancy={:.2}%, MRU, promotion_age=1)\",\n\t\t\t\titem.key, target_occupancy * 100.0\n\t\t\t);\n\t\t\tcore.put_at(Slot::Stash, InsertPosition::Back, item);\n\t\t}\n\t}\n}\n\n/// `insert_pos = Front`: promotions land at LRU end; the item must earn MRU before promoting again.\n/// `insert_pos = Back`: promotions land at MRU; one more access promotes to the next tier.\n///\n/// **H02 change**: In target→probing promotion, check promotion_age. Items with\n/// promotion_age < 1 (newly promoted to target) are ineligible for further promotion;\n/// treat them as intra-target updates instead.\nfn promotion_get<'a, K, V>(\n\tcore: &'a mut SegmentCore<K, V>,\n\tkey: &K,\n\tinsert_pos: InsertPosition,\n) -> Option<&'a Item<K, V>>\nwhere\n\tK: Hash + Eq + Copy + Debug + 'static,\n\tV: Eq + Copy + Debug + 'static,\n{\n\tlet stash_idx = get_index(key, core.stash_size);\n\tlet target_idx = get_index(key, core.segment_size);\n\tlet probing_idx = (target_idx + 1) % core.segment_size;\n\n\tif let Some(item_idx) = core.stash_buckets[stash_idx].get_position(key) {\n\t\tdebug!(\"promotion_get: key={key:?} found in stash -> target={target_idx} insert_pos={insert_pos:?}\");\n\t\tlet data = core.stash_buckets[stash_idx].remove_at(item_idx);\n\t\tpromote_to_target(core, data, insert_pos);\n\n\t\tlet Some(item_new_idx) = core.buckets[target_idx].get_position(key) else {\n\t\t\tunreachable!(\"promotion_get: key={key:?} not found after insert into target={target_idx} -- promote_to_target must leave the key resident in target or cascade it elsewhere via demote_from_normal_bucket\")\n\t\t};\n\t\treturn Some(&core.buckets[target_idx].get_items()[item_new_idx]);\n\t}\n\n\tif let Some(item_idx) = core.buckets[target_idx].get_position(key) {\n\t\tlet is_hottest = Some(item_idx) == core.buckets[target_idx].promotion_candidate_index();\n\t\tif is_hottest {\n\t\t\tlet item = &core.buckets[target_idx].get_items()[item_idx];\n\t\t\tlet promotion_age = item.promotion_age;\n\n\t\t\t// **H02 eligibility check**: Skip promotion if item was promoted fewer than 1 access cycle ago.\n\t\t\t// Increment promotion_age to track access cycles at this tier.\n\t\t\tif promotion_age < 1 {\n\t\t\t\tdebug!(\"promotion_get: key={key:?} hottest in target but ineligible (promotion_age={promotion_age} < 1) -- intra-promote\");\n\t\t\t\t// Increment promotion_age to mark this access cycle\n\t\t\t\tcore.buckets[target_idx].get_items_mut()[item_idx].promotion_age += 1;\n\t\t\t\treturn Some(core.buckets[target_idx].get_from_position(item_idx));\n\t\t\t}\n\n\t\t\tdebug!(\"promotion_get: key={key:?} found in target as hottest (promotion_age={promotion_age}) -> probing={probing_idx}\");\n\t\t\tlet data = core.buckets[target_idx].remove_at(item_idx);\n\t\t\t// Use Back insertion for target-tier hottest items, not the parameter-based insert_pos\n\t\t\tpromote_to_probing(core, data, InsertPosition::Back);\n\n\t\t\tlet Some(item_new_idx) = core.buckets[probing_idx].get_position(key) else {\n\t\t\t\tunreachable!(\"promotion_get: key={key:?} not found after insert into probing={probing_idx} -- promote_to_probing must leave the key resident in probing\")\n\t\t\t};\n\t\t\treturn Some(&core.buckets[probing_idx].get_items()[item_new_idx]);\n\t\t} else {\n\t\t\tdebug!(\"promotion_get: key={key:?} found in target item_idx={item_idx} (intra-promote)\");\n\t\t\t// Increment promotion_age for non-hottest items too (they're still accessing the tier)\n\t\t\tcore.buckets[target_idx].get_items_mut()[item_idx].promotion_age += 1;\n\t\t\treturn Some(core.buckets[target_idx].get_from_position(item_idx));\n\t\t}\n\t}\n\n\tif let Some(item_idx) = core.buckets[probing_idx].get_position(key) {\n\t\tdebug!(\"promotion_get: key={key:?} found in probing bucket={probing_idx}\");\n\t\t// Increment promotion_age for probing hits (accessing the tier)\n\t\tcore.buckets[probing_idx].get_items_mut()[item_idx].promotion_age += 1;\n\t\treturn Some(core.buckets[probing_idx].get_from_position(item_idx));\n\t}\n\n\tdebug!(\"promotion_get: key={key:?} miss\");\n\tNone\n}\n\n#[cfg(test)]\nmod tests {\n\tuse super::*;\n\tuse crate::shared::eviction_policy::EvictionPolicyKind;\n\tuse crate::test_utils::{find_keys_for_bucket, small_dash_settings};\n\n\tfn new_segment(policy: EvictionPolicyKind) -> DashSegment<i64, i64> {\n\t\tDashSegment::new(&small_dash_settings(policy))\n\t}\n\n\t#[test]\n\tfn put_goes_to_stash() {\n\t\tlet mut seg = new_segment(EvictionPolicyKind::ClassicLRU);\n\t\tseg.put(Item::new(1_i64, 10_i64));\n\t\tlet stash_idx = get_index(&1_i64, seg.core().stash_size);\n\t\tassert!(seg.core().stash_buckets[stash_idx].get_position(&1).is_some());\n\t}\n\n\t#[test]\n\tfn stash_hit_inserts_at_front_of_target() {\n\t\tlet mut seg = new_segment(EvictionPolicyKind::ClassicLRU);\n\t\tlet target_idx = 0;\n\t\tlet keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 3);\n\n\t\tseg.put(Item::new(keys[0], 10));\n\t\tseg.get(&keys[0]);\n\t\tseg.put(Item::new(keys[1], 20));\n\t\tseg.get(&keys[1]);\n\n\t\t// Cold insert pushes keys[1] to Front, displacing keys[0] backward.\n\t\tlet items = seg.core.buckets[target_idx].get_items();\n\t\tassert_eq!(items[0].key, keys[1]);\n\t\tassert_eq!(items[1].key, keys[0]);\n\t}\n\n\t#[test]\n\tfn target_hit_not_hottest_promotes_within_bucket() {\n\t\tlet mut seg = new_segment(EvictionPolicyKind::ClassicLRU);\n\t\tlet target_idx = 0;\n\t\tlet keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 2);\n\n\t\tseg.put(Item::new(keys[0], 10));\n\t\tseg.get(&keys[0]);\n\t\tseg.put(Item::new(keys[1], 20));\n\t\tseg.get(&keys[1]);\n\n\t\t// keys[1] starts at Front (cold). Hitting it should bump it Back via ClassicLRU,\n\t\t// not promote it to probing.\n\t\tseg.get(&keys[1]);\n\t\tlet items = seg.core.buckets[target_idx].get_items();\n\t\tassert_eq!(items[0].key, keys[0]);\n\t\tassert_eq!(items[1].key, keys[1]);\n\t}\n\n\t#[test]\n\tfn target_hit_hottest_blocked_by_promotion_age() {\n\t\tlet mut seg = new_segment(EvictionPolicyKind::ClassicLRU);\n\t\tlet target_idx = 0;\n\t\tlet probing_idx = (target_idx + 1) % seg.core.segment_size;\n\t\tlet keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 2);\n\n\t\tseg.put(Item::new(keys[0], 10));\n\t\tseg.get(&keys[0]);\n\t\tseg.put(Item::new(keys[1], 20));\n\t\tseg.get(&keys[1]);\n\n\t\t// First hit moves keys[1] to MRU (promotion_age=0 from stash→target).\n\t\t// Second hit: keys[1] is hottest but has promotion_age=0, so should NOT promote to probing.\n\t\tseg.get(&keys[1]);\n\t\tseg.get(&keys[1]);\n\n\t\t// keys[1] should still be in target, not promoted to probing\n\t\tassert!(seg.core.buckets[target_idx].get_position(&keys[1]).is_some());\n\t\tassert!(seg.core.buckets[probing_idx].get_position(&keys[1]).is_none());\n\t}\n\n\t#[test]\n\tfn target_hit_hottest_promotes_after_eligibility() {\n\t\tlet mut seg = new_segment(EvictionPolicyKind::ClassicLRU);\n\t\tlet target_idx = 0;\n\t\tlet probing_idx = (target_idx + 1) % seg.core.segment_size;\n\t\tlet keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 2);\n\n\t\tseg.put(Item::new(keys[0], 10));\n\t\tseg.get(&keys[0]);\n\t\tseg.put(Item::new(keys[1], 20));\n\t\tseg.get(&keys[1]);\n\n\t\t// First hit: keys[1] promoted to target with promotion_age=0.\n\t\t// As hottest, blocked by promotion_age < 1, but incremented to 1.\n\t\tseg.get(&keys[1]);\n\n\t\t// Second hit: keys[1] now has promotion_age >= 1, eligible for promotion to probing.\n\t\tseg.get(&keys[1]);\n\n\t\tassert!(seg.core.buckets[probing_idx].get_position(&keys[1]).is_some());\n\t\tassert!(seg.core.buckets[target_idx].get_position(&keys[1]).is_none());\n\t}\n\n\t#[test]\n\tfn probing_hit_touches_in_place() {\n\t\tlet mut seg = new_segment(EvictionPolicyKind::ClassicLRU);\n\t\tlet target_idx = 0;\n\t\tlet probing_idx = (target_idx + 1) % seg.core.segment_size;\n\t\tlet keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 1);\n\n\t\tseg.core.buckets[probing_idx].put(Item::new(keys[0], 10));\n\n\t\tlet val = seg.get(&keys[0]).unwrap().value;\n\t\tassert_eq!(val, 10);\n\t\tassert!(seg.core.buckets[probing_idx].get_position(&keys[0]).is_some());\n\t}\n\n\t#[test]\n\tfn probing_eviction_demotes_to_target() {\n\t\tlet mut seg = new_segment(EvictionPolicyKind::ClassicLRU);\n\t\tlet target_idx = 0;\n\t\tlet probing_idx = (target_idx + 1) % seg.core.segment_size;\n\t\tlet target_keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 1);\n\t\tlet probing_keys = find_keys_for_bucket(probing_idx, seg.core.segment_size, 2);\n\n\t\tfor &k in &probing_keys {\n\t\t\tseg.core.buckets[probing_idx].put(Item::new(k, k * 10));\n\t\t}\n\t\tassert!(seg.core.buckets[probing_idx].is_full());\n\n\t\t// Sole resident of target → already at MRU, so the next get triggers promotion.\n\t\tseg.core.buckets[target_idx].put(Item::new(target_keys[0], 100));\n\t\tseg.get(&target_keys[0]);\n\n\t\tassert!(seg.core.buckets[probing_idx].get_position(&target_keys[0]).is_some());\n\t}\n\n\t#[test]\n\tfn cascade_demotion_probing_to_full_target_to_stash() {\n\t\tlet mut seg = new_segment(EvictionPolicyKind::ClassicLRU);\n\t\tlet target_idx = 0;\n\t\tlet probing_idx = (target_idx + 1) % seg.core.segment_size;\n\t\tlet target_keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 3);\n\t\tlet probing_keys = find_keys_for_bucket(probing_idx, seg.core.segment_size, 2);\n\n\t\tseg.put(Item::new(target_keys[0], 10));\n\t\tseg.get(&target_keys[0]);\n\t\tseg.put(Item::new(target_keys[1], 20));\n\t\tseg.get(&target_keys[1]);\n\t\tassert!(seg.core.buckets[target_idx].is_full());\n\n\t\tfor &k in &probing_keys {\n\t\t\tseg.core.buckets[probing_idx].put(Item::new(k, k * 10));\n\t\t}\n\t\tassert!(seg.core.buckets[probing_idx].is_full());\n\n\t\t// Two gets on target_keys[0]: first hits as hottest but has promotion_age < 1\n\t\t// (blocked by H02 eligibility gate, incremented to 1); second hit now has\n\t\t// promotion_age >= 1 and promotes to probing, causing eviction and cascade.\n\t\tseg.get(&target_keys[0]);\n\t\tseg.get(&target_keys[0]);\n\n\t\t// The evicted item's target is `probing_idx`, so the cascade routes it to stash\n\t\t// instead of bouncing it back into the bucket it just left.\n\t\tlet evicted_key = probing_keys[0];\n\t\tlet stash_idx = get_index(&evicted_key, seg.core.stash_size);\n\t\t// Permissive assertion: the key may have re-landed elsewhere via subsequent\n\t\t// stash overflow handling — only \"vanished entirely\" would be a real bug.\n\t\tassert!(\n\t\t\tseg.core.stash_buckets[stash_idx].get_position(&evicted_key).is_some()\n\t\t\t\t|| seg.core.buckets[probing_idx].get_position(&evicted_key).is_some()\n\t\t\t\t|| seg.core.buckets[target_idx].get_position(&evicted_key).is_some(),\n\t\t\t\"evicted probing item should be demoted to stash or still in cache\"\n\t\t);\n\t}\n\n\t#[test]\n\tfn miss_returns_none() {\n\t\tlet mut seg = new_segment(EvictionPolicyKind::ClassicLRU);\n\t\tassert!(seg.get(&999).is_none());\n\t}\n\n\t/// Regression for the last-bucket probing wraparound (commit 4b0b763): keys whose target\n\t/// is `segment_size - 1` must probe at bucket 0, not at `segment_size` (out of bounds).\n\t#[test]\n\tfn last_bucket_probing_wraps_to_zero() {\n\t\tlet mut seg = new_segment(EvictionPolicyKind::ClassicLRU);\n\t\tlet seg_size = seg.core.segment_size;\n\t\tlet target_idx = seg_size - 1;\n\t\tlet probing_idx = (target_idx + 1) % seg_size;\n\t\tassert_eq!(probing_idx, 0, \"sanity: wraparound math holds\");\n\t\tlet keys = find_keys_for_bucket(target_idx, seg_size, 2);\n\n\t\tseg.put(Item::new(keys[0], 10));\n\t\tseg.get(&keys[0]);\n\t\tseg.put(Item::new(keys[1], 20));\n\t\tseg.get(&keys[1]);\n\n\t\t// Two-get promote, but blocked by promotion_age < 1.\n\t\tseg.get(&keys[1]);\n\t\tseg.get(&keys[1]);\n\n\t\t// Should still be in wrapped target bucket, not probing bucket 0\n\t\tassert!(\n\t\t\tseg.core.buckets[target_idx].get_position(&keys[1]).is_some(),\n\t\t\t\"item blocked by promotion_age should remain in target bucket\"\n\t\t);\n\t}\n\n\t/// An item evicted from its probing bucket must cascade to its true target\n\t/// (one-more-chance), not jump straight to stash.\n\t#[test]\n\tfn evicted_from_probing_cascades_to_target() {\n\t\tlet settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);\n\t\tlet mut seg = DashSegment::new(&settings);\n\t\tlet seg_size = settings.num_of_normal_buckets;\n\t\tlet target_idx = 0;\n\t\tlet probing_idx = (target_idx + 1) % seg_size;\n\t\tlet target_keys = find_keys_for_bucket(target_idx, seg_size, 2);\n\t\tlet k = target_keys[0];\n\t\tlet trigger = target_keys[1];\n\t\tlet filler = find_keys_for_bucket(probing_idx, seg_size, 1)[0];\n\n\t\t// `k` placed first so it is the LRU victim when bucket 1 next evicts.\n\t\tseg.core_mut().buckets[probing_idx].put(Item::new(k, 99));\n\t\tseg.core_mut().buckets[probing_idx].put(Item::new(filler, 50));\n\t\tassert!(seg.core().buckets[probing_idx].is_full());\n\t\tassert!(seg.core().buckets[target_idx].get_position(&k).is_none());\n\n\t\tpromote_to_probing(seg.core_mut(), Item::new(trigger, 200), InsertPosition::Back);\n\n\t\tlet in_target = seg.core().buckets[target_idx].get_position(&k).is_some();\n\t\tlet in_stash = seg.core().stash_buckets.iter().any(|b| b.get_position(&k).is_some());\n\t\tlet still_in_probing = seg.core().buckets[probing_idx].get_position(&k).is_some();\n\n\t\tassert!(!still_in_probing, \"sanity: k was evicted from its probing bucket\");\n\t\tassert!(\n\t\t\tin_target,\n\t\t\t\"promotion demotion from probing must place item into its target bucket\"\n\t\t);\n\t\tassert!(!in_stash, \"promotion demotion must not bypass the target tier\");\n\n\t\t// Verify demoted item has promotion_age = 1\n\t\tif let Some(pos) = seg.core().buckets[target_idx].get_position(&k) {\n\t\t\tassert_eq!(seg.core().buckets[target_idx].get_items()[pos].promotion_age, 1, \"demoted item should have promotion_age=1\");\n\t\t}\n\t}\n\n\t/// Foreign resident demoted via one-more-chance always lands at Back (MRU) position\n\t/// with H02 change, and has promotion_age = 1 to allow rapid re-entry.\n\tfn target_retry_position(insert_pos: InsertPosition) {\n\t\tlet settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);\n\t\tlet mut seg = DashSegment::new(&settings);\n\t\tlet seg_size = settings.num_of_normal_buckets;\n\t\tlet target_idx = 0;\n\t\tlet probing_idx = (target_idx + 1) % seg_size;\n\t\tlet target_keys = find_keys_for_bucket(target_idx, seg_size, 3);\n\t\tlet pre = target_keys[0];\n\t\tlet foreign = target_keys[1];\n\t\tlet trigger = target_keys[2];\n\t\tlet filler = find_keys_for_bucket(probing_idx, seg_size, 1)[0];\n\n\t\tseg.core_mut().buckets[target_idx].put(Item::new(pre, 1));\n\t\tseg.core_mut().buckets[probing_idx].put(Item::new(foreign, 2));\n\t\tseg.core_mut().buckets[probing_idx].put(Item::new(filler, 3));\n\n\t\tpromote_to_probing(seg.core_mut(), Item::new(trigger, 99), insert_pos);\n\n\t\tlet items = seg.core().buckets[target_idx].get_items();\n\t\tlet pos = items\n\t\t\t.iter()\n\t\t\t.position(|it| it.key == foreign)\n\t\t\t.expect(\"foreign should be in true target after retry\");\n\t\t// H02: demoted items always use Back insertion and have promotion_age = 1\n\t\tassert_eq!(pos, items.len() - 1, \"foreign should be at Back (MRU) position after demotion\");\n\t\tassert_eq!(items[pos].promotion_age, 1, \"demoted item should have promotion_age=1\");\n\t}\n\n\t#[test]\n\tfn target_retry_lands_at_back_from_front_promotion() {\n\t\ttarget_retry_position(InsertPosition::Front);\n\t}\n\n\t#[test]\n\tfn target_retry_lands_at_back_from_back_promotion() {\n\t\ttarget_retry_position(InsertPosition::Back);\n\t}\n\n\t/// When an evicted item's true target equals its source bucket, demote_from_normal_bucket\n\t/// routes it directly to stash (no retry). Verifies demotion sets promotion_age = 1.\n\tfn direct_to_stash_position(insert_pos: InsertPosition) {\n\t\tlet settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);\n\t\tlet mut seg = DashSegment::new(&settings);\n\t\tlet seg_size = settings.num_of_normal_buckets;\n\t\tlet stash_size = settings.num_of_stash_buckets;\n\t\tlet target_idx = 0;\n\t\tlet target_keys = find_keys_for_bucket(target_idx, seg_size, 3);\n\t\tlet victim = target_keys[0];\n\t\tlet resident = target_keys[1];\n\t\tlet trigger = target_keys[2];\n\t\tlet stash_idx = crate::shared::utils::get_index(&victim, stash_size);\n\t\tlet stash_pre = find_keys_for_bucket(stash_idx, stash_size, 1)[0];\n\n\t\tseg.core_mut().stash_buckets[stash_idx].put(Item::new(stash_pre, 7));\n\t\tseg.core_mut().buckets[target_idx].put(Item::new(victim, 1));\n\t\tseg.core_mut().buckets[target_idx].put(Item::new(resident, 2));\n\n\t\tpromote_to_target(seg.core_mut(), Item::new(trigger, 99), insert_pos);\n\n\t\tlet stash_items = seg.core().stash_buckets[stash_idx].get_items();\n\t\tlet pos = stash_items\n\t\t\t.iter()\n\t\t\t.position(|it| it.key == victim)\n\t\t\t.expect(\"victim should land directly in stash\");\n\t\t// H02 change: demoted items use Back insertion and have promotion_age = 1\n\t\tassert_eq!(pos, stash_items.len() - 1, \"victim should be inserted at Back (MRU) position\");\n\t\tassert_eq!(stash_items[pos].promotion_age, 1, \"demoted item should have promotion_age=1\");\n\t}\n\n\t#[test]\n\tfn direct_to_stash_at_back_from_front_promotion() {\n\t\tdirect_to_stash_position(InsertPosition::Front);\n\t}\n\n\t#[test]\n\tfn direct_to_stash_at_back_from_back_promotion() {\n\t\tdirect_to_stash_position(InsertPosition::Back);\n\t}\n\n\t/// Probing eviction → foreign-resident retry into full target → second eviction\n\t/// cascades into stash. Verifies cascade demotion sets promotion_age = 1.\n\tfn cascade_to_stash_position(insert_pos: InsertPosition) {\n\t\tlet settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);\n\t\tlet mut seg = DashSegment::new(&settings);\n\t\tlet seg_size = settings.num_of_normal_buckets;\n\t\tlet stash_size = settings.num_of_stash_buckets;\n\t\tlet target_idx = 0;\n\t\tlet probing_idx = (target_idx + 1) % seg_size;\n\t\tlet target_keys = find_keys_for_bucket(target_idx, seg_size, 4);\n\t\tlet target_filler_a = target_keys[0];\n\t\tlet target_filler_b = target_keys[1];\n\t\tlet foreign = target_keys[2]; // foreign at probing; true_target = target_idx\n\t\tlet trigger = target_keys[3];\n\t\tlet probing_filler = find_keys_for_bucket(probing_idx, seg_size, 1)[0];\n\n\t\t// `target_filler_a` is the LRU of target — evicted on retry, cascades to stash.\n\t\tlet cascaded = target_filler_a;\n\t\tlet stash_idx = crate::shared::utils::get_index(&cascaded, stash_size);\n\t\tlet stash_pre = find_keys_for_bucket(stash_idx, stash_size, 4)\n\t\t\t.into_iter()\n\t\t\t.find(|k| *k != cascaded)\n\t\t\t.expect(\"a distinct stash_pre must exist\");\n\n\t\tseg.core_mut().stash_buckets[stash_idx].put(Item::new(stash_pre, 7));\n\t\tseg.core_mut().buckets[target_idx].put(Item::new(target_filler_a, 1));\n\t\tseg.core_mut().buckets[target_idx].put(Item::new(target_filler_b, 2));\n\t\tseg.core_mut().buckets[probing_idx].put(Item::new(foreign, 3));\n\t\tseg.core_mut().buckets[probing_idx].put(Item::new(probing_filler, 4));\n\n\t\tpromote_to_probing(seg.core_mut(), Item::new(trigger, 99), insert_pos);\n\n\t\tlet stash_items = seg.core().stash_buckets[stash_idx].get_items();\n\t\tlet pos = stash_items\n\t\t\t.iter()\n\t\t\t.position(|it| it.key == cascaded)\n\t\t\t.expect(\"cascaded target item should land in stash via target overflow\");\n\t\t// H02 change: cascade items use Back insertion and have promotion_age = 1\n\t\tassert_eq!(pos, stash_items.len() - 1, \"cascaded item should be inserted at Back (MRU) position\");\n\t\tassert_eq!(stash_items[pos].promotion_age, 1, \"cascaded item should have promotion_age=1\");\n\t}\n\n\t#[test]\n\tfn cascade_to_stash_at_back_from_front_promotion() {\n\t\tcascade_to_stash_position(InsertPosition::Front);\n\t}\n\n\t#[test]\n\tfn cascade_to_stash_at_back_from_back_promotion() {\n\t\tcascade_to_stash_position(InsertPosition::Back);\n\t}\n}\n"
}
```

### 19:29:59 thinking

Good! Now I need to also create the modified item.rs file that includes the `promotion_age` field. The subject's baseline doesn't have this field, so I need to add it.

### 19:29:59 chat

Now let me create the modified `item.rs` file that adds the `promotion_age` field:

### 19:30:05 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0008/variants/v001/attempts/01/overlay/src/shared/item.rs",
  "content": "//! This module defines a key-value data object stored in the Dash cache.\n//!\n//! # Examples\n//!\n//! ```ignore\n//! use dash::shared::item::Item;\n//!\n//! let item = Item::new(1, 2);\n//! assert_eq!(item.key, 1);\n//! assert_eq!(item.value, 2);\n//! assert_eq!(item.lfu_counter, 0);\n//! ```\n\nuse std::{\n\tfmt::{Display, Formatter},\n\thash::Hash,\n\ttime::Instant,\n};\n\n#[derive(Debug, Clone)]\npub struct Item<K, V>\nwhere\n\tK: Hash + Eq + Clone,\n\tV: Eq + Clone,\n{\n\tpub key: K,\n\tpub value: V,\n\tpub lfu_counter: usize,\n\tpub timestamp: Instant,\n\t/// Tracks how many access cycles since an item was promoted to its current tier.\n\t/// Used to gate further promotions: items with promotion_age < 1 are ineligible\n\t/// for promotion to higher tiers, forcing them to prove fitness at their new tier first.\n\t/// Demoted items have promotion_age = 1, allowing rapid re-entry.\n\tpub promotion_age: usize,\n}\n\nimpl<K, V> Item<K, V>\nwhere\n\tK: Hash + Eq + Clone,\n\tV: Eq + Clone,\n{\n\tpub fn new(key: K, value: V) -> Self {\n\t\tSelf {\n\t\t\tkey,\n\t\t\tvalue,\n\t\t\tlfu_counter: 0,\n\t\t\ttimestamp: Instant::now(),\n\t\t\tpromotion_age: 0,\n\t\t}\n\t}\n}\n\nimpl<K, V> PartialEq for Item<K, V>\nwhere\n\tK: Hash + Eq + Clone,\n\tV: Eq + Clone,\n{\n\tfn eq(&self, other: &Item<K, V>) -> bool {\n\t\tself.key == other.key && self.value == other.value\n\t}\n}\n\nimpl<K, V> Eq for Item<K, V>\nwhere\n\tK: Hash + Eq + Clone,\n\tV: Eq + Clone,\n{\n}\n\nimpl<K, V> Display for Item<K, V>\nwhere\n\tK: Hash + Eq + Clone + Display,\n\tV: Eq + Clone + Display,\n{\n\tfn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {\n\t\twrite!(\n\t\t\tf,\n\t\t\t\"Bucket Item {{ key: {}, value: {}, lfu_counter: {}, timestamp: {:?}, promotion_age: {} }}\",\n\t\t\tself.key, self.value, self.lfu_counter, self.timestamp, self.promotion_age\n\t\t)\n\t}\n}\n\n#[cfg(test)]\nmod tests {\n\tuse super::*;\n\n\t#[test]\n\tfn new_initializes_fields() {\n\t\tlet item = Item::new(42, \"hello\");\n\t\tassert_eq!(item.key, 42);\n\t\tassert_eq!(item.value, \"hello\");\n\t\tassert_eq!(item.lfu_counter, 0);\n\t\tassert_eq!(item.promotion_age, 0);\n\t}\n\n\t#[test]\n\tfn equality_compares_key_and_value() {\n\t\tlet a = Item::new(1, 10);\n\t\tlet b = Item::new(1, 10);\n\t\tlet c = Item::new(1, 20);\n\t\tlet d = Item::new(2, 10);\n\n\t\tassert_eq!(a, b);\n\t\tassert_ne!(a, c); // different value\n\t\tassert_ne!(a, d); // different key\n\t}\n\n\t#[test]\n\tfn equality_ignores_lfu_counter() {\n\t\tlet mut a = Item::new(1, 10);\n\t\tlet b = Item::new(1, 10);\n\t\ta.lfu_counter = 99;\n\t\tassert_eq!(a, b);\n\t}\n\n\t#[test]\n\tfn clone_produces_independent_copy() {\n\t\tlet mut a = Item::new(1, 10);\n\t\tlet b = a.clone();\n\t\ta.value = 20;\n\t\tassert_eq!(a.value, 20);\n\t\tassert_eq!(b.value, 10);\n\t}\n}\n"
}
```

### 19:30:15 thinking

Perfect! Now I've created the overlay with:
1. The modified `dash_segment.rs` that includes:
   - The `promotion_age` field from the parent variant
   - The H01 target-first eviction priority in `promote_to_probing()`
   - The H02 eligibility delay gate in `promotion_get()`
   - The NEW occupancy-aware demotion logic in `demote_from_normal_bucket()` that:
     - Checks target bucket occupancy
     - If < 70%, demotes to target (existing behavior)
     - If >= 70%, demotes directly to stash instead

2. The modified `item.rs` that adds the `promotion_age` field

Let me verify the files are in place and ready. The next step would be to run the tests to ensure the implementation compiles and passes the test gates. But I've completed my part as the variant implementer.

Let me double-check the key modification one more time - the occupancy-aware demotion logic in `demote_from_normal_bucket()`:

```rust
} else {
    // Demoting from probing to target: check target tier occupancy
    let target_size = core.buckets[true_target].size() as f64;
    let target_capacity = core.buckets[true_target].get_max_size() as f64;
    let target_occupancy = if target_capacity > 0.0 {
        target_size / target_capacity
    } else {
        0.0
    };

    if target_occupancy < 0.70 {
        // Target not saturated: retry in target (existing behavior with Back insertion)
        debug!(
            "demote_from_normal_bucket: key={:?} probing={source_idx} -> target={true_target} (occupancy={:.2}%, MRU, promotion_age=1)",
            item.key, target_occupancy * 100.0
        );
        if let Some(mut cascade) = core.put_at(Slot::Target, InsertPosition::Back, item) {
            // Cascade to stash: also use Back insertion and preserve promotion_age = 1
            debug!(
                "demote_from_normal_bucket: cascade key={:?} target={true_target} -> stash (MRU, promotion_age=1)",
                cascade.key
            );
            core.put_at(Slot::Stash, InsertPosition::Back, cascade);
        }
    } else {
        // Target saturated: bypass and go straight to stash with Back insertion
        debug!(
            "demote_from_normal_bucket: key={:?} probing={source_idx} -> stash (occupancy={:.2}%, saturated, promotion_age=1)",
            item.key, target_occupancy * 100.0
        );
        core.put_at(Slot::Stash, InsertPosition::Back, item);
    }
}
```

The logic handles both paths: when target occupancy is below the 70% threshold, items flow through the target bucket with cascading to stash if needed; when occupancy exceeds 70%, items bypass the target tier entirely and go directly to stash. Both paths preserve the promotion_age field and use Back insertion for MRU ordering.

### 19:30:19 chat

Perfect! I've successfully created the overlay implementation for variant v001 in cycle 0008. Let me summarize what was implemented:

## Summary

### Files Created

1. **`overlay/src/dash/segments/dash_segment.rs`** - Modified segment implementation with:
   - **Inherited from parent (cycle 0007 v001)**:
     - `promotion_age` field tracking (from H02)
     - Target-first eviction priority in `promote_to_probing()` (from H01 of previous cycle)
     - Eligibility delay gate blocking promotions when `promotion_age < 1`
     - Back insertion strategy for demoted items
   
   - **New occupancy-aware demotion logic** (current cycle H01):
     - Modified `demote_from_normal_bucket()` to check target bucket occupancy
     - If target occupancy **< 70%**: demote to target with back insertion (preserves existing beneficial behavior)
     - If target occupancy **≥ 70%**: bypass congested target and demote directly to stash with back insertion
     - Reduces cascade depth during high-load periods when target is saturated

2. **`overlay/src/shared/item.rs`** - Item structure extended with:
   - `promotion_age: usize` field to track access cycles since promotion
   - Initialized to 0 for new items
   - Set to 1 for demoted items to allow rapid re-entry
   - Incremented on each access to the tier

### Key Implementation Details

The occupancy check calculates:
```rust
target_occupancy = current_size / max_capacity
```

This metric guides the routing decision:
- **Below 70%**: Target tier has capacity → use one-more-chance demotion path (item gets a retry in target before final demotion to stash)
- **At/above 70%**: Target tier is saturated → skip target entirely and demote directly to stash to reduce eviction pressure on the hot probing tier

This occupancy-aware approach coordinates eviction destination based on tier state, testing whether reducing cascade overhead during congestion can improve hit rates without abandoning the demotion-recovery principle that proved stable in prior winners.

The implementation preserves the back-insertion (MRU) strategy for all demoted items and maintains `promotion_age = 1` semantics for rapid re-entry opportunities.


---

- finished: 2026-06-05T19:30:19+00:00
- duration: 120.07s
- status: ok
- input_tokens: 68
- output_tokens: 15393
- cache_read_tokens: 303617
- cache_write_tokens: 42073
- cost_usd: 0.160928
