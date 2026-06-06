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
