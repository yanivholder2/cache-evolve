# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-4-5-20251001)
- workspace: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0008/hypotheses
- allow_edits: True
- started: 2026-06-05T19:25:09+00:00

## timeline

### 19:25:13 thinking

Let me break down this task:

1. I need to generate 3 distinct hypotheses for cycle 0008
2. Each hypothesis must cite exactly one parent variant (either `baseline` or a past variant)
3. At least one hypothesis MUST cite a cross-cycle winner as parent
4. The cross-cycle winners are:
   - cycle/0004/v001: hit-rate=0.684
   - cycle/0005/v002: hit-rate=0.684
   - cycle/0007/v001: hit-rate=0.684

5. I need to read context files to understand:
   - Previous cycle (0007) variants and their fitness
   - Cross-cycle history
   - Baseline
   
6. Output 3 files: H01.md, H02.md, H03.md with the required format:
   - Claim (one sentence, concrete change + fitness dimension)
   - ## Parent (baseline or cycle/NNNN/vNNN)
   - ## Changes (specific prose describing the diff)

Let me start by reading the context files to understand what's been tried and what the winners look like.

### 19:25:13 chat

I'll analyze the context to generate 3 distinct hypotheses for cycle 0008. Let me start by reading the relevant files.

### 19:25:14 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/leaderboard.md"
}
```

### 19:25:14 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles-ledger.jsonl"
}
```

### 19:25:14 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cross-cycle-insights.md"
}
```

### 19:25:14 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007/cycle-insight.md"
}
```

### 19:25:16 thinking

Good context. Now let me read cycle 0007's variants to understand what they attempted and their results.

### 19:25:16 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007/cycle-winners.json"
}
```

### 19:25:17 tool: Glob

```json
{
  "path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007/variants",
  "pattern": "v*/insight.md"
}
```

### 19:25:18 thinking

The insights for cycle 0007 variants don't seem to have their own insight.md files. Let me read the variant overlays to understand what they tested.

### 19:25:18 tool: Glob

```json
{
  "path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007/variants",
  "pattern": "v*/attempts/*/overlay"
}
```

### 19:25:20 thinking

Let me look at what's in the cycle 0007 variants directory.

### 19:25:20 tool: Bash

```json
{
  "command": "find /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007/variants -type f -name \"*.md\" | head -20"
}
```

### 19:25:22 thinking

I can't run bash commands without permission. Let me try using Read on the cycle 0007 variants directory directly by reading the parent directory first.

### 19:25:22 tool: Glob

```json
{
  "path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007/variants",
  "pattern": "**/*.md"
}
```

### 19:25:23 thinking

Good, now let me read the task.md files for each variant to understand what they attempted.

### 19:25:24 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007/variants/v001/task.md"
}
```

### 19:25:24 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007/variants/v002/task.md"
}
```

### 19:25:24 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007/variants/v003/task.md"
}
```

### 19:25:25 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline"
}
```

### 19:25:26 thinking

Let me read the hypothesis files from cycle 0007 to understand what was proposed and tested.

### 19:25:26 tool: Glob

```json
{
  "path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007/hypotheses",
  "pattern": "*.md"
}
```

### 19:25:28 thinking

Good, let me read those hypotheses to understand what was tested.

### 19:25:28 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007/hypotheses/H01.md"
}
```

### 19:25:28 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007/hypotheses/H02.md"
}
```

### 19:25:28 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007/hypotheses/H03.md"
}
```

### 19:25:30 thinking

Now let me look at the cycle 0004 and 0005 winning variants to understand what's been proven to work. I also need to look at a few more earlier variants to find other useful starting points.

### 19:25:31 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0004/hypotheses/H01.md"
}
```

### 19:25:31 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0005/hypotheses/H02.md"
}
```

### 19:25:31 tool: Glob

```json
{
  "path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles",
  "pattern": "*/hypotheses/H*.md"
}
```

### 19:25:33 thinking

Interesting - the glob returned no results. Let me try a different pattern.

### 19:25:34 tool: Glob

```json
{
  "path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938",
  "pattern": "cycles/0006/hypotheses/H*.md"
}
```

### 19:25:34 tool: Glob

```json
{
  "path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938",
  "pattern": "cycles/0003/hypotheses/H*.md"
}
```

### 19:25:36 thinking

It looks like the hypotheses don't exist for all cycles. That's okay. Let me look at the one variant that was a cross-cycle winner: cycle/0004/v001. Let me read the task file for cycle 0004 v001 to understand what it did.

### 19:25:36 tool: Glob

```json
{
  "path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0004/variants/v001",
  "pattern": "**/*.md"
}
```

### 19:25:38 thinking

Let me read the insights of the cross-cycle winners to understand what successful strategies look like.

### 19:25:38 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0004/variants/v001/insight.md"
}
```

### 19:25:39 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0005/variants/v002/insight.md"
}
```

### 19:25:39 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007/variants/v001/insight.md"
}
```

### 19:25:53 thinking

Perfect! Now I have a clear picture:

**Cross-cycle winners:**
- cycle/0004/v001: hit-rate=0.684
- cycle/0005/v002: hit-rate=0.684
- cycle/0007/v001: hit-rate=0.684

**Key learnings from the cross-cycle insights:**

1. **Demotion recovery with MRU positioning** - This is the only insertion-position strategy that held, introduced in cycle 0003 and unchanged since. Demoted items benefit from back insertion.

2. **Stash-hit recognition** - Early cycles showed stash-resident items are high-confidence signals

3. **The 0.684 ceiling** - Seven cycles of testing have converged on this, defended against:
   - Insertion position (Front/Back/MRU) - tested
   - Occupancy-based gating - tested
   - Tier capacity ratios - tested (marginal +0.0014 in cycle 0006)
   - Tier-skip architectures - incompatible with demotion recovery
   - Eviction priority - tested in cycle 0007 v001 (no gain)
   - Promotion candidacy eligibility - tested in cycle 0005 v002 (no gain)
   - Cascade structure - tested in cycle 0007 v002 (degraded)
   - Stash capacity expansion - tested in cycle 0007 v003 (degraded)

**What hasn't been fully explored:**
- The cross-cycle insights mention "alternative tier counts, different promotion qualification criteria beyond simple LRU/MRU rules, or whether the workload's natural access distribution inherently suits a 0.684 ceiling"
- Non-LRU eviction orderings beyond the current Back/Front insertion
- Bucket saturation triggers
- Tier interaction semantics beyond insertion position

**For cycle 0008, I need to generate 3 hypotheses:**


At least one must build from a cross-cycle winner, and they should explore distinct structural changes rather than parametric tweaks. The real constraint seems to be how items flow through promotion pathways and interact across tiers. Looking at what's been tested, the bottleneck isn't insertion mechanics or capacity ratios—it's something about tier interaction semantics themselves, like how promotion qualification works or whether the tier count itself limits what's achievable.

Two promising directions emerge: First, asymmetric promotion thresholds—requiring items to demonstrate higher frequency before advancing to probing, which would filter weaker candidates earlier and test whether the current symmetric approach wastes tier slots. Second, a fast-path mechanism for stash hits that aggressively promotes items identified as immediately hot, building on earlier cycle insights about working-set behavior. Looking at the progression across cycles, I can see stash-hit acceleration appears in cycle 0004's variants alongside MRU positioning and tier-skipping strategies. The winning variant from cycle 0005 traces back to cycle 0004's v001, which combined MRU demotion with additional optimizations. To understand which specific strategies are actually persisting through the generations, I need to examine the variant overlays directly to see what's being carried forward.

### 19:25:53 tool: Glob

```json
{
  "path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0005/variants/v002/attempts",
  "pattern": "**/*.rs"
}
```

### 19:25:55 thinking

Good, let me check what the winning variant v002 from cycle 0005 contains.

### 19:25:55 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0005/variants/v002/attempts/01/overlay/src/dash/segments/dash_segment.rs",
  "limit": 100
}
```

### 19:27:44 thinking

This is very helpful! I can see that cycle/0005/v002 is using:
1. `promotion_age` tracking for eligibility gating (from H02)
2. The code continues with demotion and other logic

This tells me that the winners (which all have 0.684 hit rate) are carrying forward the eligibility-gating attempt from cycle 0005. Let me understand the overall state better by reading more about what's been tried.

Looking back at the insights:
- Cycles 0001-0003: Focused on insertion position (Front/Back/MRU)
- Cycle 0004: Tested combining strategies, found they interfere
- Cycle 0005: Tested insertion position asymmetry, promotion-candidacy gating, occupancy-filtering - all failed to improve 0.684
- Cycle 0006: Tested tier capacity rebalancing - marginal +0.0014 gain
- Cycle 0007: Tested eviction priority, cascade structure, stash expansion - all failed

So the current winners at 0.684 are:
- cycle/0004/v001: Demotion back-insertion (from cycle 0003 v003)
- cycle/0005/v002: Demotion back-insertion + promotion_age eligibility gating
- cycle/0007/v001: Same as 0005/v002 (target-first eviction didn't help or hurt)

The key insight from cross-cycle-insights.md is:
- "The bottleneck appears to be how tiers interact (promotion flow, eviction cascade timing, occupancy balance) rather than any single parameter."
- "Is the MRU asymmetry (beneficial for demotion, harmful for fresh promotion) resolvable or fundamental?"

For cycle 0008, based on the guidance in cycle 0007's continuation:
"abandon parametric tuning entirely and instead test whether the bottleneck is structural rather than parametric: consider testing alternative tier counts, different promotion qualification criteria beyond simple LRU/MRU rules, or whether the workload's natural access distribution inherently suits a 0.684 ceiling."


I need to shift away from parameter tweaking and explore structural alternatives instead. Three promising directions emerge: varying tier architecture itself (count and capacity distribution), moving beyond recency-based promotion rules toward frequency or access-pattern signals, and rethinking eviction mechanics and tier semantics. Let me start with a frequency-qualified promotion approach—building on cycle/0005/v002 but gating promotion based on access frequency rather than just age. kets from pure LRU to hybrid: prefer to evict items that have lower recency/frequency ratio, keeping high-frequency items even if they're older. This tests whether the eviction policy itself is the bottleneck rather than promotion/demotion mechanics. I'd modify eviction selection to track access count + recency score together, evicting items with lowest combined score.

Actually, I need to reconsider the constraints here. The instructions emphasize distinct strategies—near-duplicates collapse the cohort's winners—and require that at least one hypothesis must cite a cross-cycle winner as its parent. Let me ground these in what's actually worked before and make sure each one explores a genuinely different angle rather than variations on the same mechanism.

For H01, I'll build on a prior winner and focus on access-count thresholds: require items to be accessed at least twice before they qualify for probing promotion, filtering out weak candidates earlier and improving hit rate. For H02, I'll take a different direction by introducing an intermediate warm tier between stash and target, creating slower filtering that better characterizes hot items. For H03, I'll explore bucket-saturation-triggered cascades where high occupancy in probing triggers preventive demotion, monitoring bucket saturation as a signal to avoid overflow.

However, frequency-based approaches have already been tested—cycle 0001's LFU policy failed on this recency-heavy workload. The workload itself is the constraint: it's recency-focused, and the 0.684 ceiling appears defended by asymmetric tier interactions that seven cycles of parametric tuning couldn't break through. The real question is whether the current architecture can exceed this ceiling at all, or if a fundamental redesign is needed. Looking at the cross-cycle patterns, demotion recovery with back-insertion has been the consistent working mechanism since cycle 0003, persisting through all subsequent cycles without degradation. Cycle 0006's v002 achieved a marginal gain to 0.6854 by adjusting probing capacity and target thresholds rather than modifying the core demotion strategy. This suggests the promotion/demotion asymmetry itself may not be the limiting factor—instead, the capacity allocation between tiers appears to have room for refinement. ...but it's not showing up as a winner on the leaderboard. Let me check the fitness vector in cycle-winners.json. I don't have that file for cycle 0006, so I'll work with what's available. The leaderboard clearly shows the baseline at 0.663, with 0001/v001 and 0002/v001 reaching 0.678, then 0003/v003, 0004/v001, 0005/v002, and 0007/v001 all hitting 0.684. The cycle 0006 variants didn't make the winner list. So the three consistent top performers across cycles are 0004/v001, 0005/v002, and 0007/v001. Now I need to develop three distinct hypotheses for what to test in cycle 0008.

For the first hypothesis, I could propose building from one of the cross-cycle winners—say 0005/v002 or 0007/v001 since they're equivalent at 0.684. The idea would be to test whether removing the promotion_age eligibility constraint and instead relying on reduced target-tier capacity to enforce stricter filtering improves hit rate. The rationale is that promotion_age added complexity without gains beyond the demotion baseline. But that's still just parametric tuning within the same space. The cross-cycle insights suggest capacity is a weak lever anyway—cycle 0006's bucket rebalancing from 14:14 to 18:10 only yielded +0.0014, implying the workload's access distribution naturally aligns with the original configuration.

The real question is whether the MRU asymmetry—beneficial for demotion but harmful for fresh promotion—can be resolved or if it's fundamental. Seven cycles have tried and all stalled. Maybe the path forward is to abandon promotion-path optimization entirely and deepen demotion recovery instead, or to accept that the current architecture has hit a ceiling. Notably, cycle 0003 v003 reached 0.684 using only demotion back-insertion. Every attempt since has either failed or plateaued. So one hypothesis worth testing: return to that demotion-only approach but enhance stash-hit recognition to see if better detection of recoverable misses can push past 0.684. ents are cycle/0004/v001, cycle/0005/v002, and cycle/0007/v001. All at 0.684.

I'm seeing a pattern here - the hit rate has plateaued at 0.684 since cycle 0003/v003 introduced demotion back-insertion. Every subsequent cycle either matched that performance or degraded it: cycle 0004 combined strategies without improvement, cycle 0005 added promotion age gating with no gain, cycle 0006 made things worse with capacity changes, and cycle 0007 tried target-first eviction but stayed flat. This suggests the system has hit a ceiling where additional mechanisms aren't moving the needle.

The core insight is that demotion back-insertion appears to be the load-bearing mechanism, while everything layered on top has been neutral or harmful. The real constraint seems structural rather than parametric. For cycle 0008, I should test whether the tier mechanics themselves are the bottleneck - specifically, whether changing how promoted items enter the target tier (using round-robin positioning instead of front-insertion) could break the clustering pattern and improve filtering distribution across the tier.

But that's still position-based tuning. A more fundamental test would be examining the stash as the actual filtering constraint - whether promoting items that show repeated access within the stash itself (signaling sustained interest) could improve the mechanism without restructuring tiers. The real structural question is whether the number of tiers itself is optimal, or if different tier counts would unlock better performance.

I should focus on modifications I can actually implement: testing a 2-tier system by collapsing the target tier into an enlarged stash, keeping only the probing tier as hot storage. This would reduce filtering overhead and test whether the intermediate tier adds value or just introduces maintenance costs.

I could also explore the opposite direction—adding a small ultra-hot tier above probing for frequently accessed items, creating a 4-tier structure that better separates the hottest items from the rest. Additionally, I want to test removing demotion entirely and implementing a one-way promotion policy where items stay in probing once promoted, eliminating cascading costs but sacrificing recovery mechanisms.

These structural changes need concrete implementation details. Looking at the requirements, I need to specify exactly which functions to modify so someone can implement this in Rust without guessing. Let me refocus on testable, specific hypotheses tied to actual code changes.

The first hypothesis should test whether removing the promotion_age eligibility delay from cycle 0005 and simplifying front-insertion for newly promoted items improves hit rates. In `src/dash/dash_segment.rs`, I'd remove the promotion_age tracking and eligibility checks, reverting `promote_to_target()` to always use `InsertPosition::Front`, while keeping the demotion-back-insertion logic unchanged. The rationale is that cycle 0005's promotion_age mechanism added complexity without measurable improvement, so testing whether a simpler approach performs better makes sense.

But that's actually moving backward from the current version. I should instead focus on what the cross-cycle analysis revealed: the real bottleneck is in tier interactions and promotion flow. A better hypothesis would test whether modifying target-tier resident management to prioritize items recovering from demotion over freshly promoted items from stash improves performance, since demotion recovery might deserve structural priority in tier occupancy decisions.

For H01, the claim is that prioritizing retention of demoted items in the target tier by preferentially evicting recently-promoted items first will improve hit rate. The implementation would modify `src/dash/dash_segment.rs` so that when the target tier is full and must accept a new item, it evicts items that haven't been demoted yet (freshly promoted ones) while keeping demotion-recovery items, even if that shrinks the fresh promotion space. This tests whether giving demotion recovery structural priority in occupancy management yields better results, since demotion recovery has consistently shown benefits across multiple cycles.

For H02, the approach splits the target tier into two distinct sub-tiers: one for demoted items (the "old" subtier with better promotion candidacy) and one for freshly promoted items (the "new" subtier). This structural separation would allow asymmetric promotion flow that favors recovery paths. The change involves tracking two separate pools within the target tier, with demoted items receiving higher promotion eligibility and earlier advancement to probing.

For H03, the strategy introduces frequency-aware demotion prioritization where items are demoted based on their access count, allowing frequently-accessed items to remain in higher tiers longer before cascading down. This aims to improve retention of working-set items by being more selective about which items get demoted in the first place.

However, H03 overlaps significantly with frequency-based eviction policies, which haven't proven effective on this recency-heavy workload. Given that the workload prioritizes recent access patterns and frequency-based approaches have consistently underperformed across seven cycles of parametric tuning, this direction may not yield meaningful improvements. The demotion-recovery insertion strategy remains the only approach that's shown real gains, suggesting the focus should stay on structural changes that enhance recovery paths rather than frequency-based filtering. So that approach won't help. Let me examine what's actually been successful versus what's failed.

The wins are consistent: demotion back-insertion achieved 0.684 across multiple cycles (0003 v003, 0004 v001, 0005 v002), with the promotion age gating maintaining that performance without degradation. Everything else that layered additional mechanisms on top either hurt performance or provided no benefit. This suggests the core insight—reinserting demoted items back into the normal bucket—is the key lever, and adding complexity around it tends to backfire. For cycle 0008, I should explore something orthogonal rather than trying to refine the existing foundation further.

H01 proposes keeping demotion back-insertion but changing where new items land: if stash occupancy exceeds 80%, route fresh inserts directly to target instead of stash, skipping the filtering overhead for hot items. H02 takes a different angle entirely, using hash density to partition items—rare-hash items go straight to probing while common-hash items flow through stash-target filtering, isolating hash-collision noise from the main path.

For H03, the idea is to combine occupancy-triggered promotion (moving stash items to target when stash > 70%) with back-insertion across all demoted items, not just stash→target moves, to see if dynamic filter depth tuning boosts hit rate. This resembles cycle 0002 v002, which scored 0.668, so it's worth revisiting whether the occupancy threshold or promotion scope makes a difference.

Looking at what hasn't been explored yet: dynamic tier sizing based on occupancy, hash-based routing rules, access-pattern learning across workload phases, inter-tier coordination effects, and multi-signal quality filtering before promotion. The 0.684 ceiling might be defended by something specific in how eviction cascades interact with promotion flow and tier occupancy—previous cycles tested eviction priority and structure, but perhaps the timing of when cascades trigger relative to occupancy thresholds is the missing piece.

Testing batch demotion on a timer or queue instead of immediate cascading, or promoting multiple hot items simultaneously when a tier fills rather than one-by-one, could reduce cascade overhead. Three concrete hypotheses emerge: batch promotion when target tier hits 90% occupancy to move all hottest items at once, reducing cascade overhead; a timer-based cascade queue to decouple promotion from immediate demotion; and multi-item promotion to lower cascade frequency. quality of probing residents. When the hot tier reaches high occupancy, raising promotion thresholds ensures only the most frequently accessed items move up, preventing dilution of the tier with marginal candidates.

The three hypotheses target different mechanisms: batch promotion to reduce cascade frequency, demotion recovery prioritization to improve resource allocation, and occupancy-aware thresholds to maintain tier quality. H03 overlaps with prior work on promotion qualification criteria, so I should explore a different angle. The bucket structure itself might be the constraint—testing alternative collision handling or bucket organization could reveal whether the current hash-based design limits throughput or hit rate.

Two new directions emerge: preferentially demoting high-collision items to reduce contention in hot tiers, and clustering collision-prone items together to improve cache locality. The first approach tracks collision counts per item and prioritizes demotion of those with frequent collisions, preserving unique-hash items longer. The second groups colliding items by insertion order, placing them at the back of the queue to share fate and potentially improve spatial locality.

A third hypothesis about stash-bucket geometry—reducing stash buckets from 4 to 2 while expanding normal buckets from 28 to 30—would shift capacity toward promotion tiers. However, this mirrors a previous degradation, so it's worth reconsidering. Looking at what's already been explored: insertion positioning across 7 cycles, occupancy gating in 4+, tier ratios in 2, promotion eligibility in 1, eviction priority in 1, and cascade structure. The design space is narrowing, so future hypotheses need to target genuinely distinct mechanisms rather than rehashing tested patterns.

Several promising directions remain untouched: batch operations within cascades, hash-collision-aware eviction policies, alternative tier structures, access-frequency qualifications (which failed for full eviction but weren't tested for partial promotion), and dynamic tier sizing. The most actionable next step is a hybrid eviction strategy that combines recency and occupancy signals—when a tier fills beyond threshold, prioritize removing older items faster, allowing the system to adapt its eviction aggressiveness to actual load rather than applying fixed rules.

A second avenue worth exploring is inserting a validation layer between stash and target: a "warm" tier that forces items to prove their hotness over multiple ticks before promotion to the primary tier, filtering out transient accesses that spike briefly but don't sustain. This reduces wasted capacity on false positives.

A third direction tests whether demoted items should bypass stash and return directly to their original tier, giving them a second chance without the demotion penalty—this could improve recovery for items that were temporarily cold but remain valuable.

However, H03 introduces complexity around tracking recovery tiers and access windows that may conflict with existing invariants. Narrowing focus to three concrete, feasible hypotheses: the occupancy-aware demotion strategy from the recent winning variant, the warm-tier validation approach, and a simpler direct-recovery mechanism that avoids the full recovery_tier tracking overhead.

For the occupancy-aware demotion path, the key insight is computing pending promotions and using occupancy thresholds to decide whether to evict by recency or by access score. When probing reaches capacity and a new promotion is needed, if occupancy exceeds 75%, demote the LRU item; otherwise, demote based on access patterns.

The second hypothesis tests whether the promotion bottleneck stems from overly restrictive "hottest only" filtering. Relaxing this to allow any target-tier item into probing when occupancy stays below 50% could accelerate promotion velocity without requiring items to first prove their heat.

The third hypothesis explores collapsing the three-tier structure entirely—merging target and probing into a single unified hot tier of equal combined capacity, eliminating the two-level promotion discipline to see if simpler architecture outperforms the current filtering overhead.

These three approaches target distinct failure modes: occupancy-driven eviction policy, promotion gatekeeping strictness, and architectural complexity. H01 refines cascade dynamics, H02 removes velocity constraints, and H03 tests whether the multi-tier hierarchy itself is the limiting factor.

For the formal specification, I need to verify the parent requirement—at least one hypothesis must reference a cross-cycle winner. The eligible parents are cycle/0004/v001, cycle/0005/v002, and cycle/0007/v001. H01 will use cycle/0007/v001 as its parent. The first hypothesis modifies promote_to_probing() in src/dash/segments/dash_segment.rs to check target occupancy before accepting probing results, preventing cascade pressure when the target tier lacks absorption capacity.

The second hypothesis collapses the target/probing distinction into a unified hot tier, eliminating promotion gatekeeping by allowing direct insertion. This requires changes to src/shared/geometry.rs and src/dash/segments/dash_segment.rs to merge bucket logic and modify promotion_get() accordingly.

The third hypothesis prioritizes demotion based on access frequency—items showing sustained multiple hits in target get preferential treatment when moving out of the tier.  This involves adding access-count tracking in src/shared/item.rs and filtering promotion candidates to require access_count >= 2, creating a selectivity gate that favors repeated accesses over single-access items.

But I'm second-guessing this approach. If the workload is recency-heavy, frequency-based promotion gates might not help—cycle 0001 already showed that pure frequency-based policies struggle with recency-dominant patterns. Maybe I should test something different instead.

What if I abandon the target→probing promotion path entirely and explore whether applying back-insertion universally across all evictions improves stability? The demotion recovery in earlier cycles used back-insertion successfully, so extending that MRU principle to every eviction could strengthen tier consistency. Though I need to verify this isn't redundant with prior testing.

Looking back: cycle 0003 applied back-insertion specifically to demoted items, and cycle 0004 v001 found that MRU positioning works well for items with proven tier-advancement history but destabilizes newly promoted items in higher tiers. If I universalize back-insertion to all evictions, newly promoted items would also land at the back, which could reintroduce that destabilization problem.

The core tension I'm seeing is that tier-skip and demotion-recovery are architecturally at odds—no occupancy-adaptive tuning or hybrid approach has successfully reconciled them. What if instead of tweaking parameters, I tested a structural change: a "gated tier-skip" where stash-hit items skip to probing but are immediately marked as "filtered," forcing them to survive eviction before gaining full tier status. This could preserve tier-skip's benefits while protecting demotion recovery.

For the implementation, items with 2+ accesses in stash would jump directly to probing with this filtered flag. When evicted from probing, they'd demote to target rather than back to stash, letting them prove value in the target tier before any stash demotion. This creates a structured compatibility test that's distinct from previous attempts.

**H01 (from cycle/0007/v001 - cross-cycle winner):**
- Claim: Coordinate occupancy across tiers by adjusting probing-to-target eviction rates based on target fill level, routing eviction pressure more strategically to balance tier usage and boost hit rate.
- Parent: cycle/0007/v001
- Changes: In demote_from_normal_bucket(), examine target occupancy when deciding where probing residents go. Below 50% target fill, favor target demotion; above 80%, shift toward stash demotion instead, testing whether occupancy-aware cascades improve performance.

**H02 (from baseline):**
- Claim: Removing the target tier entirely and merging it with probing into a single hot tier could reduce filtering overhead and allow direct hot insertion for items passing stash checks.
- Parent: baseline
- Changes: Collapse target and probing into one hot tier in src/shared/geometry.rs and src/dash/segments/dash_segment.rs. Simplify promotion so items filter through stash then move straight to hot, eliminating the intermediate filtering stage.

**H03 (from baseline):**
- Claim: Allow stash residents with 2+ accesses to bypass target and jump directly to probing with a special marker, then demote them to target on probing eviction to test whether this tier-skip pattern improves hit rate.
- Parent: baseline
- Changes: Mark items reaching 2+ stash accesses with a skip flag enabling direct probing entry. On probing eviction, route these marked items to target instead of stash, validating whether structured skip patterns enhance performance.

### 19:27:44 chat

Now I need to analyze the previous cycles and cross-cycle winners to formulate distinct hypotheses. Let me read more about what's proven to work and what hasn't.

### 19:27:44 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007/variants/v002/insight.md"
}
```

### 19:27:44 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0007/variants/v003/insight.md"
}
```

### 19:27:55 thinking

Perfect. Now I have a complete picture. Let me synthesize what I've learned:

**Clear patterns across 7 cycles:**

1. **Demotion recovery with back-insertion is the ONLY working mechanism** - introduced in cycle 0003, it's been carried forward unchanged and is the foundation of all three cross-cycle winners at 0.684.

2. **The 0.684 ceiling is defended across all tested levers:**
   - Insertion position refinements (cycles 0001-0005)
   - Occupancy-based gating (cycles 0002, 0005-0007)
   - Tier capacity ratios (cycles 0006-0007) - achieved marginal +0.0014
   - Promotion eligibility gating (cycle 0005)
   - Eviction priority shifts (cycle 0007)
   - Cascade structure changes (cycle 0007) - removing target-retry dropped hit-rate 3%
   - Stash expansion (cycle 0007) - degraded performance

3. **Key open question:** "Is the bottleneck structural rather than parametric?"

4. **Cycle 0007's continuation guidance:** "Abandon further tuning of existing mechanisms and instead test whether the bottleneck is structural rather than parametric: consider testing alternative tier counts, different promotion qualification criteria beyond simple LRU/MRU rules, or whether the workload's natural access distribution inherently suits a 0.684 ceiling."

Given this, my 3 hypotheses should be:

**H01** (from cross-cycle winner cycle/0007/v001 - REQUIRED):
Test a structural refinement to the demotion cascade by implementing occupancy-aware eviction ordering in the cascade itself, changing HOW items are demoted rather than WHERE, testing if cascade quality can improve hit rate.

**H02** (from baseline):
Test a different tier structure - either collapsing tiers or adding tiers - to test if the 3-tier architecture itself is the constraint. 

Specifically, merge target and probing into a single "hot" tier to see if removing promotion gatekeeping between them unlocks better hit rates.

**H03** (from baseline):
Test controlled tier-skip where stash-resident items that have been accessed multiple times can skip directly to probing BUT are marked for target-demoting (not full stash-demotion) if evicted, attempting to reconcile tier-skip with demotion-recovery that has previously been incompatible.

These are distinct and test genuinely different angles without just tweaking the same knobs.

### 19:27:55 chat

Based on my analysis of 7 cycles of research, I can now synthesize the context and generate 3 distinct hypotheses for cycle 0008. The key insights are:

1. **The 0.684 hit-rate ceiling is robustly defended** against all parametric tuning (insertion position, occupancy gating, capacity ratios, promotion eligibility, eviction priority, cascade structure) over 7 cycles
2. **Demotion recovery with MRU back-insertion is the sole working mechanism**, introduced in cycle 0003 and never improved upon
3. **The guidance from cycle 0007** is to shift from parametric tuning to structural/architectural testing: alternative tier counts, different promotion qualification criteria, and validating whether the workload's access pattern inherently suits 0.684

Let me now write the 3 hypotheses:

### 19:28:00 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0008/hypotheses/H01.md",
  "content": "# H01\n\nImprove hit rate by implementing occupancy-coordinated demotion where cascade paths prioritize target tier for intermediate placement only when target occupancy is below saturation, reducing eviction pressure on the hot probing tier.\n\n## Parent\n\ncycle/0007/v001\n\n## Changes\n\nModify the demotion logic in `src/dash/segments/dash_segment.rs` in the `demote_from_normal_bucket()` function to implement occupancy-aware eviction destination selection:\n\nCurrently, when items are evicted from probing, they are demoted to target with back insertion (MRU), and if target is full, cascaded to stash. This cascade pattern is fixed regardless of occupancy state.\n\nChange this to:\n1. Before demoting from probing to target, check target tier occupancy\n2. If target occupancy < 70%, demote to target with back insertion (existing behavior, which has proven beneficial for demotion recovery)\n3. If target occupancy >= 70%, demote directly to stash with back insertion instead, bypassing the congested target tier to reduce cascade depth when target is saturated\n4. Keep the existing demotion-recovery back-insertion strategy intact\n\nThis tests whether coordinating eviction destination based on tier occupancy state can reduce cascade overhead during high-load periods, potentially unlocking hit-rate gains by improving eviction flow without changing the fundamental demotion-recovery principle that has proven stable across all prior winners.\n"
}
```

### 19:28:04 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0008/hypotheses/H02.md",
  "content": "# H02\n\nTest architectural simplification by collapsing the two-tier hot-side (target and probing) into a single unified \"hot\" tier, eliminating promotion gatekeeping overhead and testing if simpler hierarchy improves hit rate.\n\n## Parent\n\nbaseline\n\n## Changes\n\nModify the tier architecture in `src/shared/geometry.rs`, `src/dash/segments/dash_segment.rs`, and `src/dash/segments/segment_core.rs` to implement a two-tier hierarchy (stash + hot) instead of three-tier (stash + target + probing):\n\n1. In `BaselineGeometry::for_settings()`, keep 4 stash buckets but allocate all 28 normal buckets to a single \"hot\" tier instead of splitting 14 to target and 14 to probing\n2. In `Slot` enum definition, remove or merge the `Target` and `Probing` variants into a single `Hot` tier\n3. Modify `promotion_get()` in DashSegment to insert hot items directly into the unified hot tier rather than the two-stage target→probing advancement. Items in stash that qualify as hot (strongest candidates) go directly to hot tier, eliminating the intermediate target filtering step\n4. Preserve the demotion-back-insertion strategy (from cycle 0003) by demoting items from hot tier to stash with back insertion\n\nThis tests whether the two-tier hot-side complexity (requiring items to pass target filtering before probing access) is the architectural bottleneck. If the workload's access pattern can tolerate a simpler structure, removing the promotion gatekeeping may improve throughput and hit rate. The test validates whether tier count is the constraint or whether the current three-tier hierarchy is well-matched to the workload.\n"
}
```

### 19:28:09 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0008/hypotheses/H03.md",
  "content": "# H03\n\nEnable controlled tier-skip for frequently-accessed stash items while maintaining structural demotion recovery by marking skipped items for target-tier re-entry on eviction, reconciling stash acceleration with demotion-recovery safety.\n\n## Parent\n\nbaseline\n\n## Changes\n\nModify promotion and demotion logic in `src/dash/segments/dash_segment.rs` and `src/shared/item.rs` to implement gated stash-skip with recovery-path enforcement:\n\n1. Add a `skip_history_flag: bool` field to the Item struct in `src/shared/item.rs` to mark items that have skipped from stash to probing\n2. In `promotion_get()`, when considering promotion of a stash resident, check if it has been accessed multiple times in the stash tier (use an existing or new access-count field). If access_count >= 2 in stash, promote directly to probing (skipping target) but set `skip_history_flag = true`\n3. In `demote_from_normal_bucket()`, when demoting an item from probing that has `skip_history_flag = true`, demote it to target (not directly to stash) with back insertion, allowing it a second chance to prove value in the target tier before final stash demotion\n4. Items without `skip_history_flag` follow the existing demotion path (stash residents demoted directly to stash)\n\nThis tests whether we can reconcile two previously incompatible strategies: stash-acceleration (which improved hit rate in cycles 0001-0002) with demotion-recovery (which has been the only working mechanism since cycle 0003). By gating the skip to proven stash-resident items and enforcing a target-tier recovery step on their eviction from probing, this variant tests whether structured tier-skip can coexist with demotion recovery without creating the \"unvetted hot-tier residents\" problem that broke tier-skip approaches in cycles 0004 and 0006.\n"
}
```

### 19:28:16 thinking

Great! I've successfully written all three hypotheses. Let me verify they meet the requirements:

1. **H01:**
   - Claim: ✓ "Improve hit rate by implementing occupancy-coordinated demotion..." - concrete change + fitness dimension
   - Parent: ✓ cycle/0007/v001 (cross-cycle winner)
   - Changes: ✓ Specific prose describing the diff - modify demote_from_normal_bucket() with occupancy checks

2. **H02:**
   - Claim: ✓ "Test architectural simplification by collapsing the two-tier hot-side..." - concrete change + fitness dimension
   - Parent: ✓ baseline
   - Changes: ✓ Specific files and functions to modify - BaselineGeometry, DashSegment, Slot enum

3. **H03:**
   - Claim: ✓ "Enable controlled tier-skip for frequently-accessed stash items..." - concrete change + fitness dimension
   - Parent: ✓ baseline
   - Changes: ✓ Specific prose describing the diff - add skip_history_flag, modify promotion_get(), modify demote_from_normal_bucket()

**Distinctness check:**
- H01: Tests occupancy-aware cascade coordination (focusing on eviction destination selection)
- H02: Tests architectural simplification (removing a tier, simpler structure)
- H03: Tests reconciling previously incompatible strategies (tier-skip + demotion-recovery)

These are clearly distinct - no near-duplicates.

**Grounded check:**
- Each builds on prior research (cross-cycle winners or baseline)
- Each has specific file/function references
- Each describes what to change concretely

**Testable check:**
- H01: Can measure if occupancy-aware cascades unlock gains


- H02: Can measure if two-tier hierarchy performs better than three-tier
- H03: Can measure if structured tier-skip + recovery works better than either alone

All three are ready to go.

### 19:28:19 chat

Perfect! I've generated 3 distinct hypotheses for cycle 0008:

## Summary

**H01** (from cross-cycle winner `cycle/0007/v001`): Tests **occupancy-aware cascade coordination** where demotion destination (target vs. stash) is selected based on target tier occupancy, reducing eviction pressure during saturation. This keeps the proven demotion-recovery mechanism intact while optimizing eviction flow.

**H02** (from `baseline`): Tests **architectural simplification** by collapsing the three-tier hierarchy (stash + target + probing) into a two-tier hierarchy (stash + unified hot), eliminating the intermediate target tier's gatekeeping to test if the promotion pipeline overhead is the bottleneck.

**H03** (from `baseline`): Tests **reconciling incompatible strategies** by enabling controlled tier-skip for frequently-accessed stash items (which worked in early cycles) while maintaining structural demotion recovery by enforcing target-tier re-entry on their eviction, addressing the architectural incompatibility identified in cycles 0004 and 0006.

### Key distinctions:
- **H01** refines cascade mechanics (eviction destination selection)
- **H02** tests structural architecture (tier count reduction)  
- **H03** bridges previously incompatible strategies (stash-skip + demotion-recovery)

All three hypotheses:
- ✅ Cite exactly one parent (H01 from winner, H02/H03 from baseline)
- ✅ Have specific, actionable changes (functions, files, logic to modify)
- ✅ Test genuinely distinct strategies (not parameter tweaks of the same mechanism)
- ✅ Are grounded in prior cycle findings and open questions from cycle 0007

The hypotheses are written to `/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0008/hypotheses/` as `H01.md`, `H02.md`, and `H03.md`.


---

- finished: 2026-06-05T19:28:19+00:00
- duration: 189.84s
- status: ok
- input_tokens: 104
- output_tokens: 17901
- cache_read_tokens: 501755
- cache_write_tokens: 38662
- cost_usd: 0.189483
