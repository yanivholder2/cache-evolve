# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/yaniv/projects/cache-evolve/outputs/paper_outputs/tiny_lfu/tiny_lfu_claude-haiku-4-5-20251001_2026-06-06_01-29-41-542/cycles/0010

## Winners

- v002

## Variant insights

- /home/yaniv/projects/cache-evolve/outputs/paper_outputs/tiny_lfu/tiny_lfu_claude-haiku-4-5-20251001_2026-06-06_01-29-41-542/cycles/0010/variants/v001/insight.md
- /home/yaniv/projects/cache-evolve/outputs/paper_outputs/tiny_lfu/tiny_lfu_claude-haiku-4-5-20251001_2026-06-06_01-29-41-542/cycles/0010/variants/v002/insight.md
- /home/yaniv/projects/cache-evolve/outputs/paper_outputs/tiny_lfu/tiny_lfu_claude-haiku-4-5-20251001_2026-06-06_01-29-41-542/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/yaniv/projects/cache-evolve/outputs/paper_outputs/tiny_lfu/tiny_lfu_claude-haiku-4-5-20251001_2026-06-06_01-29-41-542/cycles/0010/hypotheses/H01.md
- /home/yaniv/projects/cache-evolve/outputs/paper_outputs/tiny_lfu/tiny_lfu_claude-haiku-4-5-20251001_2026-06-06_01-29-41-542/cycles/0010/hypotheses/H02.md
- /home/yaniv/projects/cache-evolve/outputs/paper_outputs/tiny_lfu/tiny_lfu_claude-haiku-4-5-20251001_2026-06-06_01-29-41-542/cycles/0010/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

Three variants tested refinements to the frequency-cohort partitioning threshold introduced in Cycle 0008 as a breakthrough, plus an alternative partitioning signal. All three regressed: tightening the threshold to 15th percentile over-restricts the victim pool, broadening it to 40th percentile over-includes warm items, and swapping to age-based partitioning reveals frequency is the stronger primary discriminator. The cohort confirms the 25th-percentile frequency threshold is well-calibrated and locally optimal for this workload.

## Patterns

- **The 25th-percentile threshold is a precision-recall sweet spot**: v001 (15th percentile) lost availability of viable victims in the first pass, forcing fallback scans that settle for suboptimal candidates. v003 (40th percentile) lost selectivity by admitting warm-but-old items that shouldn't be evicted. The threshold balances two competing needs—ensuring enough cold candidates exist to find a victim without repeatedly falling back, and remaining selective enough to exclude moderately-warm items. This balance was not an artifact of the cycle-0008 baseline; it is structurally optimal for this trace.

- **Frequency and age are orthogonal, but frequency dominates as the primary partitioning criterion**: v002's regression (age-first vs. frequency-first partitioning) confirms both signals are independently valuable—cycle 0008 showed age-cohort partitioning alone improved hit-rate. However, when forced to choose between them as the primary filter, frequency-based cold identification outperforms age-based staleness. Frequency more precisely identifies items unused at any point, while age conflates stale-but-recent-touched items with genuinely cold ones.

- **Fine-tuning the existing architecture yields diminishing returns**: Cycles 0006–0008 escaped locked optimums by shifting architectural paradigms (relaxation failed → signal addition failed → hierarchical selection succeeded). Cycle 0009 demonstrates that once a sound architecture (hierarchical frequency partitioning) is in place, parameter tuning within that frame converges quickly. The variance around the optimal threshold is narrow; small deviations in either direction degrade performance.

- **The current architecture is robust to reasonable variations**: Unlike prior cycles where formula tweaks or signal additions failed uniformly, this cycle's single winner (v001, despite regression) is a meaningful distinction: all three regressed, but frequency-cohort partitioning at 25th percentile appears to be a true local maximum, not a broad plateau. The architecture itself is sound; only the calibration parameters have tight tolerances.

## Open questions

- Can hierarchical partitioning—frequency first, then age-ranked within the cold cohort—compound the signals without sacrificing the availability that kept v001 viable in cycle 0008, or does layering flatten gains?
- Are other signals (burstiness, inter-arrival variance, request distance) viable secondary partitioning criteria that could improve discrimination within a frequency-cohort?
- Does the 25th-percentile optimum generalize to other workloads, cache sizes, and admission-gate thresholds, or is it trace-specific to tiny-lfu?
- Have we reached a local ceiling on hit-rate under this architectural family (hierarchical victim selection + admission gate), or does a fundamentally different cache policy exist?

## Continuation

The 25th-percentile frequency-cohort threshold is locally optimal; do not pursue further threshold tuning without evidence of workload drift or architectural changes. Do not swap the partitioning signal from frequency to age without layering—frequency is the stronger primary filter. Next cycle, if pursuing gains within this architectural family, test hierarchical partitioning (frequency then age within the cold cohort) to compound orthogonal signals, or explore whether secondary selection criteria (burstiness, inter-arrival time) can improve discrimination. Alternatively, test the current winner on other workloads and configurations to establish whether the optimum is robust or trace-specific to tiny-lfu. If diminishing returns persist, consider shifts to multi-level caches, adaptive policy mixing, or empirical characterization of the theoretical hit-rate ceiling on this workload.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
