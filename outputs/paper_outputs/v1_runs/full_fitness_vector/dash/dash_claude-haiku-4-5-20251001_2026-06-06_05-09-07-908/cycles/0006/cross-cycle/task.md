# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants—back-insert promotion, geometry rebalancing, and Item struct compaction—proved successful with no negative tradeoffs. Together they reveal three independent optimization axes: promotion ordering strategy, hash-indexed capacity allocation, and memory layout efficiency. All three winners focused on hot-path efficiency improvements aligned with Zipfian workload characteristics.

## Patterns

- **Orthogonal improvements across layers**: The winners operate at three distinct levels (promotion logic, bucket geometry, struct packing) with no apparent conflicts, suggesting they could potentially compose.
- **Common theme of reducing unnecessary work**: v001 eliminates redundant LRU climbs; v002 eliminates unnecessary stash traversals; v003 eliminates memory pressure from sparse Item structs. All three avoid expensive operations on the hot path.
- **Strong alignment with workload skew**: All improvements exploit the Zipfian nature of the trace, where a small set of hot keys dominates. Back-insert keeps hot items resident longer; geometry rebalance catches cold misses more often; packing improves cache efficiency for repeated accesses.
- **Latency and hit-rate move together**: Rather than trading one for the other, improvements span both dimensions—wider bucket allocation improves hit-rate _and_ reduces traversal latency; tighter Item packing helps _both_ get and put operations.

## Open questions

- Do these three optimizations compose, or do they interact in ways that diminish returns?
- How do these strategies perform on non-Zipfian workloads (uniform or bimodal distributions)?
- Is the 16/16 bucket split the optimal sweet spot, or is there further geometry tuning to explore?

## Continuation

The first cycle's clean wins across distinct optimization axes suggest the baseline left room on multiple fronts. Next cycle should test whether these optimizations stack (e.g., v001 + v002 + v003 together) and explore other orthogonal dimensions: alternative eviction policies, different promotion thresholds, or adaptive geometry tuning. Confirm whether these patterns hold under diverse workload distributions.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

This cycle tested composition of cycle-0001's orthogonal optimizations, further geometry tuning, and a new promotion-filtering dimension. All three variants proved valuable, though with distinct trade-off profiles: back-insert + 50/50 geometry trades latency for hit-rate; aggressive 62.5/37.5 geometry improves both metrics; frequency-gated promotion improves both metrics by filtering false-positive tier transitions.

## Patterns

- **Geometry allocation remains the primary lever**: The progression from baseline (25/75) through 50/50 to 62.5/37.5 shows sustained improvement, confirming that hash-indexed capacity is consistently more valuable than stash capacity on this Zipfian workload. The allocation axis has not yet saturated.

- **Promotion gating is orthogonal and cost-free**: Frequency-gated target→probing promotion (counter ≥ 2 threshold) improves both hit-rate and latency by eliminating false-positive promotions that trigger cascading demotions. This represents a new optimization axis with no apparent downside.

- **Composition can regress latency despite design orthogonality**: While back-insert promotion and 50/50 geometry are mechanically independent, combining them incurs latency overhead from increased insertion work that offsets the hit-rate gain. This suggests careful sequencing matters even when dimensions are logically distinct.

- **Reducing cache churn improves both throughput and accuracy**: v003's success demonstrates that preventing wasteful tier transitions (fewer false-hot items) simultaneously reduces eviction cascades (improving latency) and focuses capacity on genuinely hot keys (improving hit-rate). This reinforces cycle-0001's theme of eliminating unnecessary work on the hot path.

## Open questions

- Do aggressive geometry (62.5/37.5) and frequency gating compose without regression?
- Is 62.5/37.5 the asymptote for geometry allocation, or does further concentration continue to improve both metrics?
- Can the frequency threshold be tuned adaptively (e.g., learned per workload), or is counter ≥ 2 near-optimal?

## Continuation

Geometry and promotion gating emerged as independent, complementary levers. Next cycle should test their composition (v002 + v003 together) and explore adaptive or workload-specific tuning of the frequency threshold. The fact that a simple counter-based gate achieved clean wins suggests the promotion logic has further refinement room beyond just threshold tuning—consider whether adaptive thresholds, per-bucket counters, or different gating strategies for different tiers could yield additional gains.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

This cycle tested composition of geometry and frequency gating, plus further geometry concentration. Two strategies succeeded: frequency-gated promotion combined with aggressive (62.5/37.5) geometry, and pushing geometry even further to 75/25 allocation. The third variant (frequency gating on balanced 50/50 geometry) revealed that the gate's threshold interacts poorly with less-concentrated geometry. The winning variants confirm that geometry remains the primary performance lever, while frequency gating amplifies its benefits only on appropriately tuned configurations.

## Patterns

- **Frequency-gating success depends on underlying geometry**: v002's gating achieved both latency and hit-rate wins on 62.5/37.5 geometry, but v001's identical gate hurt hit-rate on 50/50 geometry. The threshold of 2 is too conservative for balanced geometry (filters too many promotion candidates) but well-calibrated for more concentrated geometry (filters mostly false-positives from the smaller stash tier). This suggests the gate's selectivity must match the geometry's candidacy rate.

- **Geometry asymptote continues to shift upward**: The progression from baseline (25/75) through 50/50, 62.5/37.5, to 75/25 shows sustained improvement across both hit-rate and latency. The cycle-0002 hypothesis that 62.5/37.5 might be saturating was incorrect; 75/25 delivers cleaner wins on both dimensions without apparent regression, suggesting the concentration axis has further room.

- **Composition behavior is non-linear across parameter space**: While geometry and frequency gating were mechanically expected to compose orthogonally, the outcome varies sharply with their specific values. This is a departure from cycle-0001's clean stacking and cycle-0002's earlier synergies, indicating the optimization landscape has entered a region where parameter interactions matter as much as axis independence.

- **Simple geometry beats complex layering when both win**: v003 improves hit-rate, get latency, and put latency simultaneously using only geometry reallocation—no additional promotion logic. v002 achieves similar gains using gating + geometry. When both strategies succeed, the simpler one (v003) suggests the workload's optimization margin is primarily unlocked by capacity allocation, not promotion filtering.

## Open questions

- Does frequency gating (threshold 2) on v003's 75/25 geometry compose without regression, or will the gate again filter too few false-positives to justify overhead?
- Is 75/25 the asymptote, or does further geometry concentration (e.g., 85/15, 90/10) continue to improve both metrics?
- Would an adaptive frequency threshold (scaled to geometry allocation ratio) eliminate the composition sensitivity and make gating orthogonal across all geometry settings?

## Continuation

Geometry concentration has proven to be the consistent performance lever across all three cycles, with no saturation observed yet. Next cycle should test whether frequency gating works on v003's 75/25 geometry and explore the asymptote question by pushing geometry even more aggressively. Consider whether the frequency threshold should scale adaptively with geometry concentration (e.g., threshold = max(2, floor(stash_buckets / 4))) to stabilize gating's selectivity across different allocations. If adaptive thresholds prove effective, they could unlock clean composition across the parameter space; if not, prefer geometry-only variants as they deliver reliable wins with less tuning burden.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

This cycle tested three strategies on an increasingly concentrated bucket geometry: frequency-gated promotion on 75/25 geometry, an even more aggressive 85/15 geometry shift, and back-insert promotion composition. All three variants succeeded in at least one metric. Geometry extension to 85/15 delivered clean wins across both hit-rate and latency; frequency-gating improved hit-rate but incurred a latency cost; back-insert achieved the same hit-rate as its parent but with latency regression. The clear standout is the pure-geometry variant, confirming that capacity reallocation remains the dominant performance lever.

## Patterns

- **Geometry asymptote remains unblocked**: The progression from baseline (25/75) through 75/25 to 85/15 shows sustained improvement in both hit-rate and latency, with the latest geometry delivering 7.3-point hit-rate gains and meaningful latency improvements across all percentiles. The prior cycle's hypothesis that geometry might be saturating was again disproven.

- **Composition complexity continues to diverge from mechanical independence**: While frequency-gating (v001) and back-insert (v003) are logically orthogonal to geometry, their empirical outcomes differed sharply. Gating improved hit-rate (confirming cycle 0003's selectivity scales to tighter allocations) but introduced measurable get-latency overhead. Back-insert—despite composition with the most aggressive normal-tier allocation yet—failed to improve any metric and regressed latency on both get and put, suggesting the MRU-insertion cost is truly fundamental and cannot be absorbed by larger primary-tier capacity.

- **Simple variants outperform when all win**: v002 (pure geometry 85/15) achieved the cleanest wins across both dimensions—no trade-offs, no latency penalty. When multiple variants succeed, the mechanically simpler approach consistently delivers better overall outcomes. This aligns with cycle 0003's observation that geometry-only variants are more reliable than composition-based strategies.

- **Frequency-gating selectivity scales well to concentrated geometry**: v001's successful hit-rate improvement confirms that the counter ≥ 2 threshold remains well-calibrated even on 75/25 (and now, implicitly, 85/15-adjacent) geometry, filtering mostly false-positives in a shrinking stash tier. The trade-off suggests hit-accuracy and latency form a frontier under this parameter regime.

- **Back-insert's overhead is structural, not capacity-dependent**: v003's consistent latency regressions on both get and put operations, despite being paired with the most generous hash-indexed allocation tested, indicate the MRU-insertion mechanism carries an inherent cost that no amount of primary-tier capacity can overcome.

## Open questions

- Is 85/15 the asymptote, or do even more aggressive allocations (90/10, 95/5) continue to improve both metrics without saturation?
- Could frequency-gating's latency penalty be reduced by tuning the threshold downward (e.g., threshold = 1) without losing hit-rate selectivity on concentrated geometry?
- Why does back-insert fail so consistently across all tested geometries (50/50, 62.5/37.5, 75/25, 85/15), and are there alternative promotion-insertion strategies that avoid this overhead?

## Continuation

Geometry concentration has proven to be the most reliable and composable optimization axis across all four cycles, with no saturation observed and no apparent downside. Next cycle should push geometry even further (e.g., 90/10 or beyond) to identify the true asymptote and understand whether diminishing returns eventually appear. Back-insert should be dropped as a composition strategy—it is fundamentally costly and incompatible with this workload across all tested configurations. If pursuing frequency-gating refinement, focus on threshold tuning to minimize latency impact while preserving hit-rate gains. Otherwise, geometry-only variants remain the safest bet for consistent, clean improvements. Consider whether entirely different optimization axes (beyond promotion logic and bucket allocation) might unlock further gains on top of aggressive geometry.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

This cycle tested three refinements: lowering frequency-gating threshold to 1 on concentrated geometry, pushing geometry allocation to its asymptote (90/10), and implementing adaptive threshold scaling based on stash bucket count. Geometry concentration finally saturated at 90/10, revealing the boundary where hit-rate losses outweigh latency gains. Frequency-gating threshold tuning on fixed geometry proved counterproductive, but adaptive threshold computation succeeded, establishing a path toward geometry-independent gating that composes robustly across allocations.

## Patterns

- **Geometry asymptote identified at ~85/15–90/10**: The four-cycle progression of sustained improvement (25/75 → 50/50 → 62.5/37.5 → 75/25 → 85/15 → 90/10) finally encounters saturation. v002's 90/10 geometry improves get and put latency but regresses hit-rate as stash capacity shrinks to 3 buckets—insufficient for the Zipfian workload's tail. This definitively closes the question: geometry concentration is not a bottomless lever. The sweet-spot appears to be around 75/25–85/15, where both metrics improve; pushing past 85/15 optimizes for latency at the expense of hit-rate.

- **Hardcoded threshold tuning is geometry-specific, not universal**: v001's attempt to lower the frequency gate from threshold=2 to threshold=1 on 75/25 geometry demonstrates that the prior cycle's selectivity (threshold=2 on concentrated geometry) was already near-optimal. Reducing it further floods the 8-bucket probing tier with false-positives, causing collision chains and hit-rate dilution. The threshold is not a knob to turn independently; it is tightly coupled to the stash tier's capacity.

- **Adaptive threshold computation unlocks cross-geometry composition**: v003's breakthrough is shifting from a hardcoded constant (`PROMOTION_THRESHOLD: u16 = 2`) to a geometry-aware initialization parameter (`max(1, stash_buckets / 4)`). This single change eliminates the sensitivity to allocation profile: on 25/75 (24 stash buckets), the threshold becomes 6; on 75/25 (8 stash), it remains 2. By scaling the gate's selectivity with the stash size, v003 achieves 42–43% get-latency improvement and 9–19% put-latency improvement while holding hit-rate stable—proving that gating can become orthogonal to geometry when parameters adapt.

- **Simple adaptive logic beats complex per-geometry tuning**: v003 demonstrates that a single geometric scaling formula outperforms cycles of manual threshold calibration across different allocations. This shift from hardcoded constants to initialization-time parameter computation is mechanically trivial but strategically powerful: it removes the human coupling between geometry and promotion logic, enabling reliable composition.

- **Hit-rate remains the harder metric to preserve**: Across all five cycles, hit-rate degradation appears whenever either geometry or gating pushes too far (e.g., 90/10 or threshold=1). Latency improvements alone (v002's get/put gains) do not compensate for hit-rate regression on Zipfian workloads, confirming the prior insight that capacity utilization is more valuable than operation-level speed on skewed access patterns.

## Open questions

- Does adaptive threshold on aggressive geometry (e.g., 85/15 or a sweet-spot near 80/20) deliver better composition than v003's baseline results, potentially reclaiming some of the hit-rate margin?
- Is there an optimal geometry between 75/25 and 90/10 (e.g., 80/20 or 82/18) that maximizes both hit-rate and latency?
- Can the adaptive scaling formula itself be tuned (e.g., `stash_buckets / 3` or `stash_buckets / 5`) to shift the trade-off frontier, or is division-by-4 near-universal?

## Continuation

Adaptive parameter scaling is the key insight: frequency-gating selectivity must track geometry allocation to remain orthogonal, and this can be achieved via a simple initialization-time computation rather than per-operation overhead. Next cycle should test adaptive threshold on aggressive geometries (85/15 or a candidate sweet-spot like 80/20) to determine whether v003's composition success generalizes and to identify whether a geometry better than 75/25 exists. The geometry asymptote is now localized to the 85/15–90/10 range; stop pushing geometry further and focus instead on finding the optimal allocation within that band, combined with adaptive gating. If adaptive thresholds on aggressive geometry maintain hit-rate while improving latency, the composition strategy is vindicated and future cycles can explore entirely different optimization axes.


### Cycle 0006

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
