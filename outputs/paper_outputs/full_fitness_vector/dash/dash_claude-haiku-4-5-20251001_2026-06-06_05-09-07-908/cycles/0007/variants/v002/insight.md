# Variant v002 Insight

## Helped

- **hit-rate**: The 80/20 geometry doubles the normal bucket allocation (26 vs ~13 in baseline), reducing hash collisions and providing more tier-climbing opportunities; the divisor-3 adaptive threshold lowers the promotion barrier (threshold=2 for 6 stash buckets), allowing more items to reach the probing tier and remain cached longer.
- **per-op-time**: The improved hit-rate concentrates access at higher-tier buckets with better locality, while the 80/20 allocation spreads items across more buckets, reducing lookup depth and contention; p99 latency especially benefits from fewer worst-case hash collisions.

## Hurt

(none)

## Takeaway

Composing 80/20 geometry with divisor-3 adaptive promotion threshold successfully delivers simultaneous hit-rate and latency gains without regression. The geometry change alleviates the stash bottleneck by distributing capacity toward the normal tier, while the more permissive threshold formula ensures the expanded tier can absorb frequent items. Both dimensions move in the desired direction, confirming that these orthogonal improvements integrate rather than interfere.
