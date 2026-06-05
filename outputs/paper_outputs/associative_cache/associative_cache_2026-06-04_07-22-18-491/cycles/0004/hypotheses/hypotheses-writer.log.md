# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0004/hypotheses
- allow_edits: True
- started: 2026-06-04T04:55:36+00:00

## timeline

### 04:55:40 thinking

Let me read all the context files to understand the current state of the research and then generate 3 hypotheses for cycle 0004.

### 04:55:40 chat

I'll read all the context files in parallel to build a complete picture before writing the hypotheses.

### 04:55:41 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0003/variants/*/eval/fitness-vector.json"
}
```

### 04:55:42 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0003/cycle-winners.json"
}
```

### 04:55:43 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0003/cycle-insight.md"
}
```

### 04:55:43 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/leaderboard.md"
}
```

### 04:55:44 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles-ledger.jsonl"
}
```

### 04:55:44 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cross-cycle-insights.md"
}
```

### 04:55:46 thinking

Now let me read the variant overlays from cycle 0003 to understand the implementation details.

### 04:55:46 chat

Now let me read the variant overlays and insights from cycle 0003:

### 04:55:47 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0003/variants/*/insight.md"
}
```

### 04:55:48 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0003/variants/*/attempts"
}
```

### 04:55:50 thinking

The glob for variant subdirectories didn't work. Let me try a different approach and look at the variant directories directly.

### 04:55:51 tool: Bash

```json
{
  "command": "ls /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0003/variants/",
  "description": "List cycle 0003 variant directories"
}
```

### 04:55:53 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0003/variants/**"
}
```

### 04:55:56 thinking

Now let me read the overlay files for v003 (the winner) to understand its implementation, plus read the insight files and the baseline.

### 04:55:57 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0003/variants/v003/attempts/01/overlay/src/associative_cache/associative_cache_bucket.rs"
}
```

### 04:55:58 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0003/variants/v003/attempts/01/overlay/src/shared/eviction_policy.rs"
}
```

### 04:55:58 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0003/variants/v003/insight.md"
}
```

### 04:55:59 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0003/variants/v001/insight.md"
}
```

### 04:55:59 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0003/variants/v002/insight.md"
}
```

### 04:56:01 thinking

Now let me also read the baseline files to understand the starting point.

### 04:56:02 chat

Now let me read the baseline files:

### 04:56:02 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/inputs/subjects/associative_cache/baseline/**"
}
```

### 04:56:06 thinking

The baseline doesn't have a source directory listed in those files. Let me look at the baseline source files and the cycle 0002/v001 winner (which was the parent for cycle 0003 v003):

### 04:56:07 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0002/variants/v001/attempts/**"
}
```

### 04:56:08 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0003/variants/v003/task.md"
}
```

### 04:59:49 thinking

Now I have a thorough understanding of the research history. Let me synthesize what I know:

1. **Baseline**: ClassicLRU eviction, no stash, hit-rate=0.661

2. **Cycle 1 winner (cycle/0001/v001)**: Added LFU-weighted eviction (activated the existing `lfu_counter`), hit-rate=0.723

3. **Cycle 2 winner (cycle/0002/v001)**: Composed LFU eviction with second-chance stash (`stash_max_size = bucket_size/4`, FIFO ordering, +1 promotion credit), hit-rate=0.741
   
4. **Cycle 3 winner (cycle/0003/v003)**: Doubled the promotion credit to +2, achieving hit-rate=0.742

The current best configuration is `cycle/0003/v003` at 0.7424 hit-rate, which combines LFU-weighted eviction in the primary buffer with a FIFO second-chance stash sized at a quarter of the bucket capacity, where items promoted from the stash receive two additional credit points.

Looking at what's worked and what hasn't, the LFU foundation proved solid, stash composition helped, and the +2 promotion boost gave us a small win. But I'm seeing mixed signals on the other levers—larger stash caps and LFU ordering within the stash both regressed, while product scoring and counter caps were neutral. The key question now is whether +2 is the sweet spot or if there's more room to push the promotion credit higher, and whether the gains follow a monotonic curve or peak somewhere and reverse.

For cycle 4, I'm testing three hypotheses. First, I'll try bumping promotion credit to +3 to see if the improvement continues or if we've hit a ceiling. Second, I want to explore dynamic promotion credit that scales based on how many times an item has cycled through the stash—this could capture the intuition that frequently-recycled items deserve stronger promotion signals. Third, there's still that metadata-size measurement gap I haven't fully characterized, which might be hiding some of the real cost picture.

The tricky part with the dynamic scaling approach is that I'd need to track stash cycle count per item, which means either adding a new field to the `Item` struct or finding a clever way to encode it. Since the baseline overlay only touches `associative_cache_bucket.rs` and `eviction_policy.rs`, I could extend it to include `item.rs` as an overlay file to add the cycle-count tracking field. Actually, I could scale the promotion credit dynamically based on the item's lfu_counter when it gets evicted—items with higher counts that ended up in the stash are worth more effort to save. Or I could give bigger credits to items that have lingered in the stash longer. But the cleanest approach is probably tracking how many stash cycles each item has been through with a counter field, then multiplying the base credit by that count when promoting. I'm considering two concrete hypotheses to test next. The first is straightforward: bump the promotion credit from +2 to +3 when items hit the stash, to see if the performance gains continue scaling upward. The second is more nuanced—track how many times each item has cycled through demotion and give it a larger credit boost on re-entry if it's been demoted before, using a separate counter array in the bucket to avoid modifying the Item structure itself.

For the second approach, I'd need to track pass counts per key rather than per slot, so a HashMap mapping keys to their demotion history would work better than a parallel Vec.

Actually, there's a simpler path: leverage the existing `lfu_counter` as a proxy for how popular an item was before getting evicted to stash. When promoting back, I can scale the credit based on that counter—something like `2 + (stash_item.lfu_counter / 4)` capped at `+5`—so genuinely popular items get a bigger boost. This avoids adding new tracking structures and makes semantic sense since the counter already captures pre-eviction popularity.

Now exploring a different structural approach for H03... direction entirely. The cross-cycle insights note that eviction-scoring refinements fail on stable workload-a. But what about the primary eviction itself? Currently ClassicLRU evicts the minimum `(lfu_counter, position)`. What if we also protect recently-promoted items from immediate re-eviction by tagging them, and in `eviction_candidate_index`, skip any item that is "newly arrived" (position-wise, the newest item in the buffer)?

Actually, let me look at what hasn't been tried: the `get` path in primary. Currently `get_from_position` in ClassicLRU moves the item to the back and increments its counter. What if instead of always moving to back, we only move to back when the counter exceeds some threshold (so hot items move to back while cold items stay put)?

Actually, let me think of a fundamentally different approach for H03. The cycle insights suggest that all the stash-internal variants have been exhausted, and the promotion-credit lever is near its optimum. What about the initial eviction selection for the stash? Currently, the LFU-minimum from primary goes to the stash. What if instead we evict the second-lowest frequency item to the stash, keeping the absolute minimum as a "sacrifice" to be fully discarded, preserving stash slots for items that have some reuse potential?

That would change the eviction logic to discard the absolute minimum outright and send the second-minimum to the stash, but I'm worried this regresses because we'd be throwing away exactly the items the stash should be catching. Let me think of something cleaner and more distinct.

What if the real issue is that the LFU counter accumulates without bound? Items that were hot in the distant past but are now cold still carry high counter values, skewing the eviction decisions. have high counters, which means they won't be evicted even when they're no longer being accessed. A periodic decay (halving all counters in a bucket on each `put` operation) would help recently hot items compete. This is a classic mechanism to handle LFU counter staleness.

The problem: this requires a sweep of all items on each `put`, which changes the per-operation cost. But the cycle insights say we're on a stable workload where hot-set drift doesn't occur, so this would measure as neutral (same as counter cap). Let me avoid this direction since the cross-cycle insight explicitly says "Eviction-scoring refinements on stable workload-a" keep reading neutral.

**Better H03 alternative:** The `get` function currently returns `None` on a stash miss. What if on a stash miss, we perform a "ghost entry" lookup — record that a key was requested but not found, and when that same key is later `put`, give it an immediate frequency head start as if it had been seen before? This is a ghost-list / history mechanism, but it requires a ghost set data structure, which is a bigger change.

**Simpler H03:** Apply the stash mechanism to `put` upgrades too — when we `put` an item that already exists in primary, what if a write-back gives `+1` extra credit, treating it as a sign of popularity? This is minimal overhead with no structural changes. But wait, `get_from_position` already increments `lfu_counter` on access, so I need to check if this is redundant or if there's a gap in the write path.

Looking at the code, `put` on an existing key calls `get_from_position`, which does increment the counter. So that's already handled. Let me think of a genuinely distinct H03 instead: a two-tier stash with separate warm and cold lanes — items promoted at least once go to the warm stash, first-time evictees go to cold, and on primary eviction we check warm first since those have demonstrated reuse. This is a 2Q or LIRS-lite approach, but it requires significant structural changes. The real question I should focus on is whether +3 is better than +2 for the stash bonus, which is a straightforward empirical probe.

So I'm settling on three concrete hypotheses: H01 raises the promotion credit to +3 to test if +2 was suboptimal; H02 scales the credit dynamically based on the item's `lfu_counter` at eviction time, so items with higher pre-eviction frequency get more credit (ranging from +2 to +5); and I need to verify what the `lfu_counter` state actually is when an item gets evicted to the stash.

When an item lands in the stash, its counter reflects whatever hit history it accumulated—freshly inserted items start at 0, while items with prior hits carry that count forward. The idea is to reward items that showed real popularity before eviction with a bigger credit boost on promotion, so the formula gives +2 base credit plus up to +3 more depending on the counter value. But I'm realizing this approach focuses on the item's original frequency rather than where it sits in the stash queue, which was the actual insight I wanted to explore—so I should pivot H02 to instead scale credit based on the item's position in the stash FIFO.

Older items that have lingered near the front of the queue have survived longer without being found, suggesting they're under sustained miss pressure, so they deserve a slightly higher promotion credit than freshly-stashed items near the back. I can compute this purely from position without adding new tracking: items in the older half of the stash get +3 credit while younger ones get +2. This is clean to test and directly reflects the hypothesis that stash tenure correlates with demand.

For H03, I'm exploring a structurally different approach from the baseline's ClassicLRU eviction. The initial idea of deduplicating stash entries on primary gets is too trivial to matter. Instead, I'm shifting focus to how new items enter the primary buffer—currently they start with `lfu_counter = 0`, but what if I give them a small initial boost to reflect that they've just been inserted?

This is distinct from cycle 2's experiments with counter caps and product scores. The key insight is that during a cold-burst eviction, newly inserted items with counter=0 become immediate candidates for removal. By starting new items at counter=+1, they get a slight reprieve without introducing complex scoring logic.

The implementation would be straightforward: when inserting a brand-new key in the `put` function, initialize the `lfu_counter` to 1 instead of 0. However, I need to verify how the `Item` struct is constructed by the caller and whether `lfu_counter` is always set to 0 initially, since the `put` function receives the item as-is from the caller.

Looking at the broader optimization space, I'm noticing that the most successful changes have been at interface boundaries—like the stash-to-primary promotion mechanism. The primary-to-stash interface is another untapped area: when the primary cache is full and needs to evict, I'm currently sending the LFU minimum to the stash, but maybe I should be more selective about which items actually deserve stash protection versus immediate discard.

More intriguingly, there's the stash-to-discard interface itself. When the stash fills up and needs to make room for a new entry, I'm dropping the oldest item by FIFO order. But what if I used LFU scoring to decide which stash item to evict instead? This is distinct from the cycle 3 approach—that was about using LFU when promoting items *out* of the stash to primary, whereas this would be using LFU to choose which stash item gets dropped entirely when a new one needs to enter.

The problem is, the earlier insight about LFU redundancy inside the stash probably applies here too. Since every item in the stash is already there because it had the lowest frequency in primary, the frequency distribution within the stash is too flat to meaningfully discriminate. So swapping FIFO for LFU-based eviction would likely hit the same wall.

I'm wondering if the real opportunity is something structurally different—what if I don't always push items to the stash when they're evicted from primary?

Actually, there's another angle: what if stash promotions on `get` happen lazily instead of immediately? If an item gets a second hit before being promoted, that filters out one-off accesses and reduces noise. The tradeoff is needing to track hit counts per stash item, which adds overhead.

Or maybe the stash size itself is the lever—cycle-3 validated `bucket_size/4`, but what about going tighter with `bucket_size/8`? The earlier stash-cap doubling failed, so shrinking might actually help filter more aggressively, or it could backfire.

Let me focus on what cycle-3 explicitly recommends: testing +3 promotion credit, a repeat-promotion multiplier based on stash position, and exploring something structurally different from the stash/credit model entirely. For H03, I need to think about what that structural alternative could be.

Eviction scoring refinements won't help here since the workload is stable, so I should look elsewhere. One idea: when a new key gets inserted and forces an eviction to the stash, what if the new item inherits some of the evicted item's frequency counter? The logic is that items arrive in bursts, so a new item colliding with a popular one might be part of the same hot-set. But this is speculative and adds complexity.

A simpler angle might be reconsidering how items move during `get_from_position` operations—maybe not promoting every access is the right call here.

What if I decouple the counter increment from the position update? Keep incrementing the frequency counter on every hit, but only move items toward the back when their counter crosses a threshold. This way, frequently-accessed items naturally drift toward the back (staying hot), while low-frequency items stay put. The eviction logic would still use the counter-position tiebreaker, so high-frequency items remain protected even if they haven't moved recently. The tradeoff is more complex to reason about, but it might reduce unnecessary reordering churn.

Let me explore a cleaner direction instead. I'm realizing the O(n) overhead makes this approach impractical. Let me step back and focus on something simpler: H03 should be a +4 promotion credit variant branching from the cycle/0002/v001 winner, pushing the credit mechanism further. Since H01 already satisfies the requirement to cite a cross-cycle winner, H02 and H03 can explore different parents—H03 could branch from cycle/0002/v001 or even revisit a prior variant to test whether stronger promotion credits consistently outperform the baseline.

Now I'm looking at what hasn't been explored yet. The frequency counter never resets, so items that were once hot stay cached indefinitely even if they've gone cold—this is the classic LFU stagnation problem. Periodic counter aging or decay could help rebalance recency versus frequency, but the workload data shows a stable hot set with no drift, so aging wouldn't actually improve anything since the frequently-accessed items legitimately belong in the cache. Maybe the real opportunity is switching away from ClassicLRU entirely and testing a different eviction policy like TimestampLRU instead.

But looking back at the cycle 1 results, LFU already won decisively—v001 hit 0.723 while the timestamp-based v002 dropped to 0.661. The stash mechanism without LFU (v003) got 0.689, but combining LFU with the stash pushed it to 0.741, so the stash is adding real value on top of the frequency-based approach. What I haven't explored yet is adjusting the inter-stash parameters.

I'm thinking about differentiating the credit boost between read hits and write hits in the stash—maybe read hits get +3 since they're more informative about actual reuse patterns, while write hits stay at +2. That's a concrete, testable variation I could run as H03, branching from cycle/0003/v003. Though that might create too much similarity if all three variants cite the same parent, so I should consider using cycle/0002/v001 as the baseline instead, since that established the LFU plus stash composition at 0.741 before cycle 3 pushed it higher.

For a fundamentally different H03, I'm exploring a Window-TinyLFU approach at the bucket level—partitioning each bucket into a small protected FIFO window for new items and a larger LFU main area, where items from the window compete against the main area's minimum frequency when the window fills up.

But implementing this properly requires separate data structures and eviction logic that's probably too complex for a single variant attempt. Instead, I'll go with a simpler approach: raising the promotion credit to +4 to continue probing whether the gain from +3 continues monotonically. So my three hypotheses are H01 at +3, H02 with dynamic adjustment, and H03 at +4.

The problem is H01 and H03 are too similar—they'll likely collapse together and waste the cohort's discriminative power. What I really need is H03 to go in a completely different direction while staying grounded in the research. The idea: adjust the stash participation rate by only evicting to the stash with 50% probability, like a random coin flip. This reduces noise from cold items piling up and makes the stash a higher-signal waiting area where items that survive the flip get a genuine second chance. It's testable and distinct from the credit-based approaches.

The implementation challenge is finding a deterministic proxy for randomness without adding dependencies—I could use the stash length modulo 2, the item's hash modulo 2, or even the LFU counter parity. But actually, a cleaner approach sidesteps randomness entirely: only send items to the stash if they've been hit at least once (lfu_counter >= 1), discarding truly fresh cold items outright.

This deterministic filter reduces stash noise by filtering out items that never demonstrated reuse potential, freeing slots for items with proven value. I'll update the evict_to_stash logic to check the counter before pushing, making it testable and logically motivated. I'm thinking this should build on the cycle/0003/v003 parent since that's the strongest baseline so far.

Now I'm laying out my three competing hypotheses: raising promotion credit to +3, implementing position-based dynamic credit that adjusts based on how long items have been in the stash, and filtering stash entries to exclude zero-frequency items. Each targets a different mechanism lever—credit magnitude, age-sensitive promotion logic, and the eviction-to-stash boundary—so they're genuinely distinct strategies that all reference the same strong parent. of protection.

**H02: FIFO-position-scaled promotion credit**
- Parent: cycle/0003/v003
- Change: In `associative_cache_bucket.rs`, I'm modifying both the `put` and `get` stash-hit branches to scale the promotion credit based on stash position — items at the FIFO front get +3 while others get +2.
- Claim: By giving older stash items a higher promotion bonus, I can recognize that longer stash survival correlates with higher value and improve hit rate accordingly.

**H03: Stash entry filter on lfu_counter**
- Parent: cycle/0003/v003
- Change: In `evict_to_stash()`, I'm adding a frequency check before pushing evicted items — only items with `lfu_counter >= 1` enter the stash, while zero-frequency items are dropped entirely.
- Claim: Filtering out cold-start items preserves stash capacity for entries that have already demonstrated reuse potential, which should raise hit rate.

Now I'm checking whether these three hypotheses are sufficiently distinct and meet the quality bar — H01 increases the base level, H02 applies age-based scaling at promotion time, and H03 filters by frequency at eviction time, so they're clearly independent approaches.

All three are testable with concrete predictions, grounded in the incumbent v003 implementation, and specific about which functions and conditions change. Before I write the hypothesis files, I need to reconsider the FIFO ordering in H02 — the stash pushes new items at the end and removes from the front, so the oldest item is always at index 0, and when I search for a key's position, I need to account for how age-based credit should scale with that position. I'm second-guessing H02 now—an old item at position 0 in the stash could be legitimately popular or just a one-off access, so giving it extra credit might not be the right move. Instead, I should weight the credit based on how frequently the item was accessed before it got evicted, since that's a better signal of actual popularity than its current position in the stash.

Refining the approach: when a stashed item gets a hit, I'll scale the promotion credit proportionally to its stored frequency counter, capping the bonus so items with higher historical access get more credit to return to primary, while those with minimal history get a smaller boost. Computing the credit formula with this approach gives reasonable scaling: items with zero hits get the base +2 boost, while those with higher frequency counters receive up to +4 total. For H03, filtering to only promote stash items that had at least one prior access keeps slots available for items with proven utility, though this risks underutilizing the stash if most evictions are cold items. that hasn't been accessed yet—it's a borderline case where filtering might prematurely discard something that could benefit from the stash's second-chance mechanism. This uncertainty makes it worth testing empirically.

I'm settling on three hypotheses to explore: raising the promotion credit to a flat +3, using a frequency-proportional credit scaled by the LFU counter, and filtering out zero-counter items at stash entry. Each builds on the current implementation as the baseline, and I'm ready to write out the experimental configurations and output files.

Writing hypothesis files...

### 04:59:55 chat

Now I have a thorough understanding of all three cycles. Let me synthesize before writing:

**Architecture of cycle/0003/v003 (the incumbent):**
- Primary: ClassicLRU (LFU-weighted: `min_by_key(lfu_counter, position)`; hits move item to back + `lfu_counter += 1`)
- Stash: FIFO, cap = `bucket_size/4`
- Promotion credit: `+2` on both `get` and `put` stash-hit paths

**Key findings across cycles:**
- Promotion credit at the stash→primary interface is the last remaining live lever
- +2 beat +1 (small but real); monotone gain beyond +2 untested
- Stash cap enlargement and LFU-inside-stash both regressed
- Eviction scoring refinements are neutral on stable workload-a

**Distinct strategies for cycle 4 (all branching from v003):**
1. **H01**: Flat +3 credit — probes whether the gain is monotone beyond +2
2. **H02**: Frequency-proportional credit — items with higher `lfu_counter` at eviction time get extra credit on promotion (rewards demonstrably reused items without new fields)
3. **H03**: Stash entry filter — only admit items with `lfu_counter >= 1` to the stash; discard zero-frequency evictees rather than wasting a stash slot on pure cold-start items

### 05:00:02 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0004/hypotheses/H01.md",
  "content": "# H01\n\nRaising the stash-to-primary promotion credit from +2 to +3 will increase hit rate by giving promoted items one additional eviction cycle of protection against the next cold-burst displacement.\n\n## Parent\n\ncycle/0003/v003\n\n## Changes\n\nIn `associative_cache_bucket.rs`, change the promotion credit in both stash-hit code paths from `+= 2` to `+= 3`:\n\n1. In `put`, Case 2 (key found in stash):\n   ```rust\n   stash_item.lfu_counter += 3;\n   ```\n   replacing the current `stash_item.lfu_counter += 2;`\n\n2. In `get`, stash-hit branch:\n   ```rust\n   stash_item.lfu_counter += 3;\n   ```\n   replacing the current `stash_item.lfu_counter += 2;`\n\nNo other changes. The stash capacity, FIFO ordering, and all other bucket logic remain exactly as in cycle/0003/v003. This is a minimal single-constant probe to determine whether the credit lever's gain is monotone beyond +2 or whether +2 is already near the optimum.\n\n## Rationale\n\nThe cycle-3 insight found that +2 beat +1 with a real but small improvement, leaving open whether the gain continues at +3. Testing +3 is the most direct way to characterise the lever's range before exploring more structural changes.\n"
}
```

### 05:00:14 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0004/hypotheses/H02.md",
  "content": "# H02\n\nScaling the stash promotion credit by the item's pre-eviction frequency — giving more credit to items that were demonstrably popular before displacement — will raise hit rate by differentiating valuable second-chance candidates from incidental ones.\n\n## Parent\n\ncycle/0003/v003\n\n## Changes\n\nIn `associative_cache_bucket.rs`, replace the fixed `+= 2` promotion credit in both stash-hit paths with a frequency-proportional credit. The extra credit is derived from the item's `lfu_counter` value at promotion time (which equals the value it carried when it was evicted to the stash, since the stash does not modify counters).\n\nFormula: `extra = (stash_item.lfu_counter / 2).min(2)`, giving a total credit of `2 + extra`.\n- `lfu_counter == 0`: total credit +2 (same as v003 baseline)\n- `lfu_counter` in 2..=3: total credit +3\n- `lfu_counter >= 4`: total credit +4 (capped)\n\nApply this in both locations:\n\n1. In `put`, Case 2 (key found in stash):\n   ```rust\n   let extra = (stash_item.lfu_counter / 2).min(2);\n   stash_item.lfu_counter += 2 + extra;\n   ```\n   replacing `stash_item.lfu_counter += 2;`\n\n2. In `get`, stash-hit branch:\n   ```rust\n   let extra = (stash_item.lfu_counter / 2).min(2);\n   stash_item.lfu_counter += 2 + extra;\n   ```\n   replacing `stash_item.lfu_counter += 2;`\n\nNo new fields or data structures needed. Stash capacity, FIFO eviction order, and all other logic remain unchanged from cycle/0003/v003.\n\n## Rationale\n\nThe fixed +2 credit treats every promoted item identically regardless of its history. Items that arrived in the stash with a higher counter were the LFU minimum in primary despite having genuine hit history — they were displaced by a burst of newer items competing at zero frequency. Rewarding these more valuable candidates with a larger head-start credit addresses the cross-cycle open question of whether a dynamic promotion credit outperforms a fixed one.\n"
}
```

### 05:00:25 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0004/hypotheses/H03.md",
  "content": "# H03\n\nFiltering zero-frequency items from entering the stash — discarding evicted items that have never been accessed rather than storing them — will raise hit rate by reserving all stash capacity for items with at least one demonstrated access.\n\n## Parent\n\ncycle/0003/v003\n\n## Changes\n\nIn `associative_cache_bucket.rs`, modify `evict_to_stash()` to only call `stash_push` when the evicted item has `lfu_counter >= 1`. Items evicted with `lfu_counter == 0` are silently dropped rather than written into the stash.\n\nCurrent code:\n```rust\nfn evict_to_stash(&mut self) {\n    if self.items.is_empty() {\n        return;\n    }\n    let idx = self.eviction_policy.eviction_candidate_index(&self.items);\n    let evicted = self.items.remove(idx);\n    self.stash_push(evicted);\n}\n```\n\nReplace with:\n```rust\nfn evict_to_stash(&mut self) {\n    if self.items.is_empty() {\n        return;\n    }\n    let idx = self.eviction_policy.eviction_candidate_index(&self.items);\n    let evicted = self.items.remove(idx);\n    if evicted.lfu_counter >= 1 {\n        self.stash_push(evicted);\n    }\n}\n```\n\nNo other changes. The stash capacity, FIFO ordering, promotion credit (+2), and all other bucket logic remain exactly as in cycle/0003/v003.\n\n## Rationale\n\nThe stash acts as a second-chance buffer. Items evicted from primary with `lfu_counter == 0` have never been accessed since insertion; they entered the bucket only to fill a slot and were displaced on the very next cold-burst arrival. Granting these items a stash slot consumes capacity that could hold an item with real reuse history. On a Zipfian-access workload like workload-a, a meaningful fraction of primary evictions are these pure cold-start items. Filtering them out ensures every stash slot is occupied by a candidate with at least one prior hit, making the second-chance promotion mechanism more precise.\n"
}
```

### 05:00:33 chat

Three hypotheses written to the output directory. Here is a summary of what each proposes and why:

---

**H01 — flat credit +3 (parent: cycle/0003/v003)**  
The most direct probe: change `stash_item.lfu_counter += 2` to `+= 3` in both `put` and `get` stash-hit branches. Cycle-3 confirmed +2 beat +1; this test determines whether the gain is monotone or whether +2 is already near the plateau. Single-constant change, no structural risk.

**H02 — frequency-proportional credit (parent: cycle/0003/v003)**  
Replace the fixed +2 with `2 + (stash_item.lfu_counter / 2).min(2)`, yielding +2 for counter=0, +3 for counter 2–3, and +4 for counter ≥4. Items that were demonstrably popular before displacement get a larger head-start on re-entry. No new fields needed. Addresses the open question from the cross-cycle narrative about whether dynamic credit outperforms fixed.

**H03 — stash entry filter on `lfu_counter` (parent: cycle/0003/v003)**  
In `evict_to_stash()`, wrap `stash_push(evicted)` with `if evicted.lfu_counter >= 1`. Items evicted with counter=0 (never accessed since insertion) are discarded rather than occupying a stash slot. On a Zipfian workload, cold-start evictees can crowd out more valuable second-chance candidates; this filter reserves stash capacity only for items that have at least one proven access.


---

- finished: 2026-06-04T05:00:34+00:00
- duration: 297.11s
- status: ok
- input_tokens: 12
- output_tokens: 15114
- cache_read_tokens: 297057
- cache_write_tokens: 34018
- cost_usd: 0.444843
