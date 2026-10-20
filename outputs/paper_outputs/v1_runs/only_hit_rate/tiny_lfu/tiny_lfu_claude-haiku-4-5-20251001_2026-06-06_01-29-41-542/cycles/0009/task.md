# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0009

## Winners

- v001

## Variant insights

- /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0009/variants/v001/insight.md
- /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0009/variants/v002/insight.md
- /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0009/hypotheses/H01.md
- /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0009/hypotheses/H02.md
- /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

This cycle tested whether the refined eviction policy (age / frequency hybrid from prior cycles) enabled relaxations to the admission gate, or whether integrating orthogonal signals (item size, time-dependent terms) into either policy layer could unlock improvements. All three variants regressed: gate relaxation reduced selectivity, size penalties conflicted with frequency signals, and time-based admission logic became unbounded and ungrounded. The cohort confirms that the strict admission threshold and pure frequency-age eviction form a stable, locked optimum.

## Patterns

- **The strict admission gate `>` is optimal and not an artifact of prior eviction policies**: v001 retested gate relaxation (>=) under the current refined victim-selection policy and confirmed hit-rate regression. This shows the gate's strictness is not a limitation of simpler eviction logic, but a necessary discriminator. Items with equal frequency to the victim should remain rejected; this distinction cannot be compensated by improved eviction decisions downstream.

- **Size and frequency-age operate on incompatible optimization axes**: v002's attempt to incorporate item size as a multiplicative penalty showed that size-based optimization directly conflicts with frequency preservation. When a small item is hot and a large item is cold, the size penalty overrides frequency and causes incorrect evictions. This reveals that size cannot be seamlessly combined with the frequency-recency signal; any size-based optimization must be structurally decoupled or subordinate to frequency.

- **Admission and eviction are fundamentally different problems with different signal models**: v003 exposed that what works for eviction (age / frequency hybrid) cannot be directly mirrored in admission. The time-based admission gate failed because absolute elapsed time lacks grounding—it grows unbounded regardless of incoming item value. Eviction scores items already cached (with measured access history), while admission must score items arriving from external requests (with unknowable future utility). These require different signal architectures.

- **The incumbent policy is stable against targeted relaxations**: Both admission-layer modifications (v001, v003) and the eviction-layer augmentation (v002) failed without exception. The baseline eviction policy and strict gate form a tightly coupled, locally optimal system. Further improvement requires either orthogonal signals properly grounded to item properties, or a shift away from the current policy framework altogether.

## Open questions

- Have we exhausted the admission layer entirely, or are there signals other than frequency (e.g., item request pattern, inter-arrival time, cost-per-byte) that could improve gate discrimination?
- Should size be addressed as a separate optimization (e.g., size-aware replacement cost, item clustering by size), rather than folded into victim scoring?
- Is the cache approaching an asymptotic hit-rate ceiling under this workload with the current policy structure, or does a fundamentally different eviction or admission architecture exist?
- Do the failures in this cycle reflect workload-specific constraints, or are admission-gate strictness and frequency-age eviction universally robust?

## Continuation

The admission gate and eviction policies are now locked. The strict `>` threshold and age / frequency hybrid form a stable optimum; do not revisit gate relaxation or frequency-recency hybridization in admission without new, properly grounded signals. Size optimization failed when combined with frequency; if byte-efficiency matters, pursue it as an independent lever (e.g., size-clustering, cost-aware replacement) or accept that hit-rate and size-efficiency may be trading axes on this workload. The next cycle should either explore fundamentally different signals (cost, pattern clustering, access burstiness) for admission or eviction, or broaden the test matrix to other workloads and cache configurations to determine whether the current plateau is local or global.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Three variants tested whether the locked baseline policy could be improved through new signals, alternative selection methods, or refined sketch estimation. The first variant added access-burstiness tracking to victim selection (no change); the second replaced continuous scoring with frequency tiering (slight regression); the third applied collision-detection heuristics to sketch estimates (regression in admission). All three approaches failed or underperformed, reinforcing that the strict admission gate and frequency-age eviction form a robust, resistant optimum.

## Patterns

- **Adding new signals yields no improvement**: H01's burstiness factor, despite theoretical grounding in temporal clustering, produced zero change in hit-rate. This suggests frequency and age already capture the temporal-locality signals relevant to this workload, or the trace exhibits no access-burstiness patterns that distinguish from the frequency signal. The baseline's simpler frequency-age metric appears to encode all the discriminative power available from item-level signals.

- **Discretization degrades continuous scoring**: H02's frequency-tier approach attempted to improve robustness against estimation noise by grouping items into percentile buckets and always preferring the coldest tier. Instead, tiering lost fine-grained discrimination—the simpler continuous scoring (age / (frequency + 2.0)) retained more discriminative power than the coarser tier classification. This contrasts with the hypothesis that tier-based selection would be more robust; it appears the existing formula already handles the noise adequately.

- **Sketch refinement from first principles fails**: H03's collision-detection heuristic (switching to 25th percentile when coefficient of variation exceeds 0.15) was theoretically motivated to sharpen frequency estimates for the admission gate. Instead, it degraded performance, showing that the simpler minimum-based estimate is more effective. The heuristic's assumptions about collision patterns and percentile robustness did not transfer to this workload.

- **The baseline policy survives all three distinct attack vectors**: After Cycle 0006's demonstration that relaxation and structural augmentation both fail, Cycle 0007 confirms that new signals, alternative methods, and input refinement also fail. The incumbent policy is not a local optimum awaiting single-axis tuning—it is resistant to improvement from multiple independent directions, suggesting a deeper structural stability or global optimality.

## Open questions

- Have we exhausted the signal space for victim selection and admission, or do other item-level properties (e.g., request clustering patterns, inter-arrival time variance, access-distance distribution) unlock gains?
- Is the cache approaching or at an asymptotic hit-rate ceiling under this workload and policy, or does a fundamentally different architectural approach (e.g., adaptive mixing between policies, multi-level caches) exist?
- Do these findings generalize to other workloads, cache sizes, or admission-gate thresholds, or are they specific to the tiny-lfu trace's characteristics?
- Should focus shift from policy tuning to empirical characterization—measuring the gap between current hit-rate and theoretical limits?

## Continuation

The admission and eviction layers have now resisted improvement through at least six independent directions: formula refinement (Cycles 0004–0005), orthogonal signals and relaxations (Cycle 0006), new signals (H01), alternative selection methods (H02), and sketch refinement (H03). Do not pursue further policy-layer tuning—the returns are exhausted. The next cycle should either broaden the test matrix to other workloads or cache configurations to understand whether the current optimum is local or global, or shift focus to cache-infrastructure-level changes (capacity, architecture, data-structure alternatives) and empirical characterization of the performance ceiling. If pursuing the same workload, consider measuring the theoretical upper-bound hit-rate and gap analysis to ground whether further improvement is possible.


### Cycle 0008

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
