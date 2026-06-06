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
