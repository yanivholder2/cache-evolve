# Variant v001 Insight

## Hurt

- **hit-rate**: Expanding stash capacity from 4 to 6 buckets (while reducing normal buckets from 28 to 26) decreases hit rate; the 7% reduction in normal tier capacity outweighs the benefit of extended stash buffering.

## Takeaway

Extending the stash's filtering window by reallocating capacity from the normal tier does not improve hit-rate and actually harms it. This suggests the cache's performance ceiling is not constrained by insufficient stash buffering time, but by a deeper architectural factor—likely the interaction between insertion position, eviction policy, and how items progress through tiers. Rebalancing toward a wider stash without addressing tier positioning mechanics is counterproductive.
