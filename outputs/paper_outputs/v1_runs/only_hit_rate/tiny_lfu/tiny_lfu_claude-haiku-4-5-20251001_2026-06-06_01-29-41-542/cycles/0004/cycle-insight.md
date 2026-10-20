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
