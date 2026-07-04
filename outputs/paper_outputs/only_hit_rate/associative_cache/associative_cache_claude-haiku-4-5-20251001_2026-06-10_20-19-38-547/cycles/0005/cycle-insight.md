# Cycle 0005 Insight

## Cohort outcome

This cycle tested refinements at the geometric and filtering boundaries of the proven design (0.71 reduction factor + 25% stash). Both directions regressed: increasing geometric aggressiveness to 0.75 overshot the collision-reduction optimum, and collision-aware stash filtering eliminated valuable recoveries without net gain. v003's relative victory (despite hurting hit-rate) reflects convergence to a local optimum; the parent design appears to be the better-positioned point in the combined geometric-capacity-filtering design space.

## Patterns

- **Local optimum is stable**: The parent design's 0.71 geometric factor and unconditional 25%-capacity stash insertion represent a local equilibrium. Pushing geometry more aggressive (0.75) and filtering more selective (collision-aware threshold) both moved away from peak performance, indicating the system has settled into a well-balanced configuration with limited room for refinement within these axes.

- **Unconditional capture remains optimal**: v003 tested collision-aware selective insertion, filtering evictions from low-contention buckets to reduce stash pollution. The result contradicts the hypothesis—blanket FIFO insertion at 25% capacity captures all meaningful re-uses, and collision frequency does not predict recovery value. This replicates Cycle 0003's occupancy-filtering finding with a different signal; filtering is not a productive axis regardless of the signal chosen.

- **Geometric refinement has tightened bounds**: v001's regression when pushing from 0.71 to 0.75 confirms that collision-reduction gains have saturated. The design space is now constrained; v002 from Cycle 0004 (0.71) sits near the peak, and both higher and lower directions yield diminishing or negative returns. Further geometric exploration requires narrower, more expensive searches.

- **Architectural constraints continue to block capacity exploration**: v002's build failure (borrow checker error on stash insertion sequencing) persists as the third instance across cycles. Stash capacity scaling beyond 25%—a direction validated earlier—remains technically blocked, leaving the capacity saturation frontier unmapped.

## Open questions

- Is the geometric optimum precisely at 0.71, or is there a fractional refinement (0.68–0.70 or 0.72–0.73) that yields incremental gains within a narrower range?
- Can the architectural constraint preventing stash capacity scaling be resolved through refactoring, or does it reflect a fundamental limit of the bucket-level locking model?
- Has the workload-cache pairing exhausted its exploitable margin, or do entirely different levers (workload phase adaptation, eviction policies orthogonal to FIFO, collision-reduction through alternative bucket-layout strategies) remain?

## Continuation

v003's win, though accompanied by regression, confirms convergence to a local optimum around 0.71 and 25% capacity. Both geometry refinement (pushing to 0.75) and filtering (collision-aware threshold) moved away from peak, indicating the parent design is well-positioned. Next cycle should pursue one of two paths: (1) **Fine-grained local search**: test geometric factors in the 0.68–0.73 range at smaller increments to locate the precise peak and verify whether small adjustments yield marginal gains within the convergence zone, or (2) **Escape the local optimum**: accept that stash-geometry tuning has saturated and pivot to fundamentally different optimization axes (workload phase detection and dynamic adaptation, alternative collision-reduction strategies orthogonal to bucket width, or adaptive eviction policies beyond FIFO). Resolve the borrow checker constraint if pursuing stash-capacity exploration, but deprioritize it unless fine-grained local search confirms the need for larger capacity to support new geometric configurations.
