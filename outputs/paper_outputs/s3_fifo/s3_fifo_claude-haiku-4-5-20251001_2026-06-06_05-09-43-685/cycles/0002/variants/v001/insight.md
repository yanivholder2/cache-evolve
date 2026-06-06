# Variant v001 Insight

## Helped

- **per-op-time (put p50)**: Raising the promotion threshold to freq > 3 reduced average put latency by 88ns, indicating fewer items in the main queue decreases typical eviction overhead during insertions.

## Hurt

- **per-op-time (get p99)**: Tail get latency degraded from 157ns to 237ns, suggesting higher-frequency accesses now encounter more contention or processing work at the 99th percentile.
- **per-op-time (put p99)**: Tail put latency increased from 1502ns to 1616ns, indicating the stricter filter creates uneven performance on outlier operations despite improving the median.

## Takeaway

Pushing the promotion threshold from freq > 2 to freq > 3 trades median put efficiency for tail latency unpredictability. The stricter admission filter reduces main-queue churn enough to improve typical put operations, but the reduced diversity of frequencies in the main queue causes outlier operations to face more consistent blocking. Hit-rate remains negligible, confirming that the Zipfian workload tolerates requiring one additional access to earn main-queue residency. This variant reveals that tightening the admission gate beyond the freq > 2 sweet spot gains average case performance at the cost of tail latency predictability.
