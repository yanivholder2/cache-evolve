# Variant v002 Insight

## Helped

- **hit-rate**: Doubling the stash capacity from 10% to 25% extends the recovery window for evicted items, capturing more working-set re-use from hash collisions before FIFO eviction discards them entirely.

## Hurt

(None measured in this evaluation)

## Takeaway

Increasing stash capacity from 10% to 25% improves hit rate, validating the hypothesis that a larger recovery buffer reduces permanent loss of evicted data. However, the modest 1.2% improvement despite a 2.5× capacity increase suggests diminishing returns—evicted items with true re-use are largely captured by the smaller buffer, and most additional stash capacity provides negligible benefit on this workload.
