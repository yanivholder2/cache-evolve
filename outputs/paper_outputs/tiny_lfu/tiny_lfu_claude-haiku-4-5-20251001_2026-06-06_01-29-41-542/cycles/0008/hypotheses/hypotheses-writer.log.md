# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-4-5-20251001)
- workspace: /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0008/hypotheses
- allow_edits: True
- started: 2026-06-05T23:18:40+00:00

## timeline

### 23:18:44 thinking

The user is asking me to generate 3 hypotheses for cycle 0008 in a cache evolution project. Let me understand the task:

1. I need to read context files from cycle 0007 and prior cycles
2. Generate 3 distinct hypotheses
3. At least one MUST cite a cross-cycle winner as its parent
4. Each hypothesis must have:
   - A Claim (one sentence, concrete change + fitness dimension)
   - ## Parent (baseline or cycle/NNNN/vNNN)
   - ## Changes (specific modifications for the implementer)
5. Save them as H01.md, H02.md, H03.md in the output directory

The incumbents (cross-cycle winners) are:
- cycle/0004/v001: hit-rate=0.7429
- cycle/0005/v001: hit-rate=0.7429
- cycle/0007/v001: hit-rate=0.7429

All have the same hit-rate (0.7429), so they're equally strong winners.

I should start by reading the context files to understand what's been tried and what might be promising next steps. Let me plan which files to read first:

1. Previous cycle (0007) variants and their results
2. Cross-cycle history and insights
3. The baseline to understand starting point
4. Maybe look at the leaderboard

Let me start with the cycle 0007 results and the cycle summary.

### 23:18:44 chat

I'll help you generate 3 distinct hypotheses for cycle 0008. Let me start by reading the context files to understand what's been learned.

### 23:18:45 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0007/cycle-insight.md"
}
```

### 23:18:45 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0007/cycle-winners.json"
}
```

### 23:18:45 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/leaderboard.md"
}
```

### 23:18:46 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cross-cycle-insights.md"
}
```

### 23:18:47 thinking

Good, I have the cycle 0007 insights and the cross-cycle insights. Let me now look at the specific variants from cycle 0007 to understand what was tried:

### 23:18:47 chat

Now let me read the specific variants from cycle 0007 to understand what was attempted:

### 23:18:48 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0007/variants/v*/insight.md"
}
```

### 23:18:54 thinking

Let me try a different approach to find the variant information:

### 23:18:54 tool: Bash

```json
{
  "command": "ls -la /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0007/variants/"
}
```

### 23:18:55 thinking

I can't use Bash, only the read-only tools. Let me use Glob more carefully to find the variant directories:

### 23:18:56 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0007/variants/v001/attempts/*/overlay"
}
```

### 23:18:56 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0007/variants/v002/attempts/*/overlay"
}
```

### 23:18:56 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0007/variants/v003/attempts/*/overlay"
}
```

### 23:18:58 thinking

The glob pattern isn't finding the files. Let me try a broader glob to see the directory structure:

### 23:18:58 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0007/variants/*"
}
```

### 23:19:00 thinking

Good, now I can see the structure. Let me read the insight.md files for each variant, and also the task.md files to understand what hypotheses they came from:

### 23:19:00 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0007/variants/v001/task.md"
}
```

### 23:19:00 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0007/variants/v001/insight.md"
}
```

### 23:19:01 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0007/variants/v002/task.md"
}
```

### 23:19:01 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0007/variants/v002/insight.md"
}
```

### 23:19:01 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0007/variants/v003/task.md"
}
```

### 23:19:01 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0007/variants/v003/insight.md"
}
```

### 23:19:03 thinking

Great, now I have the context. Let me look at what the cycle 0005 v001 looks like, since it's a cross-cycle winner and I might want to build from there:

### 23:19:04 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0005/variants/v001/attempts/*/overlay/src/tiny_lfu/mod.rs"
}
```

### 23:19:05 thinking

Hmm, let me try the baseline to understand the basic structure:

### 23:19:06 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/tiny_lfu/baseline/overlay/src/tiny_lfu/mod.rs",
  "limit": 200
}
```

### 23:19:07 thinking

Let me try with the correct path structure. Let me find where the baseline files are:

### 23:19:07 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/inputs/subjects/tiny_lfu/baseline/**/*"
}
```

### 23:19:09 thinking

Interesting - it looks like the baseline doesn't have an overlay directory. Let me check the v001 v002 v003 from cycle 0007 to understand the actual overlay files that were modified:

### 23:19:09 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0007/variants/v001/attempts/01/overlay/src/tiny_lfu/mod.rs",
  "limit": 300
}
```

### 23:19:11 thinking

Good! Now I can see v001 from cycle 0007 has burstiness tracking. The key insight from the cycle-insight is that this added no improvement. Let me also read the hypothesis files from cycle 0007 to understand what was being tested:

### 23:19:12 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0007/hypotheses/*"
}
```

### 23:19:13 thinking

Let me read the hypotheses from cycle 0007 to understand what exactly was being tested:

### 23:19:14 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0007/hypotheses/H01.md"
}
```

### 23:19:14 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0007/hypotheses/H02.md"
}
```

### 23:19:14 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0007/hypotheses/H03.md"
}
```

### 23:19:36 thinking

Great! Now I have a clear understanding of what's been tried in cycle 0007 and what failed:

1. H01 (cycle 0005/v001 parent): Burstiness tracking - no improvement
2. H02 (baseline parent): Frequency tiering - slight regression
3. H03 (baseline parent): Collision detection for sketch - regression

From the cross-cycle insights, I can see:
- Frequency-recency hybrid (age / (frequency + constant)) is the persistent winner
- Simplicity and directness persist; complex refinements fail
- The admission gate strict `>` is optimal
- Sketch parameters are adequate
- Returns from parameter-space tuning are exhausted

The cycle-0007 insight notes that the baseline policy is resistant to improvement from multiple directions and suggests:
1. Either exhausted the signal space and need other item-level properties
2. Or approaching/at asymptotic ceiling
3. Or needs fundamentally different architectural approach (adaptive mixing, multi-level caches)
4. Or should measure theoretical upper-bound

The key question is: what new angles haven't been explored?

Looking at past cycles:
- Cycles 0001-0005: Sketch precision tuning (width/depth), admission gate variants, frequency-age formula tuning
- Cycle 0006: Relaxation and structural augmentation
- Cycle 0007: New signals, alternative selection methods, sketch refinement

Now for cycle 0008, I need to propose 3 distinct hypotheses. At least one MUST cite a cross-cycle winner (cycle 0004/v001, cycle 0005/v001, or cycle 0007/v001).

Some possible angles to explore:

1. **Workload-specific adaptive parameters**: The current solution is locked to the baseline's constants. Maybe the formula coefficient (2.0) or the burstiness/tiering constants should adapt based on observed workload characteristics? Parent: cycle/0005/v001 (the best winner) - Test if adapting the +2.0 constant in the frequency denominator based on frequency distribution improves hit-rate.
   
2. **Non-linear age transformation**: What if recency isn't weighted linearly? Applying logarithmic or exponential decay to age could capture temporal locality more effectively, prioritizing items accessed very recently over those accessed moderately recently. Parent: cycle/0005/v001 - Test whether non-linear age scaling in the scoring formula improves performance.

3. **Cost-aware eviction**: Beyond frequency and age, consider the penalty of missing certain items. Some access patterns might be more costly to evict than others, suggesting a weighted victim selection approach. Parent: baseline - Explore whether incorporating item-specific cost signals into eviction decisions yields gains.

4. **Sliding window frequency estimation**: Replace the cumulative frequency counter with a time-windowed mechanism that emphasizes recent access patterns over historical ones, potentially better capturing workload phase shifts.

5. **Dynamic admission control**: Adjust the admission threshold based on current cache hit rate, allowing the system to tighten or relax the gate depending on performance feedback. Parent: cycle/0005/v001

Looking at the experimental results across cycles, the frequency-age hybrid consistently outperforms all alternatives, while most other approaches degrade performance. The baseline proves remarkably robust. This suggests the policy space may be approaching saturation. The real questions become: Are we hitting a theoretical ceiling on this workload? Does the hybrid approach generalize to different access patterns? Could alternative frequency mechanisms—sampling, probabilistic counting, or adaptive estimation—unlock further gains? Or do we need architectural changes like multi-level caches or policy mixing? Since I'm constrained to overlay file modifications, exploring different frequency estimation techniques or empirical characterization of performance limits seems most tractable.

I can modify the Rust implementation directly, which means changing data structures, algorithms, and core logic. The key is grounding any changes in prior experimental insights. Since signal-based improvements have consistently failed and the baseline resists perturbation, I should test whether the current linear age weighting is actually optimal—logarithmic scaling might better capture recency effects, or alternative frequency bucketing strategies could improve discrimination between access patterns.

There's also the question of whether the full-scan victim selection is necessary, or if sampling or heuristic-based approaches could find equally good victims while reducing computational overhead. However, the cycle insights explicitly warn against further policy-layer tuning, suggesting diminishing returns. The real question becomes whether I'm proposing infrastructure changes or testing on different workload characteristics within the same experimental framework.

Several unexplored angles remain: using statistical measures like z-scores to identify outlier items for eviction rather than linear scoring; implementing frequency thresholds that automatically protect high-access items from consideration; restricting the eviction candidate pool to the oldest items rather than scanning everything; and applying temporal decay to frequency counts so older accesses matter less. Each of these shifts the eviction mechanism in a different direction—some focus on reducing search space, others on changing what "victim-worthy" means.

I need to clarify which variant actually won the previous cycle, since the ledger shows multiple v001 entries across different cycles. Understanding that progression will help me build on the right foundation.

### 23:19:36 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles-ledger.jsonl"
}
```

### 23:21:20 thinking

Excellent! Now I have the full context. The cycle ledger shows the progression clearly:

- Cycle 0001: Winner v001 with sketch-width increase (+0.0047 from baseline of 0.736 to 0.741)
- Cycle 0002: Winner v001 with further width increase (+0.0006, to 0.742) - diminishing returns
- Cycle 0003: Winner v001 with frequency-recency hybrid eviction (age / frequency formula) - big shift to policy tuning
- Cycle 0004: Winner v001 with linear frequency-age formula (age / (frequency + 1)) - refined formula
- Cycle 0005: Winner v001 unchanged - formula is locally optimal, fine-tuning exhausted
- Cycle 0006: Winner v001 - admission gate and eviction locked; tried relaxation, size penalties, time-based - all failed
- Cycle 0007: Winner v001 - tried burstiness (no change), tiering (regression), sketch refinement (regression)

The clear insight is:
1. The baseline is extremely robust - resists all attempts at improvement
2. The frequency-age hybrid (age / (frequency + C)) is the sweet spot
3. All signal-based improvements fail
4. The strict admission gate (>) is optimal
5. Policy-layer tuning returns are exhausted

The cycle-0007 insight explicitly says:
"Do not pursue further policy-layer tuning—the returns are exhausted. The next cycle should either broaden the test matrix to other workloads or cache configurations to understand whether the current optimum is local or global, or shift focus to cache-infrastructure-level changes (capacity, architecture, data-structure alternatives) and empirical characterization of the performance ceiling."

However, I still need to propose 3 hypotheses for cycle 0008. So within the constraint of "same codebase, same workload", what are the remaining angles?

Looking at the openquestions and the patterns:


1. **Untapped item-level signals?** Request clustering, inter-arrival variance, access-distance distribution—properties beyond frequency and recency that might correlate with future utility.
2. **Data structure optimization?** The victim selection currently scans linearly; a priority queue or heap could shift eviction accuracy or patterns without changing the policy itself.
3. **Sampling-based selection?** Rather than evaluating all candidates, sample a subset and choose the best victim from that pool—trading accuracy for computational efficiency.
4. **Adaptive tier boundaries?** Refine the frequency bucketing approach with dynamic thresholds that adjust based on the observed frequency distribution.
5. **Cost-weighted eviction?** Incorporate item size or other metadata into the victim formula, weighting the age/frequency tradeoff by what's actually worth keeping.
6. **Sliding time windows?** Replace absolute age with a recency window that captures temporal locality more precisely.
7. **Predictive admission?** Items with prior re-requests show stronger future demand signals, so admission decisions could factor in historical re-request likelihood.

The core challenge is that incremental signal additions and sketch refinements haven't moved the needle—the system appears stuck at a local optimum. Since one hypothesis must reference a cross-cycle winner (cycle/0005/v001, cycle/0004/v001, or cycle/0007/v001 at 0.7429 hit-rate), I need to ground a proposal in what those variants achieved while exploring genuinely novel directions. The remaining hypotheses can push into unexplored territory or recombine existing ideas in ways not yet attempted.

For the first hypothesis tied to a cross-cycle winner, I should examine what made that variant succeed and propose a refinement that hasn't been tested. The burstiness mechanism in cycle/0007/v001 reached the same 0.7429 hit-rate as the incumbent, suggesting the approach has merit but may need adjustment—perhaps the window size was suboptimal, or the coefficient weighting needs recalibration for this specific cache configuration. Alternatively, burstiness could shift from eviction selection to the admission gate itself, controlling which items enter the cache rather than how they leave. maintained parity rather than improving. The insight confirms that burstiness tracking didn't degrade performance—it just didn't advance it either, making it a valid but non-progressive variant.

Comparing across cycles, c0006/v001 scored 0.741, falling short of the 0.7429 threshold. The cross-cycle winners remain c0004/v001, c0005/v001, and c0007/v001, all at 0.743. I need to identify which one to use as a parent and justify the choice with a concrete hypothesis.

Looking at what remains unexplored: the open questions point toward untapped signal in item-level properties beyond victim selection and admission—specifically request clustering patterns and inter-arrival characteristics that might unlock further gains.

A few directions stand out. Inter-arrival time variance could reveal predictability in access patterns. Request distance between accesses might matter more than just age alone. Hotspot detection could preserve items experiencing concentrated access bursts. But cycle-0007 already tested burstiness and found it yielded no change, suggesting frequency and age already capture the temporal signals this workload needs.

So adding new signals seems exhausted. Instead, I should explore structural changes: using a priority queue instead of linear scan for victim selection, sampling K random items rather than scanning all, filtering by frequency tier before selecting the oldest, or applying non-linear age weighting like logarithmic or exponential scaling.

The adaptive frequency offset is worth revisiting too—replacing the fixed +2.0 constant with something that adjusts based on workload characteristics. Temporal locality bias in the admission gate could also help, weighting recently accessed items more heavily.

Looking back at the actual code, cycle-0005/v001 established the age/(frequency+1) formula with a 16-wide sketch and depth 4. Cycle-0007/v001 added burstiness tracking but produced identical hit-rates, so the burstiness code runs without harm or benefit. The winning policy remains that core formula with burstiness measurement that shows no measurable impact.

Given these constraints, I should test whether the frequency denominator constant needs adjustment. The current implementation uses +2.0 in c0007/v001, but maybe with the burstiness tracking now in place, a different constant like 0.0, 0.5, or something else would perform better. The earlier cycles settled on +1, but that was before the burstiness layer existed. at c0005's documentation to confirm the baseline formula. The H01 from c0007 explicitly states it's modifying from `score = age / (frequency + 2.0)`, which means c0005/v001 already used that denominator. So the progression appears to be: c0004 had `+ 1`, c0005 shifted to `+ 2.0`, and c0007 added the burstiness factor while keeping the `+ 2.0`. I should verify this by checking c0005's own hypothesis documentation. Maybe the constant shifted between c0004 and c0005. Looking at the c0007/v001 code, it clearly uses `+ 2.0` in the denominator. The c0007 H01 description mentions modifying from that same formula when comparing to c0005/v001, which means c0005/v001 also had `+ 2.0`. But c0004's documentation references `+ 1`, so the adjustment must have happened in one of the intermediate versions—possibly c0005/v002 or v003 introduced the change before v001 won out in the final cycle.

Wait, but c0005/v001 lists c0004/v001 as its parent and marks itself as "unchanged baseline." That should mean they're identical. Let me verify the task.md file to confirm the parent relationship and whether I'm reading it correctly.

The parent is definitely c0004/v001 according to the task.md. So if c0005/v001 is truly unchanged, the formulas should match. But c0007/v001's hypothesis explicitly states it's modifying FROM `score = age / (frequency + 2.0)`, which is c0005/v001's code. This means c0005/v001 must have the `+ 2.0` constant. That's a discrepancy—either the constant was introduced between c0004 and c0005 despite the "unchanged" label, or I'm misreading the code somewhere. So the H01 description for c0007 explicitly states it's modifying FROM `age / (frequency + 2.0)` TO `(age * burstiness_factor) / (frequency + 2.0)`, which means c0005/v001 must have used the +2.0 constant. That's the baseline it's building on.

Looking back at c0005's cycle insight about refining the victim-selection formula through parameter tuning and signal ablation—it seems like c0005 may have already settled on +2.0 as the optimal constant, possibly after testing different frequency offset values.

The pattern across cycles is consistent: c0004/v001 introduced the frequency-age hybrid, c0005/v001 kept that approach unchanged (and won), then c0007/v001 added burstiness to the same formula without regression.

For cycle 0008, I'm considering several directions. I could optimize the denominator constant now that burstiness is part of the formula, or test whether the burstiness computation itself needs refinement. Alternatively, I might explore entirely different mechanisms like sample-based victim selection or non-linear age weighting. The clearest signal from recent cycles is that incremental policy adjustments are yielding results, so I should probably continue refining the current formula rather than abandoning it.

Looking at what's already been tested—frequency tiering regressed in H02, gate relaxation failed in c0006—I need to identify gaps. The inter-arrival time variance and access-distance distribution remain unexplored. Testing victim selection based on access distance could reveal whether spatial locality patterns matter more than temporal ones.

An adaptive admission gate is worth exploring—instead of a fixed threshold, adjust it dynamically based on whether admitted items actually get hit. I could also try sampling K random candidates and selecting the best victim from that subset rather than scanning everything, which might expose whether the current exhaustive search is masking better heuristics.

For ranking, I could move away from continuous scoring and instead use ordinal comparisons—sort by frequency first, then age, then evict the worst candidate meeting some criterion. Dividing the cache into age cohorts and prioritizing eviction from the oldest cohort might also help, since it focuses pressure on truly stale items. Finally, applying decay to frequency counts so recent accesses matter more than old ones could better capture what's actually useful right now.

I should also test whether the denominator constant in c0007/v001 is actually optimal, or if removing the burstiness tracking and trying different values reveals a better formula. The burstiness component might be unnecessary overhead from an earlier approach.

For the incumbent parent requirement, I need to pick one of the proven winners and build something genuinely new from it. Reverse-engineering whether the same performance is achievable through a different mechanism seems promising—testing if the incumbent's hit rate can be matched with an entirely different approach would validate whether the formula itself is special or if multiple paths lead to the same outcome. Alternatively, I could test how sensitive the incumbent is to small perturbations in its parameters, which would reveal how robust the solution actually is.

But if prior work already exhausted constant tweaking, I should look elsewhere. What stands out is the idea of applying the policy more selectively—perhaps the burstiness signal only matters for certain item categories. Since high-frequency items are already well-protected by frequency alone, maybe burstiness should only influence decisions for low-frequency items where frequency-based signals are weaker. This would test whether the incumbent's success comes from a one-size-fits-all approach or whether adaptive application could be equally effective.

I'm also considering a couple of other angles: sampling-based victim selection instead of full scans, which might uncover different local eviction patterns, and logarithmic age weighting to better capture how recent accesses matter more than older ones. The selective burstiness approach seems most promising though—it's a natural refinement that doesn't require new mechanisms, just conditional application of the existing one. Everything else stays the same—frequency denominator and admission gate logic remain untouched.

H03 feels redundant though; it's essentially another formula tweak, and those haven't yielded improvements. Looking back at what's worked: continuous scoring performs well, but discretization and sketch refinement from first principles both failed. The issue isn't the scoring structure itself. I should focus on what hasn't been explored yet—removing the frequency denominator entirely (though C0005 H03 covered that), bucketing frequency (tested multiple times), adjusting admission strictness (C0006, C0001), tuning sketch parameters (C0001-C0003, C0007), or introducing new signals like size (C0006). Most of these have been attempted already.

What stands out as genuinely untested: victim search optimization using sampling or greedy approaches, multi-phase eviction strategies, workload-adaptive normalization of frequency and age constants (different from simple constant tweaking), or tracking evicted items to prevent re-admission within a window. That last one is particularly promising—maintaining a record of recently evicted keys and blocking their readmission for a period could prevent thrashing and improve cache stability.

I'll focus on two concrete approaches. First, adapting the frequency offset dynamically based on the median frequency observed in the current cache state rather than using a fixed constant. This lets the scoring formula self-tune to each workload's frequency distribution. Second, replacing the full-cache scan for victim selection with random sampling, which reduces computational overhead while still finding reasonable candidates for eviction.

A third option involves tracking recently evicted keys and preventing their immediate re-admission, which avoids the churn that occurs when low-value items keep cycling in and out of the cache.

The core insight is that frequency-age hybrids consistently outperform other approaches, while pure parameter variations have been exhausted. For the next cycle, I should focus on structurally different algorithms rather than formula tweaking. Sample-based victim selection, two-phase eviction strategies, heap-based data structures, or adaptive batching could offer genuinely novel directions that are still grounded in testable hypotheses.

I'm considering two main candidates: an adaptive denominator that responds to the current frequency distribution in the cache, and a randomized sampling approach that trades full-cache scans for probabilistic victim discovery. The first keeps the proven hybrid structure but makes it responsive to workload characteristics by computing the median frequency and using that to set the offset dynamically. The second shifts to a fundamentally different selection algorithm using reservoir sampling over K items, which could reduce computational overhead while potentially finding different local equilibria in the eviction space.

There's also a two-phase filtering strategy that separates cold items by frequency bucket first, then applies age-based selection within that bucket. This avoids the complexity of percentile-based approaches while still targeting genuinely infrequently-accessed items for eviction.

For H01, applying burstiness adjustment selectively—only to items below the median frequency—could be more efficient than a global adaptive offset, since it preserves temporal-locality benefits for borderline candidates without unnecessary computation on already-hot items.

But that's still just a formula adjustment. I should consider whether the real bottleneck is structural rather than algorithmic. A two-pointer optimization for victim selection could reduce scanning overhead, though it's unclear if that directly improves hit-rate or just implementation efficiency. The core problem seems to be that the frequency-age hybrid is constraining performance—the incumbent policy wins because it's already well-tuned to the workload characteristics.

Maybe the question isn't how to refine the existing approach, but whether I need frequency and age at all. Testing pure recency or pure frequency signals could reveal if one dimension is actually unnecessary. But prior experiments already showed removing age caused a 0.31% regression, and adding new signals had no measurable effect. The policy structure appears fundamentally locked in. This leaves only a few viable directions: optimizing the implementation itself through sampling or lazy updates, examining whether infrastructure constraints like capacity or cache hierarchy are the real limits, or improving how I measure and validate performance empirically.

Since I'm constrained to modifying just the tiny_lfu code, infrastructure and measurement improvements are largely out of reach. That points toward implementation-level optimizations as the most promising path forward.

One approach is victim selection with early-exit logic—instead of always finding the globally-optimal victim, I could set a "good-enough" threshold and stop scanning once I find a victim that crosses it, reducing wasted computation in pathological cases. Another is introducing randomness into victim selection through random probing rather than linear scanning, which might break up deterministic eviction patterns and improve hit rates by adding beneficial variability to which items get evicted.

A third direction involves making the scoring formula adaptive to the cache's temporal characteristics by normalizing age relative to the maximum age currently in the cache, so the formula scales appropriately regardless of the working set's age distribution. Though I recognize this is still formula-tweaking at its core.

For genuinely different approaches: pinning very hot items (those above the 90th percentile frequency) to skip them entirely from eviction consideration, reducing unnecessary computation while preserving the most-accessed entries. Another angle is constraining victim selection to a limited search window starting from the LRU tail, restarting the scan periodically to bias eviction toward recently-inactive items without examining the entire cache.

A third strategy separates items into frequency tiers using the median as a dividing line, always prioritizing eviction from the low-frequency group before touching high-frequency items, ensuring cold entries are removed first.

Two additional approaches focus on different aspects: one reserves the hottest 25% of items from eviction by computing a frequency percentile threshold, while another targets the oldest age cohort specifically, examining only the most recent 10% of items (capped at 30) to identify victims with minimal scanning overhead. -scored sampled item. This approach avoids scanning the entire cache and instead relies on random sampling to identify strong eviction candidates, which can reduce variance in the eviction path and potentially stabilize performance across different equilibrium states.

These three hypotheses target fundamentally different mechanisms: frequency-based filtering to preserve high-access items, age-based cohort prioritization to target stale entries, and randomized sampling as an alternative algorithmic approach altogether.

Now I need to structure these as proper hypothesis files. I should reconsider the parent assignment for the required incumbent-based hypothesis. The instructions specify at least one hypothesis must reference a cross-cycle winner as its parent, preferring non-baseline incumbents since they represent the strongest prior results. The candidates are cycle/0004/v001, cycle/0005/v001, and cycle/0007/v001, all with hit-rate 0.7429.

While all three perform equally, c0007/v001 is the most recent and incorporates burstiness tracking, making it the most evolved version. c0005/v001 represents a stable, verified baseline without added complexity. I'll anchor to c0007/v001 since it's the latest incumbent with the most refinement. This means H01 should build directly from c0007's foundation—either extending its burstiness mechanism, enhancing the eviction logic, or refining how it identifies candidates for removal.

For H01, I'm considering several directions: selective burstiness applied only to infrequent items, an adaptive burstiness window, bypassing burstiness for high-frequency items, or hierarchical frequency filtering. I'll frame H01 as "Frequency-percentile filtering" to extend c0007's core idea while keeping it distinct.

Actually, let me be more concrete. H01 should test whether the frequency denominator constant (currently 2.0) needs adjustment—trying lower values like 0.0, 0.5, or 1.0 to boost hit rate. The discrepancy between c0005 (supposedly unchanged from c0004) and c0007's code showing +2.0 suggests this constant might be miscalibrated.

Wait, that's just parameter tuning again. Let me refocus: H01 extends c0007's burstiness tracking to also influence the admission gate—allowing items with burst patterns to enter the cache more readily. The rationale is that c0007 applied burstiness only to eviction, not admission, so incorporating it into the gate decision could improve performance.

For H02, I'll test two-phase eviction where cold items (low frequency) are always evicted before warm items. This explicitly tests frequency-bucketing in eviction, which hasn't been directly explored yet.

H03 will use random-sampling victim selection instead of scanning the entire cache—a fundamentally different algorithmic approach rather than a formula adjustment.

Actually, H01 might be too similar to prior admission gate experiments that failed. Let me reconsider and focus on distinct, structural changes instead of parameter tweaks. H01 should be hierarchical victim selection that separates victims by frequency percentile and always evicts from the coldest tier first. H02 could test age-segmented eviction that prioritizes the oldest items.

For H03, I'll explore random-sample victim selection—sampling random candidates for eviction instead of scanning all items, which might discover local equilibria and improve hit-rate. These three approaches are fundamentally different algorithms rather than formula adjustments.

### 23:21:20 chat

Now let me examine earlier cycle winners more closely to understand the stable incumbent approach:

### 23:21:21 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0005/variants/v001/attempts/01/overlay/src/tiny_lfu/mod.rs",
  "limit": 200
}
```

### 23:21:31 thinking

Perfect! So c0005/v001 uses the formula `(age as f64) / ((victim_freq as f64) + 2.0)`.

And c0007/v001 adds burstiness but keeps the same +2.0 constant.

So the incumbent winner uses:
- Count-Min sketch with 16x width, 4 depth
- Frequency-age hybrid: `age / (frequency + 2.0)`
- Strict admission gate: `incoming_freq > victim_freq`

Now I have all the information I need. Let me write 3 distinct hypotheses:

1. **H01 (Parent: c0007/v001)**: Something grounded in the incumbent that hasn't been tried
   - NOT formula tweaking (exhausted)
   - NOT adding new signals (failed)
   - NOT alternative selection methods like tiering (regressed)
   - SHOULD be structural or algorithmic

2. **H02, H03 (Parent: baseline)**: Two other distinct strategies

Options for H01:
- Frequency percentile filtering in eviction (prioritize cold items)
- Age-bucketing in eviction (prioritize old items)
- Early-exit heuristic when victim is "bad enough"
- Selective application of burstiness

I'll go with frequency percentile filtering since it's structural and hasn't been directly tested (H02 from c0007 tested frequency tiering for ALL selection, not just victim filtering).

H01: Evict only from lowest-frequency cohort
- Parent: c0007/v001
- Claim: Restrict victim selection to items in the lowest frequency percentile to improve hit-rate by ensuring only genuinely cold items are evicted
- Changes: Compute 20th percentile of cached frequencies. Only consider items at or below this percentile for eviction. Use the age/(freq+2.0) formula within this cold cohort. If no cold cohort items exist, fall back to full scan.


H02: Age-stratified eviction targets the oldest items first by partitioning the cache into 4 temporal cohorts and evicting from the most stale cohort before moving to younger ones, applying the same age/(freq+2.0) scoring within each stratum. H03 explores random sampling of victim candidates—drawing min(sqrt(capacity)) items and selecting the worst among them—to find locally-stable equilibria without scanning the entire cache.

### 23:21:35 chat

Perfect! I now have a clear picture. Let me synthesize the context and generate 3 distinct hypotheses for cycle 0008.

## Analysis Summary

From cycles 0001-0007, the key patterns are:
- **Incumbent winner**: `age / (frequency + 2.0)` hybrid eviction (c0005/v001, c0007/v001) with 16x sketch width, depth 4, strict `>` admission gate. Hit-rate locked at 0.7429.
- **What works**: Frequency-age hybrid formula; sketch width tuning (early cycles); simple, direct metrics.
- **What fails**: New signals (burstiness in c0007 H01 = no change), frequency tiering (c0007 H02 regressed), sketch refinement heuristics (c0007 H03 regressed), gate relaxation (c0006 regressed), size penalties (c0006 regressed), formula parameter tweaking (c0005 exhausted).
- **Open frontier**: The cycle-0007 insight explicitly states policy-layer tuning is exhausted. Remaining angles are algorithmic/structural changes or infrastructure (different cache size, multi-level, etc.).

Since I can only modify the tiny_lfu subject code, I'll propose 3 structurally distinct hypotheses:

1. **H01** (Parent: c0007/v001 — the cross-cycle incumbent): Frequency-percentile filtering in victim selection
2. **H02** (Parent: baseline): Age-bucketed hierarchical eviction  
3. **H03** (Parent: baseline): Random-sample victim selection

Let me write these out:

### 23:21:41 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0008/hypotheses/H01.md",
  "content": "# H01\n\nRestrict victim eviction to items in the lowest-frequency cohort to improve hit-rate by ensuring only genuinely cold items are replaced before touching warmer cache residents.\n\n## Parent\n\ncycle/0007/v001\n\n## Changes\n\nIn the victim-selection loop within the `put()` method (where capacity is full and a victim must be chosen), implement frequency-cohort filtering before scoring:\n\n1. Compute the 25th percentile of frequency estimates across all cached items.\n2. Scan through cached items twice:\n   - **First pass** (cold cohort): Consider only items with `estimated_freq <= 25th_percentile` for eviction. Use the existing score formula `(age as f64) / (victim_freq + 2.0)` within this subset to find the best victim.\n   - **Fallback pass** (if cold cohort is empty): Revert to the current full-cache scan if no cold items are found.\n\nKeep all other logic unchanged: the admission gate threshold (`incoming_freq > victim_freq`), the burstiness tracking from c0007/v001, the sketch parameters, and the LRU ordering.\n\n## Rationale\n\nThe incumbent policy (c0007/v001) uses a single continuous score across all items, which can occasionally evict moderately-warm items if they are very old. This hypothesis tests whether explicitly partitioning candidates into a cold cohort first—similar in intent to H02 from Cycle 0007 (frequency tiering), but applied to victim selection only rather than all decision logic—can improve hit-rate by guaranteeing that eviction targets the coldest items first. Unlike H02 (which regressed on tiering), this approach is more conservative: it only filters the victim pool, not the entire scoring surface, reducing the risk of losing fine-grained discrimination. The threshold value (25th percentile) is chosen to align with typical cache working-set models where a small tail of infrequently-used items should be evicted before touching the active set.\n"
}
```

### 23:21:42 thinking

Good, H01 is written. Now let me write H02 and H03.

### 23:21:53 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0008/hypotheses/H02.md",
  "content": "# H02\n\nDivide the cache into age cohorts and always evict from the oldest cohort first to improve hit-rate by ensuring that replacement targets genuinely stale items before touching items in active working sets.\n\n## Parent\n\nbaseline\n\n## Changes\n\nIn the victim-selection loop within the `put()` method (where capacity is full), implement age-bucketed hierarchical eviction:\n\n1. Compute the age (current_time - last_access_time) for all cached items.\n2. Divide items into four age cohorts based on their position in the sorted age distribution:\n   - Cohort 1 (oldest 25%): items with age in the top quartile\n   - Cohort 2 (older 25%): items in the second quartile\n   - Cohort 3 (recent 25%): items in the third quartile\n   - Cohort 4 (newest 25%): items in the bottom quartile\n3. Victim selection proceeds in cohort order:\n   - Iterate through Cohort 1 items and compute the eviction score `(age as f64) / (frequency + 2.0)` for each\n   - Select the item with the highest score from Cohort 1 as the victim\n   - If Cohort 1 is empty (should not occur), move to Cohort 2 and repeat\n4. Keep the admission gate logic unchanged: `incoming_freq > victim_freq`.\n\n## Rationale\n\nThe current victim-selection algorithm scores all items globally and can occasionally evict items from the middle of the working set if they happen to have high age relative to their frequency. This hypothesis tests whether a hierarchical age-first approach can improve hit-rate by imposing an ordering constraint: genuinely old items must be exhausted before newer items are considered for eviction. This is distinct from frequency-tiering (which H02 in Cycle 0007 showed regresses) because it partitions by age, not frequency, and because age is known to be orthogonal to frequency in this cache (confirmed by Cycle 0005). The four-cohort structure trades off implementation overhead for finer-grained age distinction, allowing the algorithm to preferentially target the tail without abandoning frequency-age scoring within each cohort.\n"
}
```

### 23:21:54 thinking

Good, H02 is written. Now H03.

### 23:21:58 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0008/hypotheses/H03.md",
  "content": "# H03\n\nUse random sampling of victim candidates instead of full-cache linear scan to improve hit-rate by discovering locally-stable eviction equilibria without the bias of deterministic scan order.\n\n## Parent\n\nbaseline\n\n## Changes\n\nIn the victim-selection loop within the `put()` method (where capacity is full), replace the full-cache scan with random sampling:\n\n1. Instead of iterating through all cached items starting from head, randomly sample K candidate items, where `K = min((capacity / 10).max(5), 30)`. Use a simple random index selection within the nodes vector to draw the sample.\n2. For each sampled item, compute the eviction score `(age as f64) / (frequency + 2.0)` (same formula as baseline).\n3. Select and evict the sampled item with the highest score.\n4. Keep the admission gate unchanged: `incoming_freq > victim_freq`.\n5. Sketch parameters, counter saturation, and all other logic remain identical to baseline.\n\n## Rationale\n\nThe incumbent policy scans all cached items linearly from head (oldest) to tail, which may bias eviction toward items at the head of the LRU list regardless of their frequency-age score. This hypothesis tests whether randomizing the victim candidate pool can discover eviction equilibria that the deterministic scan misses. Randomization may reduce order-dependent biases and allow the frequency-age formula to operate on a less-constrained candidate distribution, potentially unlocking different hit-rate characteristics. This is a fundamentally different algorithm (sampling-based, O(K) instead of O(N)), not a formula tweak or signal addition, and explores whether the incumbent's optimality is dependent on the deterministic head-to-tail scan order or intrinsic to the scoring formula itself.\n"
}
```

### 23:22:04 thinking

Excellent! I've written all three hypotheses. Let me verify they meet the requirements:

1. **H01 - Distinct?** Yes - frequency-percentile filtering (structural change to victim pool)
2. **H02 - Distinct?** Yes - age-bucketed hierarchical eviction (different algorithm)
3. **H03 - Distinct?** Yes - random sampling (completely different approach)

All three are distinct from each other and from prior work. Let me verify they cite the required parents:

1. **H01**: Parent = cycle/0007/v001 ✓ (cross-cycle winner)
2. **H02**: Parent = baseline ✓
3. **H03**: Parent = baseline ✓

At least one cites a cross-cycle winner ✓

Are they testable?
- H01: Yes - can compute 25th percentile and filter victim pool
- H02: Yes - can divide into age cohorts and evict in order
- H03: Yes - can implement random sampling

Are they grounded?
- H01: Yes - grounded in incumbent c0007/v001, motivated by wanting to target cold items without the regression of full tiering
- H02: Yes - grounded in age being orthogonal to frequency (C0005 showed this), motivated by ensuring old items are targeted
- H03: Yes - grounded in question of whether deterministic scan order is optimal vs random sampling

Are the Changes sections specific enough to act on?
- H01: Yes - specifies computing 25th percentile, first/second pass logic, what to keep unchanged
- H02: Yes - specifies age-bucketed division, cohort-order iteration, scoring within cohorts
- H03: Yes - specifies K = min((capacity/10).max(5), 30), sampling process, scoring formula

All good! Let me verify the files are in the right place and format correctly.

### 23:22:04 chat

Perfect! Let me verify the hypotheses are properly formatted and complete:

### 23:22:05 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0008/hypotheses/H01.md"
}
```

### 23:22:05 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0008/hypotheses/H02.md"
}
```

### 23:22:05 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0008/hypotheses/H03.md"
}
```

### 23:22:11 thinking

Perfect! All three hypotheses are properly formatted and meet the requirements:

✓ **H01**
- Claim: "Restrict victim eviction to items in the lowest-frequency cohort to improve hit-rate by ensuring only genuinely cold items are replaced before touching warmer cache residents."
- Parent: cycle/0007/v001 (cross-cycle winner at 0.7429)
- Changes: Clear, specific implementation instructions
- Rationale: Grounded in incumbent c0007/v001, motivated by wanting to filter victim pool like tiering but more conservatively

✓ **H02**
- Claim: "Divide the cache into age cohorts and always evict from the oldest cohort first to improve hit-rate by ensuring that replacement targets genuinely stale items before touching items in active working sets."
- Parent: baseline
- Changes: Specific age-bucketed approach with four cohorts
- Rationale: Grounded in age being orthogonal to frequency

✓ **H03**
- Claim: "Use random sampling of victim candidates instead of full-cache linear scan to improve hit-rate by discovering locally-stable eviction equilibria without the bias of deterministic scan order."
- Parent: baseline
- Changes: Specific K formula and sampling logic
- Rationale: Tests whether deterministic scan order is optimal

All three are:
1. Distinct from each other (frequency-filtering vs age-bucketing vs random-sampling)
2. Distinct from prior work (no tiering regression like H02/c0007, no formula tweaking like c0005, no new signals like c0007/H01)
3. Grounded in the understanding from prior cycles
4. Testable with concrete modifications
5. One cites a cross-cycle winner as required

The task is complete!

### 23:22:18 chat

Excellent! All three hypotheses are now complete and ready. Let me summarize what I've generated:

## Summary of Cycle 0008 Hypotheses

I've created **3 distinct hypotheses** grounded in the deep learning from cycles 0001-0007:

### **H01** (Parent: **cycle/0007/v001** — required cross-cycle winner)
- **Claim**: Restrict victim eviction to the lowest-frequency cohort to ensure only genuinely cold items are replaced before warmer residents.
- **Strategy**: Frequency-percentile filtering—compute the 25th percentile of cached item frequencies and only consider items at/below this threshold for eviction (with fallback to full scan if cohort empties).
- **Grounding**: The incumbent c0007/v001 scores all items globally; this tests whether explicitly partitioning the victim pool can improve targeting without the full-tiering regression seen in c0007/H02.

### **H02** (Parent: **baseline**)
- **Claim**: Divide cache into age cohorts and prioritize eviction from the oldest cohort to ensure stale items are replaced before active working-set items.
- **Strategy**: Age-bucketed hierarchical eviction—split cached items into 4 age quartiles, scan Cohort 1 (oldest) first using the frequency-age formula, only move to Cohort 2 if Cohort 1 is empty.
- **Grounding**: Age and frequency are orthogonal signals (confirmed in c0005). This tests whether age-first ordering improves targeting vs. the incumbent's global score.

### **H03** (Parent: **baseline**)
- **Claim**: Use random sampling of victim candidates instead of full-cache linear scan to discover locally-stable eviction equilibria unconstrained by deterministic scan order.
- **Strategy**: Random sampling—instead of O(N) linear scan from head, sample K=min((capacity/10).max(5), 30) random items and evict the highest-scored sampled item.
- **Grounding**: Tests whether the incumbent's optimality depends on deterministic head-to-tail scan order or is intrinsic to the frequency-age scoring formula itself. A fundamentally different algorithm (sampling-based, not formula tweaking).

### **Quality Checklist**
✓ **Distinct**: Each pursues a different structural strategy (frequency filtering vs. age bucketing vs. randomization)  
✓ **Testable**: All are concrete, implementable changes with measurable outcome (hit-rate)  
✓ **Grounded**: Each is motivated by prior cycle insights (c0007 incumbent, c0005 orthogonality, scan-order bias)  
✓ **Specific**: Changes sections name concrete code modifications; no vague intent  
✓ **Parent coverage**: H01 cites required cross-cycle winner; H02, H03 explore from baseline  

The hypotheses acknowledge that policy-layer tuning has plateaued (per c0007 insight) and pivot toward **structural/algorithmic differences** rather than formula refinement or new signals—avoiding the exhausted improvement axes while staying grounded in the problem space.


---

- finished: 2026-06-05T23:22:18+00:00
- duration: 218.40s
- status: ok
- input_tokens: 116
- output_tokens: 20226
- cache_read_tokens: 578320
- cache_write_tokens: 45899
- cost_usd: 0.217897
