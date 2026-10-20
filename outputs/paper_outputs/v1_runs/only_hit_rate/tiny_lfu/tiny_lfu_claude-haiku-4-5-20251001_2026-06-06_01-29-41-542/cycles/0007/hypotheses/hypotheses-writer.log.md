# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-4-5-20251001)
- workspace: /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0007/hypotheses
- allow_edits: True
- started: 2026-06-05T23:11:41+00:00

## timeline

### 23:11:46 thinking

The user is asking me to generate 3 hypotheses for cycle 0007 of an evolve research loop. Let me break down what I need to do:

1. Read all the context files specified in the "Read for context" section
2. Generate 3 distinct hypotheses that are testable and grounded
3. At least one hypothesis MUST cite a cross-cycle winner as its parent
4. Write them as H01.md, H02.md, H03.md in the output directory

Let me start by reading all the specified context files to understand:
- What happened in cycle 0006
- What the cross-cycle winners are (from cycles 0004 and 0005)
- The overall fitness trajectory across cycles
- The baseline behavior

Files to read:
1. /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0006/cycle-winners.json
2. /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0006/cycle-insight.md
3. /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0006/variants — need to read each v<NNN> variant
4. /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/leaderboard.md
5. /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles-ledger.jsonl


6. /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cross-cycle-insights.md (if present)
7. /home/yaniv/projects/cache-evolve/inputs/subjects/tiny_lfu/baseline (to understand the baseline)

Now I'll systematically read through these files to gather the necessary context.

### 23:11:46 chat

I'll help you generate 3 distinct hypotheses for cycle 0007. Let me start by reading all the context files to understand the fitness landscape and what has been tried.

### 23:11:46 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0006/cycle-winners.json"
}
```

### 23:11:46 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0006/cycle-insight.md"
}
```

### 23:11:46 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/leaderboard.md"
}
```

### 23:11:47 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles-ledger.jsonl"
}
```

### 23:11:47 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cross-cycle-insights.md"
}
```

### 23:11:56 thinking

Excellent. I now have a comprehensive picture of what has happened over 6 cycles:

**Current State:**
- Baseline hit-rate: 0.736
- Cross-cycle winners: cycle/0004/v001 (hit-rate 0.7429) and cycle/0005/v001 (hit-rate 0.7429)
- Latest cycle 0006: v001 (winner) = 0.7414, v002 = 0.7344, v003 = 0.7135

**Key findings from cycles 0001-0006:**

1. **Admission gate** is LOCKED: strict `>` threshold is optimal, gate relaxation has failed multiple times
2. **Eviction victim selection** based on frequency-recency hybrid: age / (frequency + 1) is stable but appears locally optimal
3. **Sketch precision** has saturated: width and depth tuning showed diminishing returns
4. **Orthogonal signals failed**: 
   - Size/frequency integration conflicted (v002 in 0006)
   - Time-based admission lacked grounding (v003 in 0006)
5. **What works**: Simplicity, direct metrics, frequency+age orthogonality, strict selectivity

**Cycle 0006 insight explicitly states:**
"The admission gate and eviction policies are now locked. The strict `>` threshold and age / frequency hybrid form a stable optimum; do not revisit gate relaxation or frequency-recency hybridization in admission without new, properly grounded signals."

**Open questions from 0006:**
- Have we exhausted the admission layer entirely, or are there signals other than frequency (e.g., item request pattern, inter-arrival time, cost-per-byte) that could improve gate discrimination?
- Should size be addressed as a separate optimization (e.g., size-aware replacement cost, item clustering by size), rather than folded into victim scoring?


- Is the cache approaching an asymptotic hit-rate ceiling under this workload with the current policy structure, or does a fundamentally different eviction or admission architecture exist?
- Do the failures in this cycle reflect workload-specific constraints, or are admission-gate strictness and frequency-age eviction universally robust?

Now I need to examine the actual variants from cycle 0006 to see what exactly changed.

### 23:11:56 chat

Now let me read the cycle 0006 variants to understand what was tried:

### 23:11:56 tool: Glob

```json
{
  "path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0006/variants",
  "pattern": "v*/insight.md"
}
```

### 23:11:57 thinking

Hmm, the glob pattern didn't find insight.md files. Let me try a different approach to see what's in the variants directory.

### 23:11:58 tool: Bash

```json
{
  "command": "ls -la /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0006/variants/ | head -20"
}
```

### 23:12:00 thinking

Right, I can't use Bash in this mode. Let me use Glob more carefully.

### 23:12:00 tool: Glob

```json
{
  "path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0006/variants",
  "pattern": "*/attempts/*/overlay"
}
```

### 23:12:03 thinking

Let me try looking more broadly for the variant files:

### 23:12:03 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0006/variants/v001/**"
}
```

### 23:12:05 thinking

Good! Now I can see the files. Let me read the task.md for each variant to understand what was tried, and also the overlay files to see the actual changes.

### 23:12:05 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0006/variants/v001/task.md"
}
```

### 23:12:06 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0006/variants/v001/insight.md"
}
```

### 23:12:06 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0006/variants/v002/**"
}
```

### 23:12:06 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0006/variants/v003/**"
}
```

### 23:12:07 thinking

Good, now let me read the insights for v002 and v003 to understand what they tried:

### 23:12:08 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0006/variants/v002/insight.md"
}
```

### 23:12:08 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0006/variants/v003/insight.md"
}
```

### 23:12:08 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0006/variants/v001/attempts/01/overlay/src/tiny_lfu/mod.rs",
  "limit": 100
}
```

### 23:12:10 thinking

Good, now let me read more of the code to understand the current implementation, particularly the victim selection and admission gate logic:

### 23:12:11 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0006/variants/v001/attempts/01/overlay/src/tiny_lfu/mod.rs",
  "limit": 200,
  "offset": 100
}
```

### 23:13:14 thinking

Perfect. Now I understand the current implementation:

**Current state (Cycle 0006 v001, which is the winner but also the baseline/control):**
- Admission gate: `incoming_freq >= victim_freq` (this was the CHANGE from v001 relaxing to >=)
- Victim scoring: `age / (frequency + 2.0)`
- Sketch: width=16x, depth=4
- The admission gate was relaxed from strict `>` to `>=` but it still hurt (0.7429 → 0.7414)

Wait, let me re-read this. Looking at line 118: `if incoming_freq >= victim_freq {`. This is the >= version. But the insight says this caused regression. So cycle 0006 v001 tested relaxing the gate from `>` to `>=` and it hurt.

So the baseline before this would have had strict `>`.

Now, for cycle 0007, I need to generate 3 new hypotheses. The key constraints:
1. At least one must cite a cross-cycle winner: cycle/0004/v001 or cycle/0005/v001 (both at 0.7429)
2. Cycle 0006 showed all three variants regressed or plateaued
3. The admission gate and victim selection are "locked" per cycle 0006 insight
4. Key open questions:
   - Can admission discrimination improve without relaxing the gate?
   - Are there signals other than frequency (cost, pattern, inter-arrival time)?
   - Should size be addressed orthogonally?
   - Are there different policy frameworks or candidate selection pools worth exploring?

**What hasn't been tried:**
1. **Alternative victim selection heuristics** - instead of age/(frequency+const), could try:
   - Probabilistic/stochastic scoring
   - Rank-based selection (instead of score-based)
   - Two-phase selection (coarse filter + fine filter)
   - Non-linear scoring functions like logarithmic or exponential relationships instead of the current linear division

2. **Sketch-based improvements** - haven't fully explored:
   - Different hash seeding strategies
   - Sketch width/depth interaction
   - Double-hash counting
   - Different counter saturation strategies

3. **Admission gate signals** - could try:
   - Request pattern recognition
   - Inter-arrival time analysis
   - Cost-per-byte considerations
   - Combining with other signals orthogonally (not folded in)

4. **Candidate selection** - what if we changed HOW we select candidates:
   - Not all items (current), but a windowed subset
   - Hot-set biased selection
   - Hierarchical victim selection

5. **Cache warmup/lifecycle** - could try:
   - Probationary period for new items
   - Graduated entry into main cache
   - Time-based tiering

6. **Contention/access pattern analysis** - signals that aren't just frequency:
   - Burstiness of accesses
   - Recency of access vs. frequency weight balance
   - Working set identification

7. **Reuse distance** - measure distance between accesses rather than just frequency

**Hypothesis 1: Parent = cycle/0005/v001 (cross-cycle winner at 0.7429)**
- Claim: Strengthen victim selection by weighting both age and frequency logarithmically to reduce sensitivity to high-frequency outliers while preserving age discrimination, targeting hit-rate on workloads with skewed frequency distributions.
- Changes: Replace `age / (frequency + 2.0)` with `log(age + 1) / (log(frequency + 2))` to apply logarithmic damping symmetrically to both terms.

But cycles 0004 and 0005 already explored parameter tweaks to the victim scoring formula—sqrt-dampening, different offsets, normalization—without meaningful gains. The cycle 0005 results explicitly noted that fine-tuning the frequency-recency surface hits diminishing returns.

I should pursue something orthogonal instead. Rather than adjusting the scoring formula again, focus on the admission gate itself. The idea: add per-slot recency tracking to distinguish between old and recent cache misses, improving selectivity against repeated cold accesses.

Actually, that approach doesn't translate well to a hash-based cache structure.

Looking back at the open questions—whether we've exhausted victim-selection metrics, whether other levers like admission strictness or sketch parameters are now the constraint, whether orthogonal signals like item size or reuse distance could unlock gains—the core issue is clear: the current policy is stuck in a local optimum. Breaking out requires either fundamentally different scoring approaches, properly integrated orthogonal signals, or a different cache policy structure altogether.

Several grounded signals emerge as candidates: reuse distance (tracking distinct items between repeated accesses to the same item), access burstiness (whether accesses cluster or spread temporally), eviction pressure (varying load across cache regions), and hash collision frequency (weighting items by collision rates).

Three distinct hypotheses take shape. The first proposes incorporating item reuse distance as a secondary discriminator in victim selection—the idea being that cold items spanning large access gaps would be better identified, capturing working-set boundaries that frequency alone misses and improving hit-rate. This would require tracking the number of distinct items accessed between consecutive accesses to the same item.

The second hypothesis explores a two-phase victim selection approach: use frequency as a coarse filter to identify items below the 25th percentile coldness threshold, then select the oldest among those cold items. This separates candidate filtering from final ranking, potentially reducing search time and stabilizing victim selection under high contention.

The third hypothesis decouples size optimization from victim selection by introducing an admissions-time feasibility check. Rather than penalizing size during eviction scoring, reject incoming items at admission time if their size would exceed a fair allocation based on the target hit-rate. This tests whether size constraints belong at admission rather than in the eviction decision itself.

I need to ensure at least one hypothesis directly builds on the cross-cycle winner from 0005/v001. Looking at what's already been explored—sketch tuning, victim scoring variations, gate relaxation, size integration in scoring, time-based admission gates, and counter saturation recycling—I should focus on approaches not yet attempted. Two-phase victim selection, reuse distance tracking, hot-set probability weighting, contention-aware selection, sketch quality improvements through hash seeding, or restricting candidate pools offer genuinely distinct directions.

For the incumbent hypothesis, I'm anchoring on frequency distribution shape weighting. Rather than treating frequency uniformly in victim scoring, I'd dynamically adjust the denominator based on whether the sketch estimates show clustered or dispersed frequency values. This captures the insight that when frequencies are tightly grouped, age becomes more discriminative, but when they're spread across a wide range, frequency differences matter more. I'd track running statistics and update this factor periodically to adapt to workload shifts.

The second hypothesis branches from baseline and explores progressive victim selection that iterates through candidates with increasing selectivity. avoiding mixing optimization goals. When the cache exceeds 80% capacity, classify items as large or small relative to the median item size, then prioritize evicting from whichever tier contains more items. Within that tier, apply frequency-age scoring as normal. This keeps size as a secondary signal that guides which candidates to examine first, rather than competing directly with frequency in the scoring function itself.

H01 uses variance-weighted scoring that adapts dynamically based on the shape of the frequency distribution—updating every quarter-capacity worth of accesses. This differs from prior parameter-tuning attempts because it's data-driven rather than manually adjusted. H02 restructures the selection algorithm around frequency cohorts, examining the coldest tier first then ascending by frequency, which represents a more fundamental algorithmic shift than just changing the scoring formula. H03 integrates size orthogonally by using tier selection to guide the victim pool rather than embedding size directly into the scoring calculation, addressing the conflicts that emerged in the previous size-aware attempt.

But I'm realizing H03 still carries the same risk—size conflicted with frequency preservation before, and reframing the integration approach might not resolve that underlying tension. Instead, I should explore a signal that hasn't been thoroughly tested yet. What if I track the inter-arrival time between cache misses for incoming items and reject those with arrival gaps exceeding the working-set average, reducing repeated cold-miss thrashing?

The mechanism would be: at admission, compute the gap between current time and the last miss for that key, then add a secondary gate rejecting items where incoming frequency exceeds victim frequency AND the miss gap is abnormally large. This preserves frequency selectivity while filtering based on temporal spacing.

Actually, this still modifies admission logic, which has been heavily explored. The cycle 0006 feedback was clearer—I should either test fundamentally different signals like cost, pattern clustering, or access burstiness, or expand the test matrix across different workloads and cache configurations to see if the plateau is inherent or specific to the current setup.

Pattern clustering and access burstiness stand out as promising signals. Burstiness specifically—whether an item gets hit multiple times in rapid succession—could be a stronger indicator of temporal locality than just frequency or recency alone. Let me explore integrating burstiness into the victim selection logic instead.

**H01: Parent = cycle/0005/v001 (REQUIRED INCUMBENT)**

The idea is to track how clustered accesses are for each cached item and use that to adjust eviction priority. I'll compute a burstiness score as the ratio of recent accesses (within the last 10 steps) to total accesses, then modify the victim scoring formula to `score = (age / burstiness_score) / (frequency + 2.0)`. Items showing concentrated recent activity get lower eviction scores, making the metric responsive to temporal access patterns rather than just raw frequency.

**H02: Parent = baseline**

An alternative approach: organize candidates by frequency tiers before ranking by age. Split items into four frequency buckets (0-25th, 25-50th, 50-75th, 75-100th percentile), always evict from the coldest tier first, and only move to the next tier if that one empties. This stabilizes decisions under skewed workloads by decoupling frequency estimation noise from the actual eviction choice.

**H03 (Alternative): Parent = baseline**

Claim: Improve sketch accuracy by varying hash seeding or expanding the candidate pool examined during victim selection, reducing the variance in frequency estimates that propagates into eviction decisions.

Changes: Instead of relying on a single hash function for the frequency sketch, use multiple independent hash seeds and aggregate their estimates (e.g., median or mean). Or, when selecting a victim, examine a larger random sample of cached items rather than just the first N encountered, giving the algorithm more signal to distinguish truly cold items from those with noisy frequency counts. 25 when max-min gap is significant ✓
- Risk: Similar admission-gate tweaking has been explored, but this sharpens the frequency signal rather than relaxing the gate itself. Cycles 0001-0002 showed sketch precision improvements were effective, so this direction seems promising.

The approach feels sound—it leverages the insight that measurement precision matters while keeping the strict gate intact, which Cycle 0006 validated. H01 and H02 are sufficiently distinct: one introduces access clustering as a new victim-scoring signal, the other restructures selection from scoring-based to tier-based. Ready to formalize these hypotheses.

Looking back at the sketch exploration history, width adjustments (4→8→16) happened in Cycles 0001-0002, but collision-detection and estimate refinement at read-time haven't been systematically tested. H03 around collision-aware estimation could be worth pursuing.

For H01, I'm extending the `Node` struct with `recent_access_count` tracking accesses within a sliding window (W = capacity/10), then computing burstiness as the ratio of recent to total accesses. The victim score shifts from `age / (frequency + 2.0)` to `(age * (1 + burstiness)) / (frequency + 2.0)`, so items with clustered recent activity get lower eviction priority. Resetting the recent counter every W operations captures temporal clustering as a signal independent of raw frequency magnitude.

For H02, I'm moving away from continuous score-based selection toward frequency-tier bucketing to reduce sensitivity to estimation noise and lower hit-rate variance. Instead of scoring all items, I partition the cache into four frequency cohorts (0-25th, 25-50th, 50-75th, 75-100th percentile) and always evict the oldest item from the coldest occupied tier, only moving to the next tier once the current one is empty. This makes frequency-coldness the dominant criterion with age as a strict tiebreaker within each cohort.

For H03, I'm refining the Count-Min sketch's frequency estimates by detecting and correcting hash-collision patterns to sharpen the frequency signal without loosening the admission threshold. When estimating incoming frequency, I compute the coefficient of variation across the sketch rows—if it exceeds 0.15, indicating likely collision spread, I use the 25th percentile estimate instead of the minimum. The admission gate stays strict (`incoming_freq > victim_freq`), but the frequency estimate itself becomes more robust. The sketch structure and width remain unchanged.

All three approaches satisfy the quality bar: H01 adds a burstiness signal, H02 restructures the algorithm into tiers, and H03 refines frequency estimation through collision detection. Each is measurable (hit-rate with and without the respective change), grounded in concrete parameters (burstiness window W=capacity/10, four-tier histogram with percentile thresholds, cv threshold of 0.15), and specific enough to implement (field names, calculation formulas, escalation rules, estimate selection logic).

H01 references cycle/0005/v001, a cross-cycle incumbent at 0.7429, and all three diverge from recent cycles—burstiness weighting hasn't been attempted, the tier-based structure is new, and the collision-aware refinement is a novel read-time adjustment. Ready to format these as files.

The parent reference format follows `cycle/<NNNN>/v<NN>` for past variants, so cycle/0005/v001 fits the pattern. Now I'll write out the three hypothesis files.

### 23:13:14 chat

Now let me read the baseline to understand the current implementation starting point:

### 23:13:14 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/tiny_lfu/baseline/src/tiny_lfu/mod.rs",
  "limit": 150
}
```

### 23:13:16 thinking

Let me find the baseline directory:

### 23:13:16 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/inputs/subjects/tiny_lfu/**/*.rs"
}
```

### 23:13:18 thinking

Good, I found the baseline. Now let me read it:

### 23:13:18 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/tiny_lfu/src/tiny_lfu/mod.rs",
  "limit": 150
}
```

### 23:13:27 thinking

Good. Now I understand the baseline state. The baseline is quite simple:
- SKETCH_WIDTH_FACTOR = 4 (not the 16 from later cycles)
- No time-tracking in nodes
- No victim scoring - just uses the LRU head (oldest) as victim
- Simple admission gate: `incoming_freq > victim_freq`
- victim_idx is always the head (oldest)

Now, let me synthesize my three hypotheses based on everything I've learned. Let me refine them based on the current understanding:

The cross-cycle winners (0004/v001 and 0005/v001) both achieved 0.7429 hit-rate. Cycle 0006 showed regressions with all three variants. The current understanding is:
1. Admission gate strictness is locked (> is better than >=)
2. Victim selection frequency-age hybrid is locally optimal
3. Sketch parameters have been tuned
4. What might work: Different signals, different algorithms, different policy structures

My three hypotheses:

**H01: Parent = cycle/0005/v001 (cross-cycle winner)**
- Something new not tried: incorporate access burstiness (clustering of accesses)
- Claim: Enhance victim selection by incorporating access burstiness, improving hit-rate by recognizing clustered access patterns as stronger temporal signals
- Changes: Track recent accesses, compute burstiness score, modify victim scoring formula

**H02: Parent = baseline**
- Something different: change from score-based to tier-based victim selection
- Claim: Replace continuous victim scoring with frequency-tier-based selection to stabilize eviction decisions
- Changes: Group items by frequency percentile tiers, always select from coldest tier first

**H03: Parent = baseline**
- Something orthogonal: improve the frequency estimate quality at read time
- Claim: Improve admission decisions by detecting and correcting for hash-collision patterns in sketch estimates
- Changes: Use coefficient of variation to detect collision noise, use percentile estimate instead of minimum


- Each approach targets a different mechanism: H01 refines temporal signals, H02 stabilizes selection logic, H03 improves frequency accuracy. They're sufficiently orthogonal to avoid the issues that caused Cycle 0006 regressions.

### 23:13:30 chat

Perfect. Now I have a comprehensive understanding of the fitness landscape. Let me synthesize the three hypotheses:

**Key insights from cycles 0001-0006:**
- Cross-cycle winners: cycle/0004/v001 and cycle/0005/v001 both at 0.7429 hit-rate
- Cycle 0006 confirmed the admission gate (strict `>`) and victim-selection formula (age/(frequency+2.0)) are locally optimal
- Sketch parameters have saturated; further tuning yields no gains
- What might break the plateau: fundamentally different signals, different algorithm structures, or better frequency estimation

**Three distinct strategies:**

1. **H01 (incumbent-based)**: Incorporate access burstiness (temporal clustering) as a new signal—never tried before, grounded in the insight that rapid repeated accesses indicate stronger working-set membership than sparse old accesses

2. **H02 (algorithm structure)**: Replace score-based victim selection with frequency-cohort-based selection—changes the algorithm from "find max score globally" to "prefer coldest frequency tier first"—a structural change rather than formula tuning

3. **H03 (frequency precision)**: Improve sketch estimate quality at admission time by detecting collision patterns—orthogonal to the gate strictness itself, only sharpening the frequency signal

Now I'll write these as markdown files:

### 23:13:35 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0007/hypotheses/H01.md",
  "content": "# H01\n\nEnhance victim selection by incorporating access burstiness alongside frequency and age, improving hit-rate by recognizing clustered access patterns as stronger temporal locality signals than sparse old accesses.\n\n## Parent\n\ncycle/0005/v001\n\n## Changes\n\nExtend the `Node` struct to track `burstiness_window` (number of accesses to this item within the last W steps, where W = capacity / 10) and `total_accesses_ever` (cumulative count of all accesses). At each access via `get()`, increment both counters; reset `burstiness_window` every W global access steps.\n\nIn the victim-selection loop (the scan through cached items in `put()`), compute a burstiness factor for each candidate: `burstiness_factor = 1.0 + (burstiness_window as f64 / total_accesses_ever.max(1) as f64)`. Modify the victim scoring formula from `score = age / (frequency + 2.0)` to `score = (age * burstiness_factor) / (frequency + 2.0)`. Items with recent clustered accesses (high burstiness_factor) receive proportionally lower eviction scores, preserving the current threshold values while weighting against recently bursty access patterns.\n\nKeep all other logic (admission gate threshold, sketch parameters, counter saturation) unchanged.\n\n## Rationale\n\nAccess burstiness captures temporal clustering—when an item is accessed repeatedly in a short window, it signals immediate working-set relevance independent of absolute frequency count. The formula weighs age higher for bursty items (lower eviction score), recognizing that temporal proximity is a distinct signal from long-term frequency. This is grounded in the pattern from prior cycles that frequency and age are orthogonal signals; burstiness adds a third orthogonal dimension (access clustering) that frequency and age do not capture alone.\n"
}
```

### 23:13:37 thinking

Good, H01 is written. Now H02:

### 23:13:42 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0007/hypotheses/H02.md",
  "content": "# H02\n\nReplace continuous victim-scoring with frequency-tier-based selection to improve robustness under frequency-estimation noise, improving hit-rate by always preferring victims from the coldest frequency cohort before considering age.\n\n## Parent\n\nbaseline\n\n## Changes\n\nIn the victim-selection loop within the `put()` method (where a victim must be chosen from full cache), implement a four-tier frequency histogram instead of computing a single maximum-score victim across all items.\n\nFirst, scan all cached items and collect their frequency estimates from the sketch. Compute the 25th, 50th, and 75th percentiles of these frequencies. Create four frequency tiers:\n- Tier 1 (coldest): frequency <= 25th percentile\n- Tier 2: frequency <= 50th percentile\n- Tier 3: frequency <= 75th percentile\n- Tier 4 (hottest): frequency > 75th percentile\n\nWhen selecting a victim, iterate through tiers in order (Tier 1 first). For the first non-empty tier, find the oldest item (by position in the LRU list) and select it as the victim. Do not compute the complex frequency-age score formula; instead use frequency-coldness as the primary selection criterion with age as the strict secondary discriminator within the tier.\n\nKeep the admission gate (`incoming_freq > victim_freq`) unchanged. Sketch parameters, counter saturation, and all other logic remain the same.\n\n## Rationale\n\nCycles 0004–0006 showed that fine-tuning the continuous scoring formula (adjusting exponents, constants, normalization) yields diminishing returns. The victim-selection metric appears locally optimal, but the underlying problem may be that when frequency estimates are noisy (from hash collisions), a single global score can be misled by outliers. Tiering separates the decision into two distinct phases: first, identify which frequency cohort contains victims (frequency matters absolutely), then within that cohort, use age for ranking. This structure is more robust to estimation noise because it doesn't depend on precise score magnitudes—only on which tier a candidate falls into. The approach recognizes that victims should come from genuinely cold items, with age as a tiebreaker, rather than computing a complex formula across all items.\n"
}
```

### 23:13:44 thinking

Good, H02 is written. Now H03:

### 23:13:50 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0007/hypotheses/H03.md",
  "content": "# H03\n\nImprove admission-gate frequency estimates by detecting and correcting for hash-collision patterns in the Count-Min sketch, improving hit-rate by sharpening the frequency signal without relaxing the strict admissions threshold.\n\n## Parent\n\nbaseline\n\n## Changes\n\nIn the admission logic within the `put()` method (where the incoming frequency is estimated), modify the frequency estimation step to detect collision patterns and apply a corrective adjustment.\n\nAfter computing `incoming_freq = self.sketch.estimate(&key)` using the current approach (minimum across all depth rows), collect all individual estimates from each sketch row into a vector: `let all_estimates: Vec<u32> = (0..self.depth).map(|row| self.counts[row][pos]).collect()`. \n\nCompute the coefficient of variation (CV) of these estimates: `cv = stddev(all_estimates) / mean(all_estimates)`. If `cv > 0.15`, indicating likely collision spread across rows, use the 25th percentile of `all_estimates` instead of the minimum. Otherwise, use the minimum as before.\n\nReplace the use of `incoming_freq` in the admission gate comparison `if incoming_freq > victim_freq {` with this collision-aware estimate. The strict admission gate threshold (`>`) and all other victim-selection logic remain unchanged. Sketch width, depth, and saturation handling are unmodified.\n\n## Rationale\n\nCycle 0001 showed that sketch precision improvements (width tuning) were productive early on; Cycles 0001–0002 gained +0.0047 from width doubling. However, Cycle 0006 confirmed that further tuning sketch parameters has saturated. This hypothesis takes a different angle: instead of changing the sketch structure, improve the estimate quality at read-time by detecting collision patterns. When the coefficient of variation is high, it signals that different hash functions are returning very different counts, indicating collision noise. In such cases, using the 25th percentile (more robust than minimum to outlier collisions while more conservative than mean) provides a better estimate without changing the sketch itself. This keeps the strict admission gate's selectivity intact while sharpening the frequency signal it uses for discrimination, potentially unlocking gains that were blocked by noisy estimates.\n"
}
```

### 23:13:58 thinking

Perfect! I've written all three hypotheses. Let me verify they meet all the requirements:

1. **H01 (burstiness):**
   - Parent: cycle/0005/v001 ✓ (cross-cycle winner)
   - Claim: "Enhance victim selection by incorporating access burstiness..." ✓ (first non-empty non-header line, one sentence)
   - ## Parent section: ✓ present with single identifier
   - ## Changes section: ✓ present with specific implementation details
   - Distinct: Yes (burstiness is new signal, never tried before)
   - Testable: Yes (can measure hit-rate with and without)
   - Grounded: Yes (specific about window size, counters, scoring formula)

2. **H02 (frequency tiers):**
   - Parent: baseline ✓
   - Claim: "Replace continuous victim-scoring..." ✓
   - ## Parent section: ✓
   - ## Changes section: ✓ specific about percentile tiers, tier selection algorithm
   - Distinct: Yes (tier-based algorithm is structurally different from all prior work)
   - Testable: Yes (can measure hit-rate)
   - Grounded: Yes (specific about 25/50/75 percentiles, tier iteration order)

3. **H03 (collision detection):**
   - Parent: baseline ✓
   - Claim: "Improve admission-gate frequency estimates..." ✓
   - ## Parent section: ✓
   - ## Changes section: ✓ specific about CV calculation, percentile_25, threshold 0.15
   - Distinct: Yes (collision-aware estimation is new approach)
   - Testable: Yes (can measure hit-rate)
   - Grounded: Yes (specific about CV formula, when to trigger, which estimate to use)
   
These three approaches diverge meaningfully—H01 introduces a temporal signal, H02 restructures the scoring mechanism entirely, and H03 adds collision awareness to frequency estimation. H01 leverages the cross-cycle winner, while all three are grounded in concrete implementation details and avoid vague language.

### 23:14:03 chat

Perfect! I've created three distinct hypotheses for cycle 0007:

## Summary

**H01: Access Burstiness Integration** (Parent: cycle/0005/v001 — cross-cycle winner at 0.7429)
- Adds a new orthogonal signal (temporal clustering of accesses) to victim selection, complementing frequency and age
- Specific: Track burstiness within a rolling window (W = capacity/10), weight age inversely by burstiness in the scoring formula
- Rationale: Recognizes that rapid repeated accesses indicate immediate working-set relevance—a signal frequency count alone doesn't capture

**H02: Frequency-Tier Victim Selection** (Parent: baseline)
- Changes the algorithm structure from continuous scoring to frequency-cohort-based selection
- Specific: Build a 4-tier histogram (25th, 50th, 75th percentiles), always prefer victims from coldest tier first, use age as secondary discriminator within tiers
- Rationale: More robust to frequency-estimation noise; decouples the decision into "which cohort has victims" (frequency-absolute) vs "which one within the cohort" (age-based), avoiding precision pitfalls that plague global score computation

**H03: Collision-Aware Sketch Estimates** (Parent: baseline)
- Improves the quality of frequency estimates at read-time by detecting hash-collision patterns
- Specific: When admission-gate estimates are computed, measure coefficient of variation across sketch rows; if CV > 0.15, use 25th percentile instead of minimum
- Rationale: Cycle 0001 showed sketch precision improvements were productive; this takes a different angle—detect collision noise at read-time and use a more robust estimate without changing the sketch itself or relaxing the strict gate

All three are **grounded and specific enough for implementation**, **distinct from each other and from prior cycles**, and **address the open questions** from Cycle 0006 about exploring fundamentally different signals and algorithm structures.

The files are written to `/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0007/hypotheses/` as H01.md, H02.md, and H03.md.


---

- finished: 2026-06-05T23:14:03+00:00
- duration: 142.28s
- status: ok
- input_tokens: 1473
- output_tokens: 12738
- cache_read_tokens: 454810
- cache_write_tokens: 32259
- cost_usd: 0.15242
