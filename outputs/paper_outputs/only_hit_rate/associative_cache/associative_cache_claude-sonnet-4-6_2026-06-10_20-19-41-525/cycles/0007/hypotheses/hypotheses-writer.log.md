# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0007/hypotheses
- allow_edits: True
- started: 2026-07-04T20:38:36+00:00

## timeline

### 20:38:53 thinking

Let me read all the necessary context files to understand the current state of the research and generate informed hypotheses for cycle 0007.

### 20:38:54 tool: Bash

```json
{
  "command": "ls /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0006/variants/",
  "description": "List cycle 0006 variants"
}
```

### 20:38:54 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0006/cycle-winners.json"
}
```

### 20:38:55 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0006/cycle-insight.md"
}
```

### 20:38:55 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/leaderboard.md"
}
```

### 20:38:56 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cross-cycle-insights.md"
}
```

### 20:38:56 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles-ledger.jsonl"
}
```

### 20:39:02 thinking

Now let me read the cycle 0006 variant files and the baseline source to understand the code structure.

### 20:39:03 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0006/variants/*/eval/fitness-vector.json"
}
```

### 20:39:04 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0006/variants/*/insight.md"
}
```

### 20:39:06 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0006/variants/**"
}
```

### 20:39:09 thinking

Good. Let me now read the variant overlays and insights from cycle 0006, and the baseline source code.

### 20:39:10 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0006/variants/v001/insight.md"
}
```

### 20:39:11 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0006/variants/v002/insight.md"
}
```

### 20:39:11 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0006/variants/v003/insight.md"
}
```

### 20:39:15 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0006/variants/v001/attempts/01/overlay/src/shared/eviction_policy.rs"
}
```

### 20:39:15 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0006/variants/v002/attempts/01/overlay/src/shared/eviction_policy.rs"
}
```

### 20:39:16 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0006/variants/v003/attempts/02/overlay/src/shared/eviction_policy.rs"
}
```

### 20:39:20 thinking

Now let me read the bucket and geometry files for the incumbents (cycle/0004/v001 and cycle/0005/v003), as well as the baseline to understand the current code structure better.

### 20:39:21 chat

Now let me read the incumbent winner and the baseline source:

### 20:39:22 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0004/variants/v001/attempts/**"
}
```

### 20:39:23 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/inputs/subjects/associative_cache/baseline/**"
}
```

### 20:39:26 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0004/variants/v001/attempts/01/overlay/src/associative_cache/associative_cache_bucket.rs"
}
```

### 20:39:27 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0004/variants/v001/attempts/01/overlay/src/shared/geometry.rs"
}
```

### 20:39:30 thinking

Now let me read the cycle 0005/v003 overlay files to understand its differences from cycle/0004/v001.

### 20:39:31 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0005/variants/v003/attempts/**"
}
```

### 20:39:34 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0005/variants/v003/attempts/01/overlay/src/associative_cache/associative_cache_bucket.rs"
}
```

### 20:39:35 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0005/variants/v003/attempts/01/overlay/src/shared/eviction_policy.rs"
}
```

### 20:39:48 thinking

Now I have a thorough understanding of the codebase and the history. Let me synthesize what I know and design 3 distinct hypotheses for cycle 0007.

## Summary of what I know

### Current state
- **Best incumbents**: cycle/0004/v001 and cycle/0005/v003, both at hit-rate=0.7289
- **cycle/0004/v001** = wide buckets (factor-4: 32×64) + cold-first eviction + mid-point insertion + episodic bulk counter decay + counter cap at 4
- **cycle/0005/v003** = same configuration, confirming the counter cap at 4 was the key addition that matched the performance

Let me trace through the lineage more carefully to understand exactly which features each version inherited and what changed.

### 20:39:48 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0004/variants/v001/attempts/01/overlay/src/shared/eviction_policy.rs"
}
```

### 20:43:31 thinking

Now I have a full picture. Let me synthesize what I know:

**cycle/0004/v001** (hit-rate=0.7289):
- Wide buckets: factor-4 (32×64)
- Cold-first eviction (counter==0 preferred)
- Mid-point insertion (items inserted at len/2)
- Episodic bulk counter decay when no cold items (decrement all by 1, then evict index 0)
- In eviction_policy.rs: standard unbounded counter increment in `get_from_position`
- **cycle/0005/v003** matches this exactly but caps the counter at 4 during increments, yet both achieve the same 0.7289 hit rate

So capping the counter doesn't help or hurt. I've also ruled out CLOCK-style per-eviction decrements, selective decay on minimum-counter items, and pure LFU without positional promotions.

The approaches that came close but fell short include LFU fallback after bulk decay (which compresses counter differences too much), mid-point insertion combined with LFU, widening the geometry beyond a factor of 4, tightening thresholds to counter ≤ 1 or ≥ 2, and O(1) swap-to-back promotion.

Looking at what hasn't been explored yet, latency per operation is a dimension the winning configurations haven't touched. ARC-style or two-queue policies that explicitly separate recency from frequency signals seem like the most credible unexplored direction. Counter-decay combined with LFU fallback is theoretically interesting, though cycle 0005 v001 attempted this and failed because...

The decay operation flattens all counter differences to near-zero, which destroys the frequency signal entirely. Once every counter is decremented by the same amount, the inter-item differences become so compressed that scanning for minimum counters becomes noisier than just relying on the recency information already encoded in bucket position.

So the open questions are whether there's a latency versus hit-rate trade-off worth measuring, whether an ARC-style two-queue structure could work better than the current approach, and what the minimum viable bucket size needs to be for mid-point insertion to be effective. For cycle 0007, I need to ground at least one hypothesis in a cross-cycle winner to have something concrete to build from.

I'm considering a decay multiplier refinement where instead of decrementing all counters by 1 during a warm-lock event, I apply two passes to drop them by 2 when no cold item exists. This would more aggressively convert counter-1 items to cold status and refresh the cold pool faster. The risk is over-decay converting too many warm items simultaneously, but with a cap of 4, items re-warm quickly enough. Though honestly, this might not move the needle much compared to what's already working.

Actually, this is worth testing because if decay fires and then evicts index 0, decrementing by 2 would create multiple counter-0 candidates at once, giving the next eviction call cold items without needing to trigger another warm-lock cycle.

The ARC-style approach keeps coming up as the most credible unexplored direction — partitioning a bucket into recency and frequency segments where new items start in recency, promoted items move to frequency, and eviction prefers recency. It's a structural change that explicitly separates these concerns rather than encoding recency through position. But that requires significant refactoring, so I'm wondering if there's a simpler variant that captures the same benefit.

Actually, the existing cold-first scan already achieves something similar by preferring lower-index cold items, which naturally encodes a recency gradient. So maybe the real opportunity is adjusting the decay trigger itself — instead of firing only when the cold pool hits zero, I could trigger decay when it drops below a threshold like 10% of bucket size. This maintains a minimum cold reserve and prevents the bucket from approaching warm-lock in the first place rather than just reacting to it.

The implementation would check the cold item count before searching for one, and if it's below the threshold, run a decay pass first to restore the cold reserve. This shifts from reactive (breaking out of warm-lock) to proactive (preventing it).

Now I'm considering an alternative: instead of evicting the LRU front after decay fires, what if we evict the first item that just became cold after the decay pass?

Actually, that's essentially what the current code attempts to do—decay all counters, then evict index 0. But there's a subtle issue: after decaying by 1, items that had counter=1 drop to 0 (becoming cold), while items with counter=2 or higher remain warm at counter=1+. The current code evicts index 0 unconditionally, regardless of whether it actually became cold or stayed warm. That's a potential bug—we might be evicting a still-warm item when we should only evict items that crossed the cold threshold. After decay, I could scan for the first item with counter=0 and evict that instead of always removing index 0, which would preserve warm items that haven't cooled down yet while targeting those that just became cold.

The current approach probably works fine in practice though—with the bucket's geometry and mid-point insertion, index 0 naturally tends to hold older items with lower counters, so it's likely already at 0 or 1 after decay. But there's still room to be more precise about which item actually deserves eviction.

Instead of just removing index 0, I could scan for the first item that actually became cold (counter == 0) after the decay pass and evict that one specifically. If nothing hit zero, fall back to index 0. This targets the warm-lock path more directly—the decay makes counter-1 items cold, so picking from those cold items and choosing the oldest one among them aligns better with the recency-based strategy. The change would go in the `evict_item()` function in `associative_cache_bucket.rs`.

Another angle is applying two decay passes instead of one during a warm-lock event, decrementing by 2 total. This would make both counter-1 and counter-2 items cold at once, potentially keeping the cold pool populated longer and reducing how often we trigger decay events. The downside is that counter-2 items (moderate reuse) would get evicted immediately, which could hurt hit-rate, though with a cap of 4 they'd re-warm quickly. The real question is whether fewer warm-lock events outweighs the cost of losing those moderately-used items.

Now I'm exploring an ARC-lite approach: tracking which items were evicted from cold versus warm states, then re-admitting them to a protected warm slot if they're re-requested (a ghost hit). This would require maintaining a ghost buffer of recently evicted keys within the bucket structure itself, so on re-insertion we can detect the key and give it a high initial counter value rather than starting it cold again.

The tradeoff is that every insertion would need to scan the ghost list, but if we bound it to the bucket size, that's the same O(n) overhead as the current mid-point scan. The ghost list itself would need to be evicted when full, adding some complexity but staying within the overlay constraints.

This addresses the two-queue or ARC-style policy that the cross-cycle analysis flagged as the most promising structural alternative, though it does introduce additional bookkeeping.

Now for the first hypothesis: instead of blindly removing the item at index 0 during eviction, I'd decay all counters first, then find the first item that dropped to zero (was at counter=1 before decay) and evict that one. If no item became cold, fall back to index 0. This targets the problem where the current approach can evict still-warm items while colder ones remain.

The second hypothesis takes a different angle: increase the decay depth from 1 to 2 during warm-lock events. This way items with counter=1 or counter=2 both become cold after decay, which replenishes the cold pool faster and extends how long it lasts before the next warm-lock trigger.

But I'm realizing this could backfire—evicting moderately-used items with counter=2 might hurt hit-rate, and it's unclear whether fewer warm-lock events would actually compensate for that loss. Let me pivot to something more promising.

I'm considering a ghost buffer approach inspired by ARC: maintain a bounded queue of recently evicted keys, and when a key reappears in a `put()` call, re-admit it with an elevated counter rather than starting fresh.

For the proactive decay idea, instead of waiting until the cold pool is completely empty, I'd trigger decay when cold items drop below a threshold like `max_size / 8`. This keeps a minimum reserve available and avoids the sudden disruption of warm-lock events. The decay logic would iterate through items and reduce their counters...

But I'm realizing this might not be fundamentally different from the current approach and could run into the same counter-compression issues that plagued the selective decay attempts. The real problem is that firing decay more frequently doesn't necessarily solve the underlying issue — it just spreads the counter reduction across more evictions, which could still lead to premature flattening of the frequency distribution.

Looking back at the cross-cycle patterns, episodic bulk decay works because it's rare and concentrated, whereas steady incremental decay continuously erodes the signal. A proactive threshold would actually increase decay frequency, pushing it closer to that steady-decay failure mode, which could make things worse.

Let me pivot to hypothesis 3 instead: testing a factor-8 geometry with 16 buckets and 128 slots per bucket to see if a larger cold pool per bucket helps maintain better hit rates.

But wait—the cross-cycle insights already flagged that geometry widening beyond 32–64 slots showed diminishing returns on the baseline cold-first path. With the decay mechanism in place, wider buckets matter less because decay generates cold items on demand anyway. This is a known dead-end, so I should skip it.

Let me think about what actually hasn't been tested yet. There are a few directions worth exploring: evicting the most recently promoted warm item when warm-lock fires after decay kicks in, trying a ghost buffer or ARC-lite approach with re-admission logic, increasing decay depth beyond 1 in the warm-lock path, doing a post-decay cold scan instead of blindly evicting from index 0, capping the counter at 2 instead of 4 to speed up cold eligibility, or splitting a bucket into explicit hot and cold segments. Meanwhile, I can rule out geometry widening, threshold tightening, and O(1) swap-to-back promotion as paths that won't yield gains.

Now I'm narrowing down to three concrete hypotheses to test. The first targets hit-rate by scanning for the first cold item after decay completes, rather than always removing index 0—if nothing became cold, fall back to index 0. The second explores adding a ghost buffer to track evicted keys and re-admit them with elevated counters when they hit again, similar to ARC. The third hypothesis is still being formed. I need to distinguish between the counter cap (which limits how high a counter can go) and the eviction threshold (which determines when an item becomes cold). The cap at 4 was tested in cycle 5 v003, but lowering it to 2 is unexplored territory. This is separate from the earlier "threshold tightening" failure that tried raising the eviction threshold itself. I'm also considering an alternative approach where warm eviction during last-resort scenarios picks the highest-counter item instead of the oldest one. -decay cold-first scan with fallback to index 0, which matches hypothesis H01. Let me solidify my three main hypotheses: H01 proposes replacing the unconditional `remove_at(0)` after decay with a post-decay cold-first scan that finds the item with counter==0 (or falls back to index 0), aligning the warm-lock eviction with the same cold-first logic used elsewhere in the policy.

H02 introduces an ARC-lite ghost buffer that tracks recently evicted keys and re-admits them with elevated counter status when they're requested again, reducing the penalty of regret evictions and targeting hit-rate improvement through smarter re-admission heuristics.

H03 tightens the warm-lock recovery mechanism by lowering the counter cap from 4 to 2, allowing items to cycle back to cold eligibility faster while maintaining distinguishability between access patterns—this accelerates cold pool replenishment after decay events and reduces warm-lock severity on steady-state workloads.

These three heuristics are independent approaches: H01 refines eviction candidate selection within the normal path, H02 adds a structural ghost buffer for re-admission regret, and H03 adjusts the counter dynamics to speed recovery cycles.

Both H01 and H03 cite the cross-cycle winner from cycle/0005/v003, satisfying the requirement, and all three have clear testable claims and concrete implementation targets.

For H01, I'm modifying the eviction logic to scan for items with zero LFU counters after the decay loop, selecting the first cold item found rather than always evicting from position zero. For H02, I'm adding a ghost buffer structure to track recently evicted items and improve admission decisions on regret evictions. For H03, I'm tuning how long items maintain their warm protection through counter cap adjustments. Adding the VecDeque import, then deciding whether to build on top of the proven v003 stack or start fresh from baseline—basing it on v003 makes more sense since I can measure whether the ghost buffer specifically improves on an already-optimized foundation. I could make H02 cite baseline for structural diversity, but that means it would only add the ghost buffer without the wide buckets or cold-first eviction that baseline lacks — a partial test. Alternatively, I could have H02 also cite cycle/0005/v003, which means all three hypotheses share the same parent, though that's less diverse. I need to check the rules again about whether at least one hypothesis must cite a cross-cycle winner.

Looking at the constraint more carefully, the rules allow all three to cite the same winner as long as their strategies are genuinely distinct. So I'll go with H01 focusing on post-decay cold scan improvements, H02 implementing the full ARC-lite stack (wide buckets, ghost buffer, cold-first) on top of baseline as a complete replacement, and H03 tuning the counter cap down from 4 to 2 — all citing cycle/0005/v003 as parent since they're strategically different enough.

Actually, I'm reconsidering H02. If it starts from baseline, the implementer would need to build everything from scratch, which conflates the ghost buffer effect with the absence of other mechanisms. It makes more sense to have H02 also cite v003 and add the ghost buffer as an overlay — inheriting the counter cap, wide buckets, and other mechanisms while testing whether the ghost buffer specifically helps within that full stack.

Now I'm thinking through which parent to cite for all three hypotheses. Both cycle/0004/v001 and cycle/0005/v003 scored equally at 0.7289, but v003 is the more recent and complete implementation with the counter cap at 4. For H01 (post-decay cold scan), H02 (ghost buffer), and H03 (cap at 2), citing v003 makes the most sense since that's where the cap mechanism was introduced.

The three hypotheses involve changes isolated to specific files: the post-decay cold scan and ghost buffer logic go in the associative cache bucket, while the counter cap reduction happens in the eviction policy. But I'm realizing that lowering the cap from 4 to 2 creates a tension—items accessed once get counter=1, and after two warm-lock decays they become cold-eligible, which might conflict with the insight that single access is a reliable signal of future reuse.

The key difference between capping at 2 versus 4 is that with a lower cap, the average counter before decay events is reduced, so each decay event makes more items cold-eligible. This is distinct from threshold tightening, which would raise the eviction threshold itself rather than just limiting how high counters can climb.

The risk is that frequently-accessed items become decay-eligible too quickly and could get evicted during warm-lock resolution even when they're still relevant. The original cap=4 design was meant to prevent permanent warm-lock by requiring 4 events to return to cold, but dropping to cap=2 could cause oscillation where hot items get evicted and immediately re-requested. That said, cycle 5 showed the counter-cap had no measurable effect on this workload, so cap=2 might work as a different lever to reduce warm-lock severity.

I'm going to test this hypothesis. Now I'll write out the three hypotheses to the output files in the cycles directory.

Writing hypothesis files... The key insight is that after decay, items with a counter of exactly 0 are the freshest cold candidates—they just crossed the threshold—whereas items at the front might still be warm if they had higher counters before the decay step. By scanning for the first genuinely cold item rather than blindly evicting from index 0, the policy respects the LFU semantics and the recency signal in the vector order. The other components stay the same from the previous version, and I'm now considering adding a ghost buffer mechanism similar to ARC to re-admit recently evicted keys with elevated counters, which should improve hit rates by protecting against thrashing.

I'm modifying the bucket structure to track evicted keys in a separate ghost buffer, and when a key is re-inserted, I check if it was recently evicted. If it was, I restore it with a higher counter and place it at the MRU position instead of the normal mid-point insertion. This gives the cache a second chance to keep items it regrets evicting.

Adding ghost buffer logic... Lowering the cap to 2 means items cycle through warm states faster—those with counter=1 need just one decay event to become cold, and counter=2 items need two—so the pool of cold candidates refreshes more frequently and warm-lock episodes become shorter and less severe. This preserves meaningful frequency discrimination while reducing contention.

The other components from the previous version stay as-is. Now I'm verifying that each hypothesis is properly structured with a clear claim, parent reference, and implementable changes—all three target hit-rate improvements and include the specific code modifications needed.

I'm also checking that the strategies are genuinely distinct: one refines eviction selection during warm-lock resolution, another introduces a ghost buffer for regret-driven re-admission similar to ARC, and the third adjusts the counter cap that governs warm-lock recovery speed. Before writing the files, I need to make sure the claim lines follow the format rules—single sentence, concrete change plus fitness dimension, no markdown list syntax or code fences.

### 20:43:32 chat

Now I have all the context I need. Let me synthesize and write the three hypotheses.

### 20:43:42 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0007/hypotheses/H01.md",
  "content": "# H01\n\nRe-scanning for the first genuinely-cold item after the bulk-decay pass — instead of blindly removing index 0 — will raise hit-rate by ensuring warm-lock eviction targets only items whose counter has actually reached zero.\n\n## Parent\n\ncycle/0005/v003\n\n## Changes\n\nModify only `src/associative_cache/associative_cache_bucket.rs`; all other overlay files (eviction_policy.rs with the counter cap at 4, geometry.rs with the factor-4 wide-bucket layout) are inherited unchanged from cycle/0005/v003.\n\nIn `evict_item()`, the warm-lock fallback currently decays all counters by 1 and then unconditionally removes `index 0`:\n\n```rust\nfor item in self.items.iter_mut() {\n    item.lfu_counter -= 1;\n}\nSome(self.remove_at(0))\n```\n\nReplace `remove_at(0)` with a post-decay cold scan that mirrors the normal cold-first logic:\n\n```rust\nfor item in self.items.iter_mut() {\n    item.lfu_counter -= 1;\n}\n// After decay, evict the first item that is now genuinely cold (counter == 0).\n// Falls back to index 0 if no item reached counter == 0 (all had counter >= 2 before decay).\nlet cold_idx = self.items\n    .iter()\n    .position(|item| item.lfu_counter == 0)\n    .unwrap_or(0);\nSome(self.remove_at(cold_idx))\n```\n\n**Why this matters:** after the bulk decay, items that were at `lfu_counter == 1` before the pass become cold (`lfu_counter == 0`). Items that were at `lfu_counter >= 2` are still warm. The current code ignores this distinction and evicts index 0 regardless of its post-decay counter, which can mean evicting a still-warm item (counter >= 1) at the front while a genuinely cold item (counter == 0) sits at a higher index. The fix aligns the warm-lock resolution with the same cold-first principle used on the normal path: prefer the first cold item in insertion order; only fall back to index 0 if no item reached zero after decay. The invariant that all items have `lfu_counter >= 1` before the decay pass is preserved (unchanged from the parent), so the `saturating_sub` / direct `- 1` is still safe.\n"
}
```

### 20:44:01 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0007/hypotheses/H02.md",
  "content": "# H02\n\nAdding an ARC-style ghost buffer to the bucket — re-admitting recently evicted keys with an elevated counter and MRU position — will raise hit-rate by converting regret evictions into protected warm entries on their next access.\n\n## Parent\n\ncycle/0005/v003\n\n## Changes\n\nModify only `src/associative_cache/associative_cache_bucket.rs`; all other overlay files (eviction_policy.rs with counter cap at 4, geometry.rs with factor-4 wide-bucket layout) are inherited unchanged from cycle/0005/v003.\n\n**Struct changes:** add two fields to `AssociativeCacheBucket<K, V>`:\n\n```rust\nghost: std::collections::VecDeque<K>,\nghost_capacity: usize,\n```\n\n**`new()` initialisation:**\n\n```rust\nghost: std::collections::VecDeque::new(),\nghost_capacity: settings.bucket_size,  // ghost list is same size as the bucket\n```\n\n**`evict_item()` — record evicted keys into the ghost buffer:**\n\nAfter every call to `self.remove_at(idx)` (both the normal cold path at step 1 and the warm-lock fallback at step 2), push the evicted item's key into the ghost buffer before returning:\n\n```rust\nlet evicted = self.remove_at(idx);\nif self.ghost.len() >= self.ghost_capacity {\n    self.ghost.pop_front();  // drop oldest ghost entry to stay bounded\n}\nself.ghost.push_back(evicted.key);\nSome(evicted)\n```\n\n**`put()` — ghost-hit re-admission path:**\n\nAt the top of the new-key insertion branch (after the existing-key check, before the capacity eviction), probe the ghost buffer:\n\n```rust\nif let Some(ghost_pos) = self.ghost.iter().position(|&k| k == item.key) {\n    // Ghost hit: this key was evicted recently and is being re-requested.\n    // Re-admit it at the MRU back with an elevated counter so it immediately\n    // enters the warm-protected pool and is not cold-first-evicted right away.\n    self.ghost.remove(ghost_pos);\n    let mut re_item = item;\n    re_item.lfu_counter = 2;  // warm, survives two decay events before becoming cold\n    let evicted_item = if self.is_full() { self.evict_item() } else { None };\n    self.items.push(re_item);\n    return (&self.items[self.items.len() - 1], evicted_item);\n}\n// Non-ghost path: continue with the normal mid-point insertion at counter 0.\n```\n\nThe non-ghost path (mid-point insertion + cold-first eviction + bulk decay) is unchanged from the parent.\n\n**Invariants to preserve:**\n- Ghost list length is always `<= ghost_capacity` (enforced by the `pop_front` guard).\n- Total bucket capacity is unchanged: the ghost list is metadata only (keys, not values) and does not count toward `max_size`.\n- Re-admitted items start at `lfu_counter = 2`, consistent with the counter cap of 4; they will become cold after two decay events if not re-accessed.\n- The `VecDeque<K>` import must be added (via `use std::collections::VecDeque;` or inline path).\n"
}
```

### 20:44:20 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0007/hypotheses/H03.md",
  "content": "# H03\n\nReducing the `lfu_counter` cap from 4 to 2 in `ClassicLRU::get_from_position` will raise hit-rate by shortening warm-lock episodes: with a lower ceiling, every item returns to cold eligibility within two decay events rather than four, replenishing the cold pool more aggressively per warm-lock trigger.\n\n## Parent\n\ncycle/0005/v003\n\n## Changes\n\nModify only `src/shared/eviction_policy.rs`; all other overlay files (associative_cache_bucket.rs with mid-point insertion, cold-first eviction, and bulk counter decay, and geometry.rs with factor-4 wide-bucket layout) are inherited unchanged from cycle/0005/v003.\n\nIn `ClassicLRU::get_from_position`, change the cap from `.min(4)` to `.min(2)`:\n\n```rust\n// CURRENT (cycle/0005/v003):\nitems[position].lfu_counter = items[position].lfu_counter.saturating_add(1).min(4);\n\n// PROPOSED:\nitems[position].lfu_counter = items[position].lfu_counter.saturating_add(1).min(2);\n```\n\nNo other line in `eviction_policy.rs` changes.\n\n**What changes in practice:**\n\n| counter before GET | cap=4 result | cap=2 result |\n|--------------------|--------------|--------------|\n| 0 (cold)           | 1            | 1            |\n| 1                  | 2            | 2            |\n| 2                  | 3            | 2 (capped)   |\n| 3                  | 4            | 2 (capped)   |\n| 4                  | 4 (capped)   | 2 (capped)   |\n\nWith cap=2, any item that has been accessed at least twice saturates at counter 2. After one bulk-decay event, it drops to counter 1 (still warm). After a second decay event, it drops to counter 0 (cold, eligible for cold-first eviction). This guarantees that no item requires more than two warm-lock events to return to the cold pool, versus four with the prior cap.\n\n**Why this is different from the \"threshold tightening\" failure (cycle/0003/v002):** that experiment raised the *eviction threshold* — items with counter ≤ 1 were treated as cold-evictable before their counter reached 0, which caused hot-set members accessed just once to be evicted prematurely. This proposal leaves the eviction threshold at `counter == 0` unchanged; only the maximum value an item's counter can accumulate is halved. Items with counter=1 or counter=2 are still fully protected (not cold-evictable) under the same criterion as before.\n\n**Hypothesis:** with a cap of 2 and a bucket size of 64, warm-lock events (all counters >= 1) are resolved more quickly because a higher fraction of items have low counters (1 or 2) and become cold after fewer decay passes. The cold pool replenishes faster, so subsequent evictions find cold candidates without triggering another decay, reducing per-operation overhead during the recovery window. The risk is that highly-accessed items cycle through cold eligibility faster, but given the full-stack's proven robustness to decay events and the counter-cap-inertness observed in cycle/0005/v003, this level of change is worth probing.\n"
}
```

### 20:44:35 chat

All three hypotheses have been written to the output directory. Here is a summary of what was generated for cycle 0007:

---

## Cycle 0007 Hypotheses Summary

All three hypotheses derive from **cycle/0005/v003** (hit-rate=0.7289, the co-incumbent winner), satisfying the requirement that at least one cites a cross-cycle winner.

### H01 — Post-decay cold scan (`associative_cache_bucket.rs`)
**Claim:** Re-scanning for the first genuinely-cold item after the bulk-decay pass — instead of blindly removing index 0 — will raise hit-rate.

**Rationale:** The current warm-lock path decays all counters by 1, then unconditionally removes `index 0`. But index 0 may still be warm (counter ≥ 1) if it had counter ≥ 2 before decay, while a genuinely cold item (counter just became 0) exists elsewhere. The fix: replace `remove_at(0)` with `position(|item| item.lfu_counter == 0).unwrap_or(0)` after the decay loop — aligning warm-lock resolution with the same cold-first logic used on the normal path.

---

### H02 — ARC-lite ghost buffer (`associative_cache_bucket.rs`)
**Claim:** Adding a ghost buffer to re-admit recently evicted keys with an elevated counter and MRU position will raise hit-rate by protecting regret evictions.

**Rationale:** ARC's core insight is that items evicted while still being requested ("regret evictions") deserve preferential re-admission. This adds a bounded `VecDeque<K>` ghost list to the bucket. Every eviction records the key; every `put()` checks the ghost list first. A ghost hit re-admits the item at the back (MRU) with `lfu_counter = 2` (warm-protected), bypassing mid-point insertion.

---

### H03 — Counter cap reduced from 4 to 2 (`eviction_policy.rs`)
**Claim:** Reducing the `lfu_counter` cap from 4 to 2 will raise hit-rate by shortening warm-lock episodes — every item becomes cold-eligible within two decay events instead of four.

**Rationale:** With cap=4, items accessed repeatedly need up to 4 decay events to return to the cold pool. Halving the cap to 2 means all items saturate at a lower ceiling and return to cold eligibility faster per warm-lock trigger. The eviction threshold (`counter == 0`) is unchanged — this is distinct from the failed "threshold tightening" experiment in cycle 3.


---

- finished: 2026-07-04T20:44:35+00:00
- duration: 359.38s
- status: ok
- input_tokens: 15
- output_tokens: 17646
- cache_read_tokens: 550339
- cache_write_tokens: 73534
- cost_usd: 0.707026
