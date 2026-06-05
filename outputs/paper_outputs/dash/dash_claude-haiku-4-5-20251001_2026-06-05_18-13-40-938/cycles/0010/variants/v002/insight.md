# Variant v002 Insight

## Helped

- **hit-rate**: Expanding the stash tier to 25% of total buckets (from 12.5%) gives items more time to demonstrate their access pattern before entering competitive hot tiers, reducing false-positive promotions and allowing better-informed placement decisions.

## Hurt

_(none)_

## Takeaway

Widening the stash filtering tier validates that the original baseline was promoting items too eagerly without sufficient evidence of hotness. By allocating a larger fraction of cache capacity to characterization rather than optimizing purely for hot-tier size, the cache makes higher-confidence promotion decisions that improve overall hit rate. This suggests the 4-bucket stash was a bottleneck in the decision pipeline, not a superfluous tier.
