# Cycle 0010 Insight

## Cohort outcome

Three variants pursued refinements to the frequency-cohort partitioning architecture introduced in Cycle 0008: simplifying secondary ranking to pure age (v001), adding a burstiness-window filter (v002), and introducing dynamic percentile adaptation (v003). All three failed or showed no improvement. The winner (v002) produces no change, signaling that the frequency-cohort baseline with its age/(freq+2) hybrid scoring is well-calibrated and resistant to further tuning.

## Patterns

- **The frequency-cohort architecture has reached a new local optimum**: After Cycle 0008's breakthrough (partitioned selection beat global scoring), Cycle 0009 confirmed the 25th-percentile threshold was optimal, and Cycle 0010 shows the entire framework—threshold, partitioning signal, and secondary ranking formula—now resists improvement from multiple independent directions. This mirrors the locked-optimum plateau from Cycles 0006–0007, but at a higher baseline performance level. The architectural shift from global to hierarchical selection unlocked gains; further refinement within that frame shows diminishing returns.

- **Stripping frequency from secondary ranking degrades performance**: v001's hypothesis that pure age would better identify dormant items within the cold cohort failed. The age/(freq+2) formula provides meaningful discrimination even among low-frequency items, distinguishing between genuinely inert items and those with occasional recurrence. Removing the frequency denominator causes aggressive eviction of items with marginal value retention, confirming the hybrid formula is a well-balanced compromise.

- **Adding new dormancy signals yields no measurable improvement**: v002's burstiness-window filter, despite being theoretically motivated (items with `burstiness_window == 0` have exited the active set), produced zero change in hit-rate. This suggests the age/(freq+2) scoring already captures the relevant temporal and frequency information; additional signals about window resets or activity clustering do not refine victim selection beyond the existing formula's discriminative power.

- **The fixed 25th-percentile threshold is a genuine equilibrium, not a lucky parameter**: v003's dynamic adaptation, which aimed to keep the cold cohort in a 10–20% band by shifting between 15th and 35th percentiles, degraded performance. The fixed 25th percentile is not merely locally optimal for static conditions—it represents a balance point where availability (sufficient victim candidates) and selectivity (exclusion of items with residual value) are both optimized. Introducing variance through adaptation disrupts this balance.

## Open questions

- Is the current architecture (hierarchical frequency-cohort selection with age/(freq+2) scoring) asymptotically optimal for this workload and cache configuration, or do fundamentally different victim-selection paradigms exist (e.g., neural selection, multi-armed-bandit adaptation, admission-aware eviction)?
- Does the 25th-percentile optimum and age/(freq+2) formula generalize to other workloads, cache sizes, and admission thresholds, or is this configuration trace-specific to tiny-lfu?
- Are we approaching the theoretical hit-rate ceiling for this policy family, and if so, what is the gap between current performance and the ceiling?
- Could shifts in cache architecture (multi-level, adaptive policy mixing, sketch-based selection) yield larger gains than further parameter or formula tuning?

## Continuation

The frequency-cohort partitioning architecture has now resisted improvement from five independent refinement vectors: secondary-ranking simplification (v001), dormancy-signal addition (v002 no change), and dynamic thresholding (v003)—following Cycles 0006–0009's six prior unsuccessful attack directions. Do not pursue further tuning of the partitioning signal, threshold, or secondary ranking formula. The current baseline (v002, unchanged) is locally stable. Next cycle, either test the current winner on other workloads, cache sizes, and admission thresholds to establish robustness, or shift focus to empirical characterization: measure the theoretical hit-rate ceiling and gap analysis to ground whether further improvement exists. If pursuing the same configuration, consider cache-level architectural changes (multi-level caches, adaptive policy mixing, alternative data structures) rather than victim-selection parameter tuning.
