# Variant v002 Insight

## Helped

- **hit-rate**: Extending MAX_FREQ from 3 to 4 allows frequently-accessed items in the main queue to survive one additional eviction round before permanent discarding, enabling a modest hit-rate gain when paired with the proven 1/10 selectivity threshold.

## Hurt

(none)

## Takeaway

Adding frequency-depth (MAX_FREQ=4) to the 1/10 selectivity boundary yields a modest improvement (+0.0009), confirming that frequency-based persistence contributes to performance but does not compound significantly at this high-selectivity frontier. The result indicates that structural admission selectivity remains the dominant optimization lever on this workload; frequency-depth scaling extends item survival at the margin but does not unlock major gains. The two dimensions are complementary but not synergistic at the 1/10 boundary—tight admission filtering captures the right items, and extending their main-queue persistence provides small incremental benefit without saturating the cache.
