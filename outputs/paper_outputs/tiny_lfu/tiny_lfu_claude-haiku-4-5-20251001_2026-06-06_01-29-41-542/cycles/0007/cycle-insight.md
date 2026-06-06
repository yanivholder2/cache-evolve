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
