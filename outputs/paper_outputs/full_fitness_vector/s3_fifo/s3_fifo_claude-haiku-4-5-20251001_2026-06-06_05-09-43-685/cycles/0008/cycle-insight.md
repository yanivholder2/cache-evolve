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
