# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0006

## Winners

- v001

## Variant insights

- /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0006/variants/v001/insight.md
- /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0006/variants/v002/insight.md
- /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0006/hypotheses/H01.md
- /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0006/hypotheses/H02.md
- /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0006/hypotheses/H03.md

## Prior cycles digest

### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

This cycle shifted focus from sketch parameter tuning (Cycles 0001–0002) to cache policy layers: eviction victim selection, counter saturation handling, and admission gate optimization. The cohort revealed that sketch tuning gains have saturated and that policy-level improvements—particularly in victim selection—are now the productive axis. Only eviction-focused changes (v002) improved hit-rate; counter recycling (v001) proved inert, and admission-gate bypassing (v003) regressed performance.

## Patterns

- **Eviction victim selection is now the active lever**: v002's frequency-recency hybrid (prioritizing age and coldness) improved hit-rate, confirming that how candidates are selected for eviction—not just admission—directly impacts performance. This marks a shift from Cycles 0001–0002, where sketch precision was the constraint.

- **Frequency signals must remain intact in decision logic**: v003's dual-tier fast-path (bypassing the sketch for hot items to reduce collision noise) degraded performance, while v002's incorporation of frequency into eviction succeeded. The pattern is clear: working around frequency uncertainty by partitioning or gates breaks admission discipline. Leveraging the existing frequency estimates more intelligently, even imprecise ones, outperforms circumventing them.

- **Counter saturation is not a practical bottleneck**: v001's adaptive recycling—dividing all sketch counters by 2 on overflow to preserve discrimination—delivered no measurable improvement. This suggests that on this workload, either saturation events are rare, or the loss of absolute frequency precision (from the global right-shift) offsets gains from continued accumulation. The 16x-width sketch is collision-tolerant enough that saturation handling has negligible impact.

- **Simple arithmetic policies outperform structural complexity**: v002's straightforward scoring formula (age / frequency + 1) was more effective than v001's targeted saturation optimization or v003's two-tier structural redesign, suggesting that interpretable, direct metrics are preferable to domain-specific tricks or circuit-breaker patterns.

## Open questions

- Can v002's frequency-recency formula be further tuned (different exponents, offsets) to extract additional gains?
- How stable is v002's improvement across varied workloads, cache sizes, or access patterns?
- Are other signals—item size, reuse distance, contention—worth combining with age and frequency in victim selection?
- Does saturation recycling matter on longer-running workloads or with different sketch initialization strategies?

## Continuation

Sketch parameter tuning is exhausted; cache policy is the new frontier. v002's success validates that eviction victim selection directly impacts hit-rate when frequency estimates are available. Pursue refinements to hybrid scoring metrics: test alternative formulations, integrate additional signals (size, cost, reuse distance), and measure across diverse workloads. Counter saturation recycling should be deprioritized unless saturation becomes common on other datasets. The admission gate's strictness remains sound and should not be revisited—the constraint is now eviction-layer quality, not admission-gate precision.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

Three variants refined eviction victim-selection scoring formulas to balance frequency and recency in the decision of which cached item to evict. The simple linear hybrid (v001: age / (frequency + 1)) won by a narrow margin; a sqrt-dampened variant (v002) showed modest improvement; normalization by relative frequency (v003) degraded performance. The cohort confirms that the eviction layer is the active tuning axis and that straightforward hybrid formulas outperform complex normalizations.

## Patterns

- **Linear frequency weighting is still optimal for this workload**: v001's straightforward score of age / (frequency + 1) outperformed v002's sqrt-dampened variant (age / (sqrt(frequency) + 1)), despite v002's hypothesis that sub-linear scaling would better balance age and frequency. This suggests the workload's frequency distribution is steep enough that linear scaling captures the hot/cold boundary more accurately than damped scaling.

- **Simplicity beats normalized complexity**: v003's relative-frequency normalization (age / (1 + freq/max_freq)) failed despite addressing a plausible problem—helping identify items much colder than the working set. The added complexity of computing max_frequency and normalizing each item's frequency appears to introduce noise rather than sharpen discrimination. The simple linear hybrid (v001) remains more robust.

- **Frequency-recency hybrids consistently improve over pure-LRU baselines**: Both v001 and v002 delivered measurable hits through the same conceptual lens—incorporating both temporal recency (age) and observed frequency into victim selection. This validates Cycle 0003's pivot away from sketch tuning and toward cache-policy optimization.

- **Eviction-layer tuning is now sensitive to formula details**: The three variants employed nearly identical approaches (age divided by some frequency term), yet their outcomes diverged based on the specific formula. This indicates that the eviction-selection metric is now the active lever for improvement, and small changes in how frequency is weighted have measurable impact.

## Open questions

- Why does the linear formula (v001) outperform sub-linear damping (v002)? Is the workload's frequency distribution such that high-frequency items are genuinely rare, making damping unnecessary?
- Can the winning formula be further tuned with different constants (e.g., age^α / (frequency^β + 1)) to extract additional gains?
- Are other signals beyond frequency and age—item size, reuse distance, contention—worth incorporating into the victim-selection metric?
- How stable is v001's improvement across different workloads, cache sizes, or access patterns?

## Continuation

The eviction victim-selection approach from Cycle 0003 is productive and generalizable. v001's straightforward linear hybrid won over both sub-linear damping and normalization, validating that simplicity and directness remain advantageous. For the next cycle, focus on tuning the winning formula itself: explore different exponents and offsets within the age / (frequency + 1) framework, test across varied workloads, and measure whether introducing orthogonal signals (size, cost, reuse distance) yields additive gains. Avoid complex normalizations or domain-specific tricks—the pattern suggests the cache benefits from interpretable, transparent metrics. The admission gate and sketch parameters remain stable; the frontier is now fine-tuning the hybrid eviction metric.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

This cycle attempted to refine the eviction victim-selection formula through parameter tuning and signal ablation. Three variants tested increasing the frequency offset constant, applying super-linear age weighting, and eliminating age entirely in favor of pure frequency-based selection. None of the refinements improved hit-rate; the unchanged baseline (v001) won by virtue of producing no regression.

## Patterns

- **The linear hybrid formula has reached a local optimum**: Both parameter refinements (H01's increased offset and H02's super-linear age weighting) produced zero measurable change in hit-rate. The current formula—age / (frequency + 1)—appears to be well-tuned for this workload's characteristics, and simple constant tweaking and exponent adjustments no longer yield gains. This represents a qualitative shift from Cycles 0003–0004, where policy-level changes and formula innovations still delivered improvements.

- **Recency remains essential and cannot be substituted with frequency alone**: v003's removal of age signal caused a 0.31% hit-rate loss despite the improved sketch precision from prior cycles. This definitively shows that temporal locality and frequency capture orthogonal aspects of working-set membership. The hypothesis that sufficiently precise frequency estimates might eliminate the need for age has been falsified; the two signals are complementary, not redundant.

- **Fine-tuning the frequency-recency surface exhibits diminishing returns**: Cycles 0003–0004 showed that switching from pure-LRU to hybrid scoring and then optimizing the linear formula were productive. Cycle 0005's inability to improve through constant and exponent adjustments suggests the high-level structure is sound, but further local refinement has exhausted low-hanging fruit.

- **Simplicity continues to outperform elaboration**: The unchanged v001 baseline beat both attempts to refine it, reinforcing the pattern from prior cycles that straightforward, interpretable metrics (linear hybrid scoring) are more robust than parameterized fine-tuning or structural variation.

## Open questions

- Have we exhausted the victim-selection metric itself, or are there fundamentally different scoring approaches (e.g., probabilistic, rank-based, or information-theoretic) worth exploring?
- Are other cache levers now the constraint: admission gate strictness, sketch parameter interaction, or item-level metadata (size, cost)?
- Would introducing orthogonal signals—item size, reuse distance, or inter-arrival time—unlock new gains, or are they already implicitly captured by age and frequency?
- Does performance plateau differently on other workloads, or is this plateau workload-specific?

## Continuation

The victim-selection metric has plateaued under local parameter tuning. The linear frequency-age hybrid remains sound and should be retained as the eviction policy baseline. Do not pursue further formula refinement (exponents, offsets, normalization) on this or similar workloads unless new signals are incorporated. Consider pivoting to orthogonal cache levers: test whether admission gate logic (strict > admission) or sketch parameters need re-tuning in light of this stable eviction policy, or explore whether metadata-driven signals (item size, cost, access pattern clustering) add discriminative power beyond frequency and age. The eviction-selection frontier is now either about broadening the signal set or shifting focus entirely to other parts of the cache system.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
