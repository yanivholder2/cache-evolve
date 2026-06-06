# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three variants tested different angles on the admission gate: sketch sizing, frequency decay, and gate threshold. Only the sketch-width increase (v001) improved hit rate; attempts to refresh frequency signal via decay (v002) or relax the gate threshold (v003) both degraded it.

## Patterns

- **Precision beats permissiveness**: The winner (wider sketch) improved frequency estimation fidelity while keeping admission logic unchanged. Both hurt variants tried to change the logic itself—either by decaying frequencies or accepting tied candidates—assuming the baseline was too strict. The data suggests the baseline's strictness is not the problem; measurement noise is.

- **Sketch width is a practical bottleneck**: Hash collisions in the Count-Min sketch directly degrade admission precision. Doubling the width factor (4→8) reduced collision errors and improved hit rate, confirming that sketch sizing, not responsiveness or recency effects, was limiting performance.

- **Early frequency signal is valuable**: v002's hypothesis that periodic decay would favor recent patterns over stale ones backfired—halving counters every capacity puts removes signal too aggressively. Items that earned high frequencies early are genuinely predictive, and monotonic accumulation preserves that signal better than periodic resets.

- **Ambiguity favors residents**: v003's relaxation from strict `>` to `>=` allowed tied frequency estimates to evict hot residents. When the sketch produces equal frequency estimates, accepting the tie admits marginal candidates based on recency alone, which harms hit rate. The strict gate's selectivity protects hot items during uncertainty.

## Open questions

- Does hit rate continue to improve with even wider sketches, or is there a diminishing return above 8x?
- Are other sketch parameters (depth, counter saturation strategy) worth tuning alongside width?
- Is the baseline's monotonic counter saturation at u32::MAX itself a limiting factor, or is width the only knob needed?

## Continuation

Pursue wider sketch sizing as the primary tuning axis—the baseline sketch was genuinely undersized. Drop hypothesis categories around decay, recency-weighting, and threshold relaxation; the admission logic is sound. For the next cycle, explore whether sketch depth matters, whether saturation saturation handling can improve further, or whether alternative frequency-estimation schemes (beyond Count-Min) might yield gains.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Three variants pursued sketch parameter tuning beyond Cycle 0001's width findings: testing width escalation to 16x, exploring depth as an orthogonal tuning axis, and combining both improvements. All three helped hit-rate, with v001's further width increase winning despite diminishing returns relative to prior cycles.

## Patterns

- **Width has entered diminishing returns**: The 4→8 increase from Cycle 0001 delivered +0.0047 gain; this cycle's 8→16 jump yielded only +0.0006. Doubling again produces an order of magnitude smaller payoff, signaling that pure width expansion is saturating.

- **Depth is a productive orthogonal axis**: v002's increase to 6 hash rows achieved modest but real improvement by strengthening independence of frequency estimates, orthogonal to width. This confirms that sketch parameter tuning has multiple directions, not just horizontal scaling.

- **Compound improvements plateau quickly**: v003's combination of both width and depth improvements (8x + 8 rows) delivered measurable gains, validating that the two dimensions interact minimally. However, the composite gain is modest—sketch parameter tuning as a whole is approaching its ceiling.

- **Frequency precision is no longer the bottleneck**: Cycle 0001's hypothesis that measurement noise limited admission accuracy has been mostly exhausted through width and depth tuning. The frequency-estimation surface has been substantially refined, yet hit-rate gains are flattening.

## Open questions

- Is there an optimal width-to-depth balance, or do they saturate independently?
- Are other sketch parameters (counter saturation strategy, hash seeding) worth tuning alongside width and depth?
- Should we pivot to alternative frequency-estimation structures (beyond Count-Min) to escape this plateau?
- Are other cache levers—admission gate logic, eviction ordering, or candidate selection—now the limiting factor?

## Continuation

Sketch parameter tuning has delivered measurable but declining gains. Width has plateaued; depth is productive but modest. Before pursuing further Count-Min refinements (saturation strategy, hash quality), consider whether a fundamentally different frequency-estimation approach would unlock larger gains. Alternatively, pivot to other cache levers: test whether the admission gate's logic (strict >) is still sound with refined frequency estimates, explore non-frequency signals (age, size, access pattern clustering), or revisit LRU ordering in light of improved frequency discrimination. The precise frequency surface has been tuned within Count-Min's limits; the next breakthrough likely lies outside sketch parameter space.


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


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
