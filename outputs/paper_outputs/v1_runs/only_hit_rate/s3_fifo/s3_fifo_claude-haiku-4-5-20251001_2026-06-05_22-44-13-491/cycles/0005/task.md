# Task: cycle-insight-writer

Cycle: 0005
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-44-13-491/cycles/0005

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-44-13-491/cycles/0005/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-44-13-491/cycles/0005/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-44-13-491/cycles/0005/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-44-13-491/cycles/0005/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-44-13-491/cycles/0005/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-44-13-491/cycles/0005/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
