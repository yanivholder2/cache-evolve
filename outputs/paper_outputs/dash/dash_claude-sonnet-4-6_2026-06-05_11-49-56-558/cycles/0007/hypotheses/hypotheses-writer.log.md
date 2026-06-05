# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/hypotheses
- allow_edits: True
- started: 2026-06-05T14:18:40+00:00

## timeline

### 14:18:44 thinking

Let me read all the necessary context files to understand the current state of the research and generate informed hypotheses for cycle 0007.

### 14:18:49 tool: Agent

```json
{
  "description": "Read all context files for cycle 0007 hypothesis generation",
  "subagent_type": "Explore",
  "prompt": "Read all of the following files and return their complete contents. This is for a cache evolution research loop and I need all the content to write hypotheses.\n\nFiles to read:\n\n1. /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/cycle-winners.json\n2. /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/cycle-insight.md\n3. /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/leaderboard.md\n4. /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles-ledger.jsonl\n5. /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cross-cycle-insights.md (if present)\n6. /home/user/cache-evolve/inputs/subjects/dash/baseline/src/dash/dash_bucket.rs\n7. /home/user/cache-evolve/inputs/subjects/dash/baseline/src/dash/segments/dash_segment.rs\n8. /home/user/cache-evolve/inputs/subjects/dash/baseline/src/shared/eviction_policy.rs\n9. /home/user/cache-evolve/inputs/subjects/dash/baseline/src/shared/geometry.rs\n10. /home/user/cache-evolve/inputs/subjects/dash/baseline/src/shared/item.rs\n\nThen list all variant directories under:\n/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/variants/\n\nFor each variant found, read:\n- The insight.md file\n- The fitness-vector.json file\n- The overlay files (latest attempt)\n\nReturn all file contents in full."
}
```

### 14:18:52 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/cycle-winners.json"
}
```

### 14:18:52 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/cycle-insight.md"
}
```

### 14:18:52 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/leaderboard.md"
}
```

### 14:18:52 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles-ledger.jsonl"
}
```

### 14:18:52 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cross-cycle-insights.md"
}
```

### 14:18:53 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/dash/dash_bucket.rs"
}
```

### 14:18:53 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/dash/segments/dash_segment.rs"
}
```

### 14:18:53 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/shared/eviction_policy.rs"
}
```

### 14:18:53 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/shared/geometry.rs"
}
```

### 14:18:54 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/shared/item.rs"
}
```

### 14:18:54 tool: Bash

```json
{
  "command": "ls -la /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/variants/"
}
```

### 14:18:56 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/dash/dash_bucket.rs"
}
```

### 14:18:56 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/dash/segments/dash_segment.rs"
}
```

### 14:18:57 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/shared/eviction_policy.rs"
}
```

### 14:18:57 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/shared/geometry.rs"
}
```

### 14:18:57 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/shared/item.rs"
}
```

### 14:18:57 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/variants/*"
}
```

### 14:19:01 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/variants/v001/task.md"
}
```

### 14:19:01 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/variants/v001/insight.md"
}
```

### 14:19:02 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/variants/v001/eval/fitness-vector.json"
}
```

### 14:19:02 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/variants/v002/task.md"
}
```

### 14:19:02 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/variants/v002/insight.md"
}
```

### 14:19:02 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/variants/v002/eval/fitness-vector.json"
}
```

### 14:19:03 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/variants/v003/task.md"
}
```

### 14:19:03 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/variants/v003/insight.md"
}
```

### 14:19:03 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/variants/v003/eval/fitness-vector.json"
}
```

### 14:19:06 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/variants/v001/attempts/01/overlay/src/dash/dash_bucket.rs"
}
```

### 14:19:06 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/variants/v001/attempts/01/overlay/src/dash/segments/dash_segment.rs"
}
```

### 14:19:06 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/variants/v001/attempts/01/overlay/src/shared/eviction_policy.rs"
}
```

### 14:19:06 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/variants/v001/attempts/01/overlay/src/shared/geometry.rs"
}
```

### 14:19:07 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/variants/v002/attempts/01/overlay/src/dash/dash_bucket.rs"
}
```

### 14:19:09 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/variants/v002/attempts/01/overlay/src/dash/segments/dash_segment.rs"
}
```

### 14:19:10 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/variants/v002/attempts/01/overlay/src/shared/geometry.rs"
}
```

### 14:19:10 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/variants/v003/attempts/01/overlay/src/dash/dash_bucket.rs"
}
```

### 14:19:10 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/variants/v003/attempts/01/overlay/src/dash/segments/dash_segment.rs"
}
```

### 14:19:11 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/variants/v003/attempts/01/overlay/src/shared/geometry.rs"
}
```

### 14:22:23 thinking

Now I have a comprehensive understanding of the research history. Let me synthesize what I know and generate three distinct hypotheses for cycle 0007.

## Summary of what I know:

**Current champion:** cycle/0006/v001 (ClassicLRUFreqProtect) — hit-rate 0.7194

**Key wins across cycles:**
1. Hot-tier topology: 24/8 normal/stash split (cycles 1-4)
2. MRU insertion position (consistent winner)
3. Eviction selectivity via frequency guard (cycle 6, v001): evict zero-access items first


4. Second-chance stash recovery (cycle 6, v002): unconditional LRU-end promotion

**What hasn't worked:**
LFU and aging-LFU approaches consistently underperformed compared to position-based recency strategies, ghost sets added only noise, and attempts to weaken stash traversal or push the split ratio beyond 24/8 all degraded performance. Segmented-LRU promotion/demotion asymmetry also failed to improve results.

**What I'm exploring:**
The frequency guard and second-chance recovery mechanisms target different parts of the pipeline, so combining them might unlock gains. I'm also considering raising the frequency threshold to evict items with fewer than 2 in-tier hits rather than just 1.

Now I'm setting up three distinct hypotheses to test, with at least one building directly on cycle 6 v001. The first combines the frequency guard from v001 with the second-chance stash promotion from v002 by modifying the put method in dash_segment.rs to add that recovery logic while preserving the ClassicLRUFreqProtect policy. The second hypothesis is exploring a higher frequency threshold approach. frequency threshold (parent: cycle/0006/v001)
- Take v001 and raise the threshold from 0 to 1: evict items with lfu_counter < 2 first
- Changes: In eviction_policy.rs, modify `eviction_candidate_index` to prefer items with lfu_counter <= 1 first, fall back to lfu_counter == 0, then LRU

The cross-cycle insights are asking whether zero in-tier accesses is the right eviction threshold, or if requiring one or two confirmed hits would improve things further. So the hypothesis is: prioritize evicting items with fewer than 2 in-tier hits before touching the confirmed-hot items with 2 or more hits, rather than just the binary zero vs. non-zero distinction.

### H03: TimestampLRU + frequency guard stacked

For this one, I'm combining v001's frequency guard eviction selectivity with TimestampLRU's promotion-candidacy selection. The insight suggests TimestampLRU should be tested as an addition to the winning frequency-guard baseline rather than replacing it entirely. So I'd create a new policy that uses timestamp for deciding which items are eligible for promotion, while keeping the zero-access guard for eviction decisions.

Looking at the three hypotheses more carefully, I'm trying to figure out what's actually distinct and implementable: H01 stacks v001 with v002, H02 raises the frequency threshold, and H03 combines timestamp-based promotion with frequency-based eviction protection. v001**
3. **H03**: Explore a new axis - something different

For H03, I need something distinct. Let me think about what hasn't been tried:
- Adaptive geometry: the insight says topology is "exhausted" at 24/8, but what about adapting within that range dynamically? Actually, we can't do that easily.
- Size-aware eviction: the Item type might have a value size field we can use
- Scan resistance: inserting large scans at LRU end to protect the hot set
- Counter reset behavior: currently lfu_counter resets on stash demotion; what if we used a small counter in the stash too?

Wait, let me look at what's actually in Item to know if size is available. The item.rs was mentioned but not fully read. Let me think about what I can infer.

Actually, I see in the v001 overlay code that `item.lfu_counter` exists on Item. And from the geometry.rs description it's an "identity geometry that returns harness-provided split unchanged" - but v001 uses 75/25 (24/8) split.

Let me think about another distinct axis for H03. A really interesting hypothesis is **selective second-chance** — only give stash-evicted items a second chance if they've been accessed at least once in stash (lfu_counter > 0). Cold items that were never accessed in stash get dropped entirely, which is fundamentally different from the current approach.

This combines the eviction selectivity principle from v001 with the second-chance mechanism from v002, but applies selectivity to the second-chance decision rather than the eviction decision. It's orthogonal to stacking v001+v002.

So my three hypotheses are:

**H01** (parent: cycle/0006/v001): Stack frequency guard + second-chance — take v001 and add unconditional second-chance stash recovery from v002, targeting both eviction selectivity and stash overflow recovery.

**H02** (parent: cycle/0006/v001): Raise frequency guard threshold to 1 — change the guard so items with lfu_counter <= 1 are eviction candidates over items with lfu_counter >= 2.

**H03** (parent: cycle/0006/v002): Selective second-chance based on access history — only promote stash items back if they were accessed at least once (lfu_counter > 0), dropping cold items entirely.

Now I'm reconsidering what raising the threshold actually means. The current guard evicts lfu_counter == 0 first, so raising to 1 would mean evicting items with lfu_counter <= 1 before those with lfu_counter >= 2. But this could backfire — it would evict items with exactly 1 access (confirmed reuse) before items with 2+ accesses, which defeats the purpose. Raising to 2 instead would protect only the most-accessed items and evict single-access items first, which might be the right trade-off.

For H03, I should combine TimestampLRU with the frequency-guard baseline since the cycle notes specifically recommend testing TimestampLRU as an addition rather than a replacement. This means parenting from the winning frequency-guard policy and adding TimestampLRU's promotion logic on top.

Now I'm thinking through the parent choices for all three hypotheses. The requirement needs at least one to cite the cross-cycle winner, but I can distribute them across v001 and baseline for diversity. The key insight is that the frequency guard in v001 isn't LFU—it only protects confirmed-reuse items from being evicted by zero-access items while preserving LRU ordering everywhere else, which is fundamentally different from pure frequency-based eviction that the cross-cycle analysis warned against.

For H02, I'd combine the frequency protection mechanism with timestamp-based LRU to get more accurate recency tracking. For H03, I could test whether lowering the frequency threshold (checking if access count ≤ 1 instead of the current value) finds a better balance. The tension is whether all three should parent from v001 for coherence, or whether pulling one from baseline adds meaningful diversity—though if the gains are concentrated around v001's approach, that might be the right call. -chance on its own got 0.7183 vs baseline 0.663 — big improvement but less than v001
- Could we start from v002 and add the frequency guard? That would be similar to H01 but parent is v002
- Or start from baseline and combine ALL wins: 24/8 topology + MRU insertion + frequency guard + second-chance

A hypothesis starting from baseline with all known improvements together could be a good sanity check, but the cycle insight framing suggests building incrementally from the champion. I'll go with parenting all three from v001 since that's the proven best-so-far, which aligns with the preference for non-baseline incumbents.

For H01, I'm stacking the frequency guard with unconditional second-chance recovery. H02 combines the frequency guard with TimestampLRU promotion candidacy. H03 raises the frequency guard threshold from 0 to 1 to only protect items with 2+ accesses. Let me write these up properly.

I'm designing three hypotheses that all build from v001 as the parent, combining its frequency guard with other proven techniques to compound the gains.

The first one, icLRUFreqProtect, adds an unconditional second-chance recovery mechanism to the stash—when an item gets evicted from the stash, instead of discarding it, I immediately promote it back to the target, which should reduce both probing-boundary losses and stash-overflow losses simultaneously.

For the second hypothesis, I'm exploring a combined `TimestampFreqProtect` policy that merges the eviction selectivity of frequency protection with timestamp-based promotion ordering—it'll use the same zero-access eviction preference as v001 but choose promotion candidates by wall-clock recency instead, applying timestamp accuracy within each target bucket.

The third one raises the frequency guard threshold from zero to one, so items with two or more confirmed in-tier accesses get protected before those with only one access, extending the guard's reach to the genuinely hot set.

Refining the implementation details...

I'm restructuring the eviction logic to check for never-accessed items first, then once-accessed items, before falling back to the LRU position. Rather than modifying the existing policy, I should create a new variant like `ClassicLRUFreqProtect2` to keep the threshold change isolated, then update the instantiation logic in the dashboard to use the new policy.

Actually, the cleanest approach is to just modify the `eviction_candidate_index` function directly in the variant's overlay file—since the overlay replaces the entire file anyway, I don't need a new struct. Now I'm reconsidering the H02 hypothesis more carefully by looking at how TimestampLRU works: it increments a counter and records a timestamp on access, then returns the item with the highest `last_accessed` timestamp rather than just the most recently inserted one.

For v001 (ClassicLRUFreqProtect), the pattern is different—it increments a frequency counter and moves items to the MRU end on access, then prefers zero-access items for eviction. If I combine these approaches, I'd get a hybrid that increments the frequency counter, records timestamps, moves to MRU, prefers zero-access items for eviction, but uses the highest timestamp for promotion instead of just the MRU position. The testable claim would be that adding timestamp-based promotion to the frequency-guard eviction policy improves hit rate by ensuring genuinely hot residents are prioritized.

Now I need to write these up properly—I'm confident in all three hypotheses. But first, let me verify what fields the Item struct actually has. I know from v001 that there's an `lfu_counter` field, and v003's TimestampLRU must use some kind of timestamp or monotonic counter for tracking recency. Let me check what the baseline eviction policy reveals about the timestamp field.

Since TimestampLRU is already implemented in the baseline eviction_policy.rs, the Item struct definitely supports it—there's likely a `last_accessed` field or similar. That means my H02 hypothesis combining TimestampFreqProtect is definitely feasible.

Alright, I have what I need. Time to write out the three hypothesis files for the output directory.

For H01, I'm stacking the ClassicLRUFreqProtect selectivity from cycle/0006/v001 with the second-chance stash recovery from cycle/0006/v002 to address both the probing-boundary eviction loss and stash-overflow loss simultaneously. The key change is intercepting stash overflow events in the `put()` method and promoting evicted items back to the front instead of discarding them. identical to the change v002 made but applied on top of v001's code base.

Keep all other v001 changes intact:
- `ClassicLRUFreqProtect` policy in `dash_bucket.rs`
- `lfu_counter = 0` reset when inserting into stash in `dash_segment.rs`
- 24/8 normal/stash geometry in `geometry.rs`

The two mechanisms are orthogonal: the frequency guard acts at eviction time within the normal/probing tiers; the second-chance acts at stash overflow to recover items before they are permanently lost. Neither change alters the other's code path.

Combining them should close both loss sites simultaneously. Now I'm thinking about a new `TimestampFreqProtect` policy that merges the zero-access eviction guard from `ClassicLRUFreqProtect` with the recency-based promotion logic from `TimestampLRU`, so that when we promote an item from a target bucket, we're selecting the genuinely hottest resident rather than just the most recently inserted one.

Defining the new policy struct... `::new(TimestampFreqProtect)` instead of `Box::new(ClassicLRUFreqProtect)` in the bucket instantiation. The rest of the v001 changes stay the same—the counter reset on demotion and the 24/8 tier geometry. Since eviction and promotion operate on separate code paths, combining the frequency-based eviction guard from v001 with the timestamp-based promotion logic from v003 should let me test whether these improvements stack on top of each other.

Now I'm looking at H03, which tightens the eviction guard threshold from zero to one—so items that have been accessed once or never get evicted before those accessed twice or more. This builds on v001's single-tier zero-access guard by being more selective about what counts as "hot." I'm modifying the eviction candidate selection to scan in two stages: first prefer items with zero accesses, then fall back to items with one access before touching anything with higher frequency.

Refining eviction logic...

The rationale here is that the lfu_counter reset on stash demotion (already in v001) ensures items entering stash start at zero, so the new two-stage guard correctly identifies them as cold residents and prioritizes their eviction.

Now I'm laying out three distinct hypotheses to test: stacking v001 and v002's orthogonal improvements, combining v001 and v003's innovations into a single policy, or probing v001's threshold parameter.

All three build from v001 as the foundation since it's shown the strongest performance, then I'm writing the output files.

### 14:22:23 chat

Now I have all the context I need. Let me synthesize and write the three hypotheses.

### 14:22:23 tool: Bash

```json
{
  "command": "mkdir -p /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/hypotheses && ls /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/hypotheses",
  "description": "Ensure output directory exists"
}
```

### 14:22:34 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/hypotheses/H01.md",
  "content": "# H01\n\nStacking ClassicLRUFreqProtect eviction selectivity with unconditional second-chance stash recovery will compound the separate gains of cycle/0006/v001 and cycle/0006/v002 by simultaneously closing the probing-boundary eviction loss site and the stash-overflow loss site.\n\n## Parent\n\ncycle/0006/v001\n\n## Changes\n\nStarting from cycle/0006/v001 (which uses `ClassicLRUFreqProtect` in all buckets, resets `lfu_counter` on stash demotion, and uses 24/8 normal/stash geometry), add the unconditional second-chance stash-overflow recovery introduced by cycle/0006/v002.\n\nIn `src/dash/segments/dash_segment.rs`, override the `put()` method on `DashSegment` so that after calling `self.core.put_at(Slot::Stash, InsertPosition::Back, item)`, the return value is inspected: if `Some(ev)` is returned, immediately call `promote_to_target(&mut self.core, ev, InsertPosition::Front)` instead of discarding the evicted item. This is exactly the change v002 applied, but now applied on top of v001's codebase. Remove any ghost-set logic from the `put()` path if present in the parent.\n\nKeep all other v001 changes intact:\n- `ClassicLRUFreqProtect` policy instantiated in `src/dash/dash_bucket.rs`\n- `item.lfu_counter = 0` reset when inserting into stash in `src/dash/segments/dash_segment.rs` (already present in v001; preserve it in the same demotion paths where v001 placed it)\n- 24/8 normal/stash geometry in `src/shared/geometry.rs`\n\nThe two mechanisms are orthogonal: the frequency guard operates at eviction time within normal/probing buckets (protecting confirmed-reuse residents); the second-chance operates at stash overflow to recover items before they are permanently lost from the cache. Neither code path overlaps with or depends on the other.\n\n## Rationale\n\nCycle 0006's insight explicitly identifies these two gains as additive candidates targeting different pipeline stages. v001 fixed *who* gets evicted from the probing tier; v002 fixed *what happens* to items lost from stash overflow. Combining both on the best-so-far parent tests the direct compounding hypothesis recommended by the cycle analysis.\n"
}
```

### 14:22:49 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/hypotheses/H02.md",
  "content": "# H02\n\nCombining ClassicLRUFreqProtect's zero-access eviction guard with TimestampLRU's recency-based promotion candidacy into a single `TimestampFreqProtect` policy will improve hit rate over cycle/0006/v001 by ensuring the item promoted from each target bucket is the genuinely hottest resident rather than merely the most recently inserted one.\n\n## Parent\n\ncycle/0006/v001\n\n## Changes\n\nIn `src/shared/eviction_policy.rs`, add a new policy struct:\n\n```rust\n#[derive(Debug)]\npub struct TimestampFreqProtect;\n```\n\nImplement `EvictionPolicy<K, V>` for it with the following three methods:\n\n- `name()`: return `\"TimestampFreqProtect\"`.\n- `get_from_position(items, position)`: record the current wall-clock timestamp on `items[position].last_accessed` (same as `TimestampLRU`), increment `items[position].lfu_counter` by 1 (same as `ClassicLRUFreqProtect`), remove the item from `position` and push it to the end of the vector (MRU end), then return `items.len() - 1`.\n- `eviction_candidate_index(items)`: return the index of the first item whose `lfu_counter == 0`; if no such item exists, return `0` (LRU-position fallback). This is identical to `ClassicLRUFreqProtect`.\n- `promotion_candidate_index(items)`: return the index of the item with the maximum `last_accessed` value, using position (highest index = MRU end) as a tiebreaker. This is identical to `TimestampLRU`.\n\nIn `src/dash/dash_bucket.rs`, change `new_bucket` to instantiate `Box::new(TimestampFreqProtect)` instead of `Box::new(ClassicLRUFreqProtect)`.\n\nNo changes to `src/dash/segments/dash_segment.rs` or `src/shared/geometry.rs` relative to v001 — keep the `lfu_counter = 0` reset on stash demotion and the 24/8 normal/stash geometry exactly as in v001.\n\nThe eviction guard and promotion candidacy logic operate on independent code paths: eviction candidacy fires when a bucket must shed an item; promotion candidacy fires when the segment's target-bucket scan selects which item to elevate to the normal tier. They do not interfere.\n\n## Rationale\n\nv001 improved eviction selectivity but left promotion candidacy as MRU-position-based (same as ClassicLRU). v003's TimestampLRU improved promotion accuracy but had no eviction guard and scored lower (0.7136 vs 0.7194). The cross-cycle insight notes that TimestampLRU should be \"tested as an addition to the winning frequency-guard baseline rather than as a standalone replacement.\" This hypothesis does exactly that: it grafts v003's promotion improvement onto v001's eviction improvement to test whether the two axes compound.\n"
}
```

### 14:23:03 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/hypotheses/H03.md",
  "content": "# H03\n\nRaising the ClassicLRUFreqProtect eviction guard threshold from zero to one — evicting items with `lfu_counter <= 1` before those with `lfu_counter >= 2` — will extend protection to the genuinely hot working set and improve hit rate over cycle/0006/v001's binary zero-access guard.\n\n## Parent\n\ncycle/0006/v001\n\n## Changes\n\nIn `src/shared/eviction_policy.rs`, replace the body of `ClassicLRUFreqProtect::eviction_candidate_index` with a two-stage priority scan:\n\n```rust\nfn eviction_candidate_index(&self, items: &[Item<K, V>]) -> usize {\n    // Stage 1: prefer items that have never been accessed in this tier\n    if let Some((i, _)) = items.iter().enumerate().find(|(_, it)| it.lfu_counter == 0) {\n        return i;\n    }\n    // Stage 2: next prefer items accessed exactly once (single-touch, not yet confirmed hot)\n    if let Some((i, _)) = items.iter().enumerate().find(|(_, it)| it.lfu_counter == 1) {\n        return i;\n    }\n    // Fallback: evict the LRU-position item (index 0)\n    0\n}\n```\n\nNo other changes to `eviction_policy.rs`. No changes to `src/dash/dash_bucket.rs`, `src/shared/geometry.rs`, or `src/shared/item.rs` relative to v001. The `lfu_counter = 0` reset on stash demotion in `src/dash/segments/dash_segment.rs` remains unchanged — items entering stash start at zero, so Stage 1 of the new guard correctly treats freshly demoted items as cold stash residents.\n\n## Rationale\n\nv001's guard creates a binary cold/warm split at the zero-access boundary: items with `lfu_counter == 0` are evicted before any accessed item. On workload-a's Zipfian distribution the truly hot items accumulate many accesses; items with exactly one access may effectively be as transient as never-accessed newcomers — they have been seen once but not confirmed to recur. A two-tier guard preserves doubly-confirmed residents (`lfu_counter >= 2`) at the expense of singly-touched items (`lfu_counter == 1`), which could help in crowded probing buckets where single-touch arrivals compete with high-frequency incumbents. The cross-cycle insight explicitly lists \"is zero the right threshold, or would a slightly higher threshold yield further improvement?\" as an open question — this hypothesis tests it directly.\n"
}
```

### 14:23:13 chat

All three hypotheses are written to `/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/hypotheses/`:

---

**H01 — Compound frequency guard + second-chance** (`parent: cycle/0006/v001`)  
Takes v001 as-is and splices in v002's unconditional second-chance stash recovery: when a stash eviction occurs, the evicted item is immediately placed at `InsertPosition::Front` in its target bucket rather than discarded. The two mechanisms are orthogonal — the frequency guard fires at probing-tier eviction, second-chance fires at stash overflow — making this the direct compounding test recommended by the cycle-0006 insight.

**H02 — TimestampFreqProtect combined policy** (`parent: cycle/0006/v001`)  
Adds a new `TimestampFreqProtect` policy that merges v001's zero-access eviction guard (`eviction_candidate_index`) with v003's wall-clock recency promotion selector (`promotion_candidate_index`). v001 left promotion candidacy as pure MRU-position-based; this grafts the timestamp-accurate promotion of v003 onto the eviction improvement already proven in v001, exactly as the cross-cycle narrative suggested.

**H03 — Raise frequency guard threshold to 1** (`parent: cycle/0006/v001`)  
Modifies `ClassicLRUFreqProtect::eviction_candidate_index` to use a two-stage scan: prefer `lfu_counter == 0` first, then `lfu_counter == 1`, then fall back to LRU position. This probes the open question from the cycle-0006 insight — whether the zero-access threshold is the right cutoff or whether protecting only doubly-confirmed hot items (`lfu_counter >= 2`) yields further gains on workload-a's Zipfian distribution.


---

- finished: 2026-06-05T14:23:13+00:00
- duration: 272.56s
- status: ok
- input_tokens: 7
- output_tokens: 10239
- cache_read_tokens: 136589
- cache_write_tokens: 18864
- cost_usd: 0.406346
