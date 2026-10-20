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


### Cycle 0010

# Cycle 0010 Insight

## Cohort outcome

Three variants pursued refinements to the frequency-cohort partitioning architecture introduced in Cycle 0008: simplifying secondary ranking to pure age (v001), adding a burstiness-window filter (v002), and introducing dynamic percentile adaptation (v003). All three failed or showed no improvement. The winner (v002) produces no change, signaling that the frequency-cohort baseline with its age/(freq+2) hybrid scoring is well-calibrated and resistant to further tuning.

## Patterns

- **The frequency-cohort architecture has reached a new local optimum**: After Cycle 0008's breakthrough (partitioned selection beat global scoring), Cycle 0009 confirmed the 25th-percentile threshold was optimal, and Cycle 0010 shows the entire framework—threshold, partitioning signal, and secondary ranking formula—now resists improvement from multiple independent directions. This mirrors the locked-optimum plateau from Cycles 0006–0007, but at a higher baseline performance level. The architectural shift from global to hierarchical selection unlocked gains; further refinement within that frame shows diminishing returns.

- **Stripping frequency from secondary ranking degrades performance**: v001's hypothesis that pure age would better identify dormant items within the cold cohort failed. The age/(freq+2) formula provides meaningful discrimination even among low-frequency items, distinguishing between genuinely inert items and those with occasional recurrence. Removing the frequency denominator causes aggressive eviction of items with marginal value retention, confirming the hybrid formula is a well-balanced compromise.

- **Adding new dormancy signals yields no measurable improvement**: v002's burstiness-window filter, despite being theoretically motivated (items with `burstiness_window == 0` have exited the active set), produced zero change in hit-rate. This suggests the age/(freq+2) scoring already captures the relevant temporal and frequency information; additional signals about window resets or activity clustering do not refine victim selection beyond the existing formula's discriminative power.

- **The fixed 25th-percentile threshold is a genuine equilibrium, not a lucky parameter**: v003's dynamic adaptation, which aimed to keep the cold cohort in a 10–20% band by shifting between 15th and 35th percentiles, degraded performance. The fixed 25th percentile is not merely locally optimal for static conditions—it represents a balance point where availability (sufficient victim candidates) and selectivity (exclusion of items with residual value) are both optimized. Introducing variance through adaptation disrupts this balance.

## Open questions

- Is the current architecture (hierarchical frequency-cohort selection with age/(freq+2) scoring) asymptotically optimal for this workload and cache configuration, or do fundamentally different victim-selection paradigms exist (e.g., neural selection, multi-armed-bandit adaptation, admission-aware eviction)?
- Does the 25th-percentile optimum and age/(freq+2) formula generalize to other workloads, cache sizes, and admission thresholds, or is this configuration trace-specific to tiny-lfu?
- Are we approaching the theoretical hit-rate ceiling for this policy family, and if so, what is the gap between current performance and the ceiling?
- Could shifts in cache architecture (multi-level, adaptive policy mixing, sketch-based selection) yield larger gains than further parameter or formula tuning?

## Continuation

The frequency-cohort partitioning architecture has now resisted improvement from five independent refinement vectors: secondary-ranking simplification (v001), dormancy-signal addition (v002 no change), and dynamic thresholding (v003)—following Cycles 0006–0009's six prior unsuccessful attack directions. Do not pursue further tuning of the partitioning signal, threshold, or secondary ranking formula. The current baseline (v002, unchanged) is locally stable. Next cycle, either test the current winner on other workloads, cache sizes, and admission thresholds to establish robustness, or shift focus to empirical characterization: measure the theoretical hit-rate ceiling and gap analysis to ground whether further improvement exists. If pursuing the same configuration, consider cache-level architectural changes (multi-level caches, adaptive policy mixing, alternative data structures) rather than victim-selection parameter tuning.


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
