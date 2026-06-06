# Variant v003 Insight

## Helped

- **hit-rate**: Dynamic small-queue sizing allows more items to persist during low-load phases (occupancy ≤ 0.5), increasing their chance to reach the freq > 2 threshold and graduate to the main queue, improving overall hit-rate from 0.7162 to 0.7248.

## Hurt

- **per-op-time (get)**: Occupancy tracking and dynamic resizing logic on every put operation introduces complexity that degrades get latency; get p50 increased 36ns and p99 increased 52ns, likely due to struct size growth and overhead of the occupancy-aware path.
- **per-op-time (put)**: The occupancy-aware resizing and explicit eviction during shrinking (lines 97–117) add latency in worst-case scenarios; put p99 increased 86ns, outweighing the marginal p50 improvement.
- **metadata-size**: Tracking three new fields (`small_size_min`, `small_size_max`, `current_small_size`) plus `capacity` instead of the single static `small_size` increased struct size from 112 to 136 bytes (+24 bytes).

## Takeaway

Dynamic occupancy-aware queue sizing successfully boosts hit-rate by adapting small-queue capacity to load phases, but the added complexity and metadata overhead make it a latency trade-off. The variant demonstrates that while adaptive queue bounds improve cache effectiveness (more items graduate during low load), the per-operation cost of occupancy tracking and resizing logic is non-trivial. The hit-rate gain (1.2%) comes at the cost of degraded operation latencies across both get and put—a reminder that structural tuning levers require careful measurement when overhead is introduced on every operation path.
