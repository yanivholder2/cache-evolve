# Cross-Cycle Insights

## Recurring wins

- **Frequency-based signals are universally productive**: Width and depth tuning (Cycles 0001–0002) improved hit-rate measurably; frequency-age hybrid eviction (Cycles 0003–0010) sustains the improvements. Frequency estimation, when tuned for collision resistance, underpins all gains.

- **Simplicity and interpretability outperform parameterized complexity**: Linear formulas beat damped and normalized variants (Cycles 0004–0005); deterministic partitioning beats randomization (Cycle 0008); continuous scoring + hierarchical partitioning beats tiering (Cycles 0008–0009). Direct, transparent metrics remain robust across parameter drift.

- **Architectural shifts unlock gains when formulas plateau**: Global continuous-scoring exhausted (Cycles 0006–0007); hierarchical frequency-cohort partitioning broke the plateau (Cycle 0008) and enabled measurable gains. Changing *how* candidates are selected beats tuning the selection formula when the latter plateaus.

- **Orthogonal signals—frequency and age—remain complementary**: Removing age (v003, Cycle 0005) caused 0.31% regression despite refined sketches. Both frequency and age capture distinct working-set properties; neither subsumes the other. Partitioning by frequency (primary) with age-based ranking (secondary) is well-balanced.

## Recurring failures

- **Threshold and gate relaxation never improves performance**: Strict admission gate `>` (vs `>=`) (Cycles 0001, 0006); percentile-threshold tightening (v001, Cycle 0009) and broadening (v003, Cycle 0009); dynamic percentile adaptation (v003, Cycle 0010). When ambiguity arises, tighter gates preserve working-set selectivity. Relaxation consistently degrades hit-rate.

- **Saturation recycling and sketch-refinement heuristics are inert or harmful**: Counter recycling on overflow (v001, Cycle 0003) produced zero change; collision-detection heuristics (v003, Cycle 0007) regressed. The existing count-min infrastructure with adequate width and depth already tolerates collision noise; retrofitted refinements add complexity without signal gain.

- **Decay and forgetting mechanisms degrade monotonic frequency signals**: Periodic frequency decay (v002, Cycle 0001) caused regression. Early high-frequency items are genuinely predictive; resetting or halving counters removes signal rather than improving responsiveness. Monotonic accumulation is more reliable than time-windowed forgetting.

- **New signals fail without structural change**: Burstiness tracking (H01, Cycle 0007) yielded zero hit-rate change; windowing dormancy (v002, Cycle 0010) yielded zero change. Adding signals to existing formulas produces no gain. Signals require architectural roles—primary vs secondary partitioning, or ranking within cohorts—to meaningfully affect victim selection.

- **Parameter fine-tuning within a stable architecture converges fast**: Formula constants (Cycles 0004–0005), frequency and age exponents (Cycle 0005), percentile thresholds (Cycle 0009), dynamic adaptation (Cycle 0010) all regressed or plateaued quickly. Once an architecture is locally stable, local tuning exhausts rapidly; breakthrough requires architectural shift, not parameter drift.

## Open questions

- **Does the frequency-cohort partitioning architecture generalize?** The 25th-percentile threshold and age/(freq+2) secondary ranking are locally optimal for tiny-lfu trace. Will these parameters transfer to other workloads, cache sizes, admission-gate thresholds, and request patterns?

- **Is a theoretical hit-rate ceiling reachable, and how far is the current policy from it?** Cycles 0006–0010 show plateaus at higher levels after architectural breaks (Cycle 0008). Can gap analysis—measuring upper-bound hit-rate and current performance—ground whether further improvement is possible within this policy family?

- **Do hierarchical partitioning and new signals interact productively?** Layered partitioning (frequency-cohort then age-ranked, or frequency then inter-arrival variance) might compound orthogonal signals. Can multi-level partitioning unlock gains without the overhead that sidelined other signal combinations?

- **What architectural alternatives exist beyond hierarchical partitioning?** Multi-level caches, adaptive policy mixing (e.g., switching between frequency-dominant and age-dominant baselines), neural or multi-armed-bandit victim selection, and cost-aware replacement all remain unexplored.

## Per-policy notes

- **Sketch-parameter layer (Cycles 0001–0002)**: Width expansion (4x→8x→16x) and depth increase (4→6 rows) are both productive but show diminishing returns. The 16x+6-row configuration is stable; further sketch tuning has negligible ROI. Collision-detection heuristics and saturation recycling do not improve it.

- **Admission gate (Cycles 0001–0010)**: Strict `>` threshold remains optimal across all policy variations and signal additions. Gate relaxation, time-based logic, and structural bypasses all failed. The gate is not a bottleneck; do not revisit without fundamentally new signals (e.g., request-specific cost, class-based separation).

- **Frequency-age eviction (Cycles 0003–0010)**: Evolved from pure-LRU → linear hybrid (age/(freq+1)) → refined hybrid (age/(freq+2)) → hierarchical partitioning (25th-percentile cold-cohort with age/(freq+2) secondary ranking). The architecture is now stable; parameter tuning yields diminishing returns. Architectural shifts (e.g., multi-level, adaptive mixing) are the next frontier.

- **Victim-selection architecture shift (Cycle 0008 breakthrough)**: Global continuous scoring (Cycles 0003–0007) plateaued despite six independent refinement vectors. Hierarchical frequency-cohort partitioning broke the plateau and sustained robustness through Cycles 0009–0010. This shift from monolithic to stratified selection is the most significant structural improvement to date; future gains likely require further architectural innovation.
