# Cross-Cycle Insights

## Recurring wins

- **Structural selectivity through small-queue admission-gate reduction is the dominant axis across ten cycles**: The monotonic progression 1/4 → 1/5 → 1/6 → 1/8 → 1/10 → 1/12 → 1/14 → 1/16 delivered consistent improvements with zero degradation, establishing tighter capacity ratios as the single highest-leverage optimization lever. Even at extreme selectivity (1/16), measurable gains persist, indicating the workload tolerates severe admission filtering without utilization collapse due to pronounced reuse clustering.

- **Frequency-depth and structural selectivity compound synergistically at the saturation boundary (cycles 0007–0010)**: Once the admission gate tightens to extreme selectivity (1/12+), extending main-queue retention (MAX_FREQ ≥ 5) becomes a competitive secondary axis. Pairings like 1/14+MAX_FREQ=5 and 1/15+MAX_FREQ=6 demonstrate that the two dimensions interact; together they unlock value that tight selectivity creates, particularly for items with strong reuse signals that survive the aggressive gate.

- **Conservative, aligned tuning outperforms aggressive re-weighting**: Cycle 0001's single-level frequency increase (MAX_FREQ 3→4) beat aggressive ghost-hit protection attempts (freq=2 upfront). The pattern holds: effective changes match observed workload dynamics rather than attempting to override queue mechanics wholesale through large perturbations.

## Recurring failures

- **Ghost-hit frequency crediting fails consistently (cycles 0001–0002)**: Two independent variants granting frequency credits to re-admitted items (freq=1, freq=2) both degraded hit-rate. The implicit signal: prior survival does not predict future reuse in this workload; items must earn frequency through actual re-access, not admission history.

- **Frequency-threshold selectivity at the promotion gate underperforms size-based admission filtering (cycle 0003)**: Raising the small-to-main promotion threshold (freq>2 vs freq>1) hurt performance despite the intuition that stricter frequency-based filtering would remove noise. The lesson: frequency signals are too coarse-grained at this decision point; structural capacity tuning (admission-gate sizing) is the correct lever.

- **Moderate structural selectivity cannot be recovered by aggressive frequency depth (cycle 0009)**: The pairing 1/12+MAX_FREQ=7 lost decisively despite doubling frequency protection beyond proven baselines. Items that fail to clear the admission gate cannot benefit from extended main-queue tenure; the optimization surface is sharply peaked at extreme selectivity, not a smooth trade-off curve.

- **Frequency-depth saturation arrives before structural selectivity does**: Returns from MAX_FREQ scaling decay sharply (0.0014 → 0.0003 → plateau between MAX_FREQ=6 and MAX_FREQ=7), while structural reduction remains productive at 1/16. The binding constraint at extreme selectivity is the curated pool size itself, not tenure length.

## Open questions

- **Where are the true saturation cliffs?** Structural selectivity shows measurable but microscopic gains at 1/16; does 1/17 or 1/20 maintain productivity or cross into degradation? Frequency depth appears to saturate hard between MAX_FREQ=6 and MAX_FREQ=7. Finer-grained boundary search would pinpoint irreversible loss-of-value zones.

- **What is the optimal structure–frequency interaction pair?** Current winners (1/15+MAX_FREQ=6, 1/14+MAX_FREQ=5) occupy a narrow region; unexplored pairings like 1/16+MAX_FREQ=6 or 1/16+MAX_FREQ=5 may reveal sharper local optima, or confirm that the surface is now flat.

- **Can workload access-pattern metrics predict extreme-selectivity requirements *a priori*?** This workload converged on 1/14–1/16 structural with MAX_FREQ=5–6 frequency—an exceptional, narrow regime. Do reuse-depth histograms, inter-arrival clustering, or temporal skew uniquely predict this class from raw access logs before iterative search?

- **Is this workload's reuse-clustering signature exceptional, or does it represent broader S3-FIFO potential?** The tolerance for 1/16-ratio admission paired with frequency-based promotion is remarkable and suggests exceptional signal-to-noise separability in the access pattern. Does this signature or the extreme-selectivity requirement appear in other production workloads?

## Per-policy notes

- **S3-FIFO (all variants tested)**: The frequency mechanism (freq > 1 for small-to-main promotion) remains the core decision point across all ten cycles. Improvements stem from tuning parameters around this mechanism—small-queue capacity and MAX_FREQ—rather than replacing it. Ten cycles reveal a discontinuous optimization landscape: structural selectivity dominates until saturation (cycles 0002–0006), then frequency-depth control becomes competitive (cycles 0007–0010). The converged regime (1/14–1/16 structural, MAX_FREQ=5–6) reflects extreme admission filtering paired with moderate main-queue protection, indicating the workload exhibits pronounced reuse clustering that justifies aggressive noise-filtering at entry and selective tenure extension for proven items.
