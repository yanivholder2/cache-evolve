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
