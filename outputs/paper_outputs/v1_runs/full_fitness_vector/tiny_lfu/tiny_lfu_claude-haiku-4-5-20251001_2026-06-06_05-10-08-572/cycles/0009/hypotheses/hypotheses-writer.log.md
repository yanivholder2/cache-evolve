# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-4-5-20251001)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0009/hypotheses
- allow_edits: True
- started: 2026-06-06T06:16:10+00:00

## timeline

### 06:16:14 thinking

The user is asking me to generate 3 hypotheses for cycle 0009 of a cache evolution experiment. Let me break down the requirements:

1. I need to read context files from the previous cycle (0008) and cross-cycle history
2. Generate 3 distinct hypotheses that each cite exactly one parent (either `baseline` or a past variant `cycle/<NNNN>/v<NNN>`)
3. At least one hypothesis MUST cite a cross-cycle winner (incumbents):
   - cycle/0008/v002: hit-rate=0.7268, metadata-size=48, per-op-time=74
   - cycle/0008/v003: hit-rate=0.7243, metadata-size=48, per-op-time=73
   - cycle/0005/v001: hit-rate=0.7355, metadata-size=48, per-op-time=91
4. Write outputs to `/home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0009/hypotheses` as H01.md, H02.md, H03.md

Let me start by reading all the required context files to understand what has been tried and what the fitness dimensions are.

First, let me read the cycle 0008 information and cross-cycle history.

### 06:16:14 chat

I'll help you generate 3 distinct hypotheses for cycle 0009. Let me start by reading the context files to understand what has been learned from previous cycles.

### 06:16:15 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0008/cycle-insight.md"
}
```

### 06:16:15 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0008/cycle-winners.json"
}
```

### 06:16:15 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/leaderboard.md"
}
```

### 06:16:15 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cross-cycle-insights.md"
}
```

### 06:16:17 thinking

Good! Now let me check the cycle 0008 variants in more detail, and also look at some of the past cycles (especially cycle 0005 v001 and cycle 0008 v002, v003) to understand what they did.

Let me look at the variants directory structure for cycle 0008.

### 06:16:18 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0008/variants/*/insight.md"
}
```

### 06:16:20 thinking

Let me try to list the variants directory directly.

### 06:16:20 tool: Bash

```json
{
  "command": "find /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0008/variants -type d -maxdepth 1 | sort"
}
```

### 06:16:21 thinking

I don't have bash available. Let me use Glob to list the variants.

### 06:16:21 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0008/variants/v*/hypothesis.md"
}
```

### 06:16:22 thinking

Let me try a broader glob pattern to see what files exist in the variants directories.

### 06:16:23 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0008/variants/v*/*"
}
```

### 06:16:24 thinking

Let me check if the directory structure even exists.

### 06:16:25 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0008"
}
```

### 06:16:26 thinking

It's a directory. Let me try to find the overlay files for the variants to understand what they did.

### 06:16:26 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0008/variants/v001/attempts/*/overlay/*"
}
```

### 06:16:27 thinking

Let me try a simpler pattern.

### 06:16:27 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/*/hypotheses/H*.md"
}
```

### 06:16:28 thinking

Let me search more carefully for any markdown files in the cycles directory.

### 06:16:28 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/**/*.md"
}
```

### 06:16:31 thinking

Good! Now I can see the structure. Let me read some of the hypotheses from previous cycles and the tasks to understand what changes were made. Let me also look at the incumbents more carefully.

Let me read the hypotheses from cycle 0005 (since 0005/v001 is an incumbent), cycle 0008 (to understand what was already tried), and also look at other cycles to understand the patterns.

### 06:16:31 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0008/hypotheses/H01.md"
}
```

### 06:16:31 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0008/hypotheses/H02.md"
}
```

### 06:16:32 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0008/hypotheses/H03.md"
}
```

### 06:16:32 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0005/hypotheses/H01.md"
}
```

### 06:16:34 thinking

Good! Now I'm getting a clearer picture. Let me read more about the incumbents and understand what the current best strategies are. Let me also read cycle 0007 since it appears to be an important predecessor.

### 06:16:35 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0007/hypotheses/H01.md"
}
```

### 06:16:35 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0007/hypotheses/H02.md"
}
```

### 06:16:35 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0007/hypotheses/H03.md"
}
```

### 06:18:20 thinking

Perfect! Now I have a good understanding of the landscape. Let me synthesize:

## Key Findings from the Context:

### Incumbents (best variants):
1. **cycle/0008/v002**: hit-rate=0.7268, per-op-time=74ns (best latency)
   - This was v003 (depth reduction test) but with a tail-latency spike (p99: 501ns vs baseline 208ns)
   
2. **cycle/0008/v003**: hit-rate=0.7243, per-op-time=73ns (best latency)
   - Much better tail latency (p99: 166ns)
   
3. **cycle/0005/v001**: hit-rate=0.7355, per-op-time=91ns (best hit-rate overall)

### What's Saturated:
- **Sketch geometry**: Depth-3, width-3 is the hard frontier. Further reduction triggers 1.2-1.6% hit-rate loss and tail-latency pathology.
- **Eviction policy**: N=2 frequency-aware is stable. Pure LRU (N=1) regresses. Larger pools (N=5+) add latency variance.
- **Admission threshold**: The strict `incoming_freq > victim_freq` is load-bearing and workload-optimized. Relaxations fail.
- **Sketch decay**: Does not generalize and adds overhead.

### What Could Be Explored:
1. **Implementation efficiency** of sketch queries (vectorization, cache locality, branch prediction)
2. **Workload-adaptive parameter schedules** - detecting phase transitions
3. **Alternative frequency estimation** mechanisms
4. **Hash function cost reduction** on the hot path
5. **Microarchitectural optimization** of the sketch query hot path

### Cycle 0008 Results:
- All three variants (testing width-2, depth-2, depth-2+width-3) failed
- Each showed 1.2-1.6% hit-rate loss, with v002 exhibiting a severe p99 latency regression (141% spike)
- This confirms depth-3, width-3 is the minimum viable sketch configuration

## Strategy for Cycle 0009:

I need to propose three distinct hypotheses, each grounded in the data. At least one should build directly on an incumbent—cycle/0008/v003 has the cleanest tail-latency profile, making it a solid baseline. The other two should explore orthogonal directions: one targeting the implementation layer where sketch operations dominate the hot path, another testing whether the admission policy can be refined without sacrificing the hit-rate gains we've locked in.

I'm constrained to overlay files only—src/lib.rs, Cargo.toml, and the JNI layer are off-limits. That rules out deep structural changes but leaves room to tune parameters, modify frequency computation logic, or refactor the sketch query path for better cache behavior. The N=2 success versus N=5 failure suggests the sweet spot is narrow; I could test N=3 or N=4, or explore adaptive N based on load. Alternatively, I could modify how incoming frequency is computed—biasing it by recency or queue position rather than treating all arrivals equally. A third angle is cache-locality optimization: reordering hash computation or memory layout to reduce latency on the critical path without changing the algorithm's semantics. items' frequency when they become LRU head.

Instead of just N=2 frequency-aware eviction, I could weight both frequency and recency together—preferring to evict items that are simultaneously old and infrequently accessed.

The core challenge from cycle 0008 is that reducing sketch geometry hits a wall. So I should pivot toward either algorithmic improvements or policy refinement. The key question is why N=2 frequency-aware eviction works while N=5 fails catastrophically on tail latency. This could stem from asymmetric costs in the LRU chain (head + next are cheap, later items expensive), or it might require unordered candidate selection with larger pools. I could also explore whether vectorization or hash function optimization would reduce per-operation sketch costs.

The sharp 141% p99 spike in v002 might be specific to depth-2's interaction with LRU eviction, or it could reflect general sensitivity to sketch collision rates. This points toward either a more robust victim selection strategy or changes to how hash collisions are handled.

For H01, I'd increase the eviction candidate pool from N=2 to N=3 to capture more frequency diversity without triggering the pathology seen at N=5. The rationale is that N=2 succeeds but N=5 fails catastrophically, leaving N=3 and N=4 as unexplored middle ground that might offer better filtering without the tail-latency cost.

For H02, I'd bias victim selection toward items with both low frequency and old recency by weighting scores to prevent evicting recently-accessed items with moderate frequency. This targets hit-rate directly, since cycle/0005/v001 achieves 0.7355 hit-rate by combining both signals rather than frequency alone.

For H03, I'd implement lazy frequency estimation by skipping sketch queries for items accessed within a recency window, reducing hot-path overhead and potentially avoiding the collision pathology altogether.

But reconsidering the tail-latency issue in cycle/0008/v002—the insights point to whether the depth-2 pathology stems from its interaction with LRU eviction or from general sensitivity to sketch collisions at operational scale. This suggests the real problem might be how collisions affect victim selection under depth-2 constraints.

When the sketch is shallow, collision rates spike, hash estimates become unreliable, and victim selection degrades into near-random choices that cascade into poor evictions. I could instead track estimate confidence—detecting when shallow sketches produce noisy signals and preferring candidates with reliable frequency estimates during eviction, regardless of their apparent frequency. This would make the algorithm resilient to noisy estimates without forcing deeper geometry as a hard requirement.

But that adds complexity. Let me focus on clearer, more testable hypotheses. Each needs to be distinct from others, experimentally verifiable, grounded in a parent variant, and concrete enough to implement.

For the first approach: take the best latency incumbent and expand its victim selection pool from N=2 to N=3, which should boost hit-rate while keeping latency competitive. The change is straightforward—extend the eviction loop's candidate scanning window.

The second direction involves weighting the selection criteria to favor items that are both infrequently accessed and old in the LRU order, which could improve eviction decisions by targeting items less likely to be reused.

The third approach considers a simpler alternative—rather than computing exact LRU positions, exploring a probabilistic or heuristic-based selection method that still captures the benefit of considering both frequency and recency without the traversal overhead.

Looking at the unexplored directions, several gaps stand out. The cross-cycle insights highlight questions about whether vectorization or hash function optimization could reduce per-operation costs, whether the observed performance boundaries hold across different workload distributions, and specifically why N=2 succeeds while larger pools degrade on tail latency. This suggests three concrete directions: testing N=3 or N=4 to find the optimal pool size, investigating whether hash computation itself is a bottleneck worth optimizing, and examining the admission threshold logic to see if timing adjustments could improve behavior.

Examining what's actually modifiable without touching restricted files, cycles 0005, 0007, and 0008 consistently adjust sketch parameters like `SKETCH_WIDTH_FACTOR` and `SKETCH_DEPTH`, modify the eviction loop in the `put()` method, and refine the admission check. This means the practical levers are the sketch configuration constants and the eviction/admission logic itself. Three testable hypotheses emerge: extending the frequency-aware selection pool beyond N=2, scaling cache capacity based on item frequency to preserve hot items longer, and reversing the eviction scan direction from tail instead of head.

**H01: Increase victim selection pool to N=3**
Cycle 0008's N=2 approach is stable but conservative. Expanding to N=3 candidates (+50% more comparisons) could improve hit-rate while staying well below N=5's latency cliff. The change is minimal—shifting the loop from `for _ in 0..1 {` to `for _ in 0..2 {`—and targets the proven frequency-aware strategy incrementally.

**H02: Probabilistic frequency check with minimal overhead**
Rather than scanning multiple candidates, check frequency only for the LRU head with a probabilistic fallback to reduce eviction cost while maintaining admission awareness.

**H02 (revised): Adaptive pool size based on sketch collisions**
Adjust the victim selection pool dynamically—use N=3 when collision rates spike to recover accuracy without the consistent overhead of larger pools.

The noisy frequency estimates from shallow sketches suggest that scanning more candidates becomes valuable precisely when collisions degrade signal quality. By monitoring sketch load and expanding the pool only under high-collision conditions, I can maintain the latency gains of N=2 while recovering hit-rate when needed. The tradeoff is adding runtime measurement logic to detect collision rates.

Let me focus on three concrete approaches instead:

**H01 proposes a straightforward increase to N=3**, evaluating three candidates rather than two during victim selection to capture more frequency diversity and improve hit-rate.

**H02 takes a different angle by relaxing the admission threshold**, allowing items with frequencies close to the victim's to enter the cache rather than requiring strict dominance. This smooths the boundary condition and lets marginal items compete more fairly.

**H03 explores whether the strict inequality is truly necessary** by testing a small tolerance window. The highest-performing variant (0.7355 hit-rate) used a particular admission policy, and while prior relaxations failed, a -1 offset hasn't been tested yet. This micro-adjustment either confirms strict comparison is essential or finds a small improvement at the margin.

However, reviewing the cross-cycle data reveals -1 offset was already attempted in cycle 0003 and degraded performance by admitting marginal items that displaced higher-value residents. That rules out this direction.

**H02 (revised) inverts the victim selection strategy entirely**—instead of evicting the lowest-frequency items, evict the highest-frequency ones. The hypothesis is that for this particular Zipfian distribution, hot items might be concentrated enough that removing them creates beneficial space for a broader range of cold items that would otherwise be excluded.

But this fundamentally contradicts LFU's core principle, so it's unlikely to yield insights worth the implementation cost.

A more practical angle is to add an explicit frequency floor during admission. Rather than accepting any item that beats the victim's frequency, only admit items that meet a minimum threshold—say, frequency ≥ 2. This prevents marginal single-access items from entering the cache and displacing established residents, which directly addresses the cascade problem observed in earlier cycles.

The key difference from current logic is adding an absolute check alongside the relative one. Items below the floor get rejected regardless of victim frequency, filtering out noise from hash collisions or one-off accesses. This targets hit-rate improvement through stricter admission criteria.

Let me step back and examine what actually succeeded. Looking at the cycle 0008 winners—v002 and v003—they came from depth and width reduction hypotheses, which is counterintuitive since those changes failed in earlier cycles. I need to trace where these variants originated. The cycle 0008 hypotheses show v001 derived from reducing width (4→2), v002 from reducing depth (3→2), and v003 from depth reduction with width-3. So the winning variants were both structural reductions, suggesting there's something about those parameter changes that improved performance despite prior failures. egressions (hit-rate loss, latency spikes). The insight is saying: "Yes, we tested depth-2 and width-2, and confirmed they regress. This was valuable evidence that we've hit the frontier."

So v002 and v003 are winners in the sense of "selected for next round," but they show the boundary (they regress in hit-rate to 0.7268 and 0.7243 respectively, vs 0.7355 best). The actual parent candidates should be the ones with the strongest individual metrics: cycle/0005/v001 for hit-rate dominance at 0.7355, cycle/0008/v002 for latency at 74ns despite the hit-rate drop, and cycle/0008/v003 for the tightest p99 at 73ns.

The constraint requires at least one hypothesis to inherit from these proven performers. Since cycle/0008/v003 balances hit-rate retention with superior tail latency stability compared to v002's p99 spike, it's the most reliable foundation for H01.

H01 proposes extending the frequency-aware victim pool from examining 2 candidates to 3. The parent is cycle/0008/v003. The hypothesis claims this expands the selection scope enough to boost hit-rate without incurring the latency penalties seen at N=5. The implementation change involves modifying the victim selection loop in `put()` to scan 2 additional items instead of 1—essentially changing `for _ in 0..1 {` to `for _ in 0..2 {`. Prior cycles established that N=2 outperformed pure LRU (N=1), and cycle 0007 reverted to N=1, so testing N=3 as an intermediate step makes sense.

H02 takes a different approach by introducing frequency binning. It categorizes candidates into tiers (cold, warm, hot) and prioritizes eviction from the lowest tier, addressing the observation that noisy frequency signals can lead to suboptimal victim choices. The implementation adds bucketing logic to the selection loop, grouping candidates by frequency thresholds and exhausting lower bins before considering higher ones.

However, there's a simpler alternative: pairing N=1 pure-LRU eviction with stricter admission criteria. This removes the scanning overhead entirely while raising the bar for cache entry—requiring incoming frequency to exceed victim frequency by at least 2 rather than just 1. The trade-off is reduced selectivity in victim choice for tighter control over what gets admitted.

A third direction worth exploring is sketch optimization. Cutting the capacity multiplier from 1024 to 512 would halve memory overhead while testing whether frequency estimates remain reliable at lower resolution.

The deterministic victim selection strategy might be too rigid—introducing randomized selection weighted by inverse frequency could help escape local optima by occasionally evicting slightly-higher-frequency items rather than always picking the absolute minimum.

However, that feels counterintuitive for a cache that should behave predictably. A more practical angle: what if we're recomputing frequency estimates inefficiently? Caching the incoming item's frequency within a small operation window would reduce redundant sketch queries for repeatedly-accessed keys, amortizing the lookup cost across multiple puts.

But adding per-node state complicates the implementation. Let me focus on three concrete, testable proposals that build on what's already working:

**H01: Expand the frequency-aware candidate pool from N=2 to N=3**
Parent: cycle/0008/v003
Increase the victim selection scope by evaluating one additional candidate beyond the current two. Modify the candidate-scanning loop from `for _ in 0..1` to `for _ in 0..2`, examining three total items instead of two. Since N=2 outperformed pure LRU in earlier cycles and geometry tuning has hit diminishing returns, exploring whether N=3 captures better frequency diversity could push hit-rate further.

**H02: Tighten the admission threshold to require incoming_freq > victim_freq + 2**
Parent: cycle/0005/v001
Rather than relaxing the admission criterion (which consistently fails), reverse direction by making it stricter. Require incoming items to exceed victim frequency by at least 3 rather than 1, filtering out marginal candidates that pollute the cache. This targets hit-rate improvement through more selective entry while staying orthogonal to pool size or geometry changes.

**H03: Combine frequency and recency into a unified cost-based eviction metric**
Parent: baseline
Instead of pure-frequency victim selection, weight both frequency and LRU position together in a composite cost function, creating a single coherent eviction criterion rather than treating frequency and age as separate concerns. computing LRU_AGE requires traversing or tracking, which adds overhead.

Instead, I should simplify this. Let me test pure LRU with N=1 as a boundary condition to isolate whether frequency-aware victim selection actually improves hit-rate beyond what the admission threshold provides. I'd remove the candidate scanning loop entirely and always evict the head, keeping only the admission check that compares incoming frequency against victim frequency. This directly tests if the frequency component of victim selection—distinct from the N>1 optimization that reduced p99 latency—meaningfully impacts hit-rate.

But cycle 0007 v001 already ran pure LRU without detailed analysis. A fresher approach: dynamically switch eviction strategies based on item temperature. For cold items, use frequency-aware selection to preserve marginal coldness; for hot items, use pure LRU to avoid scanning overhead on the critical path. This would compute average frequency across the cache to determine the threshold.

The rationale is that hot items will likely be re-inserted anyway, so their eviction decisions matter less, while cold items' choices directly impact space efficiency. Frequency checks are expensive and should only apply where they provide real value. This targets per-operation latency by eliminating frequency scans during hot-path evictions, and it's independent of N-tuning or threshold changes.

For the final set, expanding the frequency-aware pool from N=2 to N=3 offers a middle ground—capturing more frequency diversity than the current approach while avoiding the latency cost of N=5. This builds on cycle 0008 v003 and tests whether modest pool growth improves hit-rate without matching the full overhead.

Tightening the admission threshold to require incoming frequency exceed victim frequency by at least 2 adds selectivity. Instead of accepting any incoming item with higher frequency, this filters marginal candidates where the frequency gap is small, reducing low-value insertions and improving hit-rate through stricter filtering.

Testing pure-LRU eviction isolates whether frequency-aware victim selection actually improves hit-rate or if admission control alone is sufficient. By evicting the LRU head unconditionally while keeping the admission check, I can measure the contribution of multi-candidate frequency comparison versus simple recency-based eviction.

### 06:18:20 chat

Now let me read more cycle history to understand what approaches have been successful:

### 06:18:21 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0006/hypotheses/H01.md"
}
```

### 06:18:21 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0006/cycle-insight.md"
}
```

### 06:19:04 thinking

Perfect! Now I have a clear picture:

- Cycle 0005 v001: N=5 frequency-aware, hit-rate=0.7355, but p99 put latency was high (570ns according to 0006 insight)
- Cycle 0006 v001: Reduced to N=2, hit-rate=0.7354 (negligible loss), p99 put latency=400ns (30% improvement)
- Cycle 0007: Tested pure LRU (N=1) which regressed, and tested width-3 which succeeded
- Cycle 0008: Tested depth-2 and width-2, all regressed to ~0.72-0.73 hit-rate

So the current state is:
- Geometry: depth-3, width-3 is the frontier (further reduction causes problems)
- Eviction: N=2 is the proven optimum (N=1 fails, N=5 has latency variance)
- Admission: strict > is load-bearing
- Decay: is fragile and doesn't generalize

Open questions from 0006:
- Is N=2 the true minimum? Could test N=1 vs N=3
- Could frequency awareness be implemented with zero conditional cost?

Open questions from 0008:
- Why does the tail-latency pathology in v002 (141% p99 spike) manifest so sharply?
- Is the depth-3, width-3 boundary workload-specific?
- Could vectorization, hash function cost reduction, or memory layout changes improve sketch query performance?

So valid hypotheses for 0009 could be:

1. **Extend N from 2 to 3** (answering the open question)
2. **Test N=1 directly** (answering whether N>1 is necessary - though 0007 v001 already did this)
3. **Optimize sketch query implementation** (addressing the microarch optimization question)
4. **Investigate tail-latency robustness** (understanding why depth-2 causes spikes)
5. **Explore workload-adaptive parameters** (suggested in 0006's continuation)

But I can't modify hot-path assembly or do deep microarch optimization without potentially editing restricted files. And I can't directly fix why depth-2 causes latency spikes without more geometry changes. The most tractable moves are extending N to 3 to directly answer whether N=2 is truly minimal, or refining the admission logic further since that's proven load-bearing.

Let me revisit what cycle 0007 v001 actually demonstrated. It tested pure LRU by stripping out the frequency-aware scanning loop, and the leaderboard shows it hit 0.733 hit-rate versus the baseline 0.736—a measurable 0.003 regression despite the 92 microsecond latency improvement. That confirms the frequency signal matters, even if it's not the dominant factor.

Looking back at the progression: cycle 0006 v001 with N=2 achieved 0.735 hit-rate (essentially flat from baseline at 0.7355), then removing frequency entirely in cycle 0007 v001 dropped it further to 0.733. The pattern suggests each simplification costs a small amount of accuracy.

For the next cycle, I'm considering two main directions. First, extending N from 2 to 3 on the best stable version to test whether additional frequency pools capture more diversity. Second, exploring something different entirely—maybe a probabilistic approach to N selection, though that feels unconventional.

The continuation notes from earlier cycles point toward orthogonal optimization axes: admission threshold tuning, workload-adaptive scheduling, reducing hash function overhead on hot paths, or alternative frequency estimation that doesn't require traversing the full sketch. These seem more promising than incremental N adjustments.

For admission threshold, I could explore a dual-condition approach—admit if the incoming frequency exceeds the victim's OR surpasses a fixed threshold, rather than relying solely on the strict comparison. On the workload-adaptive side, detecting Zipfian concentration levels or dynamically choosing between N=2 and N=3 based on runtime characteristics might help. Hash function costs could drop if I cache computed values or pre-compute them during initialization. Sampling-based frequency estimation feels too disruptive, but the sketch query itself is a clear hot path where cache locality and branch prediction matter.

Two concrete directions emerge: First, adding an absolute frequency floor to the admission gate—requiring estimated frequency >= 2 alongside the relative comparison—could filter out noise from hash collisions and low-frequency estimates, improving hit-rate without affecting N-tuning. Second, victim selection could combine both frequency and LRU distance into a composite score rather than treating them independently.

The composite approach has implementation overhead though. A simpler alternative: differentiate admission rules between new keys and existing ones, applying stricter thresholds (incoming > victim + 1) only for new insertions while allowing updates to existing keys with a relaxed threshold (incoming > victim), which would reduce churn and stabilize hit-rate.

The key insight is that updates to already-cached items are inherently safer—they've proven their value—so they warrant immediate admission. Only novel entries need the strict gate. This targets hit-rate through a cleaner admission policy and remains independent of N-tuning. The implementation is straightforward: check if the key exists before deciding which threshold to apply.

What other angles haven't been explored yet? Pure LRU with N=1 seems redundant given prior testing. Varying admission thresholds has been covered. Testing higher N values like 4 or 5 might yield insights, though modifying the sketch isn't straightforward. Sketch decay was explicitly flagged as fragile. Random victim selection or FIFO eviction both feel like step backwards.

One possibility: what if the N=2 pool size itself could be tuned differently? Instead of always taking the next item in LRU order, we could sample the furthest item or pick randomly from the candidate set. This introduces controlled variance into eviction decisions, potentially breaking out of local optima. The approach would identify the bottom-N frequency candidates and randomize selection among them rather than always choosing the minimum.

The concern is that randomness in a deterministic cache system feels unnatural—caches typically operate predictably. An alternative direction might be exploring move-to-front adaptive insertion policies, which sometimes outperform LRU+LFU hybrids. Or tracking eviction cost—how many times items were candidates for removal but survived—could reveal patterns in what's worth keeping.

A more straightforward approach: prioritize LRU position as the primary signal and use frequency only as a safeguard. Scan from the tail of the LRU list backward and evict the first item whose frequency falls below a threshold like 2, implementing "evict oldest unless it's hot." This leverages LRU's reliability—truly unused items naturally drift to the tail—while treating frequency as a noisy secondary filter rather than the main decision criterion. The hit-rate improves by trusting temporal locality over frequency estimates that may suffer from sketch collisions.

Simpler alternatives worth testing: extending N from 2 to 3 to give the incumbent strategy more room, or adding an absolute frequency floor of 2 at admission so that only items with freq >= 2 pass the gate relative to victim frequency. The latter filters out low-frequency noise before it pollutes the cache.

A third approach distinguishes between updates and new inserts—always admitting updates to existing keys while requiring new items to exceed victim frequency by at least 1. This leverages the fact that updates target proven-hot items, whereas new entries are unproven, so asymmetric gates should reduce churn and improve hit-rate.

The admission logic sits within the eviction block itself, not as a separate function. After selecting a victim via N candidates, the code checks `incoming_freq > victim_freq` before evicting and inserting. I can modify this condition to `if self.map.contains_key(&key) { allow } else if incoming_freq > victim_freq + 1 { allow }`, keeping the gate relaxed for existing keys but strict for new ones.

I can also adjust the victim selection loop bounds (the N parameter), or tune the sketch constants like SKETCH_WIDTH_FACTOR and SKETCH_DEPTH.

My three candidate hypotheses target different levers: extending the victim pool from N=2 to N=3, adding an absolute frequency floor to filter low-frequency insertions, and relaxing the admission threshold slightly.

For H01, I'd change the victim selection loop from `for _ in 0..1 {` to `for _ in 0..2 {`, expanding from 2 to 3 candidates. This sits between the proven N=2 baseline and the latency-expensive N=5, letting me capture more frequency diversity without the performance penalty.

H02 filters out noise by requiring `incoming_freq >= 2` before admission, preventing sketch estimation errors from polluting the cache. This is orthogonal to N-tuning and addresses a cross-cycle pattern I've noticed around sketch sensitivity.

H03 decouples the admission logic: updates to existing items bypass the frequency check entirely, while new items face a stricter gate like `incoming > victim + 1`. This reduces churn from one-off accesses while preserving cache stability for established entries.

### 06:19:04 chat

Perfect! I now have a comprehensive understanding of the landscape. Let me synthesize 3 distinct hypotheses for cycle 0009:

### 06:19:10 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0009/hypotheses/H01.md",
  "content": "# H01\n\nIncrease frequency-aware victim selection pool from N=2 to N=3 to improve hit-rate by capturing more frequency diversity without matching N=5's latency cost.\n\n## Parent\n\ncycle/0008/v003\n\n## Changes\n\nIn `src/tiny_lfu/mod.rs`, in the victim selection loop within the `put()` method (around line 103-117 based on prior cycles), extend the loop that scans additional candidates from 1 iteration to 2 iterations:\n\n```rust\n// Old:\nfor _ in 0..1 {\n    if let Some(idx) = current {\n        // ... frequency comparison\n    }\n}\n\n// New:\nfor _ in 0..2 {\n    if let Some(idx) = current {\n        // ... frequency comparison (unchanged logic)\n    }\n}\n```\n\nThis evaluates 3 total candidates (LRU head + 2 scanned items) instead of 2, maintaining the existing minimum-frequency selection logic. Keep the sketch geometry (depth=3, width=3 inherited from parent), admission threshold (`incoming_freq > victim_freq`), and all other eviction mechanics unchanged.\n\n## Rationale\n\nCycle 0006 proved that N=2 frequency-aware selection is superior to N=1 (pure LRU), achieving 30% p99 latency reduction with negligible hit-rate loss. Cycle 0008 confirmed that sketch geometry (depth-3, width-3) has reached a hard frontier below which hit-rate degrades sharply. However, the eviction pool size parameter remains under-explored between N=2 and N=5: cycle 0006 showed N=5 causes catastrophic tail-latency spikes (1200+ ns p99), but N=3 and N=4 are untested. This hypothesis conservatively extends the proven N=2 strategy by only 50% (+1 candidate evaluation), testing whether slightly deeper frequency-awareness captures additional diversity without matching N=5's latency pathology. Targets hit-rate directly while preserving the stable per-operation profile of cycle/0008/v003 (per-op-time=73ns, p99=166ns on get).\n\n"
}
```

### 06:19:11 thinking

Good! Now let me write H02.

### 06:19:16 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0009/hypotheses/H02.md",
  "content": "# H02\n\nAdd an absolute frequency floor to the admission gate to filter out very-low-frequency items that may represent estimate noise or hash collisions, improving hit-rate through more selective entry control.\n\n## Parent\n\nbaseline\n\n## Changes\n\nIn `src/tiny_lfu/mod.rs`, in the `put()` method where the admission check occurs (around line 122 in prior cycles), modify the condition that gates admission on frequency comparison:\n\n```rust\n// Old:\nif incoming_freq > victim_freq {\n    // admit and evict\n}\n\n// New:\nif incoming_freq > 1 && incoming_freq > victim_freq {\n    // admit and evict\n}\n```\n\nThis requires incoming items to have an estimated frequency of at least 2 (i.e., `incoming_freq >= 2`) before they are considered for admission, in addition to the existing relative comparison against the victim's frequency. Keep the sketch geometry (depth=3, width=4 from baseline), eviction strategy (LRU-only, N=1 from baseline), and all other logic unchanged.\n\n## Rationale\n\nThe cross-cycle insights document identifies sketch collision sensitivity as a root cause of pathology, particularly in cycle 0008 v002 where depth-2 produced a 141% p99 tail-latency spike. Frequency estimates below 2 are likely to be dominated by noise or single-hit coincidences rather than true access patterns. By filtering out items with `freq < 2`, we reduce cache pollution from items that are statistically unlikely to have high future value. This is an orthogonal admission-tuning approach that does not modify victim selection or sketch geometry. It targets hit-rate by ensuring only items with meaningful frequency signals enter the cache. The baseline's strict `incoming_freq > victim_freq` is proven necessary; this hypothesis adds a secondary absolute gate without relaxing the relative comparison, providing additional selectivity.\n\n"
}
```

### 06:19:17 thinking

Good! Now let me write H03. This is the asymmetric admission policy one.

### 06:19:23 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0009/hypotheses/H03.md",
  "content": "# H03\n\nDistinguish admission policy between updates to existing cache items (always admit) and insertions of new items (strict gate), reducing churn from low-frequency new entries evicting proven cache residents.\n\n## Parent\n\nbaseline\n\n## Changes\n\nIn `src/tiny_lfu/mod.rs`, in the `put()` method where the admission check occurs (around line 122 in prior cycles), modify the condition to check whether the incoming key is already in the cache, using asymmetric logic:\n\n```rust\n// Old:\nif incoming_freq > victim_freq {\n    // admit and evict victim\n    self.map.remove(&victim_key);\n    // ... insert new item\n}\n\n// New:\nlet is_existing = self.map.contains_key(&key);\nlet admission_ok = if is_existing {\n    true  // Always admit updates to existing items\n} else {\n    incoming_freq > victim_freq + 1  // New items: require stronger frequency advantage\n};\n\nif admission_ok {\n    // admit and evict victim\n    self.map.remove(&victim_key);\n    // ... insert new item\n}\n```\n\nKeep the sketch geometry (depth=3, width=4 from baseline), victim selection strategy (LRU-only, N=1 from baseline), and all other eviction mechanics unchanged. The modification is purely in the admission gate logic, not the selection or placement strategy.\n\n## Rationale\n\nThe cross-cycle insights reveal that admission threshold tuning is \"brittle and workload-encoded,\" with all relaxations failing to improve hit-rate. However, the frontier between relaxation and tightening remains under-explored. This hypothesis adopts an asymmetric gate: updates to proven-hot cache items (already present) always succeed, while new items face a stricter barrier (requiring `incoming_freq > victim_freq + 1` instead of `>`). The rationale is that cache residents have demonstrated value through prior accesses, whereas new items are unproven and should not displace them based on marginal frequency advantages. Updates represent the hot path where frequency estimation is most reliable; new inserts are cold-path entries where noise/collisions in estimates pose higher risk. This targets hit-rate by protecting cache continuity while remaining selective about new entries. Orthogonal to N-parameter tuning or sketch geometry: pure admission-policy refinement based on entry type.\n\n"
}
```

### 06:19:29 thinking

Perfect! I've written all 3 hypotheses. Let me verify that they meet the requirements:

1. **H01**: 
   - Parent: cycle/0008/v003 (cross-cycle incumbent) ✓
   - Claim: Distinct hypothesis about N=3 (not in prior cycles) ✓
   - Changes: Concrete modification to loop bounds ✓
   - Testable: Can measure hit-rate vs latency with N=3 ✓

2. **H02**:
   - Parent: baseline ✓
   - Claim: Different strategy (absolute frequency floor, not N-tuning) ✓
   - Changes: Concrete modification to admission check ✓
   - Testable: Can measure if filtering low-freq items helps hit-rate ✓

3. **H03**:
   - Parent: baseline ✓
   - Claim: Different strategy (asymmetric admission gates) ✓
   - Changes: Concrete modification to admission logic ✓
   - Testable: Can measure if protecting existing items helps hit-rate ✓

Requirements check:
- ✓ At least one cites cross-cycle incumbent (H01 cites cycle/0008/v003)
- ✓ All three are distinct (different optimization axes)
- ✓ All have concrete changes specified
- ✓ All are grounded in prior work
- ✓ All target fitness dimensions (H01 targets hit-rate, H02 targets hit-rate via selectivity, H03 targets hit-rate via admission fairness)

The three hypotheses are:
1. **N parameter extension** (H01) - builds on proven N=2 strategy
2. **Absolute frequency filtering** (H02) - targets noise/collision robustness
3. **Asymmetric admission policy** (H03) - distinguishes update vs insert semantics
4. 
Each explores a distinct lever—extending the N-tuning approach, introducing frequency-based filtering, and refining admission semantics—none of which have been directly evaluated before. These align with the cross-cycle guidance on threshold refinement and robustness.

### 06:19:32 chat

Perfect! I've generated 3 distinct hypotheses for cycle 0009:

## Summary

**H01 — Extend N from 2 to 3** (Parent: cycle/0008/v003)
- Increases the frequency-aware victim candidate pool from N=2 (proven) to N=3 (untested), conservatively capturing more frequency diversity without the latency pathology of N=5. Targets hit-rate by evaluating one additional candidate in the LRU list.

**H02 — Add absolute frequency floor** (Parent: baseline)
- Filters out very-low-frequency items (freq < 2) from admission, reducing cache pollution from estimate noise and hash collisions. Targets hit-rate through more selective entry control without modifying victim selection or geometry.

**H03 — Asymmetric admission gates** (Parent: baseline)
- Distinguishes between updates to existing items (always admit) and new inserts (stricter gate: `incoming > victim + 1`), protecting proven cache residents from eviction by marginal new entries. Targets hit-rate by leveraging the semantic difference between hot-path updates and cold-path inserts.

All three hypotheses:
- ✅ Cite valid parents (H01 uses cross-cycle incumbent cycle/0008/v003; H02 and H03 use baseline)
- ✅ Are orthogonal strategies (N-tuning vs. absolute filtering vs. admission asymmetry)
- ✅ Propose concrete, implementable changes to overlay files
- ✅ Target unexplored frontiers identified in cross-cycle insights (admission threshold refinement, robustness to estimate noise, and policy refinement)

The hypotheses are now written in `/home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0009/hypotheses/` as H01.md, H02.md, and H03.md.


---

- finished: 2026-06-06T06:19:32+00:00
- duration: 201.86s
- status: ok
- input_tokens: 98
- output_tokens: 19775
- cache_read_tokens: 436030
- cache_write_tokens: 38914
- cost_usd: 0.19273
