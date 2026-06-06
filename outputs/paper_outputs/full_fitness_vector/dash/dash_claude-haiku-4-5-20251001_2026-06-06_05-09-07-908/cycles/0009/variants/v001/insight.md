# Variant v001 Insight

## Helped

(none)

## Hurt

- **hit-rate**: Reducing normal bucket allocation from 80% to 79% increased eviction pressure in the normal tier, causing more items to be dropped before reaching the stash; the 21% stash allocation failed to compensate for the loss in hit-rate.
- **per-op-time**: The tighter normal bucket packing (fewer buckets with proportionally more items per bucket) increased hash collision rates, adding latency to get operations without improving put latency.

## Takeaway

The 80/20 geometry from v001 represents a local optimum that does not tolerate further refinement toward higher stash ratios. Reducing normal bucket allocation by even 1 percentage point degrades both hit-rate and latency, indicating that the v001 geometry is already at the frontier of the safe band and that stash-biased adjustments move backward toward the saturation boundary rather than toward improvement.
