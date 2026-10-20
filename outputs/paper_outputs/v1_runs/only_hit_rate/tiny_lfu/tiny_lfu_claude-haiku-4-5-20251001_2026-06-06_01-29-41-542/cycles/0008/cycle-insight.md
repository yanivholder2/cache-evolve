# Cycle 0008 Insight

## Cohort outcome

This cycle shifted focus from signal and formula refinement to victim-selection architecture, testing whether hierarchical partitioning of the candidate pool could unlock gains beyond the locked baseline. Two partitioning approaches (frequency-cohort and age-cohort) succeeded; one alternative selection method (randomization) failed. The frequency-cohort variant won, establishing that explicit cold-item prioritization—rather than global continuous scoring—improves hit-rate.

## Patterns

- **Hierarchical partitioning breaks the "locked optimum" plateau**: Cycles 0006–0007 showed that formula tweaks, signal additions, and gate relaxations all failed. Cycle 0008 demonstrates the plateau was specific to global-scoring architectures. Moving to a two-pass approach—partitioning candidates into a cold cohort, then applying the scoring formula within that subset—generates measurable improvement. This suggests the incumbent's stability was not fundamental optimality, but rather an artifact of the selection method.

- **Deterministic partitioning outperforms both global and random methods**: v001's frequency-cohort approach (25th-percentile threshold, cold-first) beat both the prior continuous-global baseline and v003's randomized sampling. v002's age-cohort approach also succeeded, while v003's randomization lost 1%. This reveals a clear preference: deterministic, ordered partitioning of candidates is superior to both all-at-once global scoring and unstructured sampling.

- **Age and frequency remain orthogonal and both valuable in hierarchical form**: v002's age-cohort improvement confirms that age is not redundant with frequency. However, the fact that v001 (frequency cohort) won suggests that partitioning by frequency—cold items first—may align more naturally with the working-set structure of this trace than age-based partitioning. Frequency more directly identifies genuinely unused items, while age conflates stale-but-recently-accessed items with truly cold ones.

- **The head-to-tail LRU scan is integral, not a bias to remove**: v003's failure—despite the hypothesis that randomization would eliminate ordering bias—confirms that the deterministic linear scan synergizes with the age-frequency formula. The scan naturally prioritizes older items near the head, reducing computation for frequently-evicted candidates and exploiting cache locality. Randomization loses this alignment.

## Open questions

- Can frequency and age partitioning be combined hierarchically (e.g., cold-frequency cohort within oldest-age cohort) to compound gains, or does the search space collapse into a single dominant partitioning order?
- Is the 25th-percentile threshold for frequency partitioning optimal, or would other percentiles (10th, 50th) improve hit-rate further?
- How does the winner (v001) perform on other workloads, cache sizes, or admission-gate thresholds—is the improvement general or trace-specific?
- Have we moved from a plateau to a new optimization frontier, or is the gain from v001 a one-time improvement before hitting a new stability point?

## Continuation

The baseline's stability was not fundamental; it was an artifact of global continuous scoring. Partitioned, hierarchical victim selection unlocks gains. Pursue the frequency-cohort approach (v001) as the new baseline: it is simpler to implement and understand than age-cohort schemes, and empirically outperforms them. Next cycle, test whether finer partitioning (sub-percentile thresholds), layered partitioning (frequency then age), or alternative partitioning signals (burstiness, inter-arrival time) can compound the gains. Also benchmark v001 on other traces and workload profiles to determine whether the improvement generalizes or is specific to tiny-lfu. Do not abandon partitioned selection—the evidence strongly suggests it is the right architectural direction, and the ceiling may be higher than previously thought.
