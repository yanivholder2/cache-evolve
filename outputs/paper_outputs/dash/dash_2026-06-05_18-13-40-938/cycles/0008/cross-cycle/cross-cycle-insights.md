# Cross-Cycle Insights

## Recurring wins

- **Demotion-path MRU insertion**: Back insertion for demoted (tier-advanced) items consistently improves hit rate across cycles 0003–0008 and remains the only universally beneficial insertion mechanism, validating that proven tier climbers deserve immediate re-promotion eligibility.

- **Stash-hit prioritization for acceleration**: Both successful promotion strategies (deterministic tier-skip and adaptive threshold-based) converged on targeting stash residents as high-confidence candidates, confirming survivors merit fast-tracking through the hierarchy.

- **Architectural restructuring defeats parametric ceilings**: Cycle 0008's simplification of tier hierarchy (three tiers → two) and gating of tier-skip with recovery enforcement broke through the defended 0.684 ceiling, demonstrating the bottleneck was structural (topology, filtering overhead) not parametric (insertion position, occupancy thresholds).

## Recurring failures

- **Frequency-based eviction**: LFU underperforms; temporal locality (recency) dominates this workload (cycle 0001).

- **Occupancy-only filtering**: Occupancy thresholds consistently suppress or maintain hit rate without fitness signal coupling; relief-valve approaches promote churn rather than discipline (cycles 0002, 0005, 0006, 0007).

- **Composition of successful mechanisms**: Singular strategies (MRU for demotion, stash-hit acceleration, tier-skip) succeed in isolation but fail to stack; attempted combinations degrade or match baseline, indicating architectural constraints prevent synergy (cycle 0004 onwards).

- **Parametric tuning exhaustion (0.684 plateau)**: Seven cycles tested insertion position, candidacy gating, occupancy routing, eviction priority, cascade structure, and capacity allocation. All failed to exceed baseline except marginal +0.0014 (cycle 0006), confirming parametric space is saturated.

## Open questions

- Can cycle 0008's two architectural improvements (unified hot tier and gated tier-skip recovery) be combined for additive gain, or do they compete?

- What is the hit-rate improvement magnitude from cycle 0008's architectural changes, and can it be sustained as the new baseline?

- Why does the asymmetry between demotion-path MRU (beneficial) and promotion-path MRU (destabilizing) persist despite cycles 0004–0005 specifically targeting it? Is the constraint architectural or mechanical?

- What mechanism specifically defended the 0.684 ceiling: tier-skip / probing-saturation interaction, promotion filtering discipline, or eviction cascade timing?

## Per-policy notes

- **Tier structure**: Three-tier hot-side hierarchy (target + probing filtering stages) defended the 0.684 ceiling against six cycles of parametric tuning; collapse to two tiers (unified hot) improved hit rate in cycle 0008, confirming the bottleneck was intermediate filtering overhead rather than single-parameter tuning.

- **MRU insertion discipline**: Consistently improves hit rate only for demoted (tier-advanced) items; front insertion on fresh promotions maintains baseline without gain; intra-tier MRU reordering does not drive hit rate. Effectiveness is asymmetric: a cross-tier recovery mechanism, not an intra-tier optimization.

- **Tier-skip policies**: Unconditional or recovery-unstructured tier-skip conflicts with demotion discipline (cycles 0004, 0006); conditional tier-skip gated to proven stash residents with mandatory recovery (target re-entry on eviction) reconciles acceleration with safety, proving skip remains viable when recovery is enforced (cycle 0008).

- **Occupancy sensing**: Promotion thresholds, demotion routing, and capacity allocation yield no improvement; occupancy is not a fitness signal and lacks leverage for acceleration or filtering decisions.

- **Demotion cascade structure**: Target-tier retry step (safety valve for foreign residents) is load-bearing; removal degrades hit rate significantly (cycle 0007 v002). Demotion back-insertion remains the stable foundation across all variant attempts.
