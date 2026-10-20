# Variant v001 Insight

## Helped

- **hit-rate**: Compressing `prob_cap` from `capacity/8` to `capacity/32` (96.875% protected) continued the monotonic improvement trend: more capacity held in the protected LRU means frequently-accessed items under Zipfian access are less often demoted back to probationary and subsequently evicted.

## Hurt

- **per-op-time**: The summary rose from 0 to 100; however, the parent's p50_ns of 0 for get is a physically implausible measurement-floor artifact, not a true signal — the current variant's 100 ns p50 represents a genuine timing reading. The change itself (replacing a constant divisor `8` with `32`) has no plausible mechanism to increase per-operation cost.

## Takeaway

Further squeezing probationary capacity to `capacity/32` still lands on the beneficial side of the quota-compression curve, delivering another hit-rate gain, but the absolute improvement is visibly smaller than the gains from prior halving steps, confirming the policy is well into diminishing-returns territory and that a reversal point is approaching. The per-op-time shift is attributable to the parent's anomalous zero-floor p50 rather than to the divisor change.
