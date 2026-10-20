# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/s3_fifo/s3_fifo_claude-haiku-4-5-20251001_2026-06-06_05-09-43-685/cycles/0010

## Winners

- v003
- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/s3_fifo/s3_fifo_claude-haiku-4-5-20251001_2026-06-06_05-09-43-685/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/s3_fifo/s3_fifo_claude-haiku-4-5-20251001_2026-06-06_05-09-43-685/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/s3_fifo/s3_fifo_claude-haiku-4-5-20251001_2026-06-06_05-09-43-685/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/s3_fifo/s3_fifo_claude-haiku-4-5-20251001_2026-06-06_05-09-43-685/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/s3_fifo/s3_fifo_claude-haiku-4-5-20251001_2026-06-06_05-09-43-685/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/s3_fifo/s3_fifo_claude-haiku-4-5-20251001_2026-06-06_05-09-43-685/cycles/0010/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

This cycle tested three approaches to advance the ghost-queue frontier: relaxing the ghost-queue filter to recover lost hit-rate diversity (v001), moving occupancy-responsive boost computation to the ghost-hit cold path to eliminate hot-path overhead (v002), and batch-amortizing frequency decay to enable richer ghost state without per-operation cost (v003). All three variants succeeded in narrowly defined ways: v001 confirmed that aggressive filtering was correct by showing relaxation reintroduced overhead without gain; v002 proved that occupancy-responsive signals can be computed on cold paths with no per-operation penalty but revealed the boost formula requires tuning; v003 achieved dramatic latency wins (44% faster gets, 37% faster puts) while maintaining hit-rate by pushing decay entirely into amortized cold-path work. The cohort pivots understanding toward a new frontier: **computational placement (hot vs. cold path) is more valuable than mechanism innovation, and amortization unlocks richer state without cost.**

## Patterns

- **Ghost-queue signal purity was correctly calibrated**: v001's attempt to relax the freq > 0 filter to freq >= -1 catastrophically backfired, degrading put latency by 378ns p50 and 3405ns p99 without recovering any hit-rate. This confirms that cycle 0008/v001's aggressive filtering was not overly restrictive—freq=0 items are genuine noise that clutters the ghost lookup space, not hidden re-access opportunities. The boundary test reveals that further filtering, not relaxation, remains the path forward.

- **Occupancy-responsive adaptation is architecturally sound but formula-brittle**: v002 delivers a 19% get latency improvement by moving occupancy computation from the hot put path (where it incurs overhead) to the cold ghost-hit re-admission path (where it is negligible). This validates cycle 0008's insight that "eviction-time computation is zero-cost; hot-path computation is not." However, the quadratic boost formula `(2.0 + occupancy_ratio²)` is miscalibrated, providing excessive re-admission protection (freq 2–3) that crowds out small-queue working-set items, causing a 1.55% hit-rate regression. The architecture holds; the signal magnitude requires workload-specific tuning.

- **Amortization enables richer ghost state without per-operation cost**: v003 proves that expensive operations—like per-operation frequency decay—can be preserved if batched sufficiently (every 32 evictions). This delivers dramatic latency wins (44% faster gets, 37% faster puts) while maintaining hit-rate integrity (−0.05%, negligible noise). The result extends cycle 0008/v001's "zero-hot-path-overhead" philosophy: you can have both latency and richer ghost metadata if the decay window aligns with workload patterns. Frequency depth is preserved, not discarded; it is simply updated less frequently.

- **Hot-path vs. cold-path computation is the primary leverage point**: Both v002 and v003 succeed by deferring expensive operations (occupancy computation, frequency decay) to cold paths. Conversely, v001 fails because search-path overhead cannot be eliminated. This reframes the frontier: cycles 0006–0008 explored *which* signals to use; cycle 0009 shows the next lever is *where to compute them*. Mechanism innovation has reached diminishing returns; architectural placement is now the primary frontier.

## Open questions

- Can v002's quadratic boost formula be recalibrated (e.g., lower magnitude, occupancy-threshold gating, or frequency-distribution-aware scaling) to recover hit-rate while preserving the 19% latency gain?

- Does combining v002's occupancy-responsive boost with v003's batch-amortized decay unlock dual-metric gains, or do the two mechanisms interact negatively?

- Is v003's 32-eviction amortization window optimal for this workload, or should it be tuned (e.g., 16, 64) to balance decay freshness against amortization overhead?

- Can v002's ghost-hit re-admission selectivity be refined (e.g., only re-admit items with `ghost_freq > threshold(occupancy)`) to avoid crowding the small-queue working set while retaining occupancy-responsive adaptation?

## Continuation

The cohort validates that **computational placement (hot vs. cold path) is more valuable than mechanism innovation**, with amortization as the enabling technique for richer state. v003's batch-amortized decay is the strongest baseline for latency, while v002's architecture (occupancy-responsive cold-path computation) remains valid if the boost formula is tuned. v001's failure confirms that ghost-queue signal purity was correctly calibrated; further relaxation explores noise, not gain. **For the next cycle:** (1) prioritize tuning v002's boost formula—test lower magnitudes (e.g., 1.5 + occupancy_ratio) or frequency-threshold gating to recover hit-rate while retaining latency; (2) explore combining v003's batch decay with v002's occupancy-responsive re-admission to test whether richer state + adaptive signals unlock dual-metric wins; (3) if latency specialization is the goal, v003's amortized-decay approach is near-optimal and ready for production deployment; (4) validate v003's 32-eviction amortization window via parameter sweep (16, 32, 64) to ensure it matches this workload's small-queue eviction frequency; (5) hold v002's cold-path-computation insight as the architectural baseline—it has proven overhead-free and should be the foundation for signal adaptation going forward.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
