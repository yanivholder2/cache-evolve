# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three frequency-tuning strategies were tested on an S3-FIFO baseline. Only the conservative increase to MAX_FREQ (baseline 3 → 4) succeeded; the cohort reveals that S3-FIFO's frequency-based eviction thresholds are finely calibrated, and improvements come from modest adjustments aligned with observed reuse patterns rather than aggressive re-weighting of queue membership.

## Patterns

- **Conservative tuning outperforms aggressive re-weighting**: v001 gained hit-rate through a single-level frequency depth increase, while v002 lost hit-rate despite offering the same intuition (ghost hits are proven reusable) simply because it granted protection upfront rather than requiring it to be earned. This suggests the system's frequency balance is tight; items must justify their protection through actual reuse evidence, not queue history.

- **Reuse-pattern alignment matters more than queue mechanics**: The winner (v001) succeeded by matching the frequency mechanism to moderate-reuse workload behavior—items with sustained but not-frequent access patterns now get one more eviction round. In contrast, v003's dynamic promotion threshold (stricter filtering under congestion) had no measurable impact, implying workload-a rarely hits sustained main-queue congestion at the 80% threshold, or the filtering strategy misaligns with its actual access dynamics.

- **Frequency depth and intermediate reuse exist in tension**: v001's success indicates the baseline MAX_FREQ=3 was evicting items too early that still showed moderate reuse value. Adding one level (MAX_FREQ=4) captures that value without destabilizing the frequency balance, as v002's attempted boost to ghost-hit frequency demonstrated.

## Open questions

- Why did v003 (dynamic promotion threshold) show no measurable effect? Does workload-a avoid sustained main-queue congestion, or does the 80% threshold + frequency requirement mismatch its typical access patterns?
- What is the optimal MAX_FREQ for different workload reuse-depth distributions, and can it be inferred from workload characteristics?
- Can ghost-hit protection be earned rather than gifted—e.g., by tracking re-admission patterns—to achieve v002's intuition without the crowding penalty?

## Continuation

v001's success suggests the frequency-depth dimension remains unexplored; test MAX_FREQ values beyond 4 to find the point of diminishing returns. Drop v002's approach of blanket frequency boosts; instead, investigate whether queue-load adaptive strategies (different than v003's static 80% threshold) can tune promotion selectivity to match real congestion dynamics. Consider whether workload characterization (reuse-depth histogram, access burstiness) could predict the optimal MAX_FREQ a priori, reducing future cycle search space.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

The cohort tested whether frequency-depth scaling and structural admission-gate tuning could build on Cycle 0001's MAX_FREQ success. The winner (v003) pivoted from depth to structure: tightening the small queue admission gate from 1/4 to 1/5 of capacity. This structural selectivity outperformed continuing the frequency-depth axis (v001's marginal 4→5 gain) and confirmed that ghost-hit frequency credits remain a false lead (v002).

## Patterns

- **Admission selectivity now outperforms frequency depth**: v003's tight small-queue gating (1/5 capacity, enforcing stricter frequency filtering to promote to main) delivered clear hit-rate gains, while v001's extension of MAX_FREQ from 4 to 5 showed only marginal improvement. This inverts the Cycle 0001 lesson: frequency depth was the winning lever then, but structural parameters controlling *who gets to compete for main-queue slots* now deliver more value.

- **Frequency-depth dimension exhibits diminishing returns**: The gap between v001's improvement (4→5) versus Cycle 0001's (3→4) is markedly smaller, suggesting the frequency-depth axis is approaching saturation. The system may be approaching a regime where absolute reuse depth matters less than the selectivity of item admission.

- **Ghost-hit heuristics remain counterproductive**: v002's attempt to grant modest earned protection (freq=1 on re-admission) again failed to improve hit-rate, just as Cycle 0001's more aggressive variant (freq=2) did. The pattern is consistent: prior survival does not predict future reuse. The baseline approach—treating ghost hits as cold (freq=0) and letting them earn frequency through actual re-access—aligns better with the workload's access dynamics.

## Open questions

- How far can small-queue reduction go? Does the selectivity gain continue below 1/5, or is there a utilization cliff where the queue becomes too constrained to absorb working-set diversity?
- Why do ghost-hit frequency credits consistently fail? Is it that ghost hits in workload-a lack correlation with future reuse, or that the frequency mechanism cannot distinguish between "earned" and "gifted" protection?
- Can small-queue sizing be adaptive or workload-aware (e.g., scaled by reuse-depth distribution) rather than fixed structural constants?

## Continuation

Drop ghost-hit frequency crediting entirely—two cycles of evidence suggest it misses the workload's access patterns. Shift focus to structural selectivity: test further small-queue reductions (1/6, 1/7) to find the admission-gate sweet spot, and consider whether reuse-depth distribution or burstiness could inform optimal sizing *a priori*. Retire the frequency-depth axis (MAX_FREQ beyond 4-5 appears to yield diminishing returns); any remaining improvements likely come from admission gating or frequency promotion thresholds *within* the small queue, not absolute depth expansion.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

Three refinements were tested: further tightening the small-queue admission gate (1/5 → 1/6), raising the frequency threshold for small-to-main promotion (freq>1 → freq>2), and combining structural selectivity with frequency-depth (1/5 + MAX_FREQ=4). The winner (v001) continued Cycle 0002's structural-selectivity trend, confirming that shrinking the small-queue admission barrier remains the dominant improvement lever. v003 validated that frequency-depth and structural selectivity are genuinely orthogonal axes, though v001's selectivity gain proves larger in this cycle.

## Patterns

- **Structural selectivity (small-queue reduction) remains the dominant lever**: v001's further tightening from 1/5 to 1/6 delivered clear hit-rate gains, while v002's attempt to raise the promotion frequency threshold (freq>1 → freq>2) hurt performance. This clarifies that the selectivity dimension is *size-based admission gating*, not frequency-threshold tuning. The workload continues to reward fewer but better-filtered items in the small queue.

- **Frequency-depth and structural selectivity are orthogonal but unequally weighted**: v003 confirmed that increasing MAX_FREQ from 3 to 4 while keeping the 1/5 gate produced gains, validating Cycle 0001's frequency-depth finding. However, v001's size-based reduction outperformed v003, indicating the current small-queue capacity is the higher-leverage axis. The two dimensions compound but do not carry equal importance.

- **Promotion selectivity via frequency threshold is a dead-end**: v002's stricter frequency gate contradicts the Cycle 0002 insight that "admission selectivity works." The difference is mechanistic: v002 tightens the *graduation threshold* (who leaves small for main), while v001/Cycle 0002 v003 tighten the *entry capacity* (how many can occupy small). Size-based filtering outperforms frequency-based filtering at this decision point.

## Open questions

- How far can small-queue reduction continue? Does the gain persist from 1/6 to 1/7 or 1/8, or is there a utilization cliff approaching where the queue cannot absorb sufficient working-set diversity?
- Can v001's structural gain (1/6) and v003's frequency gain (MAX_FREQ=4) compound into a single variant? If so, is the combined improvement larger than either individually?
- Why does frequency-threshold selectivity (v002) fail while size-based selectivity succeeds? Is it that frequency signals are too coarse-grained at the small-queue promotion stage, or that the frequency mechanism cannot support this decision point?

## Continuation

Drop frequency-threshold tuning at the promotion gate (v002-type experiments); three cycles of evidence—Cycles 0001 v002 (freq boost on ghost hits), 0002 v002 (freq credits), and 0003 v002 (stricter promotion frequency)—consistently show frequency-based selectivity fails at this scale. Continue aggressive small-queue reduction (test 1/7, 1/8) to locate saturation, and combine the best small-queue size with MAX_FREQ=4 to verify orthogonal compounding. If gains continue below 1/6, investigate whether reuse-depth distribution analysis could predict optimal sizing *a priori* for future workloads.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

Three refinements were tested on the small-queue structural-selectivity axis: incremental reduction to 1/7, orthogonal compounding with frequency-depth (1/6 + MAX_FREQ=4), and aggressive reduction to 1/8. The winner (v003) pushed structural selectivity further, confirming that the saturation boundary for admission-gate tightening remains unlocated. Stricter small-queue filtering continues to outperform both incremental steps and attempts to compound orthogonal optimizations, indicating the workload still sits well below its utilization cliff.

## Patterns

- **Aggressive structural selectivity outperforms incremental and orthogonal approaches**: v003's jump from 1/6 to 1/8 delivered larger gains than v001's incremental 1/6→1/7 step and v002's orthogonal combination of 1/6 + MAX_FREQ=4. Four consecutive cycles (0002 v003, 0003 v001/v003, 0004 v003) confirm that small-queue reduction remains the single most effective lever; the workload rewards increasingly selective admission filtering without sign of diminishing returns.

- **Saturation boundary is further out than expected**: Both v001 (1/7) and v003 (1/8) report positive hit-rate impact with no performance cliff. The workload demonstrates robustness to aggressive under-provisioning of the small queue—items demonstrating genuine reuse still graduate successfully to main-queue storage, and frequency selectivity (freq > 1 for promotion) effectively separates signal from noise even at tighter capacities.

- **Orthogonal compounding works but remains subordinate to structural selectivity**: v002's combination of 1/6 structural selectivity + MAX_FREQ=4 improved hit-rate and validated that frequency-depth and admission-gate sizing are genuinely orthogonal axes. However, v003's larger gains from going to 1/8 confirm that structural selectivity is currently the higher-leverage dimension—frequency-depth offers complementary but smaller absolute improvements at this stage.

## Open questions

- How far can small-queue reduction continue? Does the trend persist at 1/9 or 1/10, or does 1/8 approach the saturation cliff where working-set diversity becomes over-constrained?
- If v003's 1/8 reduction is paired with MAX_FREQ=4, do the two optimizations compound to deliver even larger gains than either achieved alone?
- Can workload characteristics (reuse-depth histogram, inter-arrival burstiness) predict the optimal small-queue ratio *a priori*, allowing future cycles to target the saturation boundary directly rather than search incrementally?

## Continuation

The structural-selectivity axis remains dominant and has not yet hit saturation. Aggressively push small-queue reduction to find the utilization cliff: test 1/9 and 1/10 to locate the boundary where performance degrades. Simultaneously, test v003's 1/8 + MAX_FREQ=4 combination to confirm whether aggressive structural selectivity and frequency-depth compound synergistically at the boundary. Once saturation is mapped, shift focus to workload-adaptive or learned admission policies that might exceed fixed-ratio boundaries, or investigate whether the observed selectivity gains transfer to different workload patterns.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Three refinements continued the structural-selectivity axis: incremental reduction to 1/9, aggressive reduction to 1/10, and orthogonal compounding of 1/8 with MAX_FREQ=5. The winner (v002) pushed small-queue reduction to 1/10 and delivered measurable improvement, extending the monotonic trend unbroken across four consecutive cycles. Notably, neither incremental reduction (v001) nor orthogonal compounding (v003) outperformed the pure structural play, confirming that tight admission-gate selectivity remains the dominant and unsaturated lever.

## Patterns

- **Structural selectivity shows no saturation sign despite five reduction steps**: The progression 1/4→1/5→1/6→1/8→1/9→1/10 has yielded consistent hit-rate improvements with no degradation or cliff. v002's aggressive 1/10 jump delivered the largest gain, and v001's incremental 1/9 step also improved despite smaller magnitude. This is a remarkable pattern: the workload continues to reward increasingly selective admission filtering at a scale (1/10 of capacity) that would be expected to trigger utilization collapse. The fact that items with genuine reuse still graduate successfully through the frequency gate suggests the workload exhibits exceptionally strong reuse locality—signal and noise are clearly separable even at extreme selectivity.

- **Diminishing returns have not yet appeared**: v001 notes "diminishing magnitude" compared to prior cycles, yet still improved. This is consistent with a gradual approach to saturation rather than an abrupt cliff. The workload appears to tolerate over-constrained small-queue sizing without failing to capture the working set, indicating the access patterns are skewed enough that frequency-based promotion (freq > 1) effectively filters repeaters even under extreme pressure.

- **Frequency-depth compounding remains complementary but subordinate**: v003's combination of 1/8 structural selectivity + MAX_FREQ=5 showed "modest but consistent improvement" and validated orthogonal compounding once more. However, v002's pure structural approach (1/10) outperformed v003, reaffirming that at this stage, tight admission-gate sizing is the higher-leverage axis. The pattern is now five cycles deep: frequency-depth and structural selectivity are orthogonal, but structural selectivity is the dominant lever by a wide margin.

## Open questions

- **Where is the true saturation boundary?** All variants this cycle improved; does the trend continue at 1/11 or 1/12, or do we find degradation soon? The absence of a cliff at 1/10 is surprising and warrants aggressive boundary search.

- **Does saturation manifest abruptly or gradually?** If saturation exists, does performance degrade sharply (utilization cliff) or gently (frequency signals become too coarse to distinguish reuse)? This affects how aggressively future cycles should search.

- **Is the extreme selectivity (1/10+) specific to this workload, or indicative of a deeper architectural mismatch?** The workload's tolerance for 1/10-ratio admission suggests either exceptional reuse clustering or that S3-FIFO's frequency mechanism is detecting signal in regimes where many cache policies would fail. Could a different admission policy (e.g., learned, adaptive, or frequency-weighted from the start) exceed fixed-ratio boundaries?

## Continuation

The structural-selectivity axis remains dominant and has not yet hit saturation—continue the aggressive boundary search. Test 1/11 and 1/12 to locate the actual utilization cliff or confirm that selectivity gains persist even further. Once saturation is mapped with confidence, investigate whether workload characteristics (reuse-depth histogram, burstiness, skew) can predict the optimal ratio *a priori*, enabling the next workload's cycles to target the saturation boundary directly rather than search incrementally. If saturation occurs beyond 1/12, pivot to adaptive or learned admission policies that might dynamically adjust selectivity per-phase rather than rely on fixed structural constants.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

Three refinements continued the structural-selectivity search: incremental reduction to 1/11, orthogonal compounding of 1/10 with MAX_FREQ=4, and aggressive reduction to 1/12. The winner (v003) extended the monotonic improvement trend unbroken across seven consecutive cycles, confirming that the admission-gate saturation boundary remains unexplored and that aggressive selectivity continues to outperform both incremental steps and frequency-depth compounding.

## Patterns

- **Aggressive structural selectivity decisively outperforms incremental and orthogonal variants**: v003's jump from 1/10 to 1/12 delivered the largest gain, surpassing v001's incremental 1/11 step and v002's frequency-depth compounding. This reinforces the consistent pattern from Cycles 0002–0005: pure structural tightening of the admission gate is the single highest-leverage optimization axis. The workload exhibits remarkable tolerance for extreme selectivity—items with genuine reuse still graduate to main-queue storage through the freq > 1 threshold even at 1/12 capacity, indicating the access patterns are skewed enough that frequency-based filtering remains effective under severe under-provisioning.

- **No saturation cliff appears even at 1/12**: The progression 1/4→1/5→1/6→1/8→1/9→1/10→1/11→1/12 has now yielded seven consecutive improvements with zero degradation. The fact that v003's aggressive jump outperformed v001's incremental step suggests the saturation boundary may lie significantly tighter than 1/12. The workload continues to reward selectivity-at-scale without showing sign of utilization collapse, pointing to either exceptional reuse clustering or that frequency-based filtering can separate signal from noise in regimes where many cache policies would fail.

- **Frequency-depth remains complementary but subordinate even at the 1/10 frontier**: v002's MAX_FREQ=4 addition to the proven 1/10 selectivity delivered only modest improvement (+0.0009), reaffirming that structural selectivity is the dominant lever by a wide margin. Eight cycles of evidence now show that frequency-depth and admission-gate sizing are orthogonal but unequally weighted; at current selectivity levels, extending item lifetime in the main queue offers marginal gains compared to stricter admission filtering.

## Open questions

- **Can aggressive reduction continue beyond 1/12?** Does the trend persist at 1/13 or 1/14, or have we finally approached the saturation cliff? The absence of degradation at 1/12 is consistent with prior cycles and warrants further aggressive probing.

- **Does 1/12 pair synergistically with frequency-depth scaling?** Cycle 0005 tested 1/8+MAX_FREQ=5 but 1/12+MAX_FREQ=4 has not been tested. Could tighter structural selectivity unlock greater compounding with frequency-depth at an even more constrained boundary?

- **Is this extreme selectivity workload-specific, or indicative of S3-FIFO's broader potential?** The tolerance for 1/12-ratio admission is exceptional and warrants investigation: could reuse-depth histograms or burstiness metrics predict whether such extreme selectivity is optimal *a priori*, or does this workload represent an outlier in access-pattern skew?

## Continuation

The structural-selectivity axis remains dominant and unambiguously unsaturated—continue the aggressive boundary search without hesitation. Test 1/13 and 1/14 to rapidly locate the actual utilization cliff or confirm that gains persist even further. Once saturation is confidently mapped, investigate whether combining the tightest viable structural selectivity with higher MAX_FREQ values (e.g., 1/12+MAX_FREQ=4 or 1/12+MAX_FREQ=5) can unlock synergistic compounding at the boundary. If structural gains plateau, pivot toward adaptive or learned admission policies that might dynamically adjust selectivity per-phase rather than rely on fixed structural ratios—this workload's pattern suggests it may benefit from policies that respond to observed reuse-depth distributions in real-time.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Three refinements tested the structural-selectivity frontier: incremental reduction to 1/13, aggressive reduction to 1/14, and orthogonal compounding of 1/12 with MAX_FREQ=5. For the first time across eight cycles, the winner (v002) was not the most aggressive structural variant, but rather the pairing of proven 1/12 selectivity with deepened frequency protection. This marks the saturation boundary's arrival—the inflection point where the dominant optimization axis shifts from pure admission-gate tightening to structure-and-frequency compounding.

## Patterns

- **Saturation boundary localized to the 1/12–1/14 range**: v001's 1/13 reduction yielded only +0.0003 marginal gain, the smallest improvement magnitude across eight consecutive structural-selectivity cycles, signaling diminishing returns. v003's 1/14 improved without degradation, confirming the workload tolerates extreme selectivity but with sharply reduced impact. Together, these variants narrow saturation to a precise zone: improvements continue, but the slope flattens dramatically compared to earlier cycles (1/4→1/10 progression).

- **Bottleneck shifts from admission gate to main-queue retention at extreme selectivity**: v002's pairing of 1/12 + MAX_FREQ=5 outperformed both v001 (incremental structural, +0.0003) and v003 (aggressive structural, 1/14). This reversal from the prior seven cycles' pure-structural dominance reveals that at 1/12 selectivity, the admission gate filters so aggressively that only the highest-confidence items reach the main queue. Further tightening the gate yields minimal additional benefit; instead, extending the tenure of proven items in the main queue delivers better returns. The workload has crossed a threshold where frequency-depth protection becomes the higher-leverage secondary axis.

- **Frequency-structure compounding validates at the saturation boundary**: v002's modest but measurable gain from pairing 1/12 + MAX_FREQ=5 confirms that frequency-depth and structural selectivity compound synergistically—but only once the admission gate reaches extreme tightness. In Cycles 0005–0006, frequency-depth additions (1/8+MAX_FREQ=5, 1/10+MAX_FREQ=4) offered complementary but marginal improvements subordinate to pure structural tightening. Now, at 1/12, frequency protection becomes a legitimate primary lever. This timing shift indicates the workload's optimization landscape is discontinuous: structural selectivity is dominant until saturation, then frequency-depth control becomes competitive.

## Open questions

- **Where is the precise saturation cliff?** v003's 1/14 shows no degradation despite diminishing returns; does the true utilization collapse occur beyond 1/14, or is the boundary softer than prior patterns suggest (gradual decay rather than cliff)?

- **Can 1/13 and 1/14 unlock greater gains when paired with MAX_FREQ=5 or higher?** v002's success with 1/12+MAX_FREQ=5 raises the question: would 1/13+MAX_FREQ=5 or 1/14+MAX_FREQ=5 (or even MAX_FREQ=6) deliver better results than 1/12's pairing? The compounding effect may enable tighter structural selectivity to remain viable.

- **Is 1/12+MAX_FREQ=5 a local optimum or global?** v002 won by a narrow margin against pure-structural v003. Is the peak somewhere else in the structure×frequency interaction space—say, 1/11+MAX_FREQ=5, or 1/12+MAX_FREQ=6?

## Continuation

The structural-selectivity axis has hit saturation between 1/12 and 1/14; stop pushing pure structural reduction. The next cycle must search the structure×frequency interaction space near the saturation boundary. Test whether tighter structural selectivity (1/13, 1/14) compounds synergistically with deeper frequency protection (MAX_FREQ=5, MAX_FREQ=6), or fine-grain variants around 1/12±1 paired with MAX_FREQ=5+ to locate the true optimal point. If 1/12+MAX_FREQ=5 proves robust, pivot to workload-characterization or learned admission policies that might adapt per-phase rather than rely on fixed structural constants—this workload's transition from structure-dominant to frequency-dominant suggests it may benefit from dynamic policy selection or adaptive boundaries.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

Three refinements tested the structure–frequency interaction space at the saturation boundary: incremental structural reduction to 1/13 paired with MAX_FREQ=5, extended frequency depth to MAX_FREQ=6 at proven 1/12 selectivity, and extreme structural selectivity at 1/14 paired with MAX_FREQ=5. The winner (v003) confirmed that aggressive admission filtering remains viable at the saturation cliff when paired with sufficient frequency protection, reversing the prior cycle's apparent saturation signal.

## Patterns

- **Extreme structural selectivity saturates only when frequency protection is insufficient**: Cycle 0007 showed 1/14 alone yielding marginal returns, suggesting structural saturation. Cycle 0008's v003 (1/14 + MAX_FREQ=5) reverses this conclusion: the apparent saturation was not a structural ceiling but rather a mismatch between aggressive admission filtering and shallow main-queue retention. Items passing the 1/14 gate are high-utility, and extending their tenure unlocks the potential that tight selectivity created. The workload's reuse clustering is strong enough to sustain both extreme admission filtering and extended item lifetime simultaneously.

- **Frequency-depth protection is the higher-leverage axis at the saturation boundary**: v002's extension from MAX_FREQ=5 to MAX_FREQ=6 yielded a measurable +0.0014 gain, substantially larger than v001's incremental structural step (+0.0002). This reversal mirrors Cycle 0007's inflection point, where compounding became the dominant axis. However, Cycle 0008 reveals the asymmetry: within the saturation zone, deepening frequency protection scales more favorably than further tightening the admission gate. The gains remain positive but diminishing in both dimensions—yet frequency depth still has unlocated leverage.

- **Structure and frequency form a coherent, discontinuous optimization surface**: Nine cycles of evidence now reveal a non-monotonic landscape: admission-gate selectivity dominates until saturation (Cycles 0002–0006), frequency protection becomes competitive at the boundary (Cycles 0007–0008), and their pairing compounds synergistically—but *only* if the admission gate is tight enough. v003's success over v002 and v001 indicates the peak lies beyond 1/12 selectivity, contingent on sufficient frequency depth. This suggests workload-aware policy tuning requires joint optimization, not sequential refinement of isolated dimensions.

## Open questions

- **What is the true limit of the structure–frequency compounding effect?** v003 wins with 1/14 + MAX_FREQ=5, but can even tighter structural selectivity (1/15, 1/16) sustain performance if paired with deeper frequency (MAX_FREQ=6, MAX_FREQ=7)? The gains remain positive at both extremes; the boundary may lie significantly further out.

- **Is there a peak in the structure×frequency interaction space, or does compounding scale indefinitely?** v002's gain (+0.0014) and v001's (+0.0002) suggest gains are still available, but can we characterize the trade-off surface to predict optimal pairing? Finer-grain search (1/12–1/14 × MAX_FREQ 5–7) may reveal whether there is a local optimum or a monotonic frontier.

- **Can reuse-depth histograms or clustering metrics predict the optimal structure–frequency pairing *a priori*?** This workload has now demonstrated that extreme selectivity (1/14) is viable with appropriate frequency protection. Do access-pattern characteristics (skew, burstiness, reuse clustering) enable prediction of the saturation boundary and optimal pairing for a new workload without iterative search?

## Continuation

The structure–frequency interaction space remains unsaturated and productive. Stop searching the margins of single dimensions; instead, systematically explore the joint structure×frequency space near and beyond the current winner (1/14 + MAX_FREQ=5). Test whether even tighter structural selectivity (1/15, 1/16) compounds with elevated frequency depth (MAX_FREQ=6, MAX_FREQ=7), or whether a more finely resolved grid around the current winner (1/13–1/14 × MAX_FREQ=5–6) reveals a sharper optimum. Once the true saturation boundary is confidently mapped in the two-dimensional space, investigate whether workload introspection (reuse-depth histogram, skew metric, burstiness) can predict the optimal point *a priori*, or pivot toward learned or adaptive policies that dynamically adjust selectivity and frequency based on observed access patterns.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

Three explorations probed the boundaries of the structure–frequency interaction space: incremental structural tightening to 1/15 paired with proven MAX_FREQ=5, extended frequency depth to MAX_FREQ=6 at extreme 1/14 selectivity, and a radically different trade-off pairing moderate 1/12 structure with aggressive MAX_FREQ=7. Both winners—the tighter structural variant and the deeper-frequency variant—yielded measurable gains, confirming the saturation zone remains productive in both dimensions. The loser decisively revealed that the optimization surface is asymmetric: aggressive frequency depth cannot recover performance lost by relaxing structural selectivity, indicating the peak lies in the extreme-structural region, not at moderate structure with extreme frequency.

## Patterns

- **Extreme structural selectivity remains the dominant axis even at saturation**: Both winners improved over the prior baseline, but v001's extension to 1/15 selectivity (paired with MAX_FREQ=5) and v002's frequency deepening to MAX_FREQ=6 (at proven 1/14) continued the pattern of nine consecutive cycles with zero degradation. This reaffirms that the workload rewards aggressive admission filtering throughout the measurable range. v003's failure—backing off to 1/12 with 2.33× deeper frequency protection—underscores that the optimization surface is not a smooth trade-off curve; items that never clear the tight admission gate cannot benefit from extended main-queue tenure, making structural selectivity the foundational axis and frequency depth a secondary multiplier dependent on structural aggressiveness.

- **Frequency-depth leverage plateaus at diminishing return magnitudes**: v002's +0.0003 gain from MAX_FREQ=6 represents the smallest improvement magnitude across the last three cycles, a near-halving of v001's +0.0007 or the prior cycle's +0.0014. This decay in frequency-depth returns suggests the main-queue retention mechanism is approaching saturation; items passing the extreme 1/14 gate are so carefully curated that further extending their tenure yields marginal benefit. The axis remains productive but at a steeply declining slope.

- **The optimization surface exhibits a discontinuous peak in the extreme-structural, moderate-frequency region**: v003's decisively larger loss (compared to the marginal wins of v001 and v002) demonstrates the surface is sharply peaked: the cost of relaxing structural selectivity from 1/14 to 1/12 is not recoverable by any frequency configuration tested to date. This mirrors Cycle 0007's inflection-point detection but with reverse asymmetry: then, frequency became "competitive"; now, frequency is revealed as subordinate to structure, unable to invert a poor structural choice. The peak appears to lie near or beyond the current 1/14–1/15 region with MAX_FREQ=5–6, not at broader structural ratios however deep their frequency protection.

## Open questions

- **Has structural selectivity finally hit its ceiling, or does 1/16–1/20 remain viable?** v001's +0.0007 gain at 1/15 is measurable but tiny; is the true saturation cliff beyond, or have we already crossed the peak?

- **What is the interaction-space peak in the extreme-structural zone?** The winners suggest different optimization directions (v001: deeper structure, v002: deeper frequency); is there a finer-grained optimum at 1/14.5-sized ratios (if continuous perturbation were possible) or combinations like 1/15+MAX_FREQ=6 not yet tested?

- **Why does the workload punish moderate structural selectivity so severely?** v003's loss with 1/12+MAX_FREQ=7 indicates that the reuse-depth clustering is so pronounced that items below the extreme admission threshold are predominantly noise; even with very deep frequency protection, they consume main-queue resources without value. Does this workload's access-pattern signature (histogram, skew, or temporal clustering metric) uniquely predict this regime?

## Continuation

Stop exploring moderate structural ratios; the asymmetry is now conclusive. Focus the next cycle on finely mapping the extreme-structural region (1/14–1/16) paired with moderate frequency depths (MAX_FREQ=5–7) to locate the true optimum. Test at least one unexplored pairing (e.g., 1/15+MAX_FREQ=6 or 1/14+MAX_FREQ=7) to check for interactions beyond the current grid, but commit to a dense sweep of 1/14–1/16×MAX_FREQ=5–7 if the next cycle yields saturating results. Once the peak is confidently mapped, pivot to characterizing the workload's access-pattern signature (reuse-depth histogram, inter-arrival clustering, frequency distribution) to test whether the extreme-selectivity requirement is predictable from raw access logs, or investigate learned/adaptive policies that dynamically adjust selectivity based on observed miss-pattern evolution in real-time rather than relying on fixed constants.


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
