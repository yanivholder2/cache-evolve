# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0010/hypotheses
- allow_edits: True
- started: 2026-06-06T01:46:09+00:00

## timeline

### 01:46:12 thinking

Let me start by reading all the context files to understand the current state of the research and what hypotheses would be most useful to generate.

### 01:46:12 chat

I'll read all the context files in parallel to understand the current state before writing hypotheses.

### 01:46:13 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0009/variants/v*/"
}
```

### 01:46:14 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0009/cycle-winners.json"
}
```

### 01:46:15 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0009/cycle-insight.md"
}
```

### 01:46:15 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/leaderboard.md"
}
```

### 01:46:16 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cross-cycle-insights.md"
}
```

### 01:46:16 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles-ledger.jsonl"
}
```

### 01:46:19 thinking

Now let me read the cycle 0009 variant details - let me first check what variant directories exist.

### 01:46:19 chat

Now let me read the cycle 0009 variants in detail:

### 01:46:20 tool: Bash

```json
{
  "command": "ls /home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0009/variants/"
}
```

### 01:46:23 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0009/variants/**"
}
```

### 01:46:27 thinking

Now let me read the key files for cycle 0009 v003 (the winner) and the baseline to understand what code I need to work with.

### 01:46:27 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0009/variants/v003/insight.md"
}
```

### 01:46:28 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0009/variants/v003/attempts/01/overlay/src/shared/eviction_policy.rs"
}
```

### 01:46:29 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0009/variants/v003/attempts/01/overlay/src/dash/dash_bucket.rs"
}
```

### 01:46:30 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0009/variants/v003/attempts/01/overlay/src/dash/segments/dash_segment.rs"
}
```

### 01:46:31 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0009/variants/v003/attempts/01/overlay/src/shared/geometry.rs"
}
```

### 01:46:34 thinking

Now let me read the baseline files to understand what we're working with at the foundation level.

### 01:46:34 chat

Now let me read the baseline and the Item struct:

### 01:46:34 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/dash/baseline/src/shared/item.rs"
}
```

### 01:46:35 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/dash/baseline/src/shared/eviction_policy.rs"
}
```

### 01:46:36 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/dash/baseline/src/dash/segments/dash_segment.rs"
}
```

### 01:46:38 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/inputs/subjects/dash/baseline/**"
}
```

### 01:46:41 thinking

There's no baseline source in the baseline directory. The overlay files are what I need to look at - those are the modified versions. Let me also look at the variant v001 and v002 for cycle 9 to understand what was tried before writing a v003 parent hypothesis.

Actually, let me look at the insights from v001 and v002 in cycle 9 to understand what failed and why.

### 01:46:42 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0009/variants/v001/insight.md"
}
```

### 01:46:43 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0009/variants/v002/insight.md"
}
```

### 01:46:44 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0009/variants/v001/attempts/01/overlay/src/shared/eviction_policy.rs"
}
```

### 01:57:08 thinking

Now I have a thorough understanding of the codebase. Let me synthesize the state and identify three distinct hypotheses for cycle 10.

**State Summary:**
- Current champion: `cycle/0009/v003` with hit-rate = 0.724
- v003's key change: Added `data.lfu_counter += 1` before `promote_to_target` in the stash-hit branch in `dash_segment.rs` (line ~200)
- Confirmed fixed features in v003:
  - `ClassicLRUFreqProtect` eviction (protects items with ≥1 in-tier hit)
  - Second-chance stash-overflow recovery (evicted stash items promoted to target at `Front`)
  - 7/8 normal/stash geometry (28+4 buckets)
  - Stash-hit counter credit (+1 before promoting to target)

These four wins have accumulated across cycles 6-9, and now I need to identify what's still missing.

The main open questions are around soft-capping the `lfu_counter` to keep hot items from going stale over time, reconsidering how timestamps get promoted (only on first hit rather than insertion), potentially tuning the stash-exit counter credit higher, and exploring intra-normal-tier eviction ordering when thresholds are non-zero.

I'm ruling out geometry changes since 7/8 is confirmed as the ceiling, and I'm avoiding ghost sets, LRU/Front insertion patterns, LFU variants, stash admission weakening, stricter promotion gates, and any expansion beyond 7/8.

Now I'm setting up three distinct hypotheses to test, starting with soft-capping the `lfu_counter`. Modifying the stash-exit credit to add 2 instead of 1 would give recently-promoted items stronger protection against eviction by starting them with a higher frequency counter. The change happens in `dash_segment.rs` where the counter increment occurs before promotion to the target tier.

But I'm realizing the current eviction policy in `ClassicLRUFreqProtect` only checks if `lfu_counter == 0`, making it a binary threshold—items with a counter of 1 or 2 are treated identically since both are above zero. The promotion candidate is always the MRU end regardless of counter value, so adjusting the stash-exit credit wouldn't actually change behavior unless I also modify the eviction threshold itself to use a graduated approach rather than just checking for zero.

The cycle 7 insights already established that protecting single-touch items is correct for this Zipfian workload, so raising the threshold to evict items with counter >= 2 doesn't align with what's been learned. Instead, I should focus on genuinely untested levers—the soft-cap on `lfu_counter` keeps surfacing as the primary unexplored mechanism, since items with very high counters get stuck at the MRU position.

But I'm realizing the concern about crowding out newly-warm items doesn't quite hold under the current `ClassicLRUFreqProtect` logic, where eviction just finds the first zero-counter item and falls back to LRU position if none exist.  A stale high counter doesn't actually change behavior since the policy only treats the counter as a binary gate. Capping the counter wouldn't help unless we also change the eviction policy itself to use the counter more directly—like protecting items above a threshold or ranking by counter value—which would be a separate design decision.

Let me look at the actual eviction logic more carefully. The function searches for the first item with a counter of zero, and if none exists, defaults to evicting the first item in the bucket. So in a mixed bucket, there might be multiple candidates with zero counters.

This means a soft cap wouldn't change the binary decision between zero and nonzero items, so capping at different values shouldn't matter much. However, if we shifted the eviction policy to look at items below some threshold rather than strictly at zero, then a cap could prevent high-counter items from indefinitely blocking newer medium-counter items—though that would require changing both the cap and the eviction logic itself.

Actually, I'm realizing the soft cap idea keeps coming up for a reason. Looking at the code more carefully, under the current binary zero-guard logic, a cap shouldn't make a difference. But what if the real change is moving from binary (zero vs. nonzero) to ranked eviction (prefer lower counter)? That's essentially a partial LFU approach, and while full LFU hurt performance due to co-resident decay and frequency discrimination, a capped version might behave differently—items can't accumulate unlimited counts, so they'd hit the cap at 4 and then compete more fairly.

The actual shift would be: instead of "evict zero-counter items first, then LRU," it becomes "evict the lowest-counter item, break ties by LRU position." This means items with counter 1 would now be preferred victims over items with counter 50, which is a meaningful change from the current scheme where both are treated equally as nonzero.

But introducing a soft cap—say, capping counters at 8—creates a different problem: newly-warm items (counter 1-3) would get evicted before more established hot items (counter 4+), which doesn't fully solve the cross-cycle decay issue where recently-accessed items still carry residual heat and shouldn't be displaced so aggressively.

Let me explore some other angles I haven't fully considered. One thing I'm noticing is the probing bucket behavior—when a key hits there, it moves to MRU within that bucket via the policy's `get_from_position` call, which seems correct since the probing bucket holds the hottest items. But I'm wondering about what happens during demotion: when an item leaves the probing bucket, it gets counter-reset to 0 on stash insertion, even though it's been accessed multiple times to reach that tier in the first place. Could we credit more on return from stash for items that previously reached the probing tier? The challenge is we'd need extra state to track that history.

Another angle: what if items with a pre-existing nonzero counter entered their target position differently on second-chance reentry—maybe at the back instead of the front? Though by the time stash overflow triggers the second-chance promotion, the counter's already been reset, so that information is lost.

The real question I'm circling is whether resetting the counter to 0 on demotion to stash is the right call. If we kept the counter intact, items with high frequency would stay protected even inside the stash through the same frequency-protect mechanism. But I need to verify whether the stash actually uses the same eviction policy as the regular buckets. the stash when hot items with high counters are present. The design choice to reset lfu_counter on demotion prevents hot items from monopolizing stash slots, ensuring the stash functions as intended—a place where cold items get a genuine second chance rather than being crowded out by items that were already popular in the normal tier. So a newly arrived cold item with counter=0 would be the first eviction candidate from stash, and if evicted, it gets a second-chance slot in the target tier—but that doesn't help since it's still cold and would just become an eviction candidate there too. The flow is: new item enters stash at MRU with zero counter, stash evicts the first zero-counter item (which could be that new arrival), and the evicted item gets promoted to target via second-chance.

Now I'm thinking about counter staleness more carefully. If an item gets accessed heavily in a burst and accumulates a high counter, then its access rate drops to zero, the counter stays high and protects it from eviction under ClassicLRUFreqProtect—but that protection becomes stale and irrelevant over time.

Actually, wait. The position in the list matters more than I initially thought. When an item stops being accessed, it gradually drifts toward the LRU end even though its counter remains high. The counter just acts as a secondary guard—it prevents priority eviction when there are zero-counter alternatives available, but it doesn't override the LRU fallback mechanism. So soft-capping the counter might not actually solve the staleness problem in any meaningful way.

And rereading the crowding-out concern more carefully: under the current binary policy, a newly-warm item with counter=1 and a long-lived item with counter=100 are treated identically since both are nonzero. There's no actual crowding happening—the ordering within nonzero items is purely positional. The promotion candidate is always the MRU end regardless of counter values.

So I'm not seeing how a counter cap alone would help without changing the eviction logic itself. The real opportunity is to move from binary protection to ranked eviction: cap the counter at something like 4 or 8, then change the eviction strategy to pick the item with the minimum counter (breaking ties by LRU position). This way hot items stay protected while cold newcomers get evicted first, and the counter actually influences which item leaves.

The risk is that this starts to look like LFU, which had issues with draining mildly-warm items in earlier attempts. But with a cap of 4, items that hit the cap all look identical and fall back to LRU ordering, so only items with 1-3 accesses get differentiated from those with 4+. That constraint might make it workable.

I'm implementing this in the eviction policy by defining a constant cap and modifying the eviction candidate selection... I'm realizing the ranking by counter first, then position for ties, means a freshly-accessed item with counter=1 could get evicted before an old item with counter=4. This is problematic if single-touch items are actually warm reuse predictors, as the workload data suggests. The only way this strategy works is if there's a sharp distinction between cold items (counter=0) and warm items (counter≥1), which is exactly what the binary guard enforces—so the ranking should be safe as long as that separation holds.

But a ranked approach feels risky. Let me try something simpler: cap the counter at a fixed value while keeping the binary eviction guard intact. This way, very long-lived items stop accumulating advantage once they hit the cap, making them equivalent to other items at that threshold. But since eviction only checks whether the counter is zero or nonzero, capping doesn't actually change the eviction behavior at all.

The real issue is that a soft cap only matters if I pair it with a ranking-based eviction policy, not a binary gate. Cycle 7 showed that raising the threshold above zero was harmful, so I need a different approach. Maybe I could keep the binary guard but add a secondary tier: prioritize evicting zero-counter items first, then among nonzero items, evict based on some other signal.

But wait—that's still a ranked approach within the nonzero tier, which brings back the same risk of evicting single-touch warm items that hurt workload-a.

Let me try something completely different: what if I give items evicted from the probing bucket a second chance to return to their original bucket at the MRU end, rather than cascading them down immediately?

Actually, looking at the demotion logic, foreign residents (items whose true home is the probing bucket) already get this second chance when demoted back to their actual target. For native residents promoted to probing, they're treated as foreign to the probing bucket, so they get demoted to their true target via the cascade—which is the current behavior.

Let me trace through the demotion function to confirm how this works... When a probing-native item gets evicted, it goes straight to stash with its counter reset, and the stash-exit credit helps it recover when it gets hit again. I'm wondering if we could give these items a second chance by routing them to their true target bucket instead of immediately demoting them to stash—since the probing bucket itself is just one hop away from that target.

I'm also considering a second optimization: when a stash overflow causes an item to get promoted back to its target bucket at the front, we could credit its counter by +1, similar to the stash-hit credit. But this feels less justified than the stash-hit case, since the item wasn't actually accessed—it was just evicted due to overflow. Adding that credit might weaken eviction selectivity by artificially boosting items that weren't recently used.

Another angle is whether probing hits should get a bonus counter increment (+2 instead of +1) since those items have already proven themselves through multiple accesses to reach the probing tier. However, this doesn't really change the relative ordering between probing items since they'd all accumulate counters the same way. Plus, the current framework doesn't cleanly expose which tier an item is in to the policy layer, so this would be messy to implement.

Let me step back and think about what's actually available to modify. I can change eviction policies, per-bucket logic, promotion mechanics, the normal/stash split, or even add new fields to the Item struct itself. What if I tracked tier information or promotion count directly on each item? I'm considering a novel approach: what if stash hits increment the LFU counter by 2 instead of 1? This would give items that survived the stash and then got accessed extra protection against eviction, since they'd only be evicted after zero-counter items are gone. But I need to check how the eviction logic actually finds candidates.

Looking at the eviction code, it searches for the first item with counter == 0, so the difference between counter=1 and counter=2 doesn't matter—anything above zero is treated the same way. I keep running into this same limitation: the binary guard makes counter magnitude irrelevant.

Let me explore a different angle: what if items that get accessed repeatedly in the probing bucket receive different treatment, like getting "locked" there with a modified eviction policy? Though that runs into the constraint that I can't change policy per-item. Another possibility is adjusting how second-chance items (stash overflow) enter the system.

Currently they go to the front of the LRU, which was confirmed as correct after testing. But what if I used the counter as a signal instead? The issue is the counter gets reset to 0 on stash entry, so I lose that information. What if I only reset the counter when a new item enters via PUT, but preserve it when an item gets demoted from the normal tier? That way I could distinguish between cold newcomers and items that were previously hot. So if we don't reset the counter on demotion, hot items with high counters end up permanently blocking stash slots—new cold items get inserted at the back with counter=0, immediately get second-chanced out to the target, and never actually benefit from being in stash. This would just add latency to cold items without any real gain.

I'm considering a different approach: when an item gets evicted from the probing bucket, instead of sending it straight to stash, give it a second chance by placing it back in its adjacent target bucket at the most-recently-used position.

But I need to think through the indexing here—I'm getting tangled up on what true_target actually means for items in the probing bucket. Let me clarify the structure: each key has a target_idx based on its hash, and the probing_idx is the next bucket over. Items in probing_idx are there because they were promoted from their target bucket when they got hot enough. natural target IS probing_idx, so its probing bucket would be (probing_idx + 1) % segment_size. The key insight is that every bucket serves dual roles: it's a target bucket for keys that hash to it, and a probing bucket for keys whose target is the previous bucket. So when demote_from_normal_bucket is called with source_idx = probing_idx, I need to figure out whether the item there is a target resident or a promoted item from elsewhere. When an item gets promoted to its target bucket and that bucket is full, the evicted item either goes to the stash if it's a native resident of that target, or retries in its own true target bucket. Similarly, when a hot item in the target gets promoted to the probing bucket and probing is full, the evicted item follows the same demotion logic based on whether it belongs in the probing bucket or needs to find its true target. So if an item is already at its home bucket when evicted, it goes straight to stash with no second chances. Items that were promoted into the probing bucket get one more attempt to land in their true target. Now I'm narrowing down the approach—I'm considering a soft-cap LFU counter strategy combined with minimum-counter eviction among non-zero items as the core mechanism.

Another angle is pre-crediting the counter when an item gets promoted from its target bucket to probing. The idea is that if an item was hot enough to be the MRU in its target and then accessed again, it's already proven itself—giving it a counter boost before it enters probing would protect it better during its tenure there. The current flow already increments the counter and moves to MRU on access, so the question is whether an extra credit at promotion time would help. would give promoted items a buffer against eviction in the probing tier, but since they likely already have nonzero counters from their target accesses, the benefit might be minimal. Let me trace through whether items can actually enter probing with counter=0 to see if this optimization matters. Ah, I see the distinction now. The `get_position` method just locates the item without modifying it, then the code checks if it's already at the MRU position using `promotion_candidate_index`. If it is the hottest item, it gets removed and promoted to probing immediately. If it's not the hottest yet, then `get_from_position` is called to increment the counter and move it toward MRU.

So the counter increments happen during normal accesses in the target bucket, but when an item finally reaches MRU status and gets promoted, the promotion itself doesn't add another increment—it just moves the item with whatever counter value it accumulated. This means items promoted from target to probing carry their access count with them, which protects them from immediate eviction.

The real problem I'm hitting is that the binary zero-guard makes the magnitude of the counter irrelevant—what matters is only whether it's zero or nonzero. So giving items extra credit at promotion time doesn't actually change the eviction behavior, since any nonzero counter already prevents eviction. I need to reconsider what would actually make a difference here.

I could change the eviction threshold itself—maybe evict items with counter zero or one instead of just zero—but cycle 7 showed that's harmful on the test workload. Or I could switch to a ranked eviction strategy based on counter magnitude, but that's essentially LFU and also performs poorly. The more promising direction is to track additional information about item provenance, like whether something came through the stash, since that signals proven reuse. Even better, I could skip the intermediate tier entirely and promote stash hits directly to the probing tier, giving those proven items immediate access to the hot zone and reducing their demotion risk.

The concern with this direct promotion is that it bypasses the target tier's filtering mechanism, potentially letting marginal items slip through to probing. Plus, probing capacity is limited since it's just a single bucket. Let me check what probing actually is in the code—it looks like the probing slot for a key maps to the next bucket in the segment, so it's a normal-sized bucket, not some special structure.

That means the hot tier spans 28 normal buckets with 2 items each, holding up to 56 items total. A stash-to-probing promotion would place the item at that next bucket, but if it's full, demotion cascades down. The tradeoff is whether skipping the target tier and going straight to probing with a fresh counter actually improves retention, or if it just creates complexity without real benefit.

The key tension: items with confirmed reuse avoid the vulnerable target position where they could be evicted by newcomers with zero counter. But jumping straight to probing risks displacing genuinely hot items already there, which then cascade down through demotion. When an item gets evicted from probing, it either goes back to stash if it's native, or retries if it's foreign.

For a Zipfian workload with a stable hot set, the probing tier holds the proven hottest items. A stash item with only one confirmed access would likely displace items with multiple accesses, which is harmful. I'm abandoning this approach.

Now I'm considering whether adding a field to the Item struct could enable new logic—something like tracking whether an item has been demoted from the normal tier before, which could inform stash insertion decisions. `demoted = true` items that get second-chanced would enter the target at Back (MRU) instead of Front, giving them more protection since they've proven themselves before. But cycle 8 v002 showed Back insertion was harmful because it gave unproven items immediate promotion candidacy—though a demoted item isn't unproven, it was in the normal tier previously. The risk is that items demoted from the normal tier may not be hot anymore, so re-entering at MRU means they'd compete for promotion immediately, which feels risky despite being an interesting hypothesis.

Now I'm thinking through three concrete hypotheses to test, starting with a soft-cap on lfu_counter combined with a min-counter eviction preference among nonzero items. I keep worrying that ranked eviction is LFU-like and harmful, but looking back at what actually failed with LFU: cycle 2 showed that symmetric decay drove mildly-warm items to eviction before they could contribute hits, cycle 3 showed that slow counter ramp-up created eviction blind spots early in an item's life, and cycle 1 tied LFU's failures to workload-a's frequency-skewed access pattern.

What I'm proposing is different—a zero-counter guard that's preserved, no decay applied to co-residents, a cap on counter growth to prevent staleness, and only mild frequency preference within the nonzero set rather than a global ranking. Single-touch items would still be evicted before two-touch items, but the zero-counter items remain the first line of defense.

The risk is that in a Zipfian workload like workload-a, single touches are reliable reuse signals, so preferring lower counters could hurt performance. But if zero-counter items are almost always available in a busy cache, this change rarely triggers and stays harmless.

Let me explore a different angle instead—applying stash-exit credit to second-chance items as well, since I've already confirmed that counter magnitude doesn't matter under a binary guard. back to the LRU eviction policy). So giving second-chance items that +1 credit would shield them from immediate eviction in target, letting them prove their value through access patterns before facing eviction pressure.

The tradeoff is real though: if these stash residents are genuinely cold and never accessed again, the +1 just delays their inevitable eviction while potentially pushing out hotter items. The stash-exit credit from the earlier version worked because it targeted items that had already demonstrated value through an explicit stash hit, whereas second-chance items are more of a gamble.

Let me explore a different approach instead. What if I focus on how the stash itself manages frequency? Right now items enter the stash with a counter of zero, and only increment when they're actually accessed there. When overflow forces an eviction, the system looks for the first zero-counter item to remove. Since everything starts at zero, the only way a stash resident gains protection is through actual hits.

But I'm realizing there's a subtlety here - when `promotion_get` finds an item in the stash, it removes it entirely and increments the counter before promoting it elsewhere. So the counter increment happens during the promotion itself, not through a stash hit operation.

This means all items currently residing in the stash have `lfu_counter=0` since they're never accessed in place - they're only removed on promotion or eviction. So the stash effectively behaves like a simple LRU structure for eviction purposes, where all items are treated equally at zero counter, and the oldest item gets evicted first.

I'm realizing that using ClassicLRUFreqProtect in the stash is functionally identical to using ClassicLRU since all items are at counter zero anyway. This opens up the possibility of using a different eviction policy specifically for stash slots, but the current `new_bucket` implementation applies the same policy uniformly across all buckets without distinguishing between target, probing, and stash slots.

The stash buckets are created through `SegmentCore::try_new` in the gate code, which I can't modify directly. However, I could potentially add a new policy in `eviction_policy.rs` and modify `dash_bucket.rs` to use it, but I'd need a way to identify which buckets are stash buckets so the policy can behave differently for them.

Looking at the geometry, the stash and normal buckets have the same `bucket_size`, so I can't distinguish them that way. Let me reconsider the problem from scratch.

I'm exploring a different approach: adding a three-tier eviction guard to `ClassicLRUFreqProtect` that uses access count thresholds instead of just binary cold/hot detection—with level 0 for zero accesses, level 1 for moderate access counts up to a cap, and level 2 for higher access counts.

Writing eviction logic... Capping the counter at a maximum value means items hit that ceiling and become indistinguishable, so the eviction falls back to pure LRU ordering among them. But for workload-a where single-touch items are warm reuse predictors, evicting counter=1 items before counter=2 items could be a real problem—those counter=1 accesses often do get reused again, so this ranking would miss those patterns. The zero-counter guard still catches the obvious cold items like the current approach, but once everything has at least one access, the counter-based ranking kicks in and could hurt performance on this workload.

I'm reconsidering the fundamental levers here. The cleanest options seem to be resetting the counter when an item gets promoted from target to probing tier—which would make it immediately evictable and test whether probing should rely purely on LRU—or instead crediting the counter on promotion to give promoted items extra protection.

There's also the question of where stash hits should be inserted in the target tier: currently they go to the back (MRU position), but moving them to the front (LRU position) would likely hurt performance since MRU insertion dominates on this workload. And I can't vary the second-chance position based on counter since the counter is zero at that point anyway.

Looking at what's actually worth pursuing versus what's just speculation: soft-capping the LFU counter only helps if paired with ranked eviction, which introduces its own risks. The timestamp promotion idea of recording only on first hit, and adjusting intra-tier eviction ordering at non-zero thresholds—these are all untested and uncertain.

For the timestamp approach specifically, the core problem was that recording timestamps at insertion time contaminates the signal with creation times, letting newer cold items outrank older hot ones. Recording only on first access avoids this, but I need to handle items that never get accessed in-tier and figure out what initial timestamp value makes sense.

Using `Option<Instant>` in the Item struct would let me mark items as "never accessed" with `None`, then prioritize those for eviction before considering actual timestamps. This requires updating the struct definition, the get operation to set `Some(Instant::now())` on first hit, and the eviction logic to prefer `None` values.

Alternatively, I could add a `first_hit_recorded` boolean flag that starts false and gets set to true on the first access, treating unrecorded items as preferred eviction candidates—though this is essentially equivalent to using `Option<Instant>` but less clean since `Instant` doesn't have a meaningful minimum value.

The real insight is that I should never use insertion timestamp at all; instead, only update the timestamp on actual hits and use `lfu_counter` to distinguish items that have never been accessed in this tier from those that have, since items with `lfu_counter == 0` are the ones that should be treated as stale. proxy for recency anyway, so the difference between LRU position and timestamp ordering is minimal. Even in edge cases where an item gets accessed frequently but then becomes stale, both approaches would correctly identify it for eviction. So I should stop overthinking this and focus on three concrete hypotheses to test, starting with the champion approach.

My first hypothesis is to cap the LFU counter at a soft limit like 4, then use a min-by-counter fallback during eviction instead of pure LRU ordering. This keeps the frequency signal strong while preventing counter saturation from obscuring recency. I'd modify the eviction logic to check the zero-guard first, then fall back to selecting the item with the lowest counter, breaking ties by position.

The risk here is that single-touch items get evicted before multi-touch ones when there are no zero-counter candidates, which could hurt workload-a's warm items. But the hypothesis is that this counter-aware ordering will improve overall hit rates by protecting frequently-accessed items more effectively.

Now I'm reconsidering the second approach—crediting items on stash overflow doesn't seem promising. Instead, I'm thinking about tracking only the first access timestamp for each item, but then I realize that's backwards from what I need. For promotion decisions, I want the most recently accessed item, not the first one accessed. If I only record the timestamp once, items that get hit many times will have an old timestamp from their initial access, which defeats the purpose.

The core problem is that cold newcomers get fresh insertion timestamps that can be newer than the last-access timestamps of hot items, causing them to displace active data. I'm exploring whether skipping timestamp updates during promotion operations and only updating on regular accesses would help—so a promoted item's timestamp reflects its last non-promotion access rather than when it was inserted. But this still doesn't really address the fundamental contamination issue where insertion time can be newer than access time.

The real solution is to treat items that have never been accessed in this tier (lfu_counter=0) as if they have infinitely old timestamps, regardless of their actual insertion time. I'm writing a comparison function that sorts these never-accessed items first for eviction, treating them as the oldest candidates. Completing the eviction logic... When items are inserted at the Front with a recent timestamp, the timestamp-based fallback would incorrectly protect them as "fresh" even though their LRU position marks them as old—this contamination problem makes the timestamp approach worse than pure LRU for the fallback path. This analysis suggests that combining timestamps with frequency guards doesn't help and may actually harm the eviction strategy, so I should probably move forward with a different approach.

Let me focus on the most defensible hypotheses instead. The soft-cap on the LFU counter is worth exploring—while it might not change behavior under the binary guard, it could prevent overflow and potentially interact with how new items are initialized, especially if I pair it with different initial values for items entering through the second-chance mechanism versus stash hits.

The real insight is what happens if I move beyond pure binary protection: items at the cap threshold could be treated differently during eviction, falling back to LRU position, while items below the cap get similar treatment. But that's still fundamentally binary, so I'm not gaining much ground there.

Let me zero in on three concrete approaches worth testing. The first one uses a soft cap at 4 where items that hit the cap are considered "hot" and get lower eviction priority than warmer items below the cap, which still get lower priority than cold items with zero counter. This gives me a tiered eviction strategy without the complexity of tracking exact frequencies beyond the cap.

To implement this, I'm adding a new eviction policy struct that caps the frequency counter at 4 on each access, then during eviction picks the item with the lowest counter, breaking ties by LRU position. The promotion logic stays the same as standard LRU. I'll swap out the existing policy in the bucket code to use this new variant.

Now I'm reconsidering the second-chance promotion mechanism—specifically what happens when a stash overflow displaces a resident item to the target. Currently that displaced item keeps its counter at zero, but I'm wondering if crediting it to 1 on promotion would better reflect that it's already been accessed once and deserves slightly more protection.

The tension is whether surviving in stash long enough to be present during overflow actually signals value or just means it happened to be there when someone else triggered eviction. Without explicit access, the credit feels unearned, but there's an argument that pre-warmed items might benefit from a small boost to avoid immediate re-eviction.

If second-chance items got counter=1 like stash-hits, they'd survive one extra eviction round before drifting back to LRU position and getting evicted anyway. The practical effect would be a slight delay in cold item eviction, which could actually help if that delay frees up space for genuinely hot items. But this seems more harmful than helpful—giving unproven items the same protection as proven reuses doesn't make sense.

I'm moving away from this approach and reconsidering the timestamp idea instead, specifically whether recording the timestamp only on first hit rather than insertion would address the cross-cycle insights.

The key insight is that "first hit" means the first time an item is accessed after entering a tier. Before that, it only has a creation timestamp. The real contamination problem would occur if we used timestamps for promotion decisions—fresh newcomers would beat hot items that haven't been accessed recently. But looking at the current implementation, promotion already uses position-based logic (MRU end), not timestamps, so the contamination issue wouldn't actually arise. Timestamps would only affect eviction ordering among already-accessed items, which is safe.

The "rehabilitated TimestampLRU" approach would combine frequency protection (keeping accessed items safe from eviction) with timestamp-based tie-breaking for eviction decisions, while only updating timestamps on actual hits to avoid insertion-time contamination. For eviction, we'd want to remove the least recently accessed items, so we'd evict the oldest timestamps first—but this means cold items with recent creation timestamps would actually survive longer, which might not be ideal. counter items). This is cleaner than the position-based fallback because timestamps reflect actual access times rather than insertion order, so the hybrid approach avoids the contamination problem entirely—nonzero items have hit-based timestamps, not creation timestamps, making the eviction decision genuinely LRU-based.

In steady state, LRU position and timestamp should align perfectly for nonzero-counter items since each access moves them to MRU and updates their timestamp simultaneously. The only divergence happens when items get inserted with stale timestamps (like stash-hit promotions with fresh counters but old entry times), but I'm realizing there's a subtlety with how stash-exit credits are applied in the current code.

When an item exits the stash with a credit, it gets counter=1 and is inserted at the MRU position, but its timestamp reflects when it was originally created—far in the past. This creates a mismatch: the item is protected from zero-guard eviction due to its nonzero counter and sits at MRU (correctly reflecting recent access), but under a hybrid oldest-timestamp fallback strategy, it could become a candidate for eviction despite being freshly promoted.

The fix is to update the timestamp when granting stash-exit credit, so the item's timestamp reflects its actual re-entry into the target cache rather than its original creation time. I need to modify the stash exit logic to refresh the timestamp alongside incrementing the counter, ensuring consistency between the item's position, counter, and temporal metadata across both the eviction policy and the segment code. Also need to update the timestamp in `dash_segment.rs` when applying stash-exit credit, and switch `dash_bucket.rs` to use the new `ClassicLRUFreqProtectTimestamp` policy. The hypothesis is that using oldest-timestamp as the eviction fallback among frequency-protected items, combined with refreshing timestamps on stash-exit promotion, should improve hit rate by providing better recency ordering. Though I'm realizing that in steady state, LRU position and oldest-timestamp should track together anyway—the real benefit would only show up for stash-hit items with stale timestamps (which we'd fix) or items inserted at specific positions without going through the normal access path.

When a demoted item from probing gets retried in its true target bucket via `put_at`, it enters with its existing counter and a timestamp from whenever it was last accessed, which could be quite old.

Under LRU-position fallback, if it lands at Front, it becomes the eviction target regardless of how recent its timestamp actually is. Under timestamp fallback, an old last-access time might get it evicted, while a recent one would protect it. There's potentially a meaningful difference here for cascade-demoted items, though it's hard to predict without testing whether this helps or hurts overall.

Since `promotion_get` specifies insert_pos=Back and cascade paths inherit that same positioning, demoted items enter their retry tier at the MRU position with a fresh timestamp, so in steady state there's probably no real difference. The only scenario where this matters is when an item's position and timestamp are misaligned, which seems uncommon enough that this hypothesis might be too subtle to show measurable improvement.

I should try a completely different angle for these hypotheses. Let me examine what parts of the code haven't been optimized yet, particularly the promotion candidate selection logic that returns the MRU item and determines whether something qualifies for promotion from target to probing tier.

What if instead of always promoting the MRU item, I promoted based on access frequency? The item with the highest counter in a bucket would be the most frequently accessed, though this conflicts with the MRU principle that's been consistently winning on the workload. I could try combining both—promote by highest counter but use recency as a tiebreaker—which would be closer to LFU promotion, but LFU has historically underperformed, and this is specifically for promotion rather than eviction.

There are a few other angles worth exploring: incrementing the counter when an item gets promoted to probing, so promoted items start with an extra count marking them as "proven hot"; or adjusting the stash bucket size relative to normal buckets, which would require tweaking the geometry configuration to give stash buckets fewer items per bucket than regular buckets.

The second-chance insertion position tuning doesn't seem promising since the counter is always reset to zero at demotion time, so it would always insert at the front regardless.

Let me focus on three orthogonal approaches: soft-cap with ranked eviction using a novel counter and eviction strategy, immediate promotion when items hit in the probing tier, and a completely independent exploration strategy separate from the baseline.

For the probing promotion idea, I realize there's actually no tier above probing to promote to—probing is already the hot tier. I could try reserving a dedicated champion slot at the MRU end of probing for items accessed beyond a threshold, but that would be complex to implement within the current framework.

Another angle: when demoting from probing to stash, I could credit the item with a partial counter instead of resetting it to zero, since items from the hot tier are likely to be reaccessed sooner. But this runs into the same problem I identified before—if demoted items keep their counter, cold newcomers get evicted first, which defeats the purpose of stash as a cold-item incubator.

Let me explore a different direction: when a stash hit gets promoted to target and displaces an existing item, credit that displaced item's counter based on the stash hit's access history.

Or maybe the issue is simpler—instead of evicting only zero-counter items, what if I raised the threshold to evict items with counter ≤ 1, treating single-access items as cold noise? But the data from cycle 7 showed that on this Zipfian workload, a single in-tier access is actually a reliable reuse signal, so that approach would backfire.

I need to stop speculating and commit to concrete hypotheses I can actually test: a soft-cap with minimum counter eviction, recording first-hit timestamps in the stash to avoid contamination, or using a counter-based promotion signal from target. But some of these don't quite work with how the code is structured. Let me step back and look at the actual GET code path with fresh eyes to find where real optimization opportunities actually exist. Continuing the cascade logic where items demoted from the target bucket either go back to the stash at the LRU position with their counter reset, or get retried in their true target bucket. I'm realizing that Front actually represents the LRU end of the list while Back is the MRU end, which means second-chance items entering at Front are positioned as immediate eviction candidates. But second-chance items entering target at Front with counter=0 become immediate eviction candidates since they're the first zero-counter item found, making them vulnerable to quick eviction despite their LRU position. The stash-exit credit protects stash-hit items by giving them counter=1 before target entry, but second-chance items lack this protection and remain at counter=0.

There's a tradeoff: protecting second-chance items with counter=1 could help items that survived in stash and might have reuse value, especially in workloads like Zipfian where the hot set is stable. But the risk is that these items weren't explicitly accessed, so protecting them might waste cache capacity on truly cold items. Still, even with counter=0, second-chance items do get a chance in target since they're inserted at the LRU position with room to be accessed before eviction. So the bucket's `put` method likely inserts first then checks capacity, meaning a second-chance item inserted at the front with counter=0 would immediately become the eviction candidate and get removed. This only works in the second-chance item's favor if target isn't already full—otherwise it gets evicted right away and provides no benefit.

I'm wondering if giving second-chance items counter=1 before they enter target would protect them from immediate eviction, but that's risky for cold items. The current approach (v003) reserves counter=1 for stash-hit items since they've been explicitly accessed, whereas second-chance items haven't. Though on Zipfian workloads, the stash serves as a cold-item filter, so maybe the distinction matters differently there.

The real question is whether second-chance items that survived stash overflow deserve some protection credit in target. I can test this empirically by comparing hit rates when second-chance items start with counter=1 versus counter=0. This would be a separate hypothesis from the soft-cap approach.

Let me outline three concrete hypotheses to test: first, soft-capping lfu_counter at 4 with min-counter-then-LRU eviction ordering from the current champion.

For the second hypothesis, I'd give second-chance items an lfu_counter of 1 when they're promoted to target, without changing the eviction policy itself.

For the third, I'm exploring updating the timestamp on every hit and using min-timestamp as a fallback eviction criterion for protected items—a fresh axis distinct from frequency-based approaches. The issue is that items created long ago but recently accessed via stash hit would have stale timestamps, making them vulnerable to eviction despite the recent access. I need to update the timestamp when promoting from stash to target tier, right before the promotion happens. Second-chance items with zero counters are already protected by the zero-guard check, so their timestamps don't affect the fallback eviction logic.

Now I'm thinking through whether H03 actually behaves differently from the current approach in practice. For cascade items coming from probing, their timestamps should be fresh since they were just accessed. But I'm realizing there's a scenario where the behavior might diverge—if an item gets accessed but doesn't move to MRU position for some reason, its timestamp could be newer than its position suggests, which would change the eviction order compared to pure LRU. Every access operation moves an item to the back and increments its counter, so the LRU position ordering naturally aligns with timestamp ordering. The only way they diverge is when items are inserted with non-default timestamps—like stash hits entering the target with old creation timestamps. But applying the timestamp refresh on stash-exit eliminates this divergence entirely, keeping positions and timestamps synchronized.

So H03 with that timestamp refresh would be functionally equivalent to the current implementation, which wouldn't give me meaningful test results. I need to step back and focus on what's actually worth testing. The cross-cycle insights consistently point to soft-capping the LFU counter to prevent staleness, and while I've analyzed why it might not help under the current binary guard, the community consensus says it's worth trying. The most concrete change would be switching the eviction fallback from pure LRU to preferring lower counter values while keeping the zero-guard first—that's a real, testable difference that could matter, though it risks hurting single-touch items unless the cap is tuned right.

I'm settling on three distinct hypotheses: first, soft-capped LFU counter with min-counter-then-LRU eviction to address the open question about capping while making the counter meaningful for ordering; second, giving stash survivors the same +1 credit as explicitly-accessed items to test whether that asymmetry matters; and third, intra-probing counter reset.

For the third hypothesis, I'm considering whether items entering the probing tier should reset their counter to zero, forcing them to prove themselves again in that tier. Combined with the zero-guard protection, items would need to survive their first probing hit to gain counter=1 protection. But this raises a question about the parent version—testing this in isolation on the baseline architecture would be different from testing it with all the v003 features present, so I need to decide which parent makes sense for each hypothesis. I'm realizing that second-chance recovery was already tested in cycle 6 and won, so exploring it again from baseline wouldn't be novel. I need to figure out what the baseline actually inherits in v003—specifically whether it already has frequency protection and other mechanisms built in, which would change what's genuinely fresh to test. Stash size based on hit patterns could work, but since the major axes are already explored, I'll go with all three hypotheses from the v003 incumbent instead—they're distinct enough and the framing prefers non-baseline strategies. My first hypothesis is a soft-capped LFU counter with ranked fallback eviction...

Implementing ranked fallback eviction... Implementing second-chance promotion for stash-overflow items by crediting them with an initial `lfu_counter` value of 1, treating them similarly to items that hit in the stash, which should improve hit rates by giving recently-evicted warm items a better chance to survive subsequent evictions.

But reconsidering this approach—the original rationale was about protecting explicitly-accessed stash hits, whereas second-chance items weren't actually accessed, just happened to be in stash when overflow occurred. Giving them counter=1 might over-protect cold items that ended up there coincidentally. Looking at workload-a specifically, most stash residents are cold newcomers sitting in FIFO order, and when overflow happens, the oldest item gets evicted—if there's temporal locality, that oldest item might actually be worth keeping around.

The tension here is real: in a Zipfian workload, an item that's been sitting in stash without being accessed probably has low rank and is cold, so protecting it seems wrong. But there's a counterargument that if the stash is filling rapidly, that oldest resident might be semi-warm and just unlucky in timing. This asymmetry between stash-hit credit and second-chance protection is worth testing empirically.

Now I'm settling on a concrete hypothesis: when an item gets promoted from the target bucket to probing, I should reset its counter to zero before the promotion happens. This way, items entering probing start fresh and aren't automatically protected by their accumulated counter from the target bucket.

The effect is that newly promoted items become vulnerable to eviction unless they're accessed again in the probing tier, which would increment their counter to 1. This creates a freshness mechanism where items that earned promotion but don't get re-accessed become the first candidates for eviction. The tradeoff is that items might get evicted from probing right after promotion if they're not immediately accessed again, which could hurt performance in certain workload patterns.

### 02:00:42 thinking

Let me now synthesize what I've learned and write the three hypotheses. Here's my analysis:

**Current champion**: `cycle/0009/v003` with hit-rate=0.724
- Has: 7/8 geometry (28 normal + 4 stash), ClassicLRUFreqProtect, second-chance stash recovery, stash-exit lfu_counter credit

**What has been exhausted**:
- Geometry: 7/8 is the ceiling, both directions confirmed
- Ghost set: noise-level
- Promotion gates: ineffective at 7/8
- Frequency-based eviction and back-insertion for second-chance both proved harmful

**The real opportunity**: A soft-cap on `lfu_counter` to prevent staleness in long-lived hot items—this is the primary untested lever that keeps surfacing. I also need to explore whether timestamp promotion works when rehabilitated to only record on first hit rather than insertion, and how intra-normal-tier eviction ordering behaves at non-zero thresholds.

The core hypothesis is that `lfu_counter` accumulates unbounded for frequently-accessed items, which blocks newly-warm items from becoming eviction candidates when truly cold items aren't available. Capping the counter at something like 8 or 16 in the get operation should let the fallback path in eviction logic be more responsive to actual access patterns rather than historical accumulation.

But looking at the actual eviction logic, it only checks whether `lfu_counter == 0` versus non-zero—the exact counter value doesn't factor into the decision. So soft-capping alone won't change anything unless I also modify the eviction logic to use counter values more granularly, like preferring items with lower counts when no zero-count items exist instead of defaulting to position 0.

The soft-cap concern from the cross-cycle notes is about long-lived hot items accumulating artificially high counts that crowd out newly-warm items, but that's only relevant if the eviction logic actually compares counter values. With the current binary zero/non-zero check, I need to think about what `lfu_counter` is really tracking here.

Looking at where it's used: it gets incremented on access and checked in eviction (zero vs non-zero), but the actual tier movement is purely LRU-based. The counter resets to zero on stash demotion and gets a +1 credit on stash-exit promotion, but once an item is back in the tier, the counter doesn't drive ordering—only whether it's been touched at all.

The staleness concern might actually point to a potential optimization: if we modified the eviction fallback to prefer items with the lowest counter value instead of just taking the LRU-front position, we could use that accumulated information as a tiebreaker to improve hit rates. Currently it just evicts the oldest item when all residents have been touched.

H01 proposes exactly this — change the fallback path to select the item with the minimum non-zero counter rather than the LRU-positional front. Even though the LRU item is reasonable, the least-frequently-accessed item might be a better target since it's been touched less relative to its peers. The history warns that pure LFU is incompatible with workload-a, but H01 only uses frequency as a secondary signal when the primary zero-count check doesn't apply, so the core protection stays intact.

H02 is exploring a soft cap on the counter itself, like capping it at 8. The concern is that if fallback eviction uses counter values, long-lived items with artificially high counts could displace genuinely warm newer items. A soft cap prevents this, but only matters if the eviction logic actually consults the counter. So I should test counter-aware fallback first—changing the fallback to evict the minimum-counter item when no zero-count candidate exists—then layer on the soft cap as a second hypothesis to isolate which change drives improvements. Actually, the soft cap wouldn't matter in the current binary scheme since we only check whether the counter is zero or non-zero, never comparing actual values. Let me reconsider the remaining optimization axes: soft-capping the counter itself, rehabilitating timestamp promotion by recording only on first hit rather than insertion to avoid creation-time contamination, and using graduated thresholds within the normal tier to guard items with multiple hits.

The core issue with `TimestampLRU` is that it updates timestamps on every access, but newly inserted cold items get fresh timestamps that can be newer than hot items' last-access times, which breaks the promotion ordering when using `max_by_key(timestamp)`. So the real issue is how the promotion candidate gets selected within the segment logic. With `ClassicLRUFreqProtect`, it picks the MRU item at the end of the list, and an item only gets promoted to probing if it's at that position and gets accessed again. I'm wondering if there's a different promotion strategy that could work better here.

What if instead of promoting based on recency, I promoted based on the frequency counter that's already being tracked? The item with the highest hit count within its bucket would be the promotion candidate, not just the most recently accessed one. This would mean items that have been consistently accessed many times are more likely to get promoted to the probing tier, which is a more reliable signal of hotness than a single recent access.

Looking back at the experiment history, frequency-based eviction was introduced in cycle 6, and various combinations were tested, but frequency-based *promotion* hasn't actually been tried yet — promotion has always used the MRU position. The current implementation moves accessed items to MRU and increments their frequency counter, so an item accessed many times would naturally end up at MRU, but that's a side effect rather than an intentional design choice.

The key insight is that LFU-based promotion would differ meaningfully from MRU-based promotion. With MRU promotion, an item needs to be at the most-recently-used position AND get accessed again to be promoted. With LFU promotion, the item with the highest frequency counter could be promoted regardless of its position in the LRU order, which is a fundamentally different mechanism.

I'm reconsidering the concern about LFU being harmful. The history shows LFU eviction is problematic because it discards low-frequency items that may be new and haven't accumulated counts yet. But LFU promotion is the opposite—we're selecting the best candidate to move up, not choosing a victim to remove. Given that the workload is Zipfian and recency-dominant, promoting the most-frequently-accessed item within a tier seems like a solid strategy.

Now I'm thinking through how items in the probing tier (the hottest bucket adjacent to the target) handle eviction and promotion. Currently they use the same `ClassicLRUFreqProtect` mechanism, and when an item gets hit in the probing tier, it stays in place with its counter incremented and moves to the back of the LRU order.

There's an unexplored angle here: when items get demoted from the normal tier, should they preserve their frequency counter instead of resetting it to zero? Right now everything going into stash starts with `lfu_counter = 0`, and the stash-exit credit gives them a +1 on re-promotion. But what if we carried over a fraction of the counter—say, half of it—to retain some heat signal while accounting for staleness?

The tension is that the original reset-to-zero design was meant to prevent demoted hot items from monopolizing stash slots against cold newcomers. But if we preserve any counter value, those demoted items would get protection from stash eviction through the same `ClassicLRUFreqProtect` mechanism, which could actually be beneficial by keeping genuinely hot items around longer.

Let me formalize this into three concrete hypotheses to test. The first modifies the fallback eviction logic to use frequency as a tiebreaker when no zero-count items exist, targeting cases where all residents have been accessed but some are much less-accessed than others. The second changes promotion to select the highest-frequency item instead of the most-recently-used one.

The third hypothesis preserves a fraction of the frequency counter on demotion rather than resetting it to zero, giving demoted-hot items some protection in the stash while capping staleness—though this conflicts with v003's stash-exit credit since we'd be removing that mechanism. The partial preservation might be a net win over the modest +1 credit, but I need to think through whether the tradeoff is worth it. For H03, I'd implement a hit-only timestamp approach where we only record access times on cache hits rather than on insertion, avoiding the problem of cold items getting contaminated timestamps. This would be a new policy variant that inherits all the prior optimizations from the v003 winner rather than starting from scratch. I need to check whether modifying `item.rs` is actually feasible given the gate-integrity constraints. Since `item.rs` isn't in the gate-integrity surface, I can safely add a `last_hit_timestamp` field to track hits separately from insertions, which would let H03 test whether hit-only timestamps eliminate the creation-time contamination problem. I'm considering a counter-range eviction approach where instead of just checking for zero, I'd prefer evicting items in the lowest third of the counter range within a bucket — but that's getting complicated. Let me think about something simpler for H03, like preserving a minimal counter value during stash demotion rather than resetting it completely.

The idea is that when a hot item gets demoted to the stash, it currently arrives with counter=0, making it just as vulnerable as a cold newcomer to eviction. If I preserve counter=1 instead, it gets some protection against zero-count-preferring eviction policies without accumulating a stale counter. But this creates tension with the existing design principle that demoted items shouldn't hoard stash slots from genuine cold newcomers, and the v003 logic that credits stash-hit access on the counter before insertion.

The deeper pattern I'm noticing is that every attempt to weaken stash admission—whether through front-door bypass for cold insertions or LFU-gated promotion—has degraded hit-rate by letting cold or slow-qualifying items displace warm residents. The stash's core function as an admission filter seems fundamental to this workload. So if I preserve counter=1 on demotion, I'd flip the dynamic: cold newcomers with counter=0 would get evicted from the stash before warm demoted items, which could either keep warm items around longer or prevent cold newcomers from filtering through and block stash admission entirely.

This would likely fail for the same reasons as other stash-weakening attempts, so I'll skip it. Let me explore a different angle with H03 instead—preserving partial counter credit when items cascade down from the probing tier. An item that reached probing is hot, but when it gets evicted from probing due to an even hotter item being promoted, it resets to counter=0, which seems wasteful.

The key insight is separating counter behavior at stash entry (which affects stash-tier eviction) from counter behavior at target entry after stash promotion (which affects target-tier eviction). v003 only modified the latter by adding +1 credit before promotion, while stash entry always resets to 0. What if I preserve the counter directly at target entry without changing stash entry behavior?

Looking at the demotion paths: items going from probing to target keep their counter, but items going from target to stash get reset to 0. The stash-exit credit adds +1 when promoting back to target, so stash-promoted items enter with counter=1 versus items entering target from probing.

I'm considering whether to preserve the counter when demoting directly to stash—maybe capping it at 2—but that creates ambiguity in the stash eviction policy since we can't distinguish true newcomers from demoted warm items without resetting. After multiple refinement cycles, the current best approach uses 7/8 geometry with ClassicLRUFreqProtect and second-chance stash recovery.

Now I'm exploring a few remaining unexplored directions: using minimum counter instead of pure LRU when no zero-count victim exists, switching promotion candidates to maximum counter rather than MRU position, or adding a hit-timestamp field to track when items were last accessed and using that for promotion decisions while keeping LRU for eviction.

The core issue with the hit-timestamp approach is that if `Item::new()` initializes the timestamp to the current moment, newly created items appear recently accessed even without any cache hits, contaminating the signal. I'd need to either initialize the timestamp to a very old value or track hits separately from creation time. in that case, a newly inserted item with a fresh timestamp would be promoted over an older accessed item, which defeats the purpose. The real problem is that both insertion and access update the timestamp to now, making them indistinguishable for the promotion logic.

The solution is to initialize new items with a far-past timestamp in `item.rs`, so only actual accesses update to the current time. This way, `TimestampLRU` correctly identifies the most recently *accessed* item for promotion and the least recently accessed item for eviction, while fresh items naturally fall to the front of the eviction queue.

However, there's a complication: `ClassicLRUFreqProtect` is hardcoded in `new_bucket()` and includes a frequency guard that's been consistently beneficial. Swapping it out entirely for `TimestampLRU` would lose that protection and likely hurt performance. A hybrid approach—keeping `ClassicLRUFreqProtect` for eviction while using `TimestampLRU` for promotion—might work, but the promotion logic is tightly coupled to the eviction policy.

Let me explore a cleaner option: **graduated eviction threshold**. Instead of the binary protected/unprotected split in `ClassicLRUFreqProtect`, I could always evict the item with the minimum counter value, eliminating the two-pass logic. But that's just pure LFU, which the history confirms fails on workload-a.

Now I'm looking at another angle around **eviction position within the counter class**. The current implementation finds the first zero-count item (which is naturally LRU-ordered since items move to the back on access), and falls back to the LRU-front item. What if instead, when multiple zero-count items exist, I evict the most recently inserted cold item—the reasoning being that a newly arrived cold item is less likely to become hot than an older cold item.

But wait, that logic flips: if an item's been sitting at zero-count for a while, it's genuinely cold, whereas a recently inserted zero-count item might just be a new arrival that hasn't had a chance to warm up yet. Actually, both are equally cold if their counter is zero.

There's something important here though—when a second-chance item arrives at the target tier, its counter gets reset to zero on stash entry, then the stash-exit credit bumps it to one before entering the target tier, so it's protected from the zero-count eviction guard. But plain cold items enter the stash at zero, and if they get evicted from stash via second-chance, they enter the target tier through a different path.

Looking at the code paths, the stash-exit credit only applies when items are retrieved from the stash (a hit), not when they're evicted due to overflow. So items that overflow from the stash and get a second chance at the target tier actually arrive with their counter still at zero—they're still vulnerable to eviction.

The fix would be to apply that same +1 credit before promoting second-chance items in the `put` method, giving them the same protection that stash hits get. This is a distinct hypothesis from the eviction fallback or promotion selection changes, and it's testable by modifying the second-chance recovery path in `dash_segment.rs`.

But there's a tension here: a stash overflow means the item was evicted without being accessed—it didn't prove its hotness through any actual use. Giving it a +1 credit might be premature protection for something that was already deemed cold enough to evict.

The counter-argument is that the item was recently in the stash and just got displaced by a cold newcomer, so it deserves a small chance to survive one eviction cycle in the target tier. Since the workload is hit-rate dominant though, protecting items that won't be hit anyway doesn't help. The current mechanism already gives it a chance by inserting it at the front—if it gets hit, it'll prove its value.

Given that second-chance plus stash-exit credit on hits both worked before, and this is a symmetrical application of that same credit idea to the overflow path, this feels like a solid candidate worth testing.

Now I'm laying out my three main hypotheses to explore.

H02 should promote the item with the highest frequency counter instead of just picking the most recent one, using recency as a tiebreaker when frequencies match. This targets the promotion mechanism to favor the most-accessed items within the tier.

H03 applies the stash-exit counter credit to second-chance recovery items during the put operation, matching the credit already given to stash hits in promotion_get. This addresses a parallel protection gap for overflow-evicted items.

All three hypotheses stem from the same parent cycle, target different code locations, and are independently testable without repeating prior attempts. Now I'm setting up the hypothesis files in the output directory.

### 02:00:44 chat

Now I have enough context. Let me synthesize the three hypotheses:

**What's confirmed exhausted**: geometry (7/8 is ceiling), ghost sets, promotion gates at 7/8, LFU eviction, Back-insertion for second-chance.

**Open axes flagged but never tested**:
1. `lfu_counter` fallback-eviction refinement (use `min_by_key` instead of LRU-front when no zero-count victim exists)
2. Frequency-based promotion candidate (promote most-in-tier-accessed, not just most-recently-accessed)
3. Second-chance recovery items get the same stash-exit counter credit that stash-hits get (symmetric application of v003's fix)

### 02:00:58 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0010/hypotheses/H01.md",
  "content": "# H01\n\nUsing `min_by_key(lfu_counter)` as the fallback eviction victim in `ClassicLRUFreqProtect` when no zero-count item exists will improve hit-rate by evicting the least-frequently-accessed resident rather than the arbitrary LRU-front item.\n\n## Parent\n\ncycle/0009/v003\n\n## Changes\n\nIn `src/shared/eviction_policy.rs`, modify `ClassicLRUFreqProtect::eviction_candidate_index()`.\n\nCurrent code:\n```rust\nfn eviction_candidate_index(&self, items: &[Item<K, V>]) -> usize {\n    items.iter()\n        .enumerate()\n        .find(|(_, item)| item.lfu_counter == 0)\n        .map(|(i, _)| i)\n        .unwrap_or(0)\n}\n```\n\nReplace `.unwrap_or(0)` with a secondary scan that picks the item with the minimum `lfu_counter` when no zero-count item is found:\n\n```rust\nfn eviction_candidate_index(&self, items: &[Item<K, V>]) -> usize {\n    if let Some((i, _)) = items.iter()\n        .enumerate()\n        .find(|(_, item)| item.lfu_counter == 0)\n    {\n        return i;\n    }\n    // All items have been accessed at least once; evict the one with the\n    // lowest in-tier access count rather than the pure LRU-front item.\n    items.iter()\n        .enumerate()\n        .min_by_key(|(_, item)| item.lfu_counter)\n        .map(|(i, _)| i)\n        .unwrap_or(0)\n}\n```\n\nNo other function changes. The first pass (zero-count preference) is preserved exactly; only the fallback path changes. The `get_from_position`, `promotion_candidate_index`, and all other logic in `ClassicLRUFreqProtect` remain identical to the parent. The geometry (7/8), second-chance recovery, and stash-exit credit from the parent are all preserved unchanged.\n\n## Rationale\n\nThe current fallback evicts index 0 (the LRU-front item), which is correct when all items are equally uncertain. However `ClassicLRUFreqProtect.get_from_position` accumulates `lfu_counter` values that are never used once the primary zero-count guard is bypassed. When the normal tier is well-provisioned (7/8 geometry, 28 normal buckets) and the workload is Zipfian, buckets frequently reach a state where every resident has been touched at least once — the fallback path fires regularly. At that point, a resident with `lfu_counter == 1` (touched once and sitting at LRU-front) is a meaningfully worse eviction choice than a resident with `lfu_counter == 8` (repeatedly accessed), even though the current code treats them identically. Using the accumulated counter as a frequency tiebreaker costs nothing beyond an additional linear scan and directly improves victim selection quality in the most-common fallback scenario.\n"
}
```

### 02:01:13 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0010/hypotheses/H02.md",
  "content": "# H02\n\nChanging `ClassicLRUFreqProtect.promotion_candidate_index()` to select the item with the highest `lfu_counter` (rather than the MRU-position item) will improve hit-rate by promoting the most in-tier-accessed item to the probing tier.\n\n## Parent\n\ncycle/0009/v003\n\n## Changes\n\nIn `src/shared/eviction_policy.rs`, modify `ClassicLRUFreqProtect::promotion_candidate_index()`.\n\nCurrent code:\n```rust\nfn promotion_candidate_index(&self, items: &[Item<K, V>]) -> usize {\n    items.len() - 1  // same as ClassicLRU: MRU end\n}\n```\n\nReplace with a scan that returns the index of the item with the maximum `lfu_counter`, breaking ties by choosing the MRU-most item (highest index):\n\n```rust\nfn promotion_candidate_index(&self, items: &[Item<K, V>]) -> usize {\n    items.iter()\n        .enumerate()\n        .max_by_key(|(pos, item)| (item.lfu_counter, *pos))\n        .map(|(i, _)| i)\n        .unwrap_or(items.len() - 1)\n}\n```\n\nThe `(item.lfu_counter, *pos)` tuple key means: first prefer the highest frequency count; among equal counts, prefer the MRU-most item (highest position index). When all items have equal counters (e.g., all zero on a fresh bucket), this degenerates to `items.len() - 1`, identical to the current behaviour.\n\nNo other function changes. `get_from_position`, `eviction_candidate_index`, geometry, second-chance recovery, and stash-exit credit from the parent are all preserved unchanged.\n\n## Rationale\n\nThe current `promotion_candidate_index` returns the MRU-front item (`items.len() - 1`). Because `get_from_position` moves the accessed item to MRU AND increments `lfu_counter`, the MRU item is always the item that was most recently touched — but not necessarily the item that has been touched the most times within this tier. Consider a bucket where item A has `lfu_counter == 1` (just accessed once, now at MRU) and item B has `lfu_counter == 6` (accessed six times, but bumped off MRU by A's recent access). The current policy promotes A; the proposed policy promotes B.\n\nPromoting the highest-frequency item to the probing tier is the correct choice for a Zipfian workload: items with high in-tier counters are genuinely hot and deserve the probing tier's protection, not just items that happened to receive the most recent single access. This is not LFU eviction (which has consistently failed) — it is LFU-guided *promotion selection*, operating on already-warm items to choose which deserves to move up, not which should be discarded. The `lfu_counter` values accumulated by `ClassicLRUFreqProtect.get_from_position` are currently unused for promotion; this change makes them useful at the promotion decision point.\n"
}
```

### 02:01:33 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0010/hypotheses/H03.md",
  "content": "# H03\n\nApplying the stash-exit counter credit to second-chance recovery items in `DashSegment::put()` will improve hit-rate by protecting overflow-evicted items from immediate re-eviction in the target tier, closing the same protection gap that was closed for stash-hit items in the parent.\n\n## Parent\n\ncycle/0009/v003\n\n## Changes\n\nIn `src/dash/segments/dash_segment.rs`, modify the `put()` method's stash-overflow second-chance path.\n\nCurrent code in `DashSegment::put()`:\n```rust\nif let Some(ev) = self.core.put_at(Slot::Stash, InsertPosition::Back, item) {\n    debug!(\n        \"DashSegment.put: stash evicted key={:?} -> second-chance -> target(Front)\",\n        ev.key\n    );\n    promote_to_target(&mut self.core, ev, InsertPosition::Front);\n}\n```\n\nChange to:\n```rust\nif let Some(mut ev) = self.core.put_at(Slot::Stash, InsertPosition::Back, item) {\n    debug!(\n        \"DashSegment.put: stash evicted key={:?} -> second-chance -> target(Front)\",\n        ev.key\n    );\n    // Credit the second-chance item before target entry, mirroring the credit\n    // applied to stash-hit items in promotion_get. The item arrives with\n    // lfu_counter == 0 (reset on stash entry); without this credit it is\n    // as eviction-vulnerable as a cold newcomer despite having been a recent\n    // stash resident.\n    ev.lfu_counter += 1;\n    promote_to_target(&mut self.core, ev, InsertPosition::Front);\n}\n```\n\nThe binding changes from `Some(ev)` to `Some(mut ev)` to allow the `lfu_counter` mutation before passing ownership. No other changes: `promotion_get` retains its existing `data.lfu_counter += 1` credit, `demote_from_normal_bucket` retains its `item.lfu_counter = 0` reset before stash insertion, and all geometry and policy logic is identical to the parent.\n\n## Rationale\n\nThe parent (v003) added `data.lfu_counter += 1` before `promote_to_target` in the stash-hit path of `promotion_get`, closing a protection gap: stash-hit items entered the target tier with `lfu_counter == 0`, indistinguishable from cold newcomers under `ClassicLRUFreqProtect`. v003's fix was confirmed to produce a genuine positive lift.\n\nThe same gap exists in the stash-overflow second-chance path. When stash overflows, the evicted item arrives at the target tier via `InsertPosition::Front` (LRU end) with `lfu_counter == 0`. Under `ClassicLRUFreqProtect.eviction_candidate_index`, this zero-count item is immediately the preferred eviction victim — it gets a second-chance insertion and can be evicted again before receiving a single access, if the target bucket is full or receives another cold insertion. The second-chance mechanism provides continued cache residency in principle, but without the counter credit the item enters the target tier maximally vulnerable.\n\nThe two paths — stash hit and stash overflow second-chance — are symmetric: both produce an item transitioning from stash to target with a zeroed counter. v003 closed the gap on the hit path; this hypothesis closes the analogous gap on the overflow path. The invariant that stash entries arrive at zero (so they don't protect stash slots from cold newcomers) is preserved, since the credit is applied after the item leaves the stash, not before it enters.\n"
}
```

### 02:01:41 chat

Three hypotheses written to the output directory. Summary:

**H01** (`cycle/0009/v003`) — Fallback eviction quality: when `ClassicLRUFreqProtect.eviction_candidate_index()` finds no zero-count victim, use `min_by_key(lfu_counter)` instead of falling back to LRU-front index 0. Exploits the counter values already accumulated by `get_from_position` as a frequency tiebreaker in the common case where all bucket residents have been touched at least once.

**H02** (`cycle/0009/v003`) — Frequency-guided promotion: change `ClassicLRUFreqProtect.promotion_candidate_index()` from `items.len() - 1` (MRU position) to `max_by_key((lfu_counter, position))`. Promotes the most in-tier-accessed item to the probing tier rather than the most recently touched, using the same counter values the policy already maintains — LFU as a promotion selector, not an eviction selector (a distinction never tested before).

**H03** (`cycle/0009/v003`) — Second-chance counter credit: add `ev.lfu_counter += 1` in `DashSegment::put()` before `promote_to_target` in the stash-overflow second-chance path. Mirrors the credit v003 applied to stash-hit items — both paths produce a stash→target transition with a zeroed counter; this closes the analogous protection gap in the overflow branch.


---

- finished: 2026-06-06T02:01:42+00:00
- duration: 932.88s
- status: ok
- input_tokens: 15
- output_tokens: 48128
- cache_read_tokens: 490900
- cache_write_tokens: 55711
- cost_usd: 1.079787
