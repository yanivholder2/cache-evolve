# Cycle 0009 Insight

## Cohort outcome

This cycle tested three distinct refinement strategies on the established geometry-and-threshold framework: further geometry fine-tuning toward stash (79/21 from the 80/20 optimum), a new orthogonal eviction policy axis using frequency-aware item ordering, and geometry-aware divisor co-optimization with larger stash capacity (75/25 with divisor 4). v001's geometry push failed, confirming the 80/20 optimum is locally stable. v002 and v003 both succeeded, but via divergent mechanisms: v002 through implementation efficiency (substantial latency gains without hit-rate cost) and v003 through parameter co-optimization (stricter divisor paired with larger stash capacity, preserving hit-rate while improving latency).

## Patterns

- **The 80/20 geometry is definitively locally optimal**: v001's single-percentage movement toward stash (79/21) degraded both hit-rate and latency compared to the 80/20 baseline. This confirms that the mid-range band identified in prior cycles (75/25–80/20) contains a sharp peak at 80/20 itself; movement in either direction incurs cost. The frontier is not further refinable by incremental geometry tuning.

- **Latency gains can be decoupled from hit-rate through two independent paths**: v002 demonstrates that implementation optimization—replacing vector-shuffling with in-place counters—delivers 57% faster get operations and 18% faster put operations without any hit-rate change. v003 demonstrates that geometry-aware parameter co-optimization (larger stash capacity paired with stricter divisor gating) delivers similar latency improvements while preserving hit-rate. These are mechanically orthogonal solutions to the same latency constraint.

- **The adaptive divisor formula continues to exhibit inverse coupling with geometry aggressiveness**: v003's success on 75/25 geometry with stricter divisor 4 confirms and extends the pattern from cycle 0007–0008: more stash capacity permits and requires stricter gating to avoid overwhelming the tier. The formula is not geometry-independent; it is geometry-aware and composition-sensitive.

- **Frequency information does not improve decision quality on this workload**: v002's key insight is that frequency-aware eviction adds implementation complexity without providing additional cache decision advantage. Recency and frequency are well-correlated on workload-a, making pure LRU sufficient. This closes an exploratory branch: frequency-biased eviction is a dead-end for this access pattern.

- **The optimization space is converging on multi-factor improvements**: Unlike cycle 0006–0007, where composition attempts often failed, both v002 and v003 succeed without regression. All three winners maintain or improve latency; v001's failure is not a regression but a confirmation of boundary conditions. This signals the optimization space has matured sufficiently to support independent improvements on orthogonal axes.

## Open questions

- Is v001's failure purely due to geometry, or does the divisor choice (2 on 79/21) also contribute? Could a different divisor on 79/21 succeed?
- Among v002 (efficiency) and v003 (geometry-formula), which represents the more sustainable optimization path? Is there a strict ordering, or do they occupy a Pareto frontier?
- Can v002's implementation efficiency and v003's geometry-aware formula be combined, or do they compete for the same latency-improvement budget?

## Continuation

The geometry frontier is now tightly bounded and locally stable at 80/20; stop pursuing incremental geometry refinement. Frequency-aware eviction is confirmed ineffective on this workload—abandon this axis. The two competing paths forward are (1) pure implementation optimization for latency (v002) and (2) geometry-aware parameter co-optimization (v003). Both succeed without hit-rate cost, meaning the optimization space now permits orthogonal gains. Next cycle should test whether these two approaches compose: can v002's efficient eviction implementation be applied to v003's 75/25 geometry with adaptive divisor to combine latency and hit-rate improvements, or does their interaction reveal interference? If composition succeeds, the result becomes the new optimized baseline. If not, choose the variant with the strongest overall outcome (hit-rate + latency balance) as the new standard and shift exploration entirely away from geometry and thresholding toward new axes (e.g., tier-crossing heuristics, promotion cascading policies, or workload-specific tiering strategies).
