# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0008/hypotheses
- allow_edits: True
- started: 2026-06-05T14:32:45+00:00

## timeline

### 14:32:49 thinking

Let me read all the context files to understand the current state of the research and generate informed hypotheses for cycle 0008.

### 14:32:55 tool: Agent

```json
{
  "description": "Read all context files for cycle 0008 hypothesis generation",
  "subagent_type": "Explore",
  "prompt": "Read the following files and return their complete contents. I need all of this information to write research hypotheses.\n\n1. /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/cycle-winners.json\n2. /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/cycle-insight.md\n3. /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/leaderboard.md\n4. /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles-ledger.jsonl\n5. /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cross-cycle-insights.md\n\nAlso list all variant directories under:\n/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/\n\nThen for each variant found (v001, v002, v003 etc.), read:\n- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/eval/fitness-vector.json\n- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/insight.md\n- The latest attempt overlay files for v001\n\nDo the same for any other variant directories found (v002, v003, etc.).\n\nAlso read the baseline files:\n- /home/user/cache-evolve/inputs/subjects/dash/baseline/src/dash/dash_bucket.rs\n- /home/user/cache-evolve/inputs/subjects/dash/baseline/src/dash/segments/dash_segment.rs\n- /home/user/cache-evolve/inputs/subjects/dash/baseline/src/shared/eviction_policy.rs\n- /home/user/cache-evolve/inputs/subjects/dash/baseline/src/shared/geometry.rs\n- /home/user/cache-evolve/inputs/subjects/dash/baseline/src/shared/item.rs\n\nReturn ALL file contents completely - do not truncate or summarize."
}
```

### 14:32:57 tool: Bash

```json
{
  "command": "find /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants -maxdepth 1 -type d -name \"v*\" | sort"
}
```

### 14:32:58 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v*/"
}
```

### 14:33:00 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/**"
}
```

### 14:33:02 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/cycle-winners.json"
}
```

### 14:33:02 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/cycle-insight.md"
}
```

### 14:33:03 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/leaderboard.md"
}
```

### 14:33:03 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles-ledger.jsonl"
}
```

### 14:33:03 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cross-cycle-insights.md"
}
```

### 14:33:05 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/eval/fitness-vector.json"
}
```

### 14:33:06 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/insight.md"
}
```

### 14:33:06 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v002/eval/fitness-vector.json"
}
```

### 14:33:06 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v002/insight.md"
}
```

### 14:33:06 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v003/eval/fitness-vector.json"
}
```

### 14:33:07 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v003/insight.md"
}
```

### 14:33:09 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/attempts/01/overlay/src/dash/dash_bucket.rs"
}
```

### 14:33:10 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/attempts/01/overlay/src/dash/segments/dash_segment.rs"
}
```

### 14:33:10 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/attempts/01/overlay/src/shared/eviction_policy.rs"
}
```

### 14:33:10 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/attempts/01/overlay/src/shared/geometry.rs"
}
```

### 14:33:13 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v002/attempts/01/overlay/src/dash/dash_bucket.rs"
}
```

### 14:33:13 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v002/attempts/01/overlay/src/shared/eviction_policy.rs"
}
```

### 14:33:13 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v003/attempts/01/overlay/src/shared/eviction_policy.rs"
}
```

### 14:33:16 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/dash/dash_bucket.rs"
}
```

### 14:33:16 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/dash/segments/dash_segment.rs"
}
```

### 14:33:16 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/shared/eviction_policy.rs"
}
```

### 14:33:17 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/shared/geometry.rs"
}
```

### 14:33:17 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/shared/item.rs"
}
```

### 14:33:18 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/inputs/subjects/dash/baseline/**/*.rs"
}
```

### 14:33:20 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/inputs/subjects/**"
}
```

### 14:33:22 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/src/dash/dash_bucket.rs"
}
```

### 14:33:22 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/src/dash/segments/dash_segment.rs"
}
```

### 14:33:23 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/src/shared/eviction_policy.rs"
}
```

### 14:33:23 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/src/shared/geometry.rs"
}
```

### 14:33:23 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/src/shared/item.rs"
}
```

### 14:33:43 thinking

Now I have a good understanding of the context. Let me also read the actual source files to get the concrete implementation details I need.

Let me read the specific implementation files to understand the exact function signatures and code that I'll be referencing in my hypotheses.

### 14:33:50 tool: Agent

```json
{
  "description": "Read v001 overlay source files for cycle 0007",
  "subagent_type": "Explore",
  "prompt": "Read the complete contents of these source files. Return them completely without any truncation or summarization — I need the exact Rust code:\n\n1. List what's in /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/attempts/ (to find the latest attempt number)\n\n2. Then read all files under the overlay directory of that latest attempt. Specifically:\n   - /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/attempts/01/overlay/src/dash/dash_bucket.rs\n   - /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/attempts/01/overlay/src/dash/segments/dash_segment.rs\n   - /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/attempts/01/overlay/src/shared/eviction_policy.rs\n   - /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/attempts/01/overlay/src/shared/geometry.rs (if it exists)\n   - /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/attempts/01/overlay/src/shared/item.rs (if it exists)\n\n3. Also read the baseline source files:\n   - /home/user/cache-evolve/inputs/subjects/dash/baseline/src/dash/dash_bucket.rs\n   - /home/user/cache-evolve/inputs/subjects/dash/baseline/src/dash/segments/dash_segment.rs\n   - /home/user/cache-evolve/inputs/subjects/dash/baseline/src/shared/eviction_policy.rs\n   - /home/user/cache-evolve/inputs/subjects/dash/baseline/src/shared/geometry.rs\n   - /home/user/cache-evolve/inputs/subjects/dash/baseline/src/shared/item.rs\n\nReturn all Rust source code completely."
}
```

### 14:33:51 tool: Bash

```json
{
  "command": "ls -la /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/attempts/"
}
```

### 14:33:53 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/attempts/*/overlay/src/dash/dash_bucket.rs"
}
```

### 14:33:54 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/attempts/**/*.rs"
}
```

### 14:33:56 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/attempts/01/overlay/src/shared/item.rs"
}
```

### 14:33:56 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/attempts/01/overlay/src/dash/dash_bucket.rs"
}
```

### 14:33:57 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/attempts/01/overlay/src/dash/segments/dash_segment.rs"
}
```

### 14:33:57 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/attempts/01/overlay/src/shared/eviction_policy.rs"
}
```

### 14:33:57 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/attempts/01/overlay/src/shared/geometry.rs"
}
```

### 14:33:57 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/dash/dash_bucket.rs"
}
```

### 14:33:58 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/dash/segments/dash_segment.rs"
}
```

### 14:33:58 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/shared/eviction_policy.rs"
}
```

### 14:33:58 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/shared/geometry.rs"
}
```

### 14:33:58 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/shared/item.rs"
}
```

### 14:34:00 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/**/*.rs"
}
```

### 14:34:01 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/inputs/**/*.rs"
}
```

### 14:34:03 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/src/dash/dash_bucket.rs"
}
```

### 14:34:04 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/src/dash/segments/dash_segment.rs"
}
```

### 14:34:04 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/src/shared/eviction_policy.rs"
}
```

### 14:34:04 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/src/shared/geometry.rs"
}
```

### 14:34:04 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/src/shared/item.rs"
}
```

### 14:35:52 thinking

I'll wait for the background agent to complete before writing the hypotheses.

### 14:35:52 chat

I've launched an agent to read the source files. I'll wait for it to complete before writing the hypotheses.

### 14:36:16 thinking

The sub-agent couldn't read the files due to permissions. Let me read them directly.

### 14:36:16 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/attempts"
}
```

### 14:36:17 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/cycle-winners.json"
}
```

### 14:36:18 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/cycle-insight.md"
}
```

### 14:36:18 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/leaderboard.md"
}
```

### 14:36:19 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cross-cycle-insights.md"
}
```

### 14:36:22 thinking

Now let me read the actual source code files from the v001 overlay and baseline.

### 14:36:24 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/attempts/01/overlay/src/shared/eviction_policy.rs"
}
```

### 14:36:25 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/attempts/01/overlay/src/dash/segments/dash_segment.rs"
}
```

### 14:36:25 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/attempts/01/overlay/src/dash/dash_bucket.rs"
}
```

### 14:36:26 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/attempts/01/overlay/src/shared/geometry.rs"
}
```

### 14:42:35 thinking

Now I have all the context I need to write the 3 hypotheses for cycle 0008. Let me analyze what I know:

## Summary of current state

**Champion (cycle/0007/v001, hit-rate 0.7222)**:
- `ClassicLRUFreqProtect` eviction policy: prefers zero-access items for eviction, falls back to LRU position
- Back-insertion (MRU) on all promotions from stash to target
- Second-chance stash overflow recovery: stash-evicted items → target at Front (LRU end)


- `lfu_counter` resets to 0 when items demote to stash

The current geometry maintains a 75/25 split between normal and stash regions, a topology that's been stable since the early cycles. Based on what I've learned across iterations, the main unknowns are whether we can optimize the stash size further and how to better manage frequency counter staleness—timestamp-based promotion might work if we only record on first access, though frequency-based approaches have consistently underperformed.

Probationary and ghost set strategies both failed, but MRU insertion has been the reliable winner, and these mechanisms seem to work well when layered together. The key remaining questions are whether the loss is coming from geometry constraints, counter staleness, or something else entirely—and whether resetting the frequency counter on demotion helps or hurts the second-chance mechanism.

I'm looking at three distinct approaches: first, tuning the stash geometry down to 1-3 slots instead of 8, building on what worked in the v001 overlay; second, exploring whether timestamp promotion can be salvaged through first-hit-only recording; and third, investigating how the frequency counter reset interacts with second-chance recovery.

For the geometry hypothesis, the current 24/8 split came from earlier cycle optimization, but the cross-cycle notes suggest sub-8 stash capacity hasn't been properly tested. I'm considering whether pushing to something like 28/4 or even tighter would reduce collision pressure in the stash while keeping the hot tier large enough to capture the working set.

The key insight is that with second-chance recovery in place, stash evictions aren't final anymore — items get a second shot at the target tier instead of being lost. That safety net means I can afford to shrink the stash and reallocate those buckets to the normal tier, which should improve hot-tier performance.

So I'm setting up hypothesis H01: reduce the stash to 4 buckets by adjusting the split ratio in the geometry calculation, keeping the total at 32 but shifting the allocation toward the hot tier.

Now moving to counter staleness — I need to think about how to manage soft caps on counter values to prevent them from growing unbounded.

The current implementation increments the counter on every access without any ceiling, so items that were hot early on stay protected even if they've gone cold. But here's the thing: the eviction logic only really cares whether the counter is zero or not, not the actual magnitude. So a soft cap wouldn't affect eviction ordering unless I'm trying to differentiate between items with different non-zero counts.

The real problem might be with how the counter resets when items get demoted from the stash.  When a hot item overflows and gets demoted back to the normal tier, it comes back with a zero counter, making it immediately vulnerable to eviction again. The staleness issue could be addressed through counter decay—periodically halving counters so items that were accessed frequently in the past but haven't been touched recently eventually become eviction candidates. This would need some kind of clock mechanism, like decaying counters every N puts. A simpler approach would be capping the counter at a fixed value like 4 or 8, though that doesn't directly solve the staleness problem—it just limits how much protection any single item can accumulate.

But there's an interesting tension here: when we do second-chance promotion from stash back to the normal tier, we're resetting the counter to zero to prevent recently-demoted hot items from hogging stash space. Yet for items being promoted back to normal, that same reset might be too aggressive—they're not newcomers, they have history. Could we preserve a non-zero counter during second-chance promotion instead?

Let me trace through the actual flow: items enter the stash without counter reset, but when the stash overflows and promotes an item to the target front, that item still has counter=0 from its original demotion. So the second-chance path is already working with a zeroed counter, which means the item immediately becomes an eviction candidate again in the target. That seems wrong — it should retain some memory of its hotness. The real issue is that when an item gets demoted to stash, its counter resets to zero, and if it then gets a second chance to return to the target tier, it arrives with no memory of being frequently used before. This means a previously hot item could end up as the first eviction candidate just because it happened to get demoted and promoted again. I should preserve the lfu_counter across that demotion-to-stash and promotion-back-to-target path so the item doesn't lose its access history. they're immediately evictable. I could instead preserve the counter when demoting to stash—setting it to 1 rather than 0 to give those items a grace period before they become vulnerable to eviction. This would protect items that get a second chance and re-enter the normal tier from being immediately kicked out again.

The change is straightforward: modify `demote_from_normal_bucket` to set `lfu_counter` to 1 instead of 0, keeping the eviction policy unchanged. But I need to think through the side effects—if demoted items keep a counter of 1, they'd be more protected in the stash than brand new cold items, which could block fresh entries from getting slots.

The issue is that both stash and the cascade-to-stash paths use the same reset logic, but they need different behavior. I need to look at how the code determines whether an item is going to the target tier versus being cascaded to stash.

Examining the demotion logic...

The second-chance eviction path already resets the counter to 0 when the item enters the stash, so setting it to 1 during demotion wouldn't affect those items—only ones that were already in the normal tier and got cascaded down. Let me reconsider what's actually driving the behavior here.

Actually, I'm wondering if the real issue is the insertion position for second-chance promotions. Currently they go to the front (coldest end) to be conservative, but since MRU insertion wins everywhere else, maybe using the back (hottest end) would be better. Though looking at the v001 implementation, it seems like stash-to-target promotions already use back insertion...

so the question is whether second-chance should follow that same pattern or stay conservative with front insertion. If I switch to back, second-chance items become immediately hot, which could cause churn if they displace legitimately popular items, but on Zipfian workloads where popular items get frequent hits, a stash-evicted item that survived long enough might actually deserve that priority.

For the third hypothesis, I want to explore something fundamentally different—maybe adjusting the bucket geometry itself. The baseline uses a 3/4 split between normal and stash buckets, but with second-chance recovery making stash eviction less terminal, I could test a more aggressive ratio like 7/8, which would mean fewer stash buckets overall since items have a recovery path anyway.

Alternatively, I could test timestamp-based promotion but only record the timestamp on first access rather than at insertion—this sidesteps the cycle-insight question about whether timestamp promotion could work if we're not constantly updating it on every hit.

The cross-cycle feedback warns this approach is risky and interacts poorly with frequency-guard eviction, so I should probably skip it and explore something simpler instead.

Let me consider a different angle: what if I selectively increment the LFU counter only under certain conditions?  Currently every access bumps it, but capping it at a small value like 3 wouldn't help since the policy only distinguishes between zero and non-zero protection. A more promising idea is to add age-based decay during eviction — when scanning for victims, items with stale timestamps get treated as unprotected regardless of their actual counter, giving cold items a better chance of being evicted even if they were historically hot.

The challenge is that ClassicLRU doesn't track timestamps, and defining the age threshold is tricky without knowing elapsed time or having a global insertion counter. Another angle is moving newly inserted cold items to the front of the stash instead of the back, so they become immediate eviction candidates rather than getting MRU protection. candidates, while older stash items (which may have been second-chance promoted and demoted) survive longer.

Wait, but the second-chance mechanism already handles stash eviction for brand-new items vs. older stash residents. Since all stash items have counter=0 (due to reset on demotion), the eviction order within stash is LRU (ClassicLRUFreqProtect falls back to LRU when all counters are equal). So changing Front vs. Back for new item insertion would just swap which ones get evicted first.

If we put new items at Front (oldest position), they'd be evicted first from stash (before older stash residents). This means newer cold items get second-chance promoted while older cold items stay in stash. This seems backward.

Current behavior (Back): newer items are at MRU, so older items (at Front, LRU) get evicted first. An older stash item that hasn't been hit gets evicted; the new item stays. This makes sense: older items are "less likely to be re-accessed soon" given the memory model. So Front for new insertions seems worse.

Now I'm considering whether second-chance items should go to the back of their target bucket instead of the front, making them immediately the hottest and requiring just one more access to promote to probing. Since MRU insertion generally wins, this could help.

I'm also thinking about whether to reset the counter only when items go to stash, rather than for all second-chance target entries. be evicted when target needs space. But if I initialize the second-chance item with counter=1 instead of 0, it would get one grace period before becoming vulnerable. The eviction logic searches for items with counter=0 first, falling back to index 0 if none exist, so a counter=1 item at the front would only be evicted if all other items also have counter≥1.

Let me settle on three concrete hypotheses to test instead of spiraling through edge cases.

The first approach is to shrink the stash tier since second-chance recovery prevents evictions from becoming permanent failures—I'd reduce it to a 7/8 split, keeping 28 items in the normal tier and 4 in the stash for a total of 32. This keeps more items accessible in the hot tier by adjusting the geometry calculation.

The second hypothesis flips where second-chance items get reinserted: instead of placing them at the front (LRU position), I'd move them to the back (MRU position) to give them immediate recency status, since MRU insertion has consistently outperformed other strategies across cycles.

The third hypothesis is still forming...

Exploring whether timestamp-based promotion becomes viable once we eliminate the contamination from creation-time initialization. I'd need to track whether an item has ever been accessed by using the counter as a sentinel—items with zero hits stay cold, while any hit increments the counter and updates the timestamp. Then the eviction policy would prefer removing untouched items first, using timestamp ordering only among items that have been accessed at least once.

This is converging toward what TimestampFreqProtect attempted before, but the key difference is deferring timestamp updates until the first access rather than setting it at insertion. I could represent this cleanly by using `Option<Instant>` where `None` indicates an item has never been touched, avoiding the need for a sentinel value in the timestamp field itself. have `Instant::now()` timestamps, which means the promotion logic in `TimestampFreqProtect` could be misled by newly created items. The real solution is to either skip timestamp initialization for new items or use `Option<Instant>` to distinguish between items that have never been accessed versus those that have, since `lfu_counter == 0` already tells us the item hasn't been hit yet. Looking back at `ClassicLRUFreqProtect`, it sidesteps this by just returning the MRU position for promotion rather than relying on timestamps, whereas the v002 `TimestampFreqProtect` ran into contamination issues because it tried to use timestamps for eviction decisions.

The fix requires modifying `item.rs` to use `Option<Instant>` and only record the timestamp on first access, then updating the eviction policy to respect this change.

But that would break existing policies like `TimestampLRU` that directly access `item.timestamp`, so I'm reconsidering the approach. Let me explore a simpler alternative: skewing eviction toward older items by weighting the scan position within the bucket, since items are already LRU-ordered with the oldest at index 0 and new items inserted at the back.

Actually, that's already what we're doing by scanning from the front. Let me try a different angle instead—what if I explore adaptive counter resets during stash demotion, where I preserve the counter for items that re-enter the normal tier instead of resetting it to zero? That's getting complex too.

Let me step back and try something more straightforward: testing a different geometry ratio starting from the baseline configuration.

Looking at the historical progression, early cycles expanded the hot tier from 8/24 up to 24/8, which peaked in cycle 3. The v001 geometry that's already winning at 0.722 uses that same 24/8 split. So the real question is whether we could push even further beyond that point.

For my three hypothesis slots, I need at least one building on v001, and the most productive approach would be: one that extends v001 by addressing another loss site, one that tries a different mechanism entirely, and one exploratory baseline variant to avoid getting locked into confirmation bias. Looking at what's failed across cycles—LFU, ghost sets, probationary insertion, bypass logic, timestamp tracking—I'm thinking about trying a skewed promotion strategy where stash-to-target moves don't just use MRU insertion but instead place items at a specific position to test whether that changes the dynamics.

Actually, the system only supports front and back insertion, so arbitrary positioning won't work. I could cycle back to the geometry-only change from baseline, but that's already proven ground. Or I could try a frequency-weighted eviction scan that looks across the entire bucket for zero-access items closest to the LRU position rather than just taking the first one linearly.

Let me focus on what's actually distinct and testable. I could apply the `ClassicLRUFreqProtect` eviction policy without the second-chance stash recovery to isolate each mechanism's contribution, though we already saw from cycle 0006 that the frequency guard alone achieved 0.719. Cycle 0007 then added the second-chance mechanism on top of that.

So the real question is whether I can push further by stacking a third mechanism onto the current champion. I'm considering a few approaches: fine-tuning the geometry by reducing the stash size, trying second-chance at the MRU end instead of the LRU end, or exploring other combinations.

Looking at the geometry parameters, I notice that `num_of_segments` controls how the key space gets partitioned — more segments means smaller buckets per segment but potentially fewer collisions. This hasn't been systematically explored yet based on the cross-cycle data. I should probably avoid over-exploring this space and focus on the most promising candidates instead.

So my final set of changes: H01 shifts the stash-to-normal bucket ratio from 3/4 to 7/8, which makes sense because second-chance recovery means stash evictions aren't permanent anymore, so we can afford a smaller stash and reclaim those slots for the hot tier. H02 changes where second-chance items get reinserted — moving them to the back (MRU position) instead of the front (LRU position) keeps them consistent with how the rest of the cache prioritizes recently-used items. H03 is about adding frequency-guard logic...

Actually, I'm realizing I need to think through the parent strategy more carefully. If I'm testing whether the LFU counter reset on stash demotion is actually necessary, I should probably build this from v001 rather than baseline, since v001 already has the ClassicLRUFreqProtect framework in place. Starting from baseline would mean rebuilding that entire structure just to remove one line. Let me reconsider what H03 should actually explore that's distinct from what I've already got.

What if instead I focus on using the timestamp field in item.rs more directly? I could initialize timestamps to an "old" sentinel value and use that to track first-hit behavior separately, or repurpose the LFU counter as a "has been accessed" flag while reserving timestamp updates for the first access only. Then in the eviction policy, I'd implement a FirstHitTimestampPromo strategy that records the timestamp on first access, increments the counter, and moves the item to MRU—essentially the same promotion logic as ClassicLRUFreqProtect but with explicit timestamp tracking.

For eviction, I'd prefer items with counter==0 (never accessed), and among items with counter>0, I'd select based on the oldest first-hit timestamp, falling back to MRU position if no counter>0 items exist. This cleanly separates cold items with insertion timestamps from accessed items with first-hit timestamps, avoiding the contamination problem that plagued the earlier version where every hit updated the timestamp. The real distinction here is that I'm only capturing the moment of first access, not refreshing it on every subsequent hit. So the key distinction is that `ClassicLRUFreqProtect` promotes based on position—the item at the MRU end gets promoted—while `FirstHitTimestampPromo` would promote based on which counter>0 item has the earliest first-hit timestamp, which might not align with recency of access. The promotion candidate logic differs fundamentally: one tracks position in the vector, the other tracks when an item first entered the hit set. In a busy Zipfian workload, hot items get repeatedly accessed which moves them around in the position list, so they'd naturally end up at the MRU position. But first-hit timestamp stays fixed to when they were first accessed long ago, so it would actually promote newer items that are just getting their first hit—likely cold items. That's worse than just using MRU position for promotion, so I'm dropping the first-hit timestamp approach.

Let me refocus on genuinely distinct strategies for H03 starting from the baseline. I could test whether removing the lfu_counter reset on stash demotion helps or hurts when keeping the frequency guard and geometry split, but without the second-chance recovery safety net. But that's just cycling back to what we already know—the frequency guard alone scored 0.719 in cycle 0006. I need to check what exactly that cycle tested to see if there's a different subset worth exploring.

Looking at the cycle 0007 insight about "the two Cycle 0006 gains" being additive, it sounds like cycle 0006 had both the eviction guard and something else—probably an earlier version of the second-chance mechanism. But I don't have the exact details of what cycle 0006 tested, so I'm trying to piece together what those two distinct gains actually were. mechanisms that were discovered independently. The insight also says: "frequency-guard eviction + second-chance stash recovery" are the two gains from cycle 0006 that v001 in cycle 0007 stacks together.

Wait, re-reading: "Only v001 — the direct stacking of both prior-cycle gains — won clearly." This says cycle 0007 v001 stacks "both prior-cycle gains." If both gains came from cycle 0006, that means cycle 0006 had two separate things discovered.

Or could it mean: one gain from cycle 0006 (frequency guard) + one gain from earlier (geometry)? The cycle 0007 insight specifically says "frequency-guard eviction + second-chance stash overflow recovery" are the two things stacked in v001.

Looking at cycle 0007 v001's implementation: it has both ClassicLRUFreqProtect AND second-chance recovery. If the second-chance was tested and shown to work in cycle 0006, then cycle 0007 v001 just combines them. But the insight says v001 in cycle 0007 was the first to combine them.

I think what's happening is: cycle 0006 v001 at 0.719 introduced frequency guard as its novelty over the previous cycle's 0.715, and cycle 0006 also discovered second-chance recovery as another gain. Then cycle 0007 v001 stacks these two together to reach 0.722. The key question now is what to try next.

So my three hypotheses are:

**H01: Reduce stash to 7/8 normal split (parent: cycle/0007/v001)**
Strategy

: geometry tuning — less stash, more hot tier. I'm adjusting the bucket allocation so the normal tier gets 7/8 of total buckets instead of the current split.

**H02: Second-chance at MRU (Back) insertion (parent: cycle/0007/v001)**
Strategy: apply MRU-wins principle to second-chance path. I'm changing stash overflow promotion to insert at the back instead of the front.

**H03: Decay-aware eviction — treat stale-hot items as cold victims (parent: baseline)**
Strategy: counter staleness management. I'm adding timestamp-based stale detection to eviction candidate selection, implementing a new policy that tries to find stale items first when evicting.

Now I'm realizing the fallback behavior might be equivalent to the existing approach — when all items have been accessed recently, both policies would end up selecting the oldest item by position or timestamp. The real difference only emerges if positions and timestamps diverge, which shouldn't happen since items move to MRU on each access. The counter never resets once an item enters the hot tier—only on demotion to stash—so an item accessed once stays permanently protected from eviction by frequency alone. But position-based LRU already handles staleness: items accessed long ago drift to position 0 and get evicted by the fallback, even if they have high counters. The highest-frequency item that's also the LRU candidate would be evicted, so staleness is indirectly covered. Now I'm trying to figure out what would actually differentiate H03...

For this alternative approach, new items start in stash and get promoted to target on first hit, then to hot when they're the hottest in target. Instead of jumping straight to the back on promotion, I could use lazy admission where items enter at a position based on their frequency counter—but that's not possible with just Front and Back insertion. Another option is to add a counter threshold: require items to be both the hottest AND have counter >= 2 before promoting to hot, which would need changes to the promotion logic.

Modifying the promotion check... items to be accessed twice in the target tier before earning their way to probing. This might help by ensuring probing only has truly hot items, not one-hit wonders.

But the cross-cycle insights say: "Raising the frequency-guard threshold beyond zero is harmful" and "Single-touch items carry genuine reuse signal." This principle applied to the EVICTION side. Could it apply differently to the PROMOTION side? Requiring 2 target hits for probing promotion keeps the same eviction threshold (0 vs >0) but raises the promotion threshold.

This is a potentially interesting hypothesis because it's a DIFFERENT axis than what was tested before (eviction threshold vs. promotion threshold). It was explicitly NOT tested before (cross-cycle insights only note frequency-threshold issues for eviction). Worth testing!

**H03: Require lfu_counter >= 2 in target before promoting to probing (parent: baseline)**
Strategy: Raise the promotion threshold from target to probing — only truly hot items (2+ in-tier hits) earn the hottest tier.
File: dash_segment.rs (modify promotion_get to check counter >= 2 before promoting)
Also need: eviction_policy.rs with ClassicLRUFreqProtect (or hardcode from dash_bucket.rs)

Now I need to figure out what the baseline dash_segment actually does. The Explore agent told me baseline uses `InsertPosition::Front` for all promotions, while v001 switched to Back. For H03 I should either work within baseline's Front insertion framework or decide whether to adopt the Back approach.

Actually, I can make H03 cleaner by building on v001 (which already has the good structural changes) and just adding the promotion gate. But that would make H03 a third hypothesis from v001, and I want to ensure H01, H02, and H03 are genuinely distinct strategies, not just rephrasing the same idea.

Let me verify they're actually different: H01 changes the geometry to reduce stash space, H02 moves the insertion position from front to back for second-chance paths, and H03 adds a frequency gate requiring counter>=2 before promoting targets to probing. These hit three different parts of the pipeline—topology, insertion logic, and promotion criteria—so they're legitimately distinct. The instructions allow all three to come from v001 as long as at least one cites the cross-cycle winner and the strategies don't collapse into near-duplicates, which these don't.

Now I'm working through the detailed implementation for each hypothesis, starting with the geometry change to shift the normal/stash split from 3/4 to 7/8.

For the second hypothesis, I'm modifying the insertion logic to apply second-chance recovery at the MRU position rather than just at eviction, which should help retain recently-accessed items in the hot tier.

Now I'm looking at the third hypothesis about gating target-to-probing promotions...

I'm examining the promotion logic in `promotion_get()` where items in the target tier get promoted to probing. The idea is to add a counter requirement — only promote an item to probing if it's been accessed at least twice while in the target tier. Items that are hottest but haven't reached that threshold would stay in target rather than immediately jumping to probing, which should help filter out items that got lucky once but aren't truly hot. non-hottest (intra-bucket promote only):

I'm checking if an item exists in the target bucket, and if it's the hottest candidate with a counter of at least 2, I promote it to the probing tier—otherwise I just return it from within the bucket. The idea is that gating promotion on this counter threshold reserves the premium probing tier for items that have been confirmed hot twice, which should raise hit rates by being more selective about what gets promoted. But I need to trace through the actual sequence of operations more carefully—when an item moves from stash to target via `promotion_get`, it gets removed from stash, inserted into the target bucket, and then accessed, so I need to verify exactly when the counter increments relative to when the promotion check happens. wait — when do we call `get_from_position` to increment the counter? Let me trace through more carefully.

For a target bucket hit (non-hottest case), we call `get_from_position` which increments the counter and moves the item to MRU. But for a target bucket hit (hottest case), we call `remove_at` instead, so the counter doesn't get incremented — it stays at whatever value it had before the promotion.

This creates an interesting edge case: when an item first arrives from stash with counter=0, it's inserted at the MRU end of target, making it the immediate promotion candidate. On the very next access, `is_hottest` becomes true, so the item gets removed and promoted to probing without ever having its counter incremented.

For items already in target that aren't at MRU, they get their counter incremented and moved to MRU on access, then promoted to probing on the next hit. So most items reach probing within 2-3 target accesses depending on competition for the MRU position.

The H03 proposal would require counter >= 2 before promotion, but since `remove_at` doesn't increment the counter, items would need to accumulate those increments through multiple `get_from_position` calls first.

Tracing through this: an item enters target with counter=0 at MRU (back insertion), and with the gate in place, it stays in target on the next access, gets its counter incremented to 1, and moves to MRU. On the following access it's still below the threshold, so counter becomes 2 and it moves to MRU again. Only on the third access does it finally get promoted to probing. This means the gate effectively requires 3 target accesses instead of 1 to reach probing for items entering at the back, which is a significant increase in the work needed to earn promotion. On Zipfian traces, hot items will still get accessed frequently enough to reach probing, but cold items that accidentally made it to target could get stuck.

The real concern is whether this slows down promotion of genuinely hot items too much—the current design allows fast promotion in 1-2 accesses, which keeps hot items climbing quickly. But there's also a potential bug: if target buckets are small (size 2), an item enters at the MRU position and needs 2 more hits to accumulate counter=2, which could cause eviction pressure before it even gets promoted. counter increments to 2 at MRU, then on the next access with counter=2 it promotes to probing—so second-chance items also need 3 target accesses before promotion. This feels like a solid strategy. I'm laying out three concrete hypotheses to test: reducing the stash allocation ratio, inserting second-chance items at the back of target, and gating promotion to probing on a counter threshold of 2.

Now reconsidering H01—the current geometry uses a 3/4 split for normal buckets, but shifting to 7/8 would give us more normal buckets and thus more target+probing pairs, though each pair consumes 2 consecutive slots. expanding the hot tier at the expense of stash. With second-chance recovery, the stash becomes less critical since evicted items get another shot at the target tier. A 4-bucket stash still filters cold items but with tighter constraints—on Zipfian workloads where most items are rarely accessed anyway, they'd cycle through quickly enough. H01 seems like a solid hypothesis to test.

Now I need to generate the output files, starting with the dashboard snapshot...

Writing hypothesis files... The rest of geometry.rs remains unchanged from the previous cycle. The eviction policy, second-chance recovery mechanism, and LFU counter reset on demotion all stay the same.

For the second change, I'm testing whether moving stash-overflow second-chance promotions from the front to the back of the insertion order will improve hit rates. The theory is that placing these items at the hot end immediately aligns with the MRU-wins pattern seen consistently across earlier cycles. The modification is a single argument swap in the `dash_segment.rs` promotion call...

Updating the promotion logic to insert at the back instead of the front, so second-chance items get immediate hot-end positioning. Since these items were typically displaced by hash collisions rather than being cold, fast-tracking them to the probing tier makes sense.

The third hypothesis gates target-to-probing promotion on requiring an LFU counter of at least 2, which would reserve the probing tier for items that have demonstrated genuine popularity... Modifying the condition to check the counter value before promoting...

Now I'm tracing through the access pattern to verify the counter won't already be at 2 when an item becomes the hottest. When a new item enters at the back with counter=0, the first access marks it as hottest but the gate fails since counter is still 0, so it stays in the bucket and gets incremented. On the second access, if it's hottest again, the counter reaches 2 and triggers promotion—but I need to verify this timing is correct.

For items entering at the front as second-chance candidates, the behavior might differ since they already have some history.

The gate mechanism creates a tradeoff: items stay in target longer, which could fill those buckets faster and trigger more evictions cascading back to stash, but it also means fewer items get promoted to probing aggressively, potentially leaving more room there. I think the approach is sound enough to move forward with implementation.

Now I need to structure the output correctly—the claim comes first as a single sentence, followed by the parent identifier and then the specific changes in prose form.

### 14:42:35 chat

Now I have all the context needed. Let me synthesize the findings and write the three hypotheses.

### 14:42:35 tool: Bash

```json
{
  "command": "ls /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0008/hypotheses 2>/dev/null || echo \"DIR_MISSING\"",
  "description": "Check if output directory exists"
}
```

### 14:42:38 thinking

Good, the directory exists. Now let me write the three hypothesis files.

### 14:42:44 chat

Good. Now I'll write all three hypotheses. The key findings from my analysis:

- **Champion (cycle/0007/v001, 0.7222)**: uses `ClassicLRUFreqProtect` (prefer evicting lfu_counter==0 items), Back/MRU insertion for stash→target promotions, second-chance stash-overflow recovery at `InsertPosition::Front`, and lfu_counter reset to 0 on normal→stash demotion, plus a 24/8 (75/25) normal/stash geometry.
- **Open loss sites identified by cycle-insight**: stash geometry fine-tuning; counter staleness; second-chance insertion position; target→probing promotion threshold (unexplored axis).
- **All three hypotheses must be distinct**: I'll target geometry (H01), second-chance insertion position (H02), and target→probing promotion gating (H03).

### 14:42:50 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0008/hypotheses/H01.md",
  "content": "# H01\n\nReducing stash capacity to a 7/8 normal-bucket split (28 normal / 4 stash for 32-bucket total) will raise hit-rate by enlarging the hot tier, now that second-chance recovery makes stash evictions non-terminal.\n\n## Parent\n\ncycle/0007/v001\n\n## Changes\n\nIn `src/shared/geometry.rs`, change the single multiply constant in `layout()` from `3 / 4` to `7 / 8`:\n\n```rust\n// current (v001):\nlet normal = (total_buckets * 3 / 4).max(2);  // 24 normal / 8 stash for total=32\n\n// proposed:\nlet normal = (total_buckets * 7 / 8).max(2);  // 28 normal / 4 stash for total=32\nlet stash = (total_buckets - normal).max(1);\n```\n\nThe edge-case clamps (`max(2)` for normal, `max(1)` for stash) and the pass-through of `num_of_segments` and `bucket_size` are unchanged from v001.\n\nNo changes to `dash_segment.rs`, `eviction_policy.rs`, `dash_bucket.rs`, or `item.rs`. The `ClassicLRUFreqProtect` policy, Back-insertion promotions, second-chance recovery at Front, and lfu_counter reset on demotion are all inherited from v001 without modification.\n\n**Invariant to preserve**: `num_of_normal_buckets >= 2` (required for probing-bucket indexing, guaranteed by the `max(2)` clamp) and `num_of_stash_buckets >= 1` (guaranteed by `max(1)`).\n"
}
```

### 14:43:03 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0008/hypotheses/H02.md",
  "content": "# H02\n\nSwitching the stash-overflow second-chance promotion from LRU-end insertion (Front) to MRU-end insertion (Back) will raise hit-rate by giving second-chance items immediate hot-end position, consistent with the universal MRU-wins principle observed across all prior cycles.\n\n## Parent\n\ncycle/0007/v001\n\n## Changes\n\nIn `src/dash/segments/dash_segment.rs`, change the `put()` method's second-chance `promote_to_target` call from `InsertPosition::Front` to `InsertPosition::Back`. This is the only diff from v001:\n\n```rust\n// current (v001):\npromote_to_target(&mut self.core, ev, InsertPosition::Front);\n\n// proposed:\npromote_to_target(&mut self.core, ev, InsertPosition::Back);\n```\n\nThe surrounding logic is unchanged: the stash insert still uses `InsertPosition::Back` for new cold items; the stash-evicted item `ev` is still unconditionally recovered (no condition added); `promote_to_target` still handles any resulting target overflow via `demote_from_normal_bucket` which resets `lfu_counter` to 0 before stash re-entry.\n\nNo changes to `eviction_policy.rs`, `geometry.rs`, `dash_bucket.rs`, or `item.rs`.\n\n**Invariant to preserve**: The second-chance item still always enters the cache rather than being discarded; only its insertion position within the target bucket changes. Any cascade from target overflow continues to be handled by the existing `demote_from_normal_bucket` path unmodified.\n\n**Rationale for plausibility**: Second-chance items are items displaced from stash by hash collision with a new cold arrival — they are not cold by choice, only by bad luck. Back/MRU insertion means one more target access promotes them to probing, rather than forcing them to wait through two target accesses from the LRU end. Cross-cycle evidence shows every other promotion path uses Back insertion; the second-chance path is the sole exception in v001.\n"
}
```

### 14:43:20 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0008/hypotheses/H03.md",
  "content": "# H03\n\nGating target-to-probing promotion on lfu_counter >= 2 in the target tier will raise hit-rate by reserving the probing bucket for doubly-confirmed-hot items and reducing probing churn caused by one-hit-in-target items earning the premium tier prematurely.\n\n## Parent\n\ncycle/0007/v001\n\n## Changes\n\nIn `src/dash/segments/dash_segment.rs`, modify the target-bucket branch of `promotion_get()` to require `lfu_counter >= 2` before executing a target-to-probing promotion. The hottest item that fails the gate is handled by the existing intra-bucket path (`get_from_position`) instead, which increments its counter and moves it to MRU — so it accumulates the required accesses on subsequent hits.\n\nLocate the block:\n\n```rust\nif let Some(item_idx) = core.buckets[target_idx].get_position(key) {\n    let is_hottest = Some(item_idx) == core.buckets[target_idx].promotion_candidate_index();\n    if is_hottest {\n        debug!(\"promotion_get: key={key:?} found in target as hottest -> probing={probing_idx}\");\n        let data = core.buckets[target_idx].remove_at(item_idx);\n        promote_to_probing(core, data, insert_pos);\n        // ...\n    } else {\n        debug!(\"promotion_get: key={key:?} found in target item_idx={item_idx} (intra-promote)\");\n        return Some(core.buckets[target_idx].get_from_position(item_idx));\n    }\n}\n```\n\nReplace the `if is_hottest` condition with a compound check:\n\n```rust\nif let Some(item_idx) = core.buckets[target_idx].get_position(key) {\n    let is_hottest = Some(item_idx) == core.buckets[target_idx].promotion_candidate_index();\n    let counter = core.buckets[target_idx].get_items()[item_idx].lfu_counter;\n    if is_hottest && counter >= 2 {\n        debug!(\"promotion_get: key={key:?} found in target as hottest (counter={counter}) -> probing={probing_idx}\");\n        let data = core.buckets[target_idx].remove_at(item_idx);\n        promote_to_probing(core, data, insert_pos);\n        // ...\n    } else {\n        debug!(\"promotion_get: key={key:?} found in target item_idx={item_idx} (intra-promote)\");\n        return Some(core.buckets[target_idx].get_from_position(item_idx));\n    }\n}\n```\n\nThe `get_items()` call is a borrow of the bucket slice and is safe before the `remove_at` on the promotion branch. `get_from_position` still handles the intra-bucket case (both non-hottest items and hottest items that fail the counter gate), incrementing `lfu_counter` via `ClassicLRUFreqProtect::get_from_position` and moving the item to MRU. Items will reach `counter >= 2` after two intra-bucket hits and then qualify for probing promotion on the third target access.\n\nNo changes to `eviction_policy.rs`, `geometry.rs`, `dash_bucket.rs`, or `item.rs`. All v001 mechanisms (ClassicLRUFreqProtect eviction, Back-insert stash→target, second-chance recovery, lfu_counter reset on demotion) are preserved untouched.\n\n**Invariant to preserve**: Items removed via `remove_at` on the promotion branch still enter `promote_to_probing` with their current counter value (which may be 0, 1, or 2 — the `remove_at` call does not increment the counter). This is existing behaviour; the only change is the eligibility gate before the `remove_at`.\n"
}
```

### 14:43:31 chat

All three hypothesis files have been written to the output directory. Here's a summary of what was produced:

---

**H01 — Geometry: tighter stash (7/8 split)** · parent: `cycle/0007/v001`  
Change `geometry.rs` multiply from `3/4` → `7/8`, giving 28 normal / 4 stash (was 24/8). Second-chance recovery already cushions stash evictions, so a smaller stash is less costly now and the freed slots go to the hot tier.

**H02 — Second-chance at MRU (Back)** · parent: `cycle/0007/v001`  
Change the one `InsertPosition::Front` argument in `put()`'s second-chance call to `InsertPosition::Back`. MRU insertion wins everywhere else in the pipeline; the second-chance path is the sole exception in v001. Stash-evicted items are displaced by collision, not by being cold — giving them immediate MRU position is consistent with how every other promotion path is handled.

**H03 — Target-to-probing promotion gate (lfu_counter ≥ 2)** · parent: `cycle/0007/v001`  
Add `counter >= 2` check before the `remove_at` / `promote_to_probing` call in `promotion_get()`. Items that are hottest but under-threshold fall through to `get_from_position` (counter++, move to MRU), accumulating the required access count across subsequent hits. This is a distinct axis from the eviction threshold that failed in cycle 0007 v003 — it targets probing *admission* rather than normal-tier *eviction*.


---

- finished: 2026-06-05T14:43:31+00:00
- duration: 646.29s
- status: ok
- input_tokens: 9
- output_tokens: 23025
- cache_read_tokens: 274909
- cache_write_tokens: 43320
- cost_usd: 0.88189
