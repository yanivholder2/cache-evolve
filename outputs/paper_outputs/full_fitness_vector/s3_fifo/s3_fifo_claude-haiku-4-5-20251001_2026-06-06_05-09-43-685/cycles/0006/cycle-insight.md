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
