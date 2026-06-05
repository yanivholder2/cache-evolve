# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0002/hypotheses
- allow_edits: True
- started: 2026-06-04T04:35:06+00:00

## timeline

### 04:35:10 thinking

Let me read all the context files to understand the current state of the research and then generate 3 distinct hypotheses for cycle 0002.

### 04:35:10 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0001/variants/*/eval/fitness-vector.json"
}
```

### 04:35:11 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0001/cycle-winners.json"
}
```

### 04:35:11 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0001/cycle-insight.md"
}
```

### 04:35:15 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0001/variants/*/insight.md"
}
```

### 04:35:15 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/leaderboard.md"
}
```

### 04:35:16 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cross-cycle-insights.md"
}
```

### 04:35:18 thinking

Now let me read the variant overlays from cycle 0001 to understand what changes were made.

### 04:35:19 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0001/variants/*/attempts/*/overlay/**/*"
}
```

### 04:35:20 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles-ledger.jsonl"
}
```

### 04:35:22 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0001/variants/**"
}
```

### 04:35:25 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0001/variants/v001/attempts/01/overlay/src/shared/eviction_policy.rs"
}
```

### 04:35:26 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0001/variants/v003/attempts/01/overlay/src/associative_cache/associative_cache_bucket.rs"
}
```

### 04:35:26 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0001/variants/v001/insight.md"
}
```

### 04:35:29 thinking

Now let me read the baseline source code to understand the full picture.

### 04:35:31 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0001/variants/v002/insight.md"
}
```

### 04:35:31 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0001/variants/v003/insight.md"
}
```

### 04:35:32 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/inputs/subjects/associative_cache/baseline/**/*"
}
```

### 04:35:34 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/associative_cache/baseline/eval/fitness-vector.json"
}
```

### 04:35:34 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/inputs/subjects/associative_cache/baseline/src/**/*"
}
```

### 04:35:36 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/inputs/subjects/associative_cache/**/*"
}
```

### 04:35:39 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/associative_cache/src/associative_cache/associative_cache_bucket.rs"
}
```

### 04:35:39 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/associative_cache/src/shared/item.rs"
}
```

### 04:35:43 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/associative_cache/src/shared/traits/bucket.rs"
}
```

### 04:35:43 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/associative_cache/src/shared/traits/eviction_policy.rs"
}
```

### 04:38:08 thinking

Now I have a comprehensive understanding of the codebase and experiment history. Let me summarize what I've learned:

**Baseline:**
- `Item<K, V>` has: `key`, `value`, `lfu_counter: usize`, `timestamp: Instant` — size is 40 bytes
- `AssociativeCacheBucket` has: `items: Vec<Item<K,V>>`, `max_size: usize`, `eviction_policy: Box<dyn EvictionPolicy<K,V>>`
- `ClassicLRU` is the default eviction policy (moves hit item to back, evicts based on lfu_counter with position-order tie-break)


- Baseline hit-rate is 0.661

In the first cycle of experiments, v001 achieved a significant improvement to 0.7231 by refining the eviction candidate selection to prioritize items with lower frequency counts and use position as a tiebreaker, leveraging the fact that `lfu_counter` was already being incremented on cache hits. Meanwhile, v002 showed no benefit from removing the timestamp field since that metadata size wasn't being measured.

v003 took a different approach by adding a small FIFO recovery buffer per bucket that catches evicted items and can restore them on subsequent hits, reaching 0.689. The interesting observation is that both v001 and v003 address the same underlying problem—hot items getting displaced during cold-item bursts—but through different mechanisms: v001 prevents the wrong evictions in the first place, while v003 recovers items that do get evicted. These strategies could be complementary, though testing their combination hasn't happened yet. There's also a measurement gap where the baseline shows 40 bytes per item but the newer variants didn't capture their metadata overhead.

For the next cycle, I need to ensure at least one hypothesis builds directly on v001 as the parent since it's the cross-cycle winner, and I need to design three distinct variants to test.

The first hypothesis would combine v001 and v003 together—layering the second-chance stash mechanism on top of v001's LFU-aware eviction logic to see if the two complementary approaches compound their benefits. The second hypothesis would address a weakness in v001's LFU counter design: the counters grow unboundedly over time, causing early items to become artificially "sticky" even when they're no longer hot, so I'd test adding periodic decay to reset or halve counters and keep the frequency signal responsive. The third hypothesis would refine v003's stash by being more selective about which evicted items get admitted—only accepting items that have been accessed at least once rather than treating all evictions equally, which would preserve stash space for genuinely reusable entries. complementary: LFU reduces wrongful evictions, stash recovers the ones that slip through
- Claim: Adding a second-chance stash to v001's frequency-aware ClassicLRU will compound the hit-rate gains of both mechanisms

**H02: LFU counter decay in ClassicLRU** — Parent: cycle/0001/v001  
- v001's counters grow monotonically; a hot item from a past burst stays "hot" forever
- Decay: when an item is eviction candidate (i.e., would be evicted), reset its counter to 0 if it beats the eviction but survives, or halve all counters when a new item is inserted
- Actually the simplest approach: when `evict_item()` runs, halve all lfu_counters in the bucket to prevent stale frequency from dominating
- This lets recently re-warming items overtake historically-hot-but-now-cold items
- Claim: Periodic counter halving during eviction will improve hit rate by giving frequency recency weight

**H02: Frequency-gated stash in ClassicLRU** — Parent: cycle/0001/v001
- Instead of adding a general stash like v003, only stash evicted items with lfu_counter > 0 (items that have been accessed at least once beyond insertion)
- Items with lfu_counter == 0 are truly cold and don't deserve a second chance
- This is cleaner than v003's full stash: smaller memory footprint, higher signal quality

**H03: LFU counter saturation cap** — Parent: cycle/0

001/v001
- v001 increments lfu_counter on every hit with no upper bound
- After N hits, the counter becomes so large that a once-very-hot item will dominate for a long time even if no longer active
- Adding a saturation cap (e.g., 255 or some reasonable max) prevents counter runaway and lets recently-accessed cold items evict stale hot items when workload patterns shift

Now I'm stepping back to think about what's actually distinct here. The cross-cycle insights point toward testing v001 + v003 together, closing the metadata-size measurement gap, and exploring whether removing timestamps from v002 simplifies things. But I need a hypothesis that stands on its own rather than just combining existing variants.

I'm considering three separate strategic directions. First, there's composing stash logic on top of v001's LFU-weighted eviction—since v003's stash was built on the baseline's simpler LRU, layering it onto v001 means fewer items hit the stash in the first place because LFU already filters out many bad evictions, leaving the stash to handle only the remaining edge cases. Second, I could evolve v001 itself by adding frequency decay when items get evicted, so the counter doesn't grow unbounded over time.

For the decay approach, the idea is to halve all frequency counters in a bucket whenever eviction happens—this preserves which items are relatively hotter while preventing old hot items from permanently dominating the cache when the workload shifts. The halving only triggers when the bucket fills and a new item arrives, so it's already part of the eviction scan. Third option is still forming... Admission filter — skip promoting cold stash items**
- Parent: cycle/0001/v001  
No wait, I need one from baseline or a prior variant that DOESN'T need to cite v001. Let me reconsider...

Actually the requirement says "At least one of the hypotheses MUST cite a cross-cycle winner above as its ## Parent." So at least one must cite v001. The others can cite baseline or other variants.

Let me think about what other exploration would be distinct:

**H03: Per-item frequency tracking reset on put** (from baseline)
- When a new item is inserted via `put`, the existing items' `lfu_counter` could be selectively decremented or aged
- Specifically, in `ClassicLRU.eviction_candidate_index`, use a scoring mechanism where age_threshold is derived from some global tick or insertion count

The `generation` approach is another angle — tracking when each item was last accessed in terms of operation count, then computing a score that normalizes frequency by how recent the access was, giving us "frequency over recent window" rather than "frequency over all time." But that requires adding fields or passing context, which adds complexity.

Actually, the cleanest distinct approach for H03 is to make eviction victim-aware by scoring based on frequency ratio rather than absolute count. Instead of just picking the minimum `lfu_counter`, I'd compute a score combining both frequency and position to favor items that are less frequently accessed but also further back in the recency order. are older, so they're penalized more
- Or: score = lfu_counter - (bucket_len - position) * weight — blending frequency and recency
- This is a RRIP-like (Re-reference Interval Prediction) approach
- Parent: baseline

Wait, let me look at what v001 already does. It uses `min_by_key(|(i, item)| (item.lfu_counter, *i))`. The `*i` (position) is used as a tiebreaker when `lfu_counter` is equal. So it's purely LFU with position-as-tiebreak.

A hypothesis that blends frequency and recency more explicitly would be different. For example:
- Score = lfu_counter * recency_weight + position_score
- Or use a log-scale: evict item with lowest `(lfu_counter + 1) * (position + 1)` (a product score that favors items accessed frequently AND recently)

This is distinct from v001 (pure LFU), v003 (stash), and the H01/H02 ideas above.

Let me finalize my 3 hypotheses:

**H01: Stash + LFU composition**
- Claim: Combining v001's frequency-aware eviction with a per-bucket second-chance stash will raise hit rate above either mechanism alone on Zipfian workloads.
- Parent: cycle/0001/v001
- Changes: Merge v003's stash mechanism into v001's overlay by adding stash storage and promotion logic to the bucket structure, while keeping v001's frequency-aware eviction policy intact.

**H02: LFU counter decay on each eviction event**
- Claim: Halving all `lfu_counter` values in the bucket at each eviction event will raise hit rate on workloads with evolving access patterns by preventing stale frequency from permanently blocking newly-hot items.
- Parent: cycle/0001/v001
- Changes: In the eviction policy, after selecting the minimum-frequency candidate, decay all counters in the bucket by right-shifting them. But there's a problem — the eviction function takes an immutable slice, so I can't mutate the counters directly.

I need to find where mutation is actually possible. The `evict_item` method calls `eviction_candidate_index` first (immutable), then `remove_at` (mutable). I could trigger the decay in the bucket's `put` method before eviction happens, since that's where we have mutable access to the items.

The approach would be to decay all counters in the bucket when it's full, right before calling `evict_item`. This means overriding `put` in the bucket implementation rather than relying on the trait default, which adds complexity since the baseline only modifies the eviction policy.

I could alternatively trigger decay on reads or add a pre-eviction hook to the trait, but those have their own drawbacks. The cleanest path is to override `put` in `AssociativeCacheBucket` to handle the decay logic before eviction, building on the LFU-weighted candidate selection from v001. I need to verify how trait method shadowing works in Rust to make sure this approach is sound.

Looking back at v003, it already did exactly this with `put` and `get` overrides, so I can follow that pattern. Since v001 only modified `eviction_policy.rs`, I'll need to include both that file and `associative_cache_bucket.rs` in the overlay for H02. The revised approach is to add a `put` override that right-shifts all LFU counter values by 1 when the bucket is full, right before calling eviction.

For H03, I'm considering a blended approach that combines frequency and recency into a single eviction score—something like multiplying the LFU counter and position together so that old items with low frequency get evicted first, while recent high-frequency items stay protected. This is similar to RRIP but simpler, and it leverages the fact that position in ClassicLRU already tracks recency. , old) → (0+1) * (0+1) = 1 → lowest score, evicted first ✓
- Hot, recent: (10+1) * (N-1+1) = 11*N → highest score, protected ✓

This formula balances frequency and recency: items that are both cold and old get evicted first, while hot recent items are protected longest. items to the back on access, so position still represents recency. This approach differs fundamentally from pure LFU with position tiebreaking or the stash variant—it's a hybrid that requires both frequency and recency to protect an item. When implementing the eviction score in `ClassicLRU.eviction_candidate_index`, I can use the items slice length to compute recency ranks, then multiply frequency and recency scores together so that items need strength in both dimensions to avoid eviction. The blended scoring approach would shift eviction decisions compared to pure LFU—protecting recently-accessed items even when they're slightly less frequent, which could matter in workloads where recency plays a meaningful role alongside frequency. I'm settling on three hypotheses to explore: the stash plus LFU composition from v001, the counter halving mechanism to address staleness, and this new blended frequency-by-recency scoring as an alternative to pure LFU.

Both H01 and H02 draw from v001 since it's the proven cross-cycle winner, and they're distinct enough—one adds structural composition while the other refines the frequency signal itself. For H03, I'm using the baseline's approach as the parent. Now I need to check whether the counter decay in H02 can stay contained within the eviction policy layer rather than requiring changes across multiple files.

Looking at the available hooks, `eviction_candidate_index` is read-only so I can't mutate state there, and `get_from_position` fires on every read which would decay at the wrong time. The cleanest approach is to override `put` in the associative cache bucket to apply decay on eviction events, which means the implementer would need to include that file in their overlay alongside the policy changes.

Alternatively, I could add a heuristic decay in `get_from_position` using a modulo check on the counter, but that's less principled since decay would happen on reads rather than when capacity pressure actually triggers eviction. Let me reconsider the whole approach for H02 — maybe capping the LFU counter itself is the way to go.

Instead of decay, I'll cap `lfu_counter` at a reasonable maximum like 16 or 32, since an item accessed thousands of times behaves the same as one accessed a hundred times in practice. This prevents counter runaway while keeping the frequency signal meaningful for lower-access items. I'd implement this in `get_from_position` by using saturating addition with a min cap, and it only requires changes to `eviction_policy.rs`.

This approach is testable: on a Zipfian workload where access patterns shift over time, the cap should let rising-frequency items overtake stale high-counter items that would otherwise remain sticky indefinitely. Now I'm settling on the final hypotheses to test—combining v001's LFU eviction with v003's second-chance stash as one variant, and adding the saturation cap as another. stash + LFU on top of v001 rather than baseline, so all three hypotheses are genuinely distinct. Now I need to write them out formally and check which files I can edit—the eviction policy, item structure, and bucket implementation are all fair game, though the JNI and factory files are locked.

For H01 I'll need both the LFU eviction logic from v001 and the stash mechanism from v003, while H02 and H03 only require tweaks to the eviction policy itself. Let me draft the markdown files next.

I'm also thinking through how v003's stash layer interacts with v001's ClassicLFU—when an item gets promoted from stash back to primary after a hit, its counter would already have some value from when it was originally evicted, so I need to trace through that logic carefully. pushed to primary without incrementing the counter on that access, but it retains its accumulated frequency from before. I should add a counter increment when promoting from stash to account for the access that triggered the promotion. Now I'm ready to write out the three hypothesis files.

### 04:38:08 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/associative_cache/CONTEXT.md"
}
```

### 04:38:09 tool: Bash

```json
{
  "command": "ls /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0002/hypotheses 2>/dev/null || echo \"DIR_NOT_FOUND\"",
  "description": "Check if output directory exists"
}
```

### 04:38:36 thinking

Good, the output directory exists. Now let me write the three hypothesis files.

Let me finalize my three hypotheses:

**H01: Stash + LFU composition (parent: cycle/0001/v001)**
- Claim: Adding a per-bucket second-chance stash to v001's frequency-aware ClassicLRU will compound hit-rate gains by recovering stash hits that LFU eviction still fails to protect.
- Changes: Start from v001's overlay (`src/shared/eviction_policy.rs` with LFU-weighted `eviction_candidate_index`). 

I also need to overlay the bucket implementation to incorporate v003's stash mechanism, where items promoted from the stash get their `lfu_counter` incremented by 1 to reflect that access as a hit.

**H02: LFU counter saturation cap to prevent runaway (parent: cycle/0001/v001)**
- Claim: Capping the `lfu_counter` at a maximum of 16 in ClassicLRU will raise hit rate by allowing rising-frequency items to eventually overtake formerly-hot items whose access rate has declined.
- Changes: In `

ClassicLRU.get_from_position()`, I'm capping the counter increment so it saturates at 16 instead of growing unbounded. Everything else in v001's overlay stays the same.

**H03: Blended frequency-recency product score for eviction (parent: baseline)**
- Claim: Replacing ClassicLRU's lexicographic (frequency, position) eviction criterion with a frequency-times-recency product score will raise hit rate by giving recency equal weight with frequency for medium-frequency items.
- Changes: In `ClassicLRU.eviction_candidate_index()`, I'm replacing the lexicographic ordering with a product-based scoring approach that multiplies frequency and recency together...

Checking the position semantics...

items move to the back when accessed, so position 0 is oldest and position len-1 is newest. With the product scoring formula, an old item with low frequency gets score 1 (evicted first), while a recent high-frequency item gets a much larger score that protects it—and even an old item with high frequency only scores 11, showing the recency factor dominates the eviction decision.

There's a slight concern that newly inserted cold items get protected by recency alone (scoring len), but that's actually desirable behavior since pure LFU would immediately evict them. I'm satisfied with this scoring approach and ready to formalize the three hypotheses.

Now I need to write them out following the claim format: starting with a concrete change and its fitness dimension, no list markers, optional header allowed.

### 04:38:41 chat

Now I have all the context I need. Let me synthesize and write the three hypotheses.

**Summary of what I've learned:**
- Baseline: ClassicLRU with pure front-of-vector eviction, hit-rate = 0.661
- v001 (winner, 0.723): LFU-weighted `eviction_candidate_index` using existing `lfu_counter` — already latent capacity activated
- v002 (0.661): Removed `timestamp` field — no effect since `metadata-size` dimension not measured for variants
- v003 (0.689): Per-bucket FIFO stash at 1/4 bucket capacity — recovers evicted items on re-access
- Both v001 and v003 combat the same root failure: cold-burst displacement of hot items
- Cycle insight explicitly recommends testing v001 + v003 composition
- v001's unbounded `lfu_counter` growth is a potential vulnerability: stale-hot items can block rising-frequency items indefinitely

### 04:38:54 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0002/hypotheses/H01.md",
  "content": "# H01\n\nAdding a per-bucket second-chance stash on top of v001's frequency-aware ClassicLRU will raise hit rate above either mechanism alone by recovering items that LFU eviction still fails to protect during cold-burst displacement.\n\n## Parent\n\ncycle/0001/v001\n\n## Changes\n\nCarry forward v001's overlay of `src/shared/eviction_policy.rs` unchanged — the LFU-weighted `eviction_candidate_index` (min by `(lfu_counter, position)`) and the `lfu_counter` increment in `get_from_position` remain intact.\n\nIn addition, add an overlay for `src/associative_cache/associative_cache_bucket.rs` that introduces the stash mechanism from v003 on top of v001's eviction policy:\n\n1. Add two fields to `AssociativeCacheBucket`: `stash: Vec<Item<K, V>>` and `stash_max_size: usize`. Initialise `stash_max_size` to `(settings.bucket_size / 4).max(1)` in `new()`.\n\n2. Add a private `stash_push` helper that enforces the cap by removing `stash[0]` (FIFO overflow) when `stash.len() >= stash_max_size` before appending.\n\n3. Override `put` in the `Bucket<K, V>` impl block:\n   - If the key already exists in `items`, update in place and call `get_from_position` as before.\n   - If the key is found in `stash`, remove it from the stash, increment its `lfu_counter` by 1 to credit the access, evict one primary item into the stash (if primary is full), then push the promoted item to the back of `items`.\n   - For a new key: evict one primary item into the stash (if full), push the new item.\n\n4. Override `get` in the `Bucket<K, V>` impl block:\n   - Primary hit: call `get_from_position` as before (this already increments `lfu_counter`).\n   - Stash hit: remove the item from the stash, increment its `lfu_counter` by 1, evict one primary item into the stash (if primary is full), push the stash item to the back of `items`, and return a reference to it.\n   - Miss: return `None`.\n\nThe LFU-weighted `eviction_candidate_index` from v001 applies on every eviction from primary — items evicted into the stash are the globally least-frequent in their bucket. The stash then provides a second chance specifically for those items, which tend to be medium-frequency items displaced by a cold burst rather than genuinely cold items.\n"
}
```

### 04:39:05 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0002/hypotheses/H02.md",
  "content": "# H02\n\nCapping `lfu_counter` at a maximum of 16 in ClassicLRU will raise hit rate by preventing stale-high-frequency items from permanently blocking rising-frequency items when the workload's hot set shifts.\n\n## Parent\n\ncycle/0001/v001\n\n## Changes\n\nModify `src/shared/eviction_policy.rs` starting from v001's overlay. The only change relative to v001 is in `ClassicLRU::get_from_position`: replace the unbounded increment\n\n```rust\nitems[back].lfu_counter += 1;\n```\n\nwith a saturating increment capped at 16:\n\n```rust\nitems[back].lfu_counter = items[back].lfu_counter.saturating_add(1).min(16);\n```\n\nAll other logic in v001's `ClassicLRU` stays identical: the move-to-back access discipline in `get_from_position`, the `min_by_key(|(i, item)| (item.lfu_counter, *i))` eviction criterion in `eviction_candidate_index`, and the `items.len() - 1` promotion candidate.\n\nThe cap of 16 is chosen to preserve discrimination among truly cold items (counters 0–3) and moderately hot items (counters 4–15) while ensuring that an item accessed thousands of times does not accumulate a counter so large that a newly-rewarming item — whose counter just crossed 1 — can never compete with it for survival. Items genuinely in the Zipfian hot set will saturate quickly and still dominate the eviction order; the benefit accrues when a formerly-hot item's access rate drops and another item is rising, which the uncapped version handles poorly.\n"
}
```

### 04:39:18 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0002/hypotheses/H03.md",
  "content": "# H03\n\nReplacing ClassicLRU's lexicographic (frequency, position) eviction criterion with a frequency-times-recency product score will raise hit rate by giving recency proportional weight alongside frequency rather than treating it as a pure tiebreaker.\n\n## Parent\n\nbaseline\n\n## Changes\n\nModify `src/shared/eviction_policy.rs` from the baseline. Only `ClassicLRU::eviction_candidate_index` changes; everything else — the struct definitions, other policy impls, `EvictionPolicyKind`, and all tests — is preserved.\n\nReplace the current body:\n\n```rust\nfn eviction_candidate_index(&self, items: &[Item<K, V>]) -> usize {\n    items\n        .iter()\n        .enumerate()\n        .min_by_key(|(i, item)| (item.lfu_counter, *i))\n        .map(|(i, _)| i)\n        .unwrap_or(0)\n}\n```\n\nwith a product-score body:\n\n```rust\nfn eviction_candidate_index(&self, items: &[Item<K, V>]) -> usize {\n    items\n        .iter()\n        .enumerate()\n        .min_by_key(|(i, item)| item.lfu_counter.saturating_add(1) * (*i + 1))\n        .map(|(i, _)| i)\n        .unwrap_or(0)\n}\n```\n\nScoring rationale: in ClassicLRU, `get_from_position` moves the accessed item to the back of the vector, so `position` encodes recency — position 0 is the least recently used slot, position `len-1` is the most recently used. The product `(lfu_counter + 1) * (position + 1)` gives a low score (high eviction priority) to items that are both cold (low counter) and old (low position), and a high score (low eviction priority) to items that are both hot and recent. Critically, a newly inserted item at position `len-1` with `lfu_counter = 0` receives score `len`, which protects it proportionally to the bucket's occupancy — avoiding the failure mode where a brand-new item is immediately the weakest LFU candidate despite being brand new. A stale item with high historical frequency (high counter, low position) receives score `(lfu+1) * 1`, making it vulnerable once it stops being accessed even if its counter is large.\n\nThe `get_from_position` access discipline (move-to-back, increment `lfu_counter`) is unchanged from baseline ClassicLRU.\n"
}
```

### 04:39:29 chat

Three hypotheses written to the output directory. Here's the cohort summary:

---

**H01 — Stash + LFU composition** (`parent: cycle/0001/v001`)
Carries v001's LFU-weighted eviction forward and adds v003's second-chance stash on top of it. The two mechanisms are complementary: LFU reduces *wrongful* evictions, and the stash recovers items that *still* slip through. The stash-hit path also credits the accessed item's `lfu_counter` on promotion, so re-promoted items are immediately protected from re-eviction.

**H02 — LFU counter saturation cap** (`parent: cycle/0001/v001`)
A single-line change to v001's `get_from_position`: cap `lfu_counter` at 16 with `saturating_add(1).min(16)`. Without a cap, a formerly-popular item accumulates a counter so large that newly-warming items can never displace it, even when the workload's hot set shifts. The cap preserves frequency discrimination among cold items (0–3) and hot items (4–15) while ensuring the signal stays fresh.

**H03 — Frequency × recency product score** (`parent: baseline`)
Replaces ClassicLRU's lexicographic `(lfu_counter, position)` eviction criterion with `(lfu_counter + 1) * (position + 1)`. In the move-to-back discipline, `position` encodes recency, so the product gives both dimensions proportional influence. A cold-and-old item (low counter, low position) scores lowest; a hot-and-recent item scores highest. Unlike pure LFU, a newly inserted item at the back of the vector receives score `len`, protecting it proportionally to bucket occupancy rather than making it the immediate weakest candidate.


---

- finished: 2026-06-04T04:39:29+00:00
- duration: 262.58s
- status: ok
- input_tokens: 16
- output_tokens: 13575
- cache_read_tokens: 434247
- cache_write_tokens: 38985
- cost_usd: 0.481558
