# Variant v001 Insight

## Helped

- **metadata-size**: Halving the sketch width factor from 4 to 2 reduces the CountMinSketch heap by 50% (49,152 → 24,576 bytes), delivering the expected memory savings proportional to the width reduction.

## Hurt

- **hit-rate**: The narrower sketch (width-2) causes 1.2% hit-rate degradation (0.7354 → 0.7264), substantially exceeding the 0.3% tolerance threshold and indicating that width-2 increases hash collision rates too severely, degrading frequency estimates and eviction quality.
- **per-op-time (get p99)**: Tail latency for get operations increases 8% (208ns → 225ns), likely due to cache-unfriendly memory access patterns in the smaller sketch and increased collision resolution overhead.
- **per-op-time (put p99)**: Tail latency for put operations increases 9% (400ns → 437ns), reflecting poor frequency estimates from sketch collisions leading to suboptimal victim selection and reallocation overhead.

## Takeaway

Width-2 breaches the practical lower bound established by cycle 0007's width-3 success. The 50% reduction in sketch geometry causes unacceptable hit-rate loss (1.2%, well beyond the tolerance window) alongside tail-latency degradation, confirming that wider sketches are necessary to maintain collision-sparse frequency estimates. For this workload, width-3 (factor-3 or factor-4) sits at the efficiency frontier; narrowing further trades unacceptable hit-rate penalties and latency spikes for modest memory savings that do not justify the performance regression.
