# Variant v001 Insight

## Helped

- **per-op-time (put)**: Occupancy-responsive thresholds in the shrinking logic (lines 105-113) improve small queue promotion decisions, reducing eviction churn and shortening put operations by ~7ns.

## Hurt

- **per-op-time (get)**: The added occupancy ratio computation (division + branching) in `evict_small()` incurs per-operation overhead that manifests in get latency, increasing p50 by 6ns because gets trigger `evict_small()` whenever the small queue fills.

## Takeaway

Restoring occupancy-responsive thresholds with finer granularity (0.4, 0.7) successfully implements the hypothesis, but the occupancy-ratio division and conditional logic required at every small-queue eviction introduces computational overhead that exceeds the hit-rate benefit. The hit-rate remains essentially flat (0.7223 vs 0.7222), falling far short of the predicted recovery, revealing that adaptive threshold mechanisms are more expensive per-operation than their marginal improvement in decision quality for this workload pattern.
