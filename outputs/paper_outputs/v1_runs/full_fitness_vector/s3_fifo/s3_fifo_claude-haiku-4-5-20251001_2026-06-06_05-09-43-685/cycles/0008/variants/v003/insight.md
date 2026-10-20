# Variant v003 Insight

## Helped

- **per-op-time (get, p99)**: Occupancy-gated ghost pruning reduces the ghost population, lowering the cost of ghost-queue searches during get operations and reducing contention on hot-path operations.

## Hurt

- **hit-rate**: Occupancy-gated retention with aggressive thresholds (freq >= -1 at >80% occupancy) discards too many warm candidates, shrinking the ghost pool below what the workload needs to sustain hits; occupancy-responsive re-admission boost cannot compensate for fewer available ghost entries.
- **per-op-time (put, p99)**: Multiple occupancy ratio computations (division + branching) in the small queue eviction loop and ghost hit path add expensive conditional logic to the hot path, causing tail latency to spike significantly when the small queue fills frequently.

## Takeaway

Occupancy-gated ghost retention creates a false trade-off: by aggressively pruning the ghost queue under load, it reduces ghost search cost but decimates hit rate by starving the re-admission pipeline of warm candidates. The workload pattern does not benefit from adaptive ghost contraction; instead, it needs consistent, generous ghost retention to preserve the warm-candidate pool. The cost of computing occupancy thresholds and applying them to every eviction decision compounds the regressive effect, turning put operations into expensive branching-heavy code paths.
