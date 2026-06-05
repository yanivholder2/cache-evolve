# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Adaptive hit-rate monitoring with interval adjustment adds per-operation overhead (hit/miss counter updates, periodic hit-rate calculations) that exceeds the performance gains from earlier decay triggering; the workload shows no significant phase shifts requiring reactive decay.

## Takeaway

Replacing the fixed capacity × 16 decay interval with adaptive hit-rate monitoring (triggering decay on 5% hit-rate drops and adjusting interval bounds between 16–24) produces a slight regression. The fixed interval was already well-tuned for this workload; the adaptive mechanism's overhead and complexity do not yield sufficient benefit. This suggests the cache's performance bottleneck lies elsewhere—either in frequency granularity, eviction selection, or workload-specific access patterns—rather than in the decay trigger timing itself.
