# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0003\hypotheses
- allow_edits: True
- started: 2026-07-19T09:00:40+00:00

## timeline

### 09:00:45 thinking

Let me read all the context files to understand the current state of the research and then generate 3 distinct hypotheses for cycle 0003.

Let me start by reading all the relevant files in parallel.

### 09:00:46 tool: Bash

```json
{
  "command": "ls \"C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0002\\variants\"",
  "description": "List cycle 0002 variants"
}
```

### 09:00:46 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0002\\cycle-winners.json"
}
```

### 09:00:47 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0002\\cycle-insight.md"
}
```

### 09:00:47 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\leaderboard.md"
}
```

### 09:00:47 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles-ledger.jsonl"
}
```

### 09:00:50 thinking

Now let me read the variant overlays and insights from cycle 0002:

### 09:00:51 tool: Glob

```json
{
  "pattern": "**",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0002\\variants"
}
```

### 09:00:51 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cross-cycle-insights.md"
}
```

### 09:00:54 thinking

Now let me read the overlay files for each variant in cycle 0002 to understand what each variant did:

### 09:00:54 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0002\\variants\\v001\\insight.md"
}
```

### 09:00:55 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0002\\variants\\v002\\insight.md"
}
```

### 09:00:55 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0002\\variants\\v003\\insight.md"
}
```

### 09:00:58 thinking

Now let me read the actual overlay source code for each variant to understand the implementation details:

### 09:00:58 chat

Now let me read the actual overlay files to understand the implementation details:

### 09:01:00 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0002\\variants\\v001\\attempts\\01\\overlay\\src\\shared\\item.rs"
}
```

### 09:01:01 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0002\\variants\\v001\\attempts\\01\\overlay\\src\\shared\\eviction_policy.rs"
}
```

### 09:01:01 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0002\\variants\\v001\\attempts\\01\\overlay\\src\\dash\\segments\\dash_segment.rs"
}
```

### 09:01:04 thinking

Now let me read the cycle 0002 v002 overlay (frequency gate) and v003 overlay (stash bypass) to understand exactly what was implemented:

### 09:01:05 chat

Now let me read the v002 and v003 overlays to understand the frequency gate and stash-bypass implementations:

### 09:01:07 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0002\\variants\\v002\\attempts\\01\\overlay\\src\\dash\\segments\\dash_segment.rs"
}
```

### 09:01:07 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0002\\variants\\v003\\attempts\\02\\overlay\\src\\dash\\segments\\dash_segment.rs"
}
```

### 09:01:11 thinking

Now let me also read the baseline to understand the foundation:

### 09:01:12 chat

Now let me read the baseline source to understand the foundation:

### 09:01:12 tool: Glob

```json
{
  "pattern": "**/*.rs",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\baseline"
}
```

### 09:01:14 tool: Glob

```json
{
  "pattern": "**",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\baseline"
}
```

### 09:03:00 thinking

The baseline doesn't have its own source files in this directory - it must be in the main project source. Let me look at the v001/v002/v003 overlays more carefully and also understand what a stash-bypass + MRU-end combination would look like, and plan the three hypotheses.

Based on what I've read:

**Cycle 0001 Winners:**
- `cycle/0001/v001` - MRU-end insertion (`InsertPosition::Back`)
- `cycle/0001/v003` - Timestamp removal from Item struct (metadata-size reduction)
- `cycle/0002/v001` - Stacking v001 and v003 together delivers the best results (hit-rate=0.6787, metadata-size=24, per-op-time=100), combining MRU-end insertion with the timestamp removal
- `cycle/0002/v002` - Frequency gating with a threshold of 2 on top of v001 shows comparable hit-rate (0.6797) but carries higher metadata overhead at 40 bytes due to the LFU counter
- `cycle/0002/v003` - Stash bypass using Front insertion (not the MRU-end change) achieves 0.6667 hit-rate with lean metadata

The cross-cycle guidance suggests stacking v003's stash-bypass logic on top of v001's combined baseline since that pairing hasn't been tested yet and could yield better results, while also exploring a higher frequency gate threshold (≥3) or making it conditional on cache fullness to reduce the metadata burden.

For cycle 0003, I need three distinct hypotheses where at least one builds on a proven winner from the previous cycles. The first experiment should stack stash-bypass on top of the v001 baseline that already includes MRU-end and timestamp removal.

The second hypothesis tests whether raising the frequency gate threshold to ≥ 3 can solve the cascade churn issue while preserving the hit-rate gains. Both of these are parented to v001, so I need to reconsider the structure to ensure proper diversity across the three experiments. to stash-bypass but applied at the probing level instead), LFU retry (which timed out before), combining stash-bypass with MRU-end logic using v003 as the parent, adjusting how demoted items land in the target, resetting the LFU counter on demotion to clear stale frequency signals, or introducing a two-hop stash mechanism for items about to be evicted. The clearest untested paths forward are stacking stash-bypass directly on v001 and running a frequency gate calibration with a threshold of 3 or higher.

Now I'm considering some additional novel approaches: promoting items to the front of probing instead of the back to make them harder to evict immediately, resetting the LFU counter when items demote back to target so they don't re-promote too quickly through the frequency gate, using LFU within stash buckets to evict the least-frequently-accessed items rather than positional ones, or testing whether stash-bypass and frequency-gate actually work well together on v001. The cycle insights suggest that combining stacking, stash-bypass, and a tuned frequency gate together is the natural direction forward.

Given the constraint of three hypothesis slots and the requirement that at least one must cite a cross-cycle winner while preferring non-baseline incumbents, I'm planning to allocate them as: H01 as a child of v001 with stash-bypass (the most critical experiment), and H02 as a child of v001 with a frequency gate that includes a fullness-conditional guard.

For H03, I'm exploring whether there are other structural optimizations beyond what's already been tried. The current Item structure at 24 bytes breaks down as key (8) + value (8) + lfu_counter (8), and while reducing the counter to u32 could save 4 bytes, alignment concerns make that tricky. I'm also considering whether the stash-bypass variant's use of Front insertion instead of Back insertion might be worth revisiting as a novel direction.

Another angle is rethinking the eviction policy specifically for the probing tier—instead of always evicting the LRU item when probing fills up, what if we evict based on lfu_counter to protect frequently-accessed items in that tier? The challenge is that the eviction policy is currently configured at the segment level for all buckets, so isolating it to just one tier would require more substantial changes to the codebase.

I'm also considering a few other approaches: using dual probing buckets to check multiple positions before evicting, implementing frequency-biased stash promotion to move high-frequency stash items directly to probing, or introducing a probing bypass mechanism where target's hottest items stay at the MRU end of target instead of evicting probing residents when probing is full. This last idea about probing bypass seems promising since it hasn't been explored yet and could reduce unnecessary churn in the probing tier. if an item gets demoted from probing back to target, I should decay its frequency counter so it doesn't immediately get promoted again on the next access. This breaks the cycle where frequently-accessed items ping-pong between tiers.

This would be a modification to the existing v002 approach rather than a standalone variant. Let me map out the three main candidates: the stash-bypass option layered on v001, this frequency decay approach as a child of v002, and I should evaluate which one actually addresses the core churn problem most effectively. The fullness condition gates promotion only when the probing bucket has room, which should eliminate the cascade-eviction latency without sacrificing hit-rate gains for items that fit without displacement. I'm also noticing that H02 and H03 both refine the frequency gate through different mechanisms—one raises the threshold, the other adds a fullness check—so they're distinct enough as separate hypotheses. But now I need to verify that all three hypotheses properly cite a cross-cycle winner as their parent, since that's a requirement. So v002 is marked as a cycle winner within cycle 0002, but it didn't make the cross-cycle winners list because its metadata-size of 40 bytes hurt its overall score compared to v001 and v003, which both kept it at 24 bytes. That larger metadata footprint is what disqualified it from being a cross-cycle winner despite winning within its own cycle.

The issue is that v002 inherited from `cycle/0001/v001` rather than `cycle/0002/v001`, so it never got the timestamp removal that would've brought it down to 24 bytes. If I want to properly test the frequency gate with a ≥3 threshold, I should parent it on `cycle/0002/v001` instead, which already has the MRU-end optimization and no timestamp.

Now I'm laying out two competing hypotheses: one tests stash-bypass logic by checking if the target bucket is full before promoting, routing around saturation to reduce cascade evictions; the other tests frequency gating at a higher threshold to filter out low-frequency items from promotion. dash_segment.rs`: add a probing-vacancy guard to the frequency gate so that early promotion only triggers when `!core.buckets[probing_idx].is_full()`, preventing cascade evictions from starting in the first place. This targets the same per-op-time regression as H02 but through a different mechanism—gating on available space rather than raising the threshold.

But wait, H02 and H03 are both tweaking the frequency gate, just in different ways. Let me think about whether H03 should explore something entirely different instead.

I could try a few other angles: keeping hot items pinned at the target tier's MRU when probing is full to avoid cascades altogether, giving stash items a second chance by promoting high-frequency ones directly to target before they're discarded, or capping the LFU counter to prevent stale items from hoarding inflated counts.

The stash eviction angle is worth exploring more carefully—when an item gets evicted from a full stash bucket, if it has a meaningful access history, promoting it to target instead of throwing it away could preserve valuable frequency information. The tricky part is that the bucket's `put_at` method probably doesn't have visibility into the item's LFU state, so I'd need to hook into the eviction logic at a higher level where I can make that decision. ading cascades. A cleaner approach would be to prevent cascades entirely when probing is full—instead of evicting the LRU item from probing, I'd just leave the target item in place at the MRU end of its bucket. This avoids the complexity of chained evictions while still keeping hot items accessible.

The trade-off is interesting though: if I suppress promotion when probing is full, hot items stay trapped in target longer, which could either help or hurt depending on whether they're blocking better candidates from entering probing or whether they're actually getting more hits there anyway.

I should test this hypothesis along with a couple others I'm considering. Before diving in, let me check if there are other relevant files like the bucket traits or segment core implementation that might inform the approach.

### 09:03:00 chat

Let me also read the v003 overlay files for cycle 0002 (attempts 02 which was the final successful attempt):

### 09:03:01 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0002\\variants\\v003\\attempts\\02\\overlay\\src\\shared\\traits\\bucket.rs"
}
```

### 09:03:02 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0002\\variants\\v003\\attempts\\02\\overlay\\src\\shared\\eviction_policy.rs"
}
```

### 09:03:04 thinking

Now I have a comprehensive picture. Let me also check the output directory to make sure it exists:

### 09:03:05 chat

Now I have all the context I need. Let me check the output directory exists and then write the three hypotheses:

### 09:03:07 tool: Glob

```json
{
  "pattern": "**",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\hypotheses"
}
```

### 09:05:55 thinking

The output directory exists. Now I'll write the three hypothesis files.

Let me design the three hypotheses:

**H01: Stack stash-bypass on cycle/0002/v001** (cross-cycle winner)
- Parent: `cycle/0002/v001` (best scored winner: MRU-end + timestamp-free)
- Change: Add target-fullness check in the stash hit path. If `core.buckets[target_idx].is_full()`, redirect to `promote_to_probing` instead of `promote_to_target`. Only modifies `src/dash/segments/dash_segment.rs`.
- Claim: Layering saturation-aware stash routing onto the MRU-end + timestamp-free base will improve hit rate by preventing cascade evictions triggered on each stash hit.

**H02: Frequency gate at threshold ≥ 3 on cycle/0002/v001** (cross-cycle winner)
- Parent: `cycle/0002/v001`
- Change: In the non-hottest target branch of `promotion_get`, increment `lfu_counter` and promote early only when `lfu_counter >= 3`, matching the mechanism from cycle/0002/v002 but with a stricter threshold.
- Claim: Raising the frequency-gate promotion threshold from ≥ 2 to ≥ 3 will reduce cascade-eviction frequency enough to eliminate v002's per-op-time regression while retaining most of its hit-rate gain.

**H03: Probing-vacancy guard on frequency-gate (cycle/0002/v002)**
- Parent: `cycle/0002/v002`
- Change: In the non-hottest target branch where `lfu_counter >= 2` triggers `promote_to_probing`, add a guard that checks whether the probing bucket has spare capacity before firing the promotion, ensuring the path never cascades.
- Claim: Adding a probing-vacancy pre-check to the frequency gate will suppress cascade evictions entirely.

These three strategies represent distinct approaches: saturation-aware stash promotion on v001, frequency-based early promotion with a recalibrated threshold on v001, and frequency-based early promotion with a vacancy guard on v002. Two of them reference cross-cycle winners, while H03 builds on a cycle winner that didn't achieve cross-cycle success, which aligns with the requirement that at least one strategy must cite a cross-cycle winner. So the constraint is permissive — at least one hypothesis needs a cross-cycle winner as parent, but the others can branch from baseline or any prior variant. All three of my hypotheses should be valid then. But I'm noticing something odd: cycle/0002/v003 is using `InsertPosition::Front` throughout, which contradicts what I'd expect if it was built on cycle/0001/v001 that had `InsertPosition::Back`. Let me check the task files to understand what actually happened.

Looking at the rejection and attempt records, it seems v003 was re-attempted and the evaluated version uses `InsertPosition::Front`, meaning it's baseline plus stash-bypass, not v001 plus stash-bypass. That means the pairwise experiment the cycle insight recommended — combining v001's stacked approach with v003's stash-bypass logic — hasn't been tested yet. This should be H01: taking cycle/0002/v001 as the parent and layering the stash-bypass on top while preserving `InsertPosition::Back`. For H03, I could explore something entirely different like gating the frequency mechanism on probing vacancy instead.

Now I'm weighing whether H03 should branch from v002 or v001. If it uses v002 as the parent, it inherits the frequency gate at threshold ≥ 2 and I'd add the vacancy guard to test whether that fixes the cascade cost. If it uses v001, I'd add both the frequency gate and the vacancy check together. Since H02 already tests threshold ≥ 3 on v001, H03 could take a different path.

I'm also realizing that cycle/0002/v002 still has the timestamp field in the Item struct because it only overrides the segment code, not the item definition itself—it inherited that from cycle/0001/v001, which explains the 40-byte metadata size.

So for H03, I should parent on cycle/0002/v001 instead and add both the frequency gate at threshold ≥ 2 and the vacancy guard together as a complete package. This way H02 tests threshold calibration at ≥ 3, while H03 tests the vacancy-conditional approach at ≥ 2, giving me two distinct mechanisms to compare.

Actually, there's another angle worth exploring: instead of relying on the frequency gate, I could suppress the cascade entirely when the probing bucket is full. When the target's hottest item would normally trigger a cascade by promoting to a full probing bucket, I'd just keep it in place at the MRU position instead. I'm implementing this in the promotion logic...

Writing promotion logic...

The key insight is that this targets the probing-saturation cascade specifically, which is different from the stash-saturation issues H01 and H02 address. If the workload consistently shows cascade problems, it suggests the probing tier is frequently full, making this a viable hypothesis to test.

Now I'm considering whether all three hypotheses should branch from the same parent variant or if I should use different bases. The instructions require at least one cross-cycle winner as a parent, but I could test one hypothesis on a different successful variant to see if the mechanism behaves differently in isolation.

After weighing the options, I'm settling on a clean approach: all three hypotheses will branch from cycle/0002/v001 since it's the best-performing variant. H01 adds the stash-bypass mechanism, H02 introduces a frequency gate with a threshold of 3 or higher, and H03 prevents the target-to-probing cascade when the probing queue is full. This gives me three distinct experimental directions on the same proven base, and they all satisfy the cross-cycle winner requirement.

Now I'm drafting the actual hypothesis documents.

For H01, I'm modifying the stash hit handling in the promotion logic to check if the target bucket is full before promoting. If it is, I route directly to the probing queue instead, which avoids the cascade eviction that normally happens on a stash hit.

The rest of the codebase stays the same—MRU-end insertion and the existing branching logic remain untouched. This variant tests whether stash bypass works as well on the stronger MRU baseline as it did on the LRU baseline, and whether the combined approach improves both hit rate and tail latency.

For H02, I'm raising the frequency-gate threshold from two to three accesses to reduce cascade evictions and eliminate the per-operation regression from the previous variant while keeping most of its hit-rate improvements. The change involves incrementing a counter on non-hottest hits and adding early-promotion logic...

Writing promotion logic... Writing the conditional logic... Writing the promotion logic... ency, but the trade-off is that existing probing residents stick around longer. The real question is whether that stability gain justifies delaying some hot items from getting promoted to probing.

Now I'm checking the `is_hottest` logic in the promotion path—specifically whether comparing the item index directly against the promotion candidate index actually identifies the hottest item correctly.

The `probing_idx` is already calculated earlier as `(target_idx + 1) % core.segment_size`, so I can reference it directly in H03 without recomputing it.

Looking at the template format again, the Changes section uses prose rather than code fences, though the spec does allow free-form content, so keeping code blocks should be fine.

Now I'm weighing whether H03's approach of holding items in place in the probing bucket is better than the alternative strategies I've been considering. I'm realizing the core issue: if probing stays full, holding the hottest item in target means it never gets promoted at all, which defeats the purpose. The hottest item would just keep checking `is_hottest` on each access, staying in target indefinitely while probing residents remain untouched. This only helps if items eventually get demoted from probing through other mechanisms, but that's not guaranteed to happen quickly enough. Under a Zipfian workload, hot items quickly reach probing and stay there, while medium-hot items cycle through target and occasionally trigger cascades into probing. If probing fills up frequently, H03 would block new items from entering via the positional hottest path, which seems counterproductive since cascades were the mechanism getting items in. Instead of suppressing promotion, I could compare the target-hottest item against the LRU probing resident and evict whichever has a lower lfu_counter, choosing the better victim based on actual coldness rather than just position.

Writing comparison logic... The issue is that in the current version, all items start with `lfu_counter=0`, so comparing counters doesn't actually differentiate between items—everything gets promoted, which defeats the purpose. I need a simpler approach that works within the existing constraints. What if instead of discarding evicted stash items, I give them a second chance by trying to re-promote them back to their target bucket before they're truly lost? Modifying the stash insertion strategy to always use Front position for demotions would make newly demoted items the first candidates for eviction, keeping longer-resident stash items stable. This is worth testing as an independent variable from the target insertion position to see if it improves performance depending on the workload. MRU items at the back stay protected longer. This approach would evict the most recently demoted items first, which could be better if those items are genuinely cold, or worse if they're still useful. It's speculative though, and probably not worth testing.

Let me shift focus to the put path instead—when items are inserted into the cache, they go straight to stash without any frequency awareness. If we knew at insertion time that an item would be frequently accessed, we could start it at a higher tier, but we don't have that information upfront. The workload would need to provide it.

A more practical angle is capping the LFU counter to prevent stale frequency data from dominating. If the counter grows unboundedly, an item that was popular long ago but is now cold keeps its high count. By capping it at something like 8 or 16, older frequency signals get overtaken by more recent activity, making the frequency metric more temporally sensitive. I'd implement this in the frequency gate path where the counter is incremented, so items can't accumulate unbounded counts.

But there's a cleaner direction I'm considering: a two-tier stash structure. Instead of a single stash tier, I could split it into a cold region (LRU-evicted) and a warm region for items that've been accessed once from stash. This adds another tier between target and true eviction, though it would require new data structures to track the promotion logic.

Looking at what I can actually modify, the overlay scope allows changes to most source files except the gate-integrity surface. That means I can edit segment logic, item structs, and eviction policies, but I need to check what's available in segment_core to understand the existing data structures before committing to this approach.

I'm considering a simpler alternative: when an item gets demoted from the target bucket back to stash via `demote_from_normal_bucket`, I could reset its `lfu_counter` to zero so it starts fresh when accessed again, preventing stale frequency data from skewing its behavior on re-entry.

But that only matters if there's a frequency gate in place—on v001 where all counters are zero anyway, it's a no-op unless the implementer adds that gate separately. I'm going in circles trying to find a mechanism that works cleanly on v001 alone, so let me settle on something concrete rather than keep iterating. The issue is that ClassicLRU never increments the lfu_counter, so all items stay at zero—this approach won't work with the parent v001. I could add counter increment logic like v002 does, but that's mixing two changes at once. Instead, I'll stick with the frequency-vacancy guard as H03 since it's mechanically distinct from the other hypotheses and actually implementable. The plan is to modify the promotion logic in the segment code to add a frequency gate in the non-hottest target branch, using the same threshold and counter increment pattern from v002. All three hypotheses reference cycle/0002/v001, which satisfies the cross-cycle winner requirement. Now I'm ready to write out the hypothesis files.

### 09:05:55 chat

Now I have all the context needed to write three well-grounded, distinct hypotheses. Let me create the output files:

### 09:06:07 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\hypotheses\\H01.md",
  "content": "# H01\n\nLayering saturation-aware stash routing (bypass a full target to probing) onto the MRU-end plus timestamp-free base will improve hit rate by eliminating the cascade eviction triggered on every stash hit when target is saturated.\n\n## Parent\n\ncycle/0002/v001\n\n## Changes\n\nModify `promotion_get` in `src/dash/segments/dash_segment.rs`. In the stash hit branch — the `if let Some(item_idx) = core.stash_buckets[stash_idx].get_position(key)` block — after removing the item with `core.stash_buckets[stash_idx].remove_at(item_idx)`, add a fullness check on `core.buckets[target_idx]` before choosing the promotion path:\n\n```rust\nif core.buckets[target_idx].is_full() {\n    // Target saturated: skip it and land directly in probing.\n    promote_to_probing(core, data, insert_pos);\n    let item_new_idx = core.buckets[probing_idx]\n        .get_position(key)\n        .expect(\"promote_to_probing must leave key in probing (stash bypass)\");\n    return Some(&core.buckets[probing_idx].get_items()[item_new_idx]);\n} else {\n    // Target has capacity: use the normal path.\n    promote_to_target(core, data, insert_pos);\n    let item_new_idx = core.buckets[target_idx]\n        .get_position(key)\n        .expect(\"promote_to_target must leave key in target\");\n    return Some(&core.buckets[target_idx].get_items()[item_new_idx]);\n}\n```\n\n`InsertPosition::Back` is kept throughout. No other branches are modified. No changes to `item.rs` or `eviction_policy.rs` are needed (cycle/0002/v001 already removed the timestamp field).\n\n## Rationale (optional)\n\ncycle/0002/v003 proved stash bypass improves hit rate but was built on the LRU-end baseline (InsertPosition::Front). cycle/0002/v001 is the stronger base — it already has MRU-end insertion and a compact 24-byte Item. These two improvements have never been tested together; this variant closes that gap and tests whether the bypass benefit is additive on the best-in-class foundation.\n"
}
```

### 09:06:21 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\hypotheses\\H02.md",
  "content": "# H02\n\nRaising the frequency-gate promotion threshold from two to three accesses will reduce cascade-eviction frequency enough to recover cycle/0002/v002's per-op-time regression while retaining most of its hit-rate gain.\n\n## Parent\n\ncycle/0002/v001\n\n## Changes\n\nModify `promotion_get` in `src/dash/segments/dash_segment.rs`. In the non-hottest target branch — the `else` block inside `if let Some(item_idx) = core.buckets[target_idx].get_position(key)` where `is_hottest` is false — replace the current single-line `get_from_position` intra-promote return with counter tracking and threshold-gated early promotion:\n\n```rust\n} else {\n    core.buckets[target_idx].get_items_mut()[item_idx].lfu_counter += 1;\n    let counter = core.buckets[target_idx].get_items()[item_idx].lfu_counter;\n    if counter >= 3 {\n        debug!(\"promotion_get: key={key:?} found in target item_idx={item_idx} lfu_counter={counter} (frequency-promote to probing={probing_idx})\");\n        let data = core.buckets[target_idx].remove_at(item_idx);\n        promote_to_probing(core, data, insert_pos);\n        let item_new_idx = core.buckets[probing_idx]\n            .get_position(key)\n            .expect(\"promote_to_probing must leave key in probing\");\n        return Some(&core.buckets[probing_idx].get_items()[item_new_idx]);\n    }\n    debug!(\"promotion_get: key={key:?} found in target item_idx={item_idx} lfu_counter={counter} (intra-promote)\");\n    return Some(core.buckets[target_idx].get_from_position(item_idx));\n}\n```\n\nThe only difference from cycle/0002/v002's gate is `>= 3` instead of `>= 2`. `InsertPosition::Back` is kept throughout. No changes to `item.rs` or `eviction_policy.rs` (cycle/0002/v001 already removed the timestamp field; lfu_counter is already present in Item).\n\n## Rationale (optional)\n\ncycle/0002/v002 showed the frequency signal is real — the workload contains re-accessed non-hottest items — but threshold ≥ 2 fires broadly enough that cascade churn erases most of the latency budget. The cycle insight explicitly nominated ≥ 3 or ≥ 4 as the next calibration point. This variant tests ≥ 3 on the best-scored base (cycle/0002/v001, which cycle/0002/v002 never had access to since v002 was parented on cycle/0001/v001 and retained the 40-byte Item).\n"
}
```

### 09:06:34 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\hypotheses\\H03.md",
  "content": "# H03\n\nAdding a probing-vacancy pre-check to frequency-gated promotion will preserve the frequency signal while eliminating the cascade cost entirely by ensuring gate-fires only occur when probing has spare capacity.\n\n## Parent\n\ncycle/0002/v001\n\n## Changes\n\nModify `promotion_get` in `src/dash/segments/dash_segment.rs`. In the non-hottest target branch — the `else` block inside `if let Some(item_idx) = core.buckets[target_idx].get_position(key)` where `is_hottest` is false — add counter tracking and a vacancy-gated early-promotion path that only fires when `probing_idx` has room:\n\n```rust\n} else {\n    core.buckets[target_idx].get_items_mut()[item_idx].lfu_counter += 1;\n    let counter = core.buckets[target_idx].get_items()[item_idx].lfu_counter;\n    let probing_has_space = !core.buckets[probing_idx].is_full();\n    if counter >= 2 && probing_has_space {\n        debug!(\"promotion_get: key={key:?} found in target item_idx={item_idx} lfu_counter={counter} probing={probing_idx} has space (vacancy-gated frequency-promote)\");\n        let data = core.buckets[target_idx].remove_at(item_idx);\n        promote_to_probing(core, data, insert_pos);\n        let item_new_idx = core.buckets[probing_idx]\n            .get_position(key)\n            .expect(\"promote_to_probing must leave key in probing\");\n        return Some(&core.buckets[probing_idx].get_items()[item_new_idx]);\n    }\n    debug!(\"promotion_get: key={key:?} found in target item_idx={item_idx} lfu_counter={counter} (intra-promote)\");\n    return Some(core.buckets[target_idx].get_from_position(item_idx));\n}\n```\n\nThe gate threshold stays at `>= 2` (same as cycle/0002/v002), but the second condition `!core.buckets[probing_idx].is_full()` must also be true. When probing is full the gate is silenced regardless of counter value; the item receives the normal intra-bucket touch via `get_from_position`. `InsertPosition::Back` is maintained throughout. No changes to `item.rs` or `eviction_policy.rs`.\n\n## Rationale (optional)\n\ncycle/0002/v002's latency regression was attributed to cascade evictions triggered each time the frequency gate fired into a full probing bucket — each gate-fire displaced a probing resident and cascaded downstream. Raising the threshold (H02) reduces gate-fire rate but still allows cascades when they do fire. This variant instead eliminates the cascade at its source: gate-fires are structurally guaranteed to find room in probing, so no eviction cascade can occur. The trade-off is that items whose frequency qualifies them for promotion must wait until probing has vacancy, which may delay their residency there. Whether the zero-cascade guarantee outperforms the higher-threshold approach (H02) is the question this variant answers alongside it.\n"
}
```

### 09:06:45 chat

I've written all three hypothesis files to `C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0003\hypotheses\`. Here's a summary of what was produced:

---

**H01 — Stash-bypass stacked on cycle/0002/v001** (`hit-rate` target)
*Parent: `cycle/0002/v001`*
Adds a target-fullness check in the stash hit path of `promotion_get`: if `core.buckets[target_idx].is_full()`, redirect to `promote_to_probing` instead of `promote_to_target`. This is the direct recommendation from cycle/0002's insight — stash-bypass has only been tested on the LRU-end baseline (v003), never on the MRU-end + timestamp-free base (v001). The change is isolated to the stash hit branch and preserves `InsertPosition::Back` throughout.

**H02 — Frequency gate at threshold ≥ 3 on cycle/0002/v001** (`hit-rate` + `per-op-time` target)
*Parent: `cycle/0002/v001`*
Copies the frequency-gate mechanism from cycle/0002/v002 exactly, except the threshold is raised from `>= 2` to `>= 3`. This is the calibration experiment the cycle insight nominated by name. Running it on v001 (rather than v002's parent, cycle/0001/v001) also removes the 40-byte metadata penalty that hurt v002's score.

**H03 — Vacancy-gated frequency promotion on cycle/0002/v001** (`per-op-time` target)
*Parent: `cycle/0002/v001`*
Adds the same frequency gate as v002 (threshold ≥ 2) but guards it with `!core.buckets[probing_idx].is_full()` — the gate only fires when probing has spare capacity, structurally guaranteeing zero cascade cost on every gate-fire. Distinct from H02 (which lowers fire rate via higher threshold but still allows cascades) and from H01 (which operates on the stash path, not the target non-hottest path).


---

- finished: 2026-07-19T09:06:46+00:00
- duration: 365.34s
- status: ok
- input_tokens: 14
- output_tokens: 20347
- cache_read_tokens: 657909
- cache_write_tokens: 76505
- cost_usd: 0.791024
