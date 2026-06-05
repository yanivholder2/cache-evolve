# Cross-Cycle Insights

## Recurring wins

- **Stash-hit acceleration**: Both cycles improved hit rate by prioritizing items that had already proved their value through stash residency. This signal (recent survival through eviction pressure) is more actionable than occupancy alone.
- **MRU positioning within tiers**: Inserting at back (MRU) rather than front improved outcomes in both cycles. Position within a tier meaningfully affects promotion eligibility and competition.
- **Fitness-gated acceleration**: Successful strategies (v001, v002 in cycle 0002; v001, v003 in cycle 0001) paired acceleration with explicit hotness checks. Acceleration without a fitness criterion creates churn.
- **Recency over frequency**: Access count–based promotion consistently underperformed on this workload; recency signals (stash membership, MRU position) are stronger predictors of near-term reuse.

## Recurring failures

- **Occupancy without fitness checks**: v003 (cycle 0002) promoting any target when stash exceeded 60% occupancy failed by promoting cold items alongside hot ones. Occupancy is a load indicator, not a fitness signal; relief strategies must gate on hotness.
- **Frequency-based eviction policies**: v002 (cycle 0001) using LFU showed no gain despite correct implementation. This workload exhibits temporal locality; access count is a weaker signal than recency on this traffic pattern.

## Open questions

- **Deterministic vs. adaptive acceleration**: Both v001 (deterministic tier-skip) and v002 (occupancy-adaptive tier-skip, cycle 0002) improved hit rate. Does adaptive logic offer advantage under sustained cache pressure, or is the simpler deterministic approach equivalent across load conditions?
- **Threshold sensitivity in adaptive strategies**: v002's 75% occupancy threshold succeeded; v003's 60% threshold failed. Is there a principled way to set occupancy thresholds, or does the fitness check (stash hit vs. occupancy) dominate the threshold choice?
- **Combination synergy**: Can tier-position optimization (v001's MRU back insertion) be combined with occupancy-aware gating (v002's logic) to create a hybrid that handles both steady-state and peak loads?
- **Selection rationale**: Why was v001 chosen as winner in cycle 0002 over v002, despite both improving hit rate? Magnitude, consistency, interaction with other subsystems, or simplicity?

## Per-policy notes

- **Stash-hit promotion**: Direct tier-skipping (v001) and occupancy-adaptive tier-skipping (v002) both work when promotion is gated on stash membership. This is the most reliable lever identified so far.
- **Frequency-based policies**: Underperform on this recency-dominated workload. Prioritize temporal signals (MRU position, stash residence) over access count.
- **Occupancy-adaptive strategies**: Succeed when fitness-gated (v002 cycle 0002); fail when used for unconditional relief (v003 cycle 0002). Occupancy should modulate *which* hot items get accelerated, not *whether* to accelerate without fitness checks.
- **Position and promotion rank**: Back (MRU) insertion within target tiers showed consistent benefit; this suggests query order within a tier is a meaningful lever independent of tier-transition logic.
