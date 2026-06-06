# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-4-5-20251001)
- workspace: /home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0002/hypotheses
- allow_edits: True
- started: 2026-06-06T05:21:15+00:00

## timeline

### 05:21:18 thinking

Let me break down this task:

1. I'm the hypotheses-writer for cycle 0002
2. I need to generate 3 hypotheses
3. Each hypothesis must cite exactly one parent variant (baseline or a past variant from cycle 0001)
4. At least one MUST cite a cross-cycle winner (v001, v002, or v003 from cycle 0001)
5. I should aim for distinct strategies

First, let me read all the context files to understand:
- What the baseline looks like
- What variants were created in cycle 0001
- What fitness vectors they achieved
- What insights were learned

Let me start reading the files systematically.

### 05:21:18 chat

I'll start by reading the context files to understand the baseline, previous cycle results, and what strategies have been tested.

### 05:21:18 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0001/cycle-insight.md"
}
```

### 05:21:18 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0001/cycle-winners.json"
}
```

### 05:21:19 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/leaderboard.md"
}
```

### 05:21:19 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0001/variants/v*/insight.md"
}
```

### 05:21:20 thinking

Let me read the variant insights directly. Let me check if the insight files exist:

### 05:21:21 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0001/variants/*/insight.md"
}
```

### 05:21:22 thinking

Interesting, the insight files don't exist. Let me check the structure:

### 05:21:23 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0001/variants/v*"
}
```

### 05:21:24 thinking

Let me look for the overlay files and fitness vectors:

### 05:21:24 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0001/variants/v*/eval/fitness-vector.json"
}
```

### 05:21:25 thinking

Let me check what's actually in the variants directory:

### 05:21:25 tool: Bash

```json
{
  "command": "find /home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0001/variants -type f | head -30"
}
```

### 05:21:27 thinking

I can't use bash. Let me try using glob to list files:

### 05:21:27 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0001/variants/**/*"
}
```

### 05:21:29 thinking

Good, now I found the files. Let me read the key information:
1. The task files for each variant to understand what was changed
2. The overlay files to see the actual changes
3. The insight files to understand the results

Let me start with the tasks and insights:

### 05:21:29 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0001/variants/v001/task.md"
}
```

### 05:21:30 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0001/variants/v001/insight.md"
}
```

### 05:21:30 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0001/variants/v002/task.md"
}
```

### 05:21:30 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0001/variants/v002/insight.md"
}
```

### 05:21:32 thinking

Great, now let me read v003's insight and the hypotheses from the first cycle to understand what was attempted:

### 05:21:33 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0001/variants/v003/task.md"
}
```

### 05:21:33 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0001/variants/v003/insight.md"
}
```

### 05:21:33 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0001/hypotheses/H01.md"
}
```

### 05:21:34 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0001/hypotheses/H02.md"
}
```

### 05:21:35 thinking

Good, let me also read H03:

### 05:21:38 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0001/hypotheses/H03.md"
}
```

### 05:21:38 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/insight.md"
}
```

### 05:21:42 thinking

Now I have a good understanding of what happened in cycle 0001:
- v001: Back-insert promotion (moved from Front to Back insertion position)
- v002: Geometry rebalancing (changed normal buckets from 8 to 16, stash from 24 to 16)
- v003: Item struct compaction (reduced lfu_counter from usize to u16)

All three were winners and showed that they operate orthogonally (no conflicts). The cycle insight notes that:
1. These three optimizations compose well (they don't conflict)
2. They all align with Zipfian workload characteristics
3. Next cycle should test whether they can be combined
4. Should explore alternative eviction policies, promotion thresholds, adaptive geometry tuning

Now I need to read the baseline to understand the current code structure, and then look at the overlay files to see exactly what was changed:

### 05:21:42 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0001/variants/v001/attempts/01/overlay/src/dash/segments/dash_segment.rs"
}
```

### 05:21:43 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0001/variants/v002/attempts/02/overlay/src/shared/geometry.rs"
}
```

### 05:21:43 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0001/variants/v003/attempts/01/overlay/src/shared/item.rs"
}
```

### 05:21:45 thinking

Excellent! Now I have a solid understanding of what happened in cycle 0001. Let me read the baseline now to understand what it looks like:

### 05:21:45 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/overlay/src/shared/item.rs"
}
```

### 05:21:46 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/overlay/src/shared/geometry.rs"
}
```

### 05:21:46 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/overlay/src/dash/segments/dash_segment.rs",
  "limit": 100
}
```

### 05:21:47 thinking

The baseline doesn't have overlay files. Let me check the structure:

### 05:21:47 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/inputs/subjects/dash/baseline/**/*"
}
```

### 05:22:22 thinking

OK, so the baseline doesn't have overlay files. That's fine - it's the baseline. Let me now understand what I need to propose for cycle 0002.

Based on what I've learned:

**Cycle 0001 Winners:**
- v001: Back-insert promotion (hit-rate=0.6787, per-op-time=104)
- v002: Geometry rebalancing (hit-rate=0.6965, per-op-time=120) - BEST HIT-RATE
- v003: Item struct compaction (hit-rate=0.6634, per-op-time=105)

**Key insights from cycle 0001:**
- These three optimizations are orthogonal (operate at different layers)
- All exploit Zipfian workload characteristics
- The cycle-insight notes these could potentially compose
- Per-op-time and hit-rate move together, not trade-offs
- All three winners focus on reducing unnecessary work on the hot path

**For Cycle 0002, I need to generate 3 hypotheses:**

1. At least one MUST cite a cross-cycle winner (v001, v002, or v003)
2. The remaining hypotheses can cite baseline or prior losers
3. Aim for distinct strategies

**Strategy ideas:**

Idea 1: **Compose v001 + v002** - Combine back-insert promotion with geometry rebalancing. This addresses the cycle insight's open question about whether they compose well. Parent: cycle/0001/v002 (the best hit-rate winner). Expected outcome: even better hit-rate while maintaining reasonable per-op-time.

Idea 2: **Explore adaptive/threshold-based promotion** - Instead of always back-inserting on promotion, use a threshold on the promotion chain. Back-insert for early-tier promotions, front-insert for final-tier promotion to prevent stash thrashing. 

This refines the back-insert approach from v001 by introducing conditional logic based on promotion depth. Parent: cycle/0001/v001. The hypothesis is that we can preserve the per-op-time gains while avoiding pathological cases where aggressive back-insertion causes unnecessary churn.

Idea 3: **Eviction policy alternative** - The cycle insight notes "alternative eviction policies" as unexplored. Try a different eviction policy (e.g., TLFU - Time-aware LFU, or frequency-biased LRU) in addition to the classic LRU. This is orthogonal to geometry and promotion. Parent: baseline (fresh approach to a new dimension).

Actually, looking more carefully:
- v002 had the best hit-rate (0.6965) but slightly slower per-op-time (120 vs 104)
- v001 had the best per-op-time (104) with good hit-rate (0.6787)
- v003 had the worst hit-rate but was still a winner

For cycle 0002, the natural progression would be:
1. Compose v001 + v002 to see if they stack
2. Add v003 (compaction) on top, or
3. Explore another dimension

Let me think about what would be most valuable:

**Option A: Composition**
- H01: v002 (geom) + v001 (back-insert) → better hit-rate + latency
- H02: v002 (geom) + v003 (compaction) → hit-rate + latency via cache density
- H03: Some new idea (eviction policy, adaptive thresholds)

**Option B: Refinement**
- H01: v002 + more aggressive geometry (wider normal, smaller stash)
- H02: v001 + conditional back-insert (threshold-based)
- H03: Alternative

The composition approach seems more promising—stacking v002's geometry improvements with v001's latency gains could yield the best of both. I should test whether these optimizations compound or interfere with each other. the geometry shift further by testing allocations like 20/12 or 24/8 to see if we can push beyond the 50/50 split and reduce stash misses even more. A tier-aware eviction strategy is worth exploring too—using LFU for the probing tier and LRU for target/stash tiers since they have different retention patterns. This would be a genuinely new direction compared to what we tested before. eviction policies for different tiers—probing tier resists eviction since it holds hot items, stash tier is aggressive for overflow, and target tier handles transitions. This targets hit-rate primarily with per-op-time as secondary.

I should verify which files are actually editable. The cycle variants modified src/dash/segments/dash_segment.rs for promotion logic, src/shared/geometry.rs for bucket allocation, and src/shared/item.rs for struct layout—all within scope. For H02, pushing the geometry split further than 16/16 needs careful consideration; going to 24/8 seems too extreme. The cycle notes suggest these improvements compound without conflicts, so combining approaches might be worth exploring.

Since at least one hypothesis must cite a cross-cycle winner as its parent, I can structure this around the three cycle-0001 variants. H01 layers v001's back-insert promotion and v003's item compaction onto v002's geometry split while keeping hit-rate as the primary target. H02 takes v002's geometry as a base and tests wider allocations like 20/12 or 22/10 to find the optimal split beyond 16/16. H03 applies v001's back-insert selectively—using it only for stash-to-target transitions while keeping front-insert for target-to-probing to avoid stash overflow and manage per-op-time better. : Widening the indexed normal tier reduces stash misses and boosts hit-rate.

**H03: Add LFU-aware retention to probing tier**
- Parent: baseline
- Changes: Incorporate LFU weights into eviction decisions within the probing bucket alongside LRU ordering
- Targets: Hit-rate + per-op-time
- Claim: Frequency-weighted retention in the probing tier captures hot-key patterns and prevents eviction of frequently accessed items

I'm identifying which dimensions remain unexplored. Promotion strategies were tested in v001, geometry adjustments in v002, and struct layout in v003. This suggests exploring retention policies in the probing tier as a fresh angle.

Several promising directions emerge: alternative eviction strategies like LFU or TLFU, adaptive thresholds for when items get promoted, stash-specific optimizations with faster eviction, and tier-specific bucket sizing. 

**H01: Compose v002 (geometry) + v001 (back-insert)**
- Parent: cycle/0001/v002
- Targets: hit-rate (primary), per-op-time (secondary)

**H02: Further geometry -

widen normal buckets to 20 (20/12 split)**
- Parent: cycle/0001/v002
- Targets: hit-rate (primary)
- Rationale: v002 showed 16/16 is better than 8/24; test if even wider helps

**H03: LFU-weighted eviction in probing tier**
- Parent: baseline
- Targets: hit-rate (primary), per-op-time (secondary)
- Rationale: Probing tier holds the hottest items; LFU weights preserve frequently-accessed items better than pure LRU

Actually, H03 is getting complicated. Let me try a different angle.

What about **H03: Stash-first eviction strategy**?
- When demoting from normal tiers, prioritize placing in stash vs full eviction
- Or: when stash is full, evict less-frequently accessed items instead of the oldest ones

.

**H03: Frequency-gated promotion - only promote after N accesses**
- Parent: baseline
- Changes: Modify promotion logic to only promote items from target to probing after they've been hit N times (e.g., N=2)
- Targets: hit-rate + per-op-time
- Rationale: Reduces noisy promotion of occasionally-accessed items; keeps stash occupied with genuinely hot items

This addresses a key inefficiency in the baseline's promote-on-hit behavior—single-hit items shouldn't immediately jump to the probing tier. By gating promotion on access frequency, I can filter out transient accesses and preserve stash capacity for items that actually matter.

My final three hypotheses span different optimization angles: H01 combines proven strategies from the previous cycle, H02 refines the geometric allocation further, and H03 introduces an entirely new control lever around promotion thresholds. They're independent enough to reveal whether improvements come from composition, tuning, or architectural changes.

### 05:22:22 chat

Now let me look at the cross-cycle context to better understand what's been explored:

### 05:22:22 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles-ledger.jsonl"
}
```

### 05:22:26 thinking

Perfect! Now I have a clear picture. The cycle insight explicitly says:
1. The three winners are orthogonal and could potentially compose
2. Open questions include: do they compose? Is 16/16 optimal? What about alternative eviction policies and promotion thresholds?
3. Next cycle should test composition and explore new dimensions

So my H01, H02, H03 strategy is well-grounded:

**H01: Compose v001 + v002** - addresses the composition question
**H02: Further geometry tuning** - addresses whether 16/16 is optimal
**H03: New dimension - frequency gating or tier-aware policy** - addresses alternative eviction/promotion strategies

Now let me write the hypothesis files. I need to output them to:
`/home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0002/hypotheses/`

Three files: H01.md, H02.md, H03.md

### 05:22:28 chat

Perfect! I now have all the context I need. Let me synthesize the three hypotheses for cycle 0002:

Based on cycle 0001's findings:
- Three orthogonal winners (v001: back-insert, v002: geometry rebalance, v003: compaction)
- Open question: Do these compose?
- Open question: Is 16/16 bucket split optimal?
- Suggested exploration: alternative eviction policies, promotion thresholds, adaptive geometry

I'll create three distinct hypotheses:

1. **H01**: Compose v001 (back-insert) + v002 (geometry) — tests the composition question
2. **H02**: Further geometry exploration (push normal buckets even wider) — tests the sweet spot
3. **H03**: Frequency-gated promotion — explores a new promotion threshold dimension

### 05:22:33 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0002/hypotheses/H01.md",
  "content": "# H01\n\nCombining back-insert promotion with geometry rebalancing amplifies both hit-rate and latency gains.\n\n## Parent\n\ncycle/0001/v002\n\n## Changes\n\nApply both v001's back-insert promotion strategy and v002's geometry rebalancing together:\n\n1. In `src/dash/segments/dash_segment.rs`, line 68: change the `get()` method to pass `InsertPosition::Back` to `promotion_get()` (already done in v001 at lines 157, 170).\n\n2. Keep `src/shared/geometry.rs` at the v002 configuration: 16 normal + 16 stash buckets (50/50 split) instead of baseline's 8/24 split.\n\nThe combination layers back-insert's benefit (keeping promoted items at MRU end longer) with geometry's benefit (reducing stash misses via wider indexed tier). Both changes operate independently: promotion timing is orthogonal to bucket allocation. Expected outcome: higher hit-rate from geometry + lower latency from back-insert, without diminishing returns.\n\n## Rationale\n\nCycle 0001's insight notes the three winners are orthogonal with no apparent conflicts. This hypothesis tests whether v001 and v002 compose beneficially. v002 achieved the highest hit-rate (0.6965) but with slightly higher per-op-time (120 ns). v001 achieved the best per-op-time (104 ns). Combining them should yield the hit-rate strength of v002 + the latency strength of v001 without trade-offs, since back-insert reduces demotion cascades (v001's mechanism) while wider normal buckets reduce stash traversals (v002's mechanism).\n"
}
```

### 05:22:39 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0002/hypotheses/H02.md",
  "content": "# H02\n\nPushing normal bucket allocation further to 20/12 ratio raises hit-rate by maximizing hash-indexed capacity.\n\n## Parent\n\ncycle/0001/v002\n\n## Changes\n\nIn `src/shared/geometry.rs`, modify the `BaselineGeometry::for_settings()` function to allocate buckets more aggressively toward the normal tier:\n\n- Change `num_of_normal_buckets = (total_buckets + 1) / 2` to `num_of_normal_buckets = (total_buckets * 5) / 8` (rounding down)\n- Change `num_of_stash_buckets = total_buckets / 2` to `num_of_stash_buckets = total_buckets - num_of_normal_buckets`\n\nFor baseline's total of 32 buckets, this yields 20 normal + 12 stash (62.5% / 37.5%), up from v002's 50/50 split.\n\nKeep `bucket_size` and `segment_size` unchanged. The total capacity remains constant; capacity is reallocated from the unindexed stash to the hash-indexed normal tier.\n\n## Rationale\n\nv002 proved that wider normal buckets improve hit-rate (from 0.663 to 0.6965) by reducing cold misses that fall through to the stash. The cycle insight asks: is 16/16 the optimal sweet spot, or is there further geometry tuning to explore? This hypothesis tests whether pushing even further—to a 62.5% normal allocation—continues to yield improvements. The tradeoff is a smaller stash (more demotions may fully evict instead of landing in stash), but Zipfian workloads are dominated by hot keys that stay in higher tiers, so stash size may matter less than catching cold misses in the primary tier.\n"
}
```

### 05:22:43 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0002/hypotheses/H03.md",
  "content": "# H03\n\nFrequency-gated promotion reduces noisy tier transitions and raises hit-rate by conserving stash capacity for genuinely hot items.\n\n## Parent\n\nbaseline\n\n## Changes\n\nIn `src/dash/segments/dash_segment.rs`, modify the `promotion_get()` function (around lines 141–189) to gate target→probing promotion on a frequency threshold:\n\n1. Add a promotion counter threshold constant: `const PROMOTION_THRESHOLD: u16 = 2;` at the module level.\n\n2. In the section checking \"is_hottest\" (line 166), add a condition before promoting:\n   ```\n   let is_hottest = Some(item_idx) == core.buckets[target_idx].promotion_candidate_index();\n   if is_hottest && core.buckets[target_idx].get_items()[item_idx].lfu_counter >= PROMOTION_THRESHOLD {\n       // proceed with promotion to probing\n   } else if is_hottest {\n       // increment lfu_counter and stay in target (intra-promote only)\n   }\n   ```\n\nStash→target promotion remains ungated (all stash hits promote immediately to target). The threshold gates only target→probing, the most expensive tier transition.\n\nKeep all demotion logic and cascade behavior unchanged.\n\n## Rationale\n\nThe baseline promotes items to probing on first hit within their target bucket (if hottest), which can elevate occasionally-accessed items to the hottest tier and waste probing capacity. Frequency-gating (N=2 threshold) ensures only items with sustained access patterns claim space in the expensive probing tier. This reduces churn in the probing bucket, keeping it reserved for genuinely hot keys, and reduces the frequency of cascading demotions when probing overflows. The tradeoff is slightly delayed promotion of truly hot items (they need 2 hits before promotion), but on Zipfian traces with small hot sets, this cost is minimal compared to the benefit of reducing false-positive promotions.\n"
}
```

### 05:22:44 thinking

Great! I've successfully created all three hypothesis files. Let me verify they were created correctly by reading them back to make sure the format is correct:

### 05:22:45 chat

Perfect! I've created three distinct hypotheses for cycle 0002. Let me verify they meet the quality bar:

### 05:22:45 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0002/hypotheses/H01.md"
}
```

### 05:22:45 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0002/hypotheses/H02.md"
}
```

### 05:22:45 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0002/hypotheses/H03.md"
}
```

### 05:22:52 thinking

Excellent! All three hypotheses are written and meet the quality bar:

✓ **Distinct strategies**: 
- H01: Composition of v001 + v002 (testing interaction)
- H02: Further geometry tuning (pushing boundaries)
- H03: New dimension - frequency-gated promotion (new exploration axis)

✓ **Testable claims**: Each one makes a concrete, measurable claim about fitness improvements

✓ **Grounded**:
- H01 and H02 cite cycle/0001/v002 (cross-cycle winner incumbent) ✓
- H03 cites baseline
- All cite valid parents, no made-up variants

✓ **Specific**:
- H01: Names exactly which lines/functions to modify
- H02: Gives concrete arithmetic for new allocation (20/12 split, (total_buckets * 5) / 8)
- H03: Provides pseudocode for the gating logic, specifies PROMOTION_THRESHOLD=2

✓ **Valid claims**:
- H01: "Combining back-insert promotion with geometry rebalancing amplifies both hit-rate and latency gains." ✓
- H02: "Pushing normal bucket allocation further to 20/12 ratio raises hit-rate by maximizing hash-indexed capacity." ✓
- H03: "Frequency-gated promotion reduces noisy tier transitions and raises hit-rate by conserving stash capacity for genuinely hot items." ✓

All claims are:
- Concrete (no vague intent like "try something different")
- Start with a concrete change (not a list marker or numbering)
- End with the fitness dimension targeted
- No leading markers or numbering

At least one hypothesis cites a cross-cycle winner (both H01 and H02 cite cycle/0001/v002) ✓

The hypotheses are grounded in the cycle 0001 findings and the open questions noted in the cycle insight. 

These are ready to move forward.

### 05:22:55 chat

Excellent! All three hypotheses have been created successfully. Let me summarize what was generated:

## Cycle 0002 Hypotheses Summary

**H01: Compose v001 + v002** ✓
- **Parent**: `cycle/0001/v002` (cross-cycle winner—highest hit-rate at 0.6965)
- **Strategy**: Layer back-insert promotion with geometry rebalancing
- **Targets**: Hit-rate + per-op-time
- **Rationale**: Tests the cycle-0001 insight that these optimizations are orthogonal and could compose beneficially

**H02: Further geometry exploration** ✓
- **Parent**: `cycle/0001/v002` (geometry incumbent)
- **Strategy**: Push normal/stash split from 50/50 (16/16) to 62.5/37.5 (20/12)
- **Targets**: Hit-rate
- **Rationale**: v002 showed wider normal buckets help; tests whether 16/16 is optimal or if further allocation helps

**H03: Frequency-gated promotion** ✓
- **Parent**: `baseline`
- **Strategy**: Gate target→probing promotions to only occur after N=2 hits (new dimension)
- **Targets**: Hit-rate + per-op-time
- **Rationale**: Reduces wasteful promotion of occasionally-accessed items; preserves stash/probing for genuinely hot keys

All three hypotheses:
- ✓ Cite valid parents (two cite v002 incumbent winner; one cites baseline)
- ✓ Have distinct strategies across different optimization dimensions
- ✓ Include concrete, testable claims starting with specific changes
- ✓ Provide implementation details specific enough for a variant-implementer to act on
- ✓ Are grounded in cycle-0001's empirical findings and open questions

The cohort is ready for cycle 0002 evaluation.


---

- finished: 2026-06-06T05:22:56+00:00
- duration: 101.05s
- status: ok
- input_tokens: 98
- output_tokens: 9671
- cache_read_tokens: 414945
- cache_write_tokens: 33926
- cost_usd: 0.133806
