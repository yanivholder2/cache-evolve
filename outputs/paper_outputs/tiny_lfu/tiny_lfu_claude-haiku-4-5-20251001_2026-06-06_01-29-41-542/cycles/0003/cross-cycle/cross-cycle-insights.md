# Cross-Cycle Insights

## Recurring wins

- **Direct frequency precision improvements yield gains**: Sketch width (Cycle 0001) and depth (Cycle 0002) both improved hit-rate; width delivered larger returns but entered diminishing returns earlier than depth, suggesting orthogonal but saturable improvement axes.

- **Intelligent use of existing signals beats working around them**: Cycle 0003's successful eviction victim selection (frequency-recency hybrid) leveraged imperfect frequency estimates directly, outperforming attempts to bypass or partition them (Cycle 0003 v003's dual-tier fast-path).

- **Simple, interpretable policies outperform domain-specific optimization tricks**: Cycle 0003's straightforward age/frequency scoring formula for eviction victim selection defeated both saturation-recycling optimization (v001) and structural redesigns (v003), confirming that clarity and directness win over targeted complexity.

- **Multi-dimensional tuning of orthogonal parameters is productive**: Cycle 0002's combination of width and depth improvements (v003) confirmed that sketch parameters are largely independent; gains stack but plateau as each axis saturates.

## Recurring failures

- **Working around measurement uncertainty breaks admission discipline**: Cycle 0003's dual-tier fast-path (bypassing the sketch for hot items to reduce collision noise) regressed performance despite the intent to reduce noise. Admission logic that evades frequency signals, even imperfect ones, undermines selectivity.

- **Aggressive manipulation of frequency signals removes genuine predictive value**: Cycle 0001's frequency decay (v002) assumed stale frequencies were noise, but periodic halving removed signal too aggressively; early-earned high frequencies remain predictive. Cycle 0003's counter saturation recycling (v001) assumed overflow was a problem, but it proved inert—absolute precision loss outweighs gains from continued accumulation.

- **Relaxing gate selectivity under uncertainty harms precision**: Cycle 0001's relaxation from strict `>` to `>=` for admission allowed tied frequency estimates to evict hot residents based on recency alone, confirming that the baseline's strictness protects hot items during uncertainty rather than being overly restrictive.

## Open questions

- **Can eviction victim selection formulas be further tuned?** Cycle 0003's v002 hybrid succeeded with a simple age/frequency metric; testing alternative exponents, offsets, and weights could extract additional gains.

- **What is the optimal sketch width-to-depth balance, or do they saturate independently?** Cycle 0002 treated them as orthogonal, but the interaction and saturation thresholds are unclear.

- **Are alternative frequency-estimation structures (beyond Count-Min) worth pursuing?** Cycles 0001–0002 saturated sketch parameter tuning; a fundamentally different approach might escape this plateau.

- **What non-frequency signals should be integrated into victim selection?** Item size, reuse distance, access contention, and cost could complement age and frequency; Cycle 0003's success suggests the eviction layer is now the productive axis.

- **How stable is Cycle 0003's eviction improvement across workloads, cache sizes, and access patterns?** Current success is workload-specific; generalization requires cross-validation.

## Per-policy notes

- **Sketch parameters (Cycles 0001–0002)**: Width > Depth in initial impact; width diminishing returns (4→8: +0.0047, 8→16: +0.0006); depth productive but modest. Counter saturation handling inert on current workload; deprioritize unless saturation becomes frequent on other datasets. Measurement noise was the primary constraint; it has been substantially reduced via width and depth.

- **Admission gate (Cycles 0001, 0003)**: Strictness (`>` not `>=`) is sound. Attempts to relax or bypass it (Cycle 0001 v003, Cycle 0003 v003) backfire. Gate is not the limiting factor; sketch precision and eviction policy are.

- **Eviction victim selection (Cycle 0003)**: Frequency-recency hybrid (v002) is productive; direct scoring formula beats saturation optimization or structural redesigns. Eviction is now the active lever; the admission and sketch layers have been tuned near their limits.
