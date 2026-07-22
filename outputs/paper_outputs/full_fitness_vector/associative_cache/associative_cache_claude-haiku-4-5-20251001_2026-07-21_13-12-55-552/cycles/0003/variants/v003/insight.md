# Variant v003 Insight

## Helped

- **hit-rate**: Extending to 32×64 buckets yielded minimal additional hit-rate gain (+0.0005), suggesting that hit-rate improvements from wider buckets have largely saturated and consolidating beyond 64×32 yields diminishing returns.

## Hurt

- **per-op-time**: Tail latencies (p99) degraded significantly: get operations went from 100ns to 200ns (+100%) and put operations from 200ns to 300ns (+50%), as ClassicLRU's O(n) intra-bucket scan became more expensive on 64-item buckets compared to 32-item buckets in v002.

## Takeaway

The hypothesis that hit-rate improvements would continue as buckets widened from 64×32 to 32×64 encountered a wall. While v002 demonstrated a favorable trade-off where hit-rate gains outweighed latency costs, v003 reveals that the frontier has shifted: even though hit-rate barely moved, the tail-latency penalty from scanning longer buckets with a linear-search eviction policy grew sharply. The cache has reached an inflection point where continuing to widen buckets worsens performance without commensurate benefit.
