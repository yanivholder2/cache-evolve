# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0005\hypotheses
- allow_edits: True
- started: 2026-07-20T16:03:48+00:00

## timeline

### 16:04:23 thinking

Let me read all the context files to understand the current state of the research and formulate 3 distinct hypotheses for cycle 0005.

Let me start by reading multiple files in parallel.

### 16:04:27 tool: Agent

```json
{
  "description": "Read cycle 0004 context files",
  "prompt": "Read the following files and return their full contents. I need all of them to write research hypotheses.\n\n1. C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\cycle-winners.json\n2. C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\cycle-insight.md\n3. C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\leaderboard.md\n4. C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cross-cycle-insights.md\n5. C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles-ledger.jsonl\n\nReturn the full text of each file, labeled by filename."
}
```

### 16:04:29 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\cycle-winners.json"
}
```

### 16:04:30 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\cycle-insight.md"
}
```

### 16:04:31 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\leaderboard.md"
}
```

### 16:04:32 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cross-cycle-insights.md"
}
```

### 16:04:32 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles-ledger.jsonl"
}
```

### 16:04:33 tool: Agent

```json
{
  "description": "Read cycle 0004 variant files",
  "prompt": "Read the following files and return their full contents. I need these to write research hypotheses.\n\nFor each variant v001, v002, v003 in C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\variants\\, read:\n- The fitness-vector.json at: C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\variants\\v001\\eval\\fitness-vector.json\n- The insight.md at: C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\variants\\v001\\insight.md\n- The insight.md at: C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\variants\\v002\\eval\\fitness-vector.json\n- The insight.md at: C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\variants\\v002\\insight.md\n- C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\variants\\v003\\eval\\fitness-vector.json\n- C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\variants\\v003\\insight.md\n\nAlso list what's in C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\variants\\ to see which variants exist.\n\nThen for each variant that exists, read the latest attempt's overlay directory by first listing C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\variants\\v001\\attempts\\ and then reading all Rust source files in the latest attempt's overlay.\n\nReturn all content, labeled by filepath."
}
```

### 16:04:36 tool: Bash

```json
{
  "command": "ls \"C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\variants\\\"",
  "description": "List variants in cycle 0004"
}
```

### 16:04:36 tool: Agent

```json
{
  "description": "Read baseline source files",
  "prompt": "Read all Rust source files in the baseline directory at C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\baseline\\ and return their full contents labeled by filepath. Also list the directory structure first.\n\nAlso read the cross-cycle winner variants' source overlay files:\n- List C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\variants\\v002\\attempts\\ and read the latest attempt's overlay Rust files\n- List C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\variants\\v003\\attempts\\ and read the latest attempt's overlay Rust files\n- List C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0002\\variants\\v001\\attempts\\ and read the latest attempt's overlay Rust files\n\nReturn all content labeled by filepath."
}
```

### 16:04:37 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\variants\\v001\\eval\\fitness-vector.json"
}
```

### 16:04:37 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\variants\\v001\\insight.md"
}
```

### 16:04:38 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\variants\\v002\\eval\\fitness-vector.json"
}
```

### 16:04:39 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\variants\\v002\\insight.md"
}
```

### 16:04:40 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\variants\\v003\\eval\\fitness-vector.json"
}
```

### 16:04:40 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\variants\\v003\\insight.md"
}
```

### 16:04:41 tool: Bash

```json
{
  "command": "ls -R \"C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\baseline\\\"",
  "description": "List baseline directory structure recursively"
}
```

### 16:04:42 tool: Bash

```json
{
  "command": "ls \"C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\variants\\v002\\attempts\\\"",
  "description": "List v002 attempts directory"
}
```

### 16:04:43 tool: Bash

```json
{
  "command": "ls \"C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\variants\\v003\\attempts\\\"",
  "description": "List v003 attempts directory"
}
```

### 16:04:43 tool: Bash

```json
{
  "command": "ls \"C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0002\\variants\\v001\\attempts\\\"",
  "description": "List v001 attempts directory"
}
```

### 16:04:44 tool: Glob

```json
{
  "pattern": "*",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\variants\\v001\\attempts"
}
```

### 16:04:45 tool: Glob

```json
{
  "pattern": "*",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\variants\\v002\\attempts"
}
```

### 16:04:45 tool: Glob

```json
{
  "pattern": "*",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\variants\\v003\\attempts"
}
```

### 16:04:46 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\baseline"
}
```

### 16:04:47 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\variants\\v002\\attempts"
}
```

### 16:04:48 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\variants\\v003\\attempts"
}
```

### 16:04:48 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0002\\variants\\v001\\attempts"
}
```

### 16:04:50 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\variants\\v001\\attempts\\01\\overlay\\src\\dash\\segments\\dash_segment.rs"
}
```

### 16:04:51 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\variants\\v002\\attempts\\01\\overlay\\src\\dash\\segments\\dash_segment.rs"
}
```

### 16:04:51 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\variants\\v003\\attempts\\01\\overlay\\src\\dash\\segments\\dash_segment.rs"
}
```

### 16:04:54 tool: Glob

```json
{
  "pattern": "**/*.rs",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash"
}
```

### 16:04:55 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\variants\\v002\\attempts\\01\\overlay\\src\\shared\\item.rs"
}
```

### 16:04:55 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\variants\\v002\\attempts\\01\\overlay\\src\\shared\\eviction_policy.rs"
}
```

### 16:04:56 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\variants\\v002\\attempts\\01\\overlay\\src\\dash\\segments\\dash_segment.rs"
}
```

### 16:04:57 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\variants\\v003\\attempts\\01\\overlay\\src\\shared\\item.rs"
}
```

### 16:04:57 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\variants\\v003\\attempts\\01\\overlay\\src\\shared\\eviction_policy.rs"
}
```

### 16:04:58 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\variants\\v003\\attempts\\01\\overlay\\src\\dash\\segments\\dash_segment.rs"
}
```

### 16:04:59 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0002\\variants\\v001\\attempts\\01\\overlay\\src\\shared\\item.rs"
}
```

### 16:05:00 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0002\\variants\\v001\\attempts\\01\\overlay\\src\\shared\\eviction_policy.rs"
}
```

### 16:05:00 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0002\\variants\\v001\\attempts\\01\\overlay\\src\\dash\\segments\\dash_segment.rs"
}
```

### 16:05:04 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\src\\lib.rs"
}
```

### 16:05:05 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\src\\factory.rs"
}
```

### 16:05:05 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\src\\shared\\item.rs"
}
```

### 16:05:06 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\src\\shared\\eviction_policy.rs"
}
```

### 16:05:06 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\src\\shared\\geometry.rs"
}
```

### 16:05:07 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\src\\shared\\logging.rs"
}
```

### 16:05:08 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\src\\shared\\mod.rs"
}
```

### 16:05:08 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\src\\shared\\utils.rs"
}
```

### 16:05:08 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\src\\shared\\traits\\mod.rs"
}
```

### 16:05:09 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\src\\shared\\traits\\bucket.rs"
}
```

### 16:05:10 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\src\\shared\\traits\\eviction_policy.rs"
}
```

### 16:05:10 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\src\\shared\\traits\\geometry.rs"
}
```

### 16:05:11 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\src\\dash\\mod.rs"
}
```

### 16:05:11 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\src\\dash\\dash_bucket.rs"
}
```

### 16:05:12 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\src\\dash\\dash_settings.rs"
}
```

### 16:05:12 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\src\\dash\\jni.rs"
}
```

### 16:05:13 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\src\\dash\\segments\\mod.rs"
}
```

### 16:05:13 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\src\\dash\\segments\\dash_segment.rs"
}
```

### 16:05:14 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\src\\dash\\segments\\segment.rs"
}
```

### 16:05:15 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\src\\test_utils\\mod.rs"
}
```

### 16:05:15 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\src\\test_utils\\invariants.rs"
}
```

### 16:05:16 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\src\\bin\\evolve_metadata_size.rs"
}
```

### 16:05:16 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\src\\bin\\evolve_per_op_time.rs"
}
```

### 16:05:17 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\tests\\common\\mod.rs"
}
```

### 16:05:18 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\tests\\contract_gate.rs"
}
```

### 16:05:18 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\tests\\dash_integration.rs"
}
```

### 16:05:19 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\tests\\invariant_gate.rs"
}
```

### 16:05:19 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\tests\\segment_external_fixture.rs"
}
```

### 16:05:20 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\tests\\universal_invariant_gate.rs"
}
```

### 16:12:23 thinking

Now I have a comprehensive picture of the state of the research. Let me synthesize everything before writing the hypotheses.

## Key findings to synthesize:

### Stable anchors (from cycle/0003/v003, the best parent):
- `item.rs`: `timestamp` removed (Item has only key, value, lfu_counter) — reduces metadata-size from 40 to 24 bytes
- `eviction_policy.rs`: `TimestampLRU` is now positional (no Instant import)
- `dash_segment.rs`: MRU-end insertion (Back) + vacancy-gated frequency promotion (>= 2 hits, only when probing not full)

### The critical cross-cycle lesson:
**Overlay coverage gaps.** Cycle 4 variants only modified `dash_segment.rs` and silently reverted `item.rs` struct reduction (going back to 40 bytes). Any child of a multi-file parent MUST include ALL three files (item.rs, eviction_policy.rs, dash_segment.rs) in the overlay.

### What's been tried and ruled out:
1. Threshold tuning (>= 3):  harmful, the >= 2 threshold is near-optimal
2. Vacancy-gated stash bypass (joint target-full + probing-vacant): fires too rarely
3. lfu_counter reset on stash promotion: no measurable signal

The next cycle should pivot away from promotion sensitivity and instead explore orthogonal levers like eviction ordering within a tier or how items accessed in the probing tier get promoted. I'm tracking three optimization dimensions: hit-rate (currently ~0.68, top is 0.6796), metadata-size (best at 24 bytes versus the 40-byte baseline), and per-operation time (currently around 100ns).

Looking at the leaderboard, the best performers cluster tightly around 0.6796–0.6787 hit-rate with 24-byte metadata and 100ns latency, with cycle 0003 v002 and v003 both emerging as winners alongside cycle 0002 v001. So cycle/0002/v002 achieved that 0.68 hit-rate with the frequency gate set to >= 2, but it came at the cost of higher metadata overhead (40 bytes) and didn't reduce struct size. Looking at the actual cross-cycle winners, I'm seeing that cycle/0003/v002 with the >= 3 frequency threshold is nearly matching that performance at 0.6796 hit-rate while keeping metadata lean at 24 bytes.

The key difference is that cycle/0003/v002 was built on a cleaner base—the MRU-end with timestamp removal and the full three-file overlay—whereas cycle/0004/v001 tried to apply the same >= 3 threshold but only partially updated the codebase, missing the item.rs reduction that was critical to the earlier wins.

Now I'm thinking through three distinct hypotheses to test. The first direction focuses on changing how items get evicted within a tier—instead of always removing the oldest item when a bucket fills up, I'd prioritize evicting items with low access frequency, keeping the recently or frequently accessed ones around longer. This builds on cycle/0003/v003 as the parent foundation.

The challenge is that the eviction policy trait doesn't directly access the items themselves, so I'd need to either extend the trait to pass item data to the eviction candidate selection, or move the lfu_counter logic into the segment's eviction method where it can actually see the items. So the eviction policy can actually see all items in the bucket when selecting a victim, which means I can modify `ClassicLRU` to pick the item with the lowest frequency counter instead of always returning index 0. This keeps the change localized to `eviction_policy.rs` without needing to touch the promotion logic.

For the second direction, I'm looking at what happens when an item in the probing tier gets accessed — it checks the target tier first, then the probing tier, and if found there it updates the item's position within that tier before...

Actually, I need to trace through the `get` method more carefully to see if probing items ever get promoted to the target tier or if they just stay in probing. If they're not crossing over to target even with repeated hits, that's wasted opportunity. I could add a promotion path where probing items with a frequency counter of 2 or higher get moved to target if there's space available, gating it on actual vacancy so we don't evict unnecessarily.

Now looking at the metadata overhead — the Item struct is currently 24 bytes for a key-value pair of u64 each, which means the lfu_counter is taking up 8 bytes. I'm wondering if I can shrink that field or pack it more efficiently, though without seeing the exact struct layout it's hard to know if there's padding I can exploit.

Another angle: what if I could eliminate the lfu_counter entirely by using position in the list as a frequency proxy? Items near the MRU end would implicitly have higher frequency since we insert fresh items there. But this conflicts with the vacancy-gated promotion logic that depends on the counter, so I'd need to redesign that mechanism.

A more promising direction might be rethinking the eviction policy for the stash tier — the overflow bucket that holds items that don't fit in the main probing structure. Currently it uses the same positional LRU policy, but what if I prioritized stash evictions differently, like removing the item with the lowest frequency counter first?

Actually, that's just a variant of frequency-aware eviction applied to a specific tier. Let me map out three genuinely distinct strategies: one that scans all items in a bucket during eviction and removes the coldest one regardless of position, another focused on promoting items from probing to a target tier based on access patterns, and a third I'm still working through. Dropping items on first access is too aggressive since it'd hurt workloads where items get accessed twice. A decay mechanism where counters gradually decrease over time would help prevent stale items from blocking newer hot ones, but that's complex to implement. Instead, I could adapt the insertion tier based on how full the target tier is — when there's room, new items go straight to target, and once it fills up, fall back to the normal stash insertion flow.

But that risks undermining the whole tier hierarchy, which exists to make items prove themselves before entering target. The real levers to explore are eviction selection within tiers, whether items in probing can be promoted to target, and a different approach to the stash itself — maybe implementing a gate where stash items need to meet a frequency threshold before moving to probing.

Actually, I should check the current code path for stash hits. If they're going directly to target, adding a stash-to-probing intermediate step could filter out one-hit wonders that never get accessed again, reducing false promotions.

Now I'm settling on three concrete hypotheses to test. The first one focuses on frequency-aware eviction within tiers — instead of always evicting the oldest item when a bucket overflows, I'd scan for the item with the lowest frequency counter and evict that instead, keeping higher-frequency items around longer. This should improve hit-rate by being smarter about what gets thrown out when space runs tight.

The second hypothesis targets promotion of frequently-accessed items from the probing tier up to the target tier. When an item in the probing section gets accessed and its frequency counter hits 2 or higher, and there's room in the target bucket, I'd promote it up — similar to how the stash-to-target promotion already works. This addresses the observation that hot items stuck in the probing tier should graduate to premium placement.

For the third hypothesis, I'm reconsidering the direction. Since I need at least one variant citing a cross-cycle winner, and I've covered that with the first two, the third could either return to baseline or revisit something unexplored. The LFU eviction override from cycle 1 timed out before, but maybe it's worth retesting at a smaller scale. Alternatively, I could investigate the `promotion_candidate_index` mechanism in the eviction policy — how probing items currently get selected for promotion.

I'm exploring two distinct directions for H03: one where resetting the `lfu_counter` when items move between tiers forces them to re-earn their access count within that tier, though cycle 4 already tested this on stash-to-target promotion without success. The other direction routes stash hits into probing first rather than directly to target, creating a two-hop promotion path that might better filter items before they reach the target tier.

For the two-hop approach, I'd modify the stash-hit handler to check if probing has space and demote the stash item there instead of promoting directly to target, falling back to target only if probing is full. This is the inverse of a previous attempt that bypassed probing entirely, so I'm curious whether forcing items through probing first will reduce target pollution.

The key insight is that a stash hit landing in probing might then compete fairly and earn target promotion through the frequency gate, rather than immediately polluting target with one-time hits. But I need to clarify the current architecture—specifically how stash hits are handled versus how the vacancy-gated promotion works between tiers. Looking at the tier hierarchy more carefully: target is the hottest tier, probing is middle, and stash is coldest. When a probing item gets promoted via the frequency gate, it moves up to target, which makes sense. Similarly, if a stash item is accessed, it can be promoted into the higher tiers. But I'm still confused about what happens when a target item itself triggers the frequency gate — that description suggests it might move to probing, which would be a demotion and doesn't align with the logic. I need to look at the actual code comments more carefully to understand what's really happening here. Looking at how the `Segment` trait routes items: new entries go to Stash with `InsertPosition::Back`, and when accessed there, they get promoted up the tier hierarchy. The baseline variant uses `InsertPosition::Front` for promotions while the evolved MRU-end variants use `InsertPosition::Back`. In the target tier, the hottest item sits at position N-1, and accessing it just calls `get_from_position`, but non-hottest items trigger the frequency-gated promotion logic I was examining earlier. So the tier ordering seems to be: Target (hottest) → Probing → Stash (coldest). I think the core insight is that "vacancy-gated frequency promotion" means items in lower tiers get promoted to higher tiers when they've been accessed multiple times AND there's actually space available—this prevents cascading evictions. So I'm working with three concrete hypotheses: frequency-aware eviction that targets low-counter items, a promotion path from probing up to target for frequently-accessed items, and exploring a fundamentally different angle from the baseline approach.

The baseline relies on timestamp-based LRU with high overhead and cold-start insertion at the LRU end, with no frequency gating at all. I could try pure frequency-based eviction without the promotion gating, or revisit the LFU eviction override from an earlier cycle—though that one timed out before, so I need to be careful about scale. Let me think about what genuinely differs from baseline without repeating failed approaches.

I'm considering whether to build on cycle 0002/v001 (which already has MRU-end ordering and no timestamp) and layer frequency-aware eviction on top, but that would be too similar to H01, which does the same thing from a different parent. And removing the timestamp to get MRU-end ordering for free is just reproducing what's already been done in earlier cycles. I need something truly different.

I'm exploring whether I could implement an adaptive LFU counter decay mechanism where accessing an item triggers probabilistic decay of other counters in the bucket, creating a rough LRFU policy, though that might be too complex. Alternatively, I could adjust the geometry split between target and probing slots to give more capacity to the target region rather than using a frequency mechanism at all.

Looking at the geometry system, `BucketGeometry` is a trait I can implement, and the baseline just returns the harness-provided split unchanged. I could create a custom geometry that skews allocation toward target—say 80% instead of the default—to give the hot tier more space and potentially improve hit rates. Since `geometry.rs` isn't in the gate-integrity surface, this should be in scope. The main question is how shifting more capacity to target affects probing, and whether that interaction causes issues elsewhere.

That said, let me focus on something more straightforward for H03: taking the cycle/0003/v002 variant (MRU-end, no timestamp, frequency gate >= 3 without vacancy guard) which had good hit-rate but latency regression, and layering frequency-aware eviction on top—evicting whichever item has the lowest frequency counter. The tension is that v002's threshold without the vacancy guard caused the latency problem, so I need to be careful about how this combines with H01's vacancy-gated approach.

Actually, a cleaner distinct strategy for H03 would be insertion-site selection based on frequency: instead of always placing promoted items at the MRU end, items with low frequency counters get inserted at an intermediate position (like 3/4 through the bucket) while high-frequency items go all the way to the back.

This creates a graduated temperature structure within the bucket where items earn their way to the hottest position through repeated reuse rather than landing there immediately. Both H02 and H03 would modify `dash_segment.rs` from the same parent, but they're testing completely different mechanisms — one focuses on the promotion path itself, the other on where promoted items land in the bucket.

Now I need to check what insertion positions are actually available in the `InsertPosition` enum. If it only supports `Front` and `Back`, then the graduated insertion idea won't work as I've described it, and I should stick to changes that fit within the existing API constraints. Let me refocus on three clean, distinct hypotheses that work with what's available: frequency-aware eviction victim selection, probing-based promotion paths, and position-based insertion strategies. insert into target at the MRU end. Carry forward item.rs and eviction_policy.rs unchanged.
- Rationale: Hot probing items exceeding the frequency threshold should earn promotion to target if space is available.

Actually, I need to reconsider the logic here. Looking back at cycle/0003/v003, the frequency gate sits in the non-hottest branch of target's get method. So a target hit on a non-hottest item triggers intra-promotion, and if lfu_counter >= 2 with probing not full, it promotes to probing. But that doesn't make sense—probing is a lower tier than target, so why promote upward to a lower tier?

I think I've been misunderstanding the Dash architecture. The cache has target (main), probing (holding area for demoted items), and stash (overflow). Items flow from stash up to target on hits, and from target down to probing when target overflows. Probing isn't hotter than stash—it's a middle ground for items worth keeping but not hot enough for target. So when a non-hottest target item hits with lfu_counter >= 2, the frequency gate determines whether it stays in target or gets demoted to probing.

Actually, I think the promotion works differently than I thought. When a non-hottest target item is accessed and meets the frequency threshold, it moves to the MRU end of target, which displaces the LRU item down to probing. The vacancy gate checks probing capacity because that displaced item needs somewhere to go—without space in probing, it would cascade further down the hierarchy.

For probing hits, an item could stay in probing with an intra-promotion, or potentially move back to target if it meets a frequency gate, creating a path for items to re-enter the hotter tier. me think through what H03 should explore. I could reduce metadata overhead, try a fundamentally different approach from baseline, focus on the stash tier, or revisit the LFU eviction override concept more lightly. One option is to implement frequency-aware eviction selection without the MRU-end insertion or promotion gates—just pure LFU eviction to see if that alone improves hit-rate. But I'm also considering something new: decaying the lfu_counter when items are demoted, so they lose frequency credit as they move down tiers.

For H03, I'm thinking about resetting the lfu_counter to zero (or halving it) whenever an item gets demoted from target to probing or from probing to stash. This way, items that were hot in target but got evicted due to capacity pressure don't carry their historical frequency count into lower tiers—they have to re-earn it. This differs from cycle 4's approach, which reset counters on promotion; here I'm resetting on demotion. The implementation would modify the demotion logic in dash_segment.rs to clear the counter, while carrying forward the existing item.rs and eviction_policy.rs. The reasoning is that an item bumped down due to capacity constraints should prove it's still worth promoting back to target, rather than relying on stale frequency data. When an item gets evicted from the target bucket to probing, I'd reset its lfu_counter to 0 before the demotion happens. This means a demoted item would need at least 2 more accesses to earn promotion back to target via the vacancy gate, which could actually be beneficial—hot items would quickly re-earn their count on a Zipfian workload, while cold items that accidentally landed in target wouldn't clutter it after demotion. Let me formalize this as one of my hypotheses to test.

I'm laying out three candidate strategies: frequency-aware victim selection, a promotion path for frequently-accessed items in probing, and resetting the counter on demotion. All three reference the same parent cycle, which satisfies the constraint but might be too similar. I should consider pulling one from a different parent to add more diversity to the cohort.

For H03, I could test frequency-aware eviction on the simpler baseline without the vacancy gate, isolating just the eviction effect. But that's the same mechanism as H01, just applied to a different parent—the core change is identical even if the combined effects differ. I need to find a genuinely distinct direction instead.

An ARC-style approach with separate lists for recent and frequent items would be too invasive. A better angle is to shift capacity allocation between tiers—say 80/20 instead of 50/50—so the target tier holds more items and performs better on Zipfian workloads. This is fundamentally different from eviction policy or promotion logic; it's about tier geometry itself.

The tricky part is that `BucketGeometry` returns a fixed `DashGeometry` struct, and `factory.rs` (which is off-limits) creates the cache using `BaselineGeometry`. If I override the layout method in `BaselineGeometry` to return a skewed split, it would affect all instances without needing to touch the factory. But there's a risk to consider.

The total capacity must stay the same—shifting from 50/50 to 80/20 target/probing just redistributes the slots, not the overall count. I think this approach is viable but requires modifying `geometry.rs`, which is in scope.

Looking at the three-hypothesis constraint again, I need at least one citing a cross-cycle winner, distinct strategies, and ideally some parent diversity. I'm settling on frequency-aware eviction victim selection for the eviction axis, probing-to-target promotion for hot items on the promotion axis, and custom geometry with expanded target capacity for the sizing axis—all from the same cycle but hitting different levers.

Alternatively, I could pull all three from the same parent but touch different mechanisms: frequency-aware eviction, probing-to-target promotion, and LFU-aware item decay on demotion. The risk is they'd all modify the same three files, but since each changes a different part, the coverage should still be distinct if implemented carefully.

I'll go with all three from cycle/0003/v003 since the quality bar is about strategy distinctness, not parent variety. The key is being explicit in the Changes section about which files each hypothesis needs to carry forward to avoid gaps in cycle-4 coverage.

For H01, I'm proposing to replace the position-based victim selection in the eviction logic with a frequency-based scan that picks the item with the lowest counter, which should improve hit-rate by keeping higher-frequency items around during bucket overflows. The changes involve carrying forward the item structure and the timestamp-free eviction policy, then modifying the candidate selection method to scan for the minimum frequency rather than just returning the first position.

For H02, I'm adding a re-promotion mechanism in the get operation that moves probing-tier items back to the target tier when they show persistent reuse (counter >= 2) and there's available space, recognizing that demoted items with evidence of frequent access deserve to return to the premium tier. the item stays in probing. Now I'm considering a second hypothesis: resetting the `lfu_counter` to zero when an item gets demoted from the target tier back into probing should help hit-rate by preventing stale frequency information from causing premature promotion attempts after the item re-enters probing. The implementation would involve zeroing the counter during the eviction or demotion logic in the segment code.

This is conceptually similar to what cycle 4 tested with promotion, but in the opposite direction — instead of resetting on promotion from stash to target, I'd reset on demotion from target to probing. The earlier experiment showed no measurable improvement when resetting on promotion, but the demotion case might behave differently since it's preventing stale counts from interfering with re-entry logic.

Actually, I'm concerned about the overlap with prior work. Let me pivot H03 to something more distinct: instead of tweaking the counter reset logic, I'll test a custom tier geometry that allocates more capacity to the target bucket — say 70% target and 30% probing instead of the default split. The hypothesis is that reducing eviction pressure on the primary hot tier should improve hit-rate on Zipfian workloads.

I need to modify the `BaselineGeometry::layout` method to compute this new split dynamically, allocating 70% of total capacity to target and 30% to probing with integer division favoring target, then construct the `DashGeometry` with these adjusted bucket sizes while keeping the stash count at baseline. I should verify that `src/shared/geometry.rs` is editable — checking the constraints, it is — but I'm uncertain whether the `DashGeometry` struct has the fields I need to set this up correctly.

Looking at what the agent described about `BaselineGeometry`, the `layout` method currently ignores `total_capacity` (hence the underscore prefix), which suggests the geometry is baked into settings rather than computed dynamically. This means I'd need to compute the split in `for_settings` using the capacity from `DashSettings` instead of trying to override it in `layout`. The `DashSettings` struct has a `capacity()` method, so I can use that to calculate the target and probing allocation there.

The main uncertainty is the exact structure of `DashGeometry` and what fields it expects—whether it's bucket counts, segment sizes, or something else—but the `with_topology()` method on settings and the fact that `BaselineGeometry` wraps a `DashGeometry` suggests this approach should work.

Given that uncertainty, I'm reconsidering H03 and settling on three concrete hypotheses instead. H01 focuses on frequency-minimum eviction in `ClassicLRU` to improve hit-rate by selecting victims based on access frequency rather than just recency.

H02 targets the same hit-rate metric but through a different mechanism: promoting frequently-accessed items from the probing section to the target section when space is available.

For H03, I'm moving away from the vacancy-gated promotion idea since it would essentially duplicate what H02 already does. Instead, I'm considering whether applying frequency-minimum victim selection specifically to stash eviction (rather than globally) could be a distinct improvement.

But there's a problem: if H01 already changes the eviction policy globally to use frequency-minimum selection, then H03's stash-specific override becomes redundant—the global change would already affect stash eviction anyway.

Let me reframe H03 instead: remove the timestamp field, switch to MRU insertion order, and change the eviction policy to select the minimum-frequency item, but without adding any frequency gate for promotion. This isolates whether the eviction policy change alone can achieve v003's hit rate without the vacancy-gated promotion logic, making it genuinely distinct from H01 (which layers LFU eviction on top of v003's frequency gate) and H02. ch has the full v003 feature set plus probing promotion).

So my three hypotheses are: H01 tests LFU-minimum eviction selection by modifying how ClassicLRU picks victims; H02 tests vacancy-gated promotion for hot probing items in the dash_segment logic; and H03 combines MRU-end insertion with LFU-minimum eviction but skips the promotion gate entirely, creating a novel configuration that isolates eviction behavior from promotion mechanics.

I need to be careful when writing the markdown files to explicitly enumerate all inherited file changes across the overlay, not just the ones each hypothesis modifies—that's the key insight from the cycle-4 retrospective about avoiding coverage gaps.

For H01 and H02, since they both inherit from v003, I'll list all three files that v003 touches: `src/shared/item.rs`, `src/shared/eviction_policy.rs`, and `src/dash/segments/dash_segment.rs`. H03 descends from baseline, so it gets its own fresh overlay with those same three files.

Now I'm ready to write out the Changes sections with full file enumeration. For H01, the key change is modifying how `ClassicLRU::eviction_candidate_index` works—instead of always returning 0, it needs to select the least frequently used item. So `item.lfu_counter` is the correct field access. For the probing hit case in `DashSegment::get`, I need to update the MRU position within the probing bucket, increment the counter, and if it reaches the promotion threshold and the target bucket has space, remove it from probing and insert it at the back of the target bucket instead.

I'm realizing there's some confusion about the architecture — I need to clarify whether items in the target bucket that get accessed are intra-promoted to the MRU end, and whether that's separate from the frequency-based promotion logic that moves items between buckets. the reverse of what happens when target overflows into probing. We remove the item from probing, insert it into target at the MRU end, and check that target isn't full before doing this to avoid cascading evictions. There's already a `promote_to_target` method in the existing code that handles this kind of movement, so I can use a similar approach for probing-to-target promotion. Now I'm ready to write out the actual implementation files.

I need to set up the output directory and compose the three hypothesis files. Writing H01.md... The `TimestampLRU` variant and other policy structs remain as-is. In `dash_segment.rs`, I'm keeping the MRU-end insertion and the vacancy-gated frequency promotion logic unchanged. The key insight here is that by selecting the victim with the minimum `lfu_counter` rather than always evicting position 0, I can preferentially remove cold items that haven't been accessed recently, even if they're positioned later in the list.

Now I'm looking at H02, which builds on the same foundation but adds a re-promotion path for items that were demoted from the target tier to the probing tier—if they continue accumulating hits, they become candidates to move back to hot storage, improving hit-rate on Zipfian workloads.

In the probing-hit branch of `get`, after updating the item's position within the probing bucket, I'm incrementing the frequency counter on the accessed item (unless the parent already does this), then checking if the counter has reached 2 and the target bucket has space available. If both conditions hold, I remove the item from its probing position and reinsert it at the MRU end of the target bucket, mirroring the vacancy-gated promotion pattern used elsewhere.

I need to add two unit tests: one verifying that items with a counter of 2 or higher promote to the target when space exists, and another confirming they stay in probing when the target is full.

Now I'm looking at the H03 variant, which removes timestamp-based logic entirely and simplifies the eviction policy. The approach strips out the `Instant` field from the item struct and replaces timestamp-driven LRU with a positional implementation in the eviction policy.

For `ClassicLRU`, I'm changing the eviction candidate selection from a constant return to a linear scan that finds the item with the lowest frequency counter, breaking ties by position. On the promotion side in the segment code, I'm shifting newly promoted items to the back of the target (MRU end) instead of the front, while keeping the non-hottest branch as a simple intra-bucket update without any frequency gating or inter-tier promotion conditions.

These three hypotheses test distinct mechanisms: H01 layers LFU eviction onto the existing v003 parent, H02 adds a new probing-to-target promotion path orthogonal to H01, and H03 tests LFU eviction in isolation without any promotion gating. All three need to cite at least one cross-cycle winner as their parent, and H01 and H02 both reference cycle/0003/v003 which qualifies.

Now I'm verifying the claims are single sentences and preparing to write out the files.

### 16:12:23 chat

Now I have comprehensive context. Let me synthesize the findings and write the three hypotheses.

### 16:12:23 tool: Bash

```json
{
  "command": "ls \"C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0005\\hypotheses\" 2>/dev/null || echo \"Directory does not exist\"",
  "description": "Check if output directory exists"
}
```

### 16:12:29 thinking

Good, the directory exists. Now I can write the three hypothesis files. Let me write them.

### 16:12:44 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0005\\hypotheses\\H01.md",
  "content": "# H01\n\nReplacing position-first victim selection in `ClassicLRU::eviction_candidate_index` with a linear scan that evicts the item with the lowest `lfu_counter` will improve hit-rate by retaining frequently-accessed items when a bucket overflows.\n\n## Parent\n\ncycle/0003/v003\n\n## Changes\n\nThis variant inherits all three overlay files from cycle/0003/v003 and modifies only `eviction_policy.rs`. All three files must be present in the overlay to avoid the coverage-gap regression seen in cycle/0004.\n\n**`src/shared/item.rs`** — Carry forward unchanged from cycle/0003/v003: `Item<K, V>` has `key`, `value`, and `lfu_counter` fields only; no `timestamp` field; `new()` does not call `Instant::now()`; `Display` does not print a timestamp.\n\n**`src/shared/eviction_policy.rs`** — Carry forward all struct definitions and the `TimestampLRU` positional-clone variant unchanged. In the `ClassicLRU` implementation of `EvictionPolicy`, change `eviction_candidate_index` from returning the constant `0` to performing a linear scan: iterate over the `items: &[Item<K, V>]` slice with `.iter().enumerate()`, track the index that yields the minimum `lfu_counter`, and return that index. Use `0` as the tiebreaker when multiple items share the minimum counter value. No other methods or structs change. The `use std::time::Instant` import must remain absent (as in the parent).\n\n**`src/dash/segments/dash_segment.rs`** — Carry forward entirely unchanged from cycle/0003/v003: MRU-end insertion via `InsertPosition::Back`; vacancy-gated frequency promotion in the non-hottest target branch (`lfu_counter >= 2` AND probing bucket not full); all existing unit tests including `vacancy_gated_frequency_promote_fires_when_probing_has_space` and `vacancy_gated_promote_does_not_fire_when_probing_full`.\n\n## Rationale\n\nThe current `ClassicLRU` always evicts the item at position 0 (the LRU end), which is correct when recency is the only signal. But after MRU-end insertion and intra-bucket promotion, position 0 holds the item that has gone longest without *recent* access — its `lfu_counter` may still be high (it was a hot item that had a brief access gap), while a freshly inserted cold item at position N-2 carries `lfu_counter = 0`. Frequency-minimum victim selection evicts the genuinely coldest item rather than the most-recently-cold item, which on the Zipfian workload-a trace should reduce false evictions of hot items temporarily sitting at cold positions.\n"
}
```

### 16:13:02 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0005\\hypotheses\\H02.md",
  "content": "# H02\n\nAdding a vacancy-gated frequency promotion path in `DashSegment::get`'s probing-hit branch that moves items with `lfu_counter >= 2` back into the target tier when target has space will improve hit-rate by recognizing demoted items that continue to demonstrate reuse as candidates for re-entry into the hot tier.\n\n## Parent\n\ncycle/0003/v003\n\n## Changes\n\nThis variant inherits all three overlay files from cycle/0003/v003 and modifies only `dash_segment.rs`. All three files must be present in the overlay to avoid the coverage-gap regression seen in cycle/0004.\n\n**`src/shared/item.rs`** — Carry forward unchanged from cycle/0003/v003.\n\n**`src/shared/eviction_policy.rs`** — Carry forward unchanged from cycle/0003/v003.\n\n**`src/dash/segments/dash_segment.rs`** — Carry forward the MRU-end insertion (`InsertPosition::Back`) and the vacancy-gated frequency promotion in the target-tier non-hottest branch exactly as in cycle/0003/v003. Make the following addition in the probing-hit branch of `get` (the branch that executes after the key is found in the probing bucket):\n\n1. After the existing intra-probing MRU update (`get_from_position` on the probing bucket), increment the accessed item's `lfu_counter` by 1.\n2. Immediately after the increment, evaluate the promotion condition: if `data.lfu_counter >= 2` AND `!core.buckets[target_idx].is_full()`, remove the item from its current probing slot (`remove_at(pos)` on the probing bucket) and insert it into the target bucket at `InsertPosition::Back` (MRU end). This mirrors the vacancy-gate pattern already used in the target non-hottest branch, applied symmetrically to items found in probing.\n3. If the condition is not met (counter below threshold or target full), leave the item in probing with its position updated by step 1's `get_from_position` call and return normally.\n\nAdd two unit tests: `probing_item_with_counter_ge2_promotes_to_target_when_target_has_space` (verifies the promotion fires and the item is in target afterward) and `probing_item_stays_in_probing_when_target_is_full` (verifies the vacancy gate blocks promotion). All existing tests from the parent must continue to pass.\n\n## Rationale\n\nItems reach probing primarily by being evicted from a full target tier during insertion pressure. An item that was evicted from target but continues to receive cache hits in probing is demonstrating persistent reuse — exactly the evidence the frequency gate was designed to capture. Currently these items remain in probing indefinitely, missing the opportunity to reclaim a target slot when one opens up. The proposed path is cascade-free by construction (the vacancy gate ensures target has room before the move) and reuses the same threshold already validated in the target promotion path, so it introduces no new parameters.\n"
}
```

### 16:13:22 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0005\\hypotheses\\H03.md",
  "content": "# H03\n\nBuilding from the baseline with MRU-end insertion, timestamp removal, and LFU-minimum eviction victim selection — but deliberately omitting any frequency-gated promotion — will improve hit-rate and metadata-size by retaining high-frequency items through better eviction choices alone, isolating the eviction mechanism's contribution without the cascade risk of a promotion gate.\n\n## Parent\n\nbaseline\n\n## Changes\n\nThis variant starts from the unmodified baseline and introduces three coordinated changes across the same three files used by the leading incumbents.\n\n**`src/shared/item.rs`** — Remove the `pub timestamp: Instant` field from `Item<K, V>`. Remove the `use std::time::Instant` import. In `Item::new()`, remove the `timestamp: Instant::now()` field initializer. In `impl Display for Item`, remove the timestamp line. The resulting struct has only `key`, `value`, and `lfu_counter` fields, matching the incumbent variants. This eliminates the per-item syscall and reduces the per-item footprint toward the 24-byte target.\n\n**`src/shared/eviction_policy.rs`** — Remove `use std::time::Instant`. Replace the real timestamp-based `TimestampLRU` implementation with a positional clone of `ClassicLRU`: `get_from_position` does a `remove(position)` then `push(item)` at the back (no timestamp write or read); `eviction_candidate_index` returns `0`; `promotion_candidate_index` returns `items.len() - 1`. In `ClassicLRU::eviction_candidate_index`, replace the constant-`0` return with a linear scan over `items: &[Item<K, V>]` using `.iter().enumerate()` to find and return the index with the minimum `lfu_counter`, using `0` as the tiebreaker on ties. All other policy structs (`Lfu`, `Lifo`, `Fifo`, `Swap`) carry forward unchanged.\n\n**`src/dash/segments/dash_segment.rs`** — Change the insertion position for items promoted from stash to target from `InsertPosition::Front` to `InsertPosition::Back`, so newly promoted items land at the MRU end. Do NOT add any frequency gate or vacancy-gated inter-tier promotion logic — leave the non-hottest target branch as a plain intra-bucket MRU update with no `lfu_counter` check and no probing-vacancy check. This intentionally isolates the effect of LFU-minimum eviction from the effect of frequency-gated promotion, which is already validated in cycle/0003/v003.\n\n## Rationale\n\nEvery incumbent variant stacks MRU-end insertion, timestamp removal, and vacancy-gated frequency promotion together. This conflates two independent mechanisms: (a) promoting items based on frequency and (b) evicting items based on frequency. Hypothesis H01 adds LFU-minimum eviction on top of v003's full feature set; this hypothesis tests whether LFU-minimum eviction alone — without any inter-tier promotion gating — can achieve meaningful hit-rate improvement over the baseline. If it can, it points to eviction selection as the dominant lever. If it cannot match v003's result, it confirms that promotion gating carries unique value not captured by eviction selection alone.\n"
}
```

### 16:13:36 chat

Three hypothesis files written to the output directory. Here's a summary of what was produced:

---

## Cycle 0005 — Hypotheses Summary

**Strategic context:** All three cross-cycle winners (`cycle/0003/v002`, `v003`, `cycle/0002/v001`) share the same three-file overlay (item.rs: no timestamp; eviction_policy.rs: positional LRU; dash_segment.rs: MRU-end + vacancy-gated ≥2 promotion). Cycle 4 stalled because (a) all variants lost the item.rs struct reduction through overlay coverage gaps, and (b) the remaining promotion-gate tuning axes appear exhausted. The cycle-4 continuation explicitly recommends shifting to **eviction ordering** and **probing-tier promotion paths** as the next orthogonal axes.

---

### H01 — Frequency-minimum eviction victim selection
**Parent:** `cycle/0003/v003`  
**Axis:** Eviction ordering within a tier  
**Key change:** `ClassicLRU::eviction_candidate_index` in `eviction_policy.rs` — replace `return 0` with a linear scan returning the index of the item with minimum `lfu_counter`. All three parent files carried forward explicitly (to prevent the cycle-4 coverage-gap regression).

### H02 — Probing-to-target re-promotion for hot probing items
**Parent:** `cycle/0003/v003`  
**Axis:** Promotion path for items accessed in the probing tier  
**Key change:** `dash_segment.rs` probing-hit branch — increment `lfu_counter`; if ≥ 2 AND target not full, remove from probing and insert into target at MRU end. Mirrors the vacancy-gate pattern already proven in the target branch. Two new unit tests required.

### H03 — MRU + timestamp removal + LFU eviction, no promotion gate (from baseline)
**Parent:** `baseline`  
**Axis:** Isolating eviction-selection contribution without promotion gating  
**Key change:** All three files modified from scratch — removes timestamp, switches to MRU-end insertion, adds LFU-minimum eviction to ClassicLRU — but deliberately omits the vacancy-gated frequency promotion gate. This ablation tests whether better eviction selection alone can account for the incumbents' gains, or whether the promotion gate carries independent value.


---

- finished: 2026-07-20T16:13:37+00:00
- duration: 588.52s
- status: ok
- input_tokens: 6525
- output_tokens: 23311
- cache_read_tokens: 195959
- cache_write_tokens: 53229
- cost_usd: 1.730694
