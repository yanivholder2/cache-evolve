# Variant v003 Insight

## Helped

- (none measurable): The hypothesis targeted per-op-time (single index computation, early-exit scan, minimal-range rotate promotion, reuse of the miss-scan for victim choice). The fitness vector has no per-op-time dimension, so any speedup is not visible in the evaluation.

## Hurt

- (none): hit-rate is unchanged from the parent. This is the required result for a refactor that keeps the policy bit-for-bit.

## Takeaway

This variant restructured the lookup, promotion and insert paths of the rotate-based segmented policy without changing which entries are hit or evicted, and hit-rate stayed identical to the parent. That confirms the refactor preserved the policy. The evaluation only reports hit-rate, so it can't tell whether the cost-side goal was met. A per-op-time measurement has to be in the fitness vector before this kind of change can be judged. I could not inspect the overlay source in this session, so I haven't verified how the changes were implemented.
