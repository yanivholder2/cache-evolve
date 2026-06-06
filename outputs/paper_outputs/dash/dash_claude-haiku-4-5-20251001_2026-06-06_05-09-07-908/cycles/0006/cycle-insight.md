# Cycle 0006 Insight

## Cohort outcome

This cycle tested three refinements aimed at optimizing the hit-rate/latency frontier: aggressive geometry (85/15) paired with adaptive threshold scaling, pure geometry at 80/20 as a midpoint within the saturation zone, and adaptive threshold formula refinement (divisor 3 instead of 4) on balanced baseline geometry. All three variants succeeded, but with divergent trade-offs. The clear standout is v002's 80/20 pure-geometry variant, which improved both hit-rate and get latency with only modest put-latency cost. v003's formula refinement proved that decreasing the threshold divisor from 4 to 3 on balanced geometry unlocks simultaneous gains across both hit-rate and latency. v001's attempt to compose aggressive geometry (85/15) with adaptive thresholding failed to reconcile the opposing forces: while promoting items more aggressively helped individual get operations, the combined effect overwhelmed the stash tier, regressing both hit-rate and put latency.

## Patterns

- **Optimal geometry lies in the 75/25–80/20 band, not further**: v002's 80/20 geometry delivers strong hit-rate improvement (7.5% gain) and exceptional get-latency reduction (37% improvement) with a manageable put-latency cost. This confirms that the hit-rate/latency frontier is well-calibrated in the mid-aggressive range, closer to the 75/25 incumbent than the 90/10 saturation boundary identified in cycle 0005. Pushing geometry further (v001's implicit pressure via aggressive threshold on 85/15) introduces unbalanced trade-offs; pulling back slightly (v002's 80/20) maximizes the favorable region.

- **Adaptive threshold formula sensitivity is tunable**: v003's shift from divisor 4 (cycle 0005) to divisor 3 demonstrates that formula constants directly control promotion selectivity and can be refined to shift the hit-rate/latency frontier. On baseline 25/75 geometry, threshold=8 (divisor 3) permits more candidates to promote than threshold=6 (divisor 4), simultaneously improving hit-rate and reducing latency. This suggests the prior formula over-filtered on balanced geometry, leaving hot items trapped in lower tiers. The formula's adaptation to stash bucket count makes these refinements portable across geometries.

- **Composition succeeds when it aligns rather than opposes geometry pressure**: v001 fails because 85/15 geometry (aggressive) paired with threshold=1 (also aggressive) creates compounding promotion pressure—more items flood the probing tier faster, overwhelming the 5-bucket stash and triggering eviction cascades. v003 succeeds because divisor 3 on balanced geometry aligns with the stash's capacity: 8 stash buckets on baseline can absorb the extra promotion candidates without churn. Composition is not orthogonal; it is constrained by the geometry's capacity envelope.

- **Simple geometry outperforms complex composition when both win**: Among the three winners, v002 (pure 80/20 geometry) achieves the cleanest outcome—no surprising trade-offs, no latency regressions, transparent hit-rate/latency coupling. v003 also succeeds, but via formula tuning on conservative geometry, leaving v001's composition approach as the least reliable. This reinforces the prior cycle's insight: when multiple strategies succeed, simpler mechanisms typically deliver more robust results.

- **Hit-rate degradation remains the principal cost of aggressive optimization**: v001's latency gains on get operations are negated by hit-rate regression and put-latency overhead. Both metrics together matter more than either in isolation; workloads reward capacity preservation over operation-level acceleration.

## Open questions

- Would adaptive threshold divisor 3 on v002's optimal 80/20 geometry combine the hit-rate and latency gains of both v002 and v003, or would the composition again encounter interference?
- Is there an even finer-tuned divisor (e.g., between 3 and 4) that optimizes the adaptive formula for mid-range geometries, or does divisor 3 represent a stable attractor?
- Can the formula divisor itself be made adaptive (scaled by geometry ratio) to achieve universal optimality across all allocations, or is each geometry band best served by a different constant?

## Continuation

The optimal geometry is now localized to the 75/25–80/20 range; stop exploring beyond 80/20. The key breakthrough in this cycle is that adaptive threshold formula refinement (divisor 3) unlocks clean composition by increasing promotion permissiveness on balanced geometry without incurring trade-offs. Next cycle should test whether divisor 3 composes with v002's 80/20 geometry to yield the best of both: hit-rate and latency gains from geometry plus gating refinement. If that composition succeeds, the optimization frontier has been pushed forward and future cycles can explore entirely new axes (e.g., eviction policies, bucket-size tuning, or tier-crossing heuristics beyond frequency). If composition at the optimal geometry still encounters interference, the evidence will be strong that simple geometry-only variants are the most reliable path forward and that frequency gating, despite its theoretical promise, has been fully optimized on this workload. Prioritize the divisor 3 + 80/20 composition test as the definitive next step.
