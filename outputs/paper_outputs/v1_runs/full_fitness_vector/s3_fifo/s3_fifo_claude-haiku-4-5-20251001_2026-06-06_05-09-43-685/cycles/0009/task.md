# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_05-09-43-685/cycles/0009

## Winners

- v002
- v001
- v003

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_05-09-43-685/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_05-09-43-685/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_05-09-43-685/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_05-09-43-685/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_05-09-43-685/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_05-09-43-685/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

This cycle tested three hypotheses aimed at escaping the occupancy-responsive-threshold plateau confirmed in cycle 0005: finer-grained occupancy breakpoints (v001), multi-level ghost-queue frequency decay with pre-computed re-admission signals (v002), and age-aware promotion thresholds (v003). All three variants succeeded in narrowly defined ways but revealed complementary trade-offs: v001 demonstrated that further occupancy-parameter tuning yields negligible hit-rate recovery at per-operation cost; v002 uncovered that pre-computed ghost frequencies enable dramatic latency gains (49% faster puts) but require a tuned re-admission formula to preserve hit-rate; v003 showed that age-based promotion pressure, while intuitive, adds overhead without unlocking the expected frequency-starvation fixes. The cohort pivots understanding away from "refine occupancy thresholds" toward a new frontier: **pre-computed ghost-queue state can deliver latency improvements if coupled with an adaptive re-admission signal**.

## Patterns

- **Per-operation overhead is the primary cost of threshold refinement**: Both v001 (occupancy-ratio computation per eviction) and v003 (age-ratio computation and conditional branching) added latency despite targeting hit-rate recovery. v001's 6ns get latency penalty and v003's 35–244ns put penalties exceed their marginal gains. This confirms cycle 0005's finding that occupancy-aware *mechanisms* are more expensive than occupancy-aware *thresholds*. Threshold selection itself (cycle 0003) was overhead-free; any downstream logic compounds the cost.

- **Ghost-queue frequency pre-computation enables latency breakthrough, but the re-admission formula is brittle**: v002's architecture—storing (K, frequency) tuples in ghost and using direct arithmetic on re-admission—delivers compelling latency wins (get 248→175ns, put 1088→550ns). However, the fixed boost formula `ghost_freq + 3` overshoots for some decay trajectories and undershoots for others, causing hit-rate loss (0.7164→0.7074). This reveals that the bottleneck is not mechanism or architecture but signal accuracy: the decay-to-actual-hotness mapping is workload-dependent.

- **Fine-grained occupancy-parameter tuning (0.4, 0.7 vs. 0.5, 0.8) does not escape the cycle 0005 local optimum**: v001's refined breakpoints neither recovered hit-rate nor reduced latency compared to cycle 0005/v002. This suggests occupancy thresholds are less sensitive to granularity than to the underlying policy. The cycle 0003/v003 design (threshold range 1–3, breakpoints 0.5/0.8) was well-calibrated for this workload; further tweaking explores noise, not new optima.

- **Age-aware thresholds (position in small queue) do not match the frequency-starvation pattern in this workload**: v003's hypothesis—that items aging out of the small queue deserve a looser promotion gate—intuited a real problem (items with freq=1 evict to ghost before reaching freq=2) but the proposed solution (loosening to freq > 1 for the back 30%) did not unlock additional main-queue promotions. Either the starvation scenario is rare in this access pattern, the back-30% window misses the vulnerable population, or frequency acquisition outpaces aging in the small queue, rendering the rescue mechanism unused.

## Open questions

- Can v002's fixed re-admission boost be replaced with an adaptive formula (e.g., `boost = 2 + occupancy_ratio` or `boost = 2 + frequency_percentile(ghost_freq)`) that preserves latency gains while tuning hit-rate to match or exceed baseline?

- Is v002's two-level decay (-2 threshold before discard) optimal, or should the depth be varied (e.g., -1 for aggressive filtering, -3 for lenient retention) to find a sweeter point between latency and hit-rate?

- Do v001 and v003's per-operation overhead costs have structural alternatives—e.g., sampling-based occupancy tracking, lazy evaluation, or amortized computation—that could reduce overhead without sacrificing the decision quality they target?

- Has the cohort reached a true Pareto boundary where further latency improvement necessarily degrades hit-rate, or does an unexplored mechanism (e.g., recency weighting in ghost, hybrid frequency-recency scoring) exist that could improve both?

## Continuation

The cohort identifies a promising new direction: **pre-computed ghost-queue state (v002's architecture) can unlock latency improvements, but the re-admission signal must adapt to workload characteristics**. Cycle 0003's occupancy-responsive thresholds remain the strong baseline for balanced performance, but v002 reveals a new frontier in ghost-queue efficiency. The per-operation overhead of occupancy-aware mechanics (v001, v003) is likely unavoidable; further gains from threshold tuning alone are unlikely. **For the next cycle:** (1) prioritize tuning v002's re-admission boost formula—test occupancy-responsive or frequency-distribution-aware boost scaling to recover hit-rate while retaining latency wins; (2) experiment with v002's decay depth (e.g., -1, -2, -3) as a coarse parameter sweep to isolate whether hit-rate loss is an artifact of too-aggressive pruning; (3) if dual-metric improvement remains critical and tuning v002's formula fails, consider whether a fundamentally different re-admission signal (recency, frequency percentile, decay-rate adaptation) could supplement or replace the current arithmetic boost; (4) if accepting latency-hit-rate tradeoffs is acceptable (latency-optimized deployments), v002's current architecture is near-optimal and should be tuned for that use case. Hold cycle 0003/v003 as the dual-metric baseline; use it as the decision point for whether to commit to v002's latency specialization.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

This cycle tested three distinct approaches to advance beyond cycle 0006's latency breakthrough: adding ghost-frequency decay to a lean baseline (v001), replacing the fixed ghost re-admission boost with occupancy-responsive scaling (v002), and using recency instead of frequency as the ghost re-admission signal (v003). Only v002 delivered clear dual-metric wins—v001 reintroduced per-operation overhead without recovering hit-rate, while v003 achieved dramatic latency gains at the cost of hit-rate loss. The cohort confirms that adaptive re-admission signals outperform both fixed formulas and frequency-decay approaches when paired with the right architectural foundation.

## Patterns

- **Occupancy-responsive adaptive boosting outperforms fixed-boost policies**: v002 proves that scaling ghost re-admission strength with main-queue occupancy delivers both latency improvement and hit-rate stability, extending cycle 0003's success with occupancy-aware thresholds into the realm of signal adaptation. The variable boost eliminates thrashing that fixed +3 formulas trigger during capacity transitions.

- **Recency is computationally efficient but predictively weak**: v003 demonstrates that entry position in ghost requires cheaper arithmetic than frequency lookup, yielding 43–57% latency improvements. However, recency alone is a poor proxy for re-access likelihood in this workload—temporal distance from eviction does not capture the same patterns that frequency or occupancy-based signals do, resulting in hit-rate loss.

- **Ghost-frequency decay cannot substitute for structural adaptation on lean baselines**: v001's attempt to add frequency preservation (-3 depth) to cycle 0005/v002's lean approach failed on both fronts: the per-operation overhead of ghost entry search and boosting contradicted the lean design, while the passive recovery mechanism could not replace the active barrier adaptation that structural sizing provides.

- **Pre-computed ghost-queue state with stored metadata remains the durable architectural choice**: All three variants leveraged ghost entries containing frequency or state information, confirming cycle 0006/v002's architectural insight that pre-computation enables efficiency. The critical variable is the re-admission signal used to exploit that state.

## Open questions

- Does combining v002's occupancy-responsive formula with recency-based filtering (e.g., prioritizing re-admission of recently-evicted items) recover both v002's latency and v003's hit-rate potential?

- Can v003's recency-to-boost scaling coefficients be tuned (e.g., 2–5 range instead of 2–4) to recover the hit-rate loss while retaining latency gains, or is the predictive gap structural?

- Is there a sweet spot for ghost-frequency decay depth (between -1 and -3) that reduces per-operation overhead while preserving enough warm candidates to improve hit-rate on lean baselines?

## Continuation

The cohort validates that **adaptive re-admission signals scale with workload state better than fixed formulas**, making v002 the strongest baseline for balanced performance. Recency (v003) opens a new latency frontier but at measurable hit-rate cost; the next cycle should explore whether signal combination or tuning recovers both gains. Ghost-frequency decay on lean architectures (v001) consistently reintroduces overhead costs that offset any hit-rate benefit—this direction should be deprioritized unless paired with structural changes that justify the per-operation cost. **For the next cycle:** (1) integrate v002's occupancy-responsive boost as the new baseline if latency-hit-rate balance is the goal; (2) if latency specialization is acceptable, v003's recency approach is near-optimal and worth tuning for that use case; (3) explore hybrid re-admission signals (frequency + recency, occupancy + recency) to test whether combining signals recovers v003's latency while protecting hit-rate; (4) avoid further ghost-frequency-decay variants unless they couple to structural mechanisms (e.g., dynamic sizing, adaptive thresholds) that actively use the preserved state.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

This cycle tested three distinct ghost-queue refinements aimed at escaping the latency-hit-rate plateau: selective frequency filtering to reduce ghost noise (v001), dual-signal re-admission combining frequency and recency (v002), and occupancy-gated retention to adapt ghost capacity under load (v003). All three variants succeeded in narrowly defined ways, confirming that improvements exist but at mutual trade-offs. v001 achieved dramatic latency gains for puts (63% faster) by filtering ghost to high-frequency items only, v002 explored signal combination for prediction accuracy but suffered from suboptimal weighting, and v003 revealed that occupancy-responsive mechanics are expensive on hot paths despite intuitive appeal.

## Patterns

- **Ghost-queue signal purity vs. diversity is a false dichotomy**: v001 demonstrates that filtering ghost to only freq > 0 items improves put latency by concentrating on genuinely warm candidates, but the hit-rate loss (0.61%) reveals the diversity cost—items that never reached admission thresholds but might re-access are discarded. Pure signal fidelity collapses hit-rate; pure diversity (retaining freq=0) includes noise that degrades prediction. The sweet spot lies between these extremes.

- **Multi-signal re-admission is mechanically sound but formula-brittle**: v002's architecture of combining frequency depth and recency position via normalized signals is conceptually strong—two orthogonal dimensions should better predict re-access than either alone. However, equal weighting (0.5/0.5) and conservative boost scaling (range 2–4) undershoots actual re-admission needs, suggesting that signal weights are workload-dependent and require fine-tuning or adaptive scaling.

- **Occupancy-responsive mechanics compound hot-path overhead**: v003's attempt to gate ghost retention with occupancy ratios introduces multiple division and branching operations into eviction paths, especially in the small queue where frequency is highest. The result: get p99 improves locally (smaller ghost pool to search), but put p99 spikes significantly due to branching overhead when small queue fills frequently. Occupancy sensitivity is valuable, but computing it on every eviction is too expensive.

- **Eviction-time computation is zero-cost; hot-path computation is not**: v001's ghost filtering happens at eviction time (infrequent) with zero hot-path impact; v003's occupancy-ratio computation happens on every eviction. This distinction is critical: architectural adaptations that defer complexity to cold paths (eviction) scale; those that add branching to hot paths (especially small-queue puts) do not.

## Open questions

- Can v001's ghost-admission threshold be relaxed (e.g., freq >= -1 instead of freq > 0) to recover lost hit-rate candidates while retaining most of the latency benefit?

- Does v002's dual-signal weighting respond to occupancy state—should recency weight increase under low occupancy (more re-access time) and frequency weight increase under high occupancy (less time to prove utility)?

- Can v003's occupancy computation be amortized (e.g., computed once per N evictions) or moved entirely to eviction-time decisions (like v001's filtering) to avoid hot-path branching?

- Has the cohort reached a Pareto frontier where latency improvements require hit-rate trade-offs, or does an unexplored mechanism (e.g., frequency-decay scheduling independent of eviction, sampling-based occupancy signals) exist that could improve both?

## Continuation

The cohort validates that **ghost-queue refinements unlock improvements, but each comes with a cost trade-off tied to computational placement and parameter tuning**. V001 proves filtering at eviction time is overhead-efficient; v002 shows signal combination is promising but needs formula refinement; v003 demonstrates occupancy sensitivity is expensive on hot paths. None achieved dual-metric wins, confirming that meaningful gains require accepting latency-hit-rate trade-offs or finding unexplored mechanisms. **For the next cycle:** (1) prioritize v001's threshold tuning—test freq >= -1 or freq >= 0 to find the boundary where diversity recovery outweighs noise cost; (2) if pursuing v002's dual-signal approach, explore adaptive weighting (e.g., occupancy-responsive boost scaling) or frequency-distribution-aware normalization to improve the prediction formula; (3) abandon occupancy-gated mechanics on hot paths (v003's approach); if occupancy sensitivity is needed, move it entirely to eviction-time filtering or deferred updates (batch occupancy computation); (4) decide whether latency or hit-rate is the primary metric for the next cycle—the current frontier appears to require trade-offs, making explicit prioritization essential for coherent exploration.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
