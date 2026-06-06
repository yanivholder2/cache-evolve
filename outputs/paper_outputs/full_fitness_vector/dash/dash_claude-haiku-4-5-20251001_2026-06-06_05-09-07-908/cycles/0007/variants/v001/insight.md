# Variant v001 Insight

## Helped

- **hit-rate**: Doubling the promotion threshold from 1 to 2 filters out marginal single- and low-frequency items that v001's aggressive geometry (85/15) was indiscriminately promoting, recovering the hit-rate lost to over-promotion cascades.

- **per-op-time (get)**: Fewer false-positive promotions reduce the churn of eviction cascades that stall get operations waiting for bucket reorganization, yielding marginal latency gains.

- **per-op-time (put)**: Reduced cascading demotion chains between tiers (probing → target → stash) lower the per-operation cost of managing items that should never have promoted in the first place.

## Hurt

None.

## Takeaway

Increasing selectivity (threshold 1→2) on aggressive geometry successfully recovers what the threshold decrease sacrificed: the promotion gate now enforces stricter frequency requirements, allowing only truly hot items to occupy the scarce probing tier. This transforms the geometry–threshold tradeoff from a loss (accelerate latency at the cost of hit-rate decay) into a win, proving that aggressive cache composition can coexist with precision selectivity when the two are properly tuned together.
