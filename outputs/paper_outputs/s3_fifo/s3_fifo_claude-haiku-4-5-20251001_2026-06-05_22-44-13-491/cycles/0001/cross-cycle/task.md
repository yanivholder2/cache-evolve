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
