# Variant v003 Insight

## Helped

- None. hit-rate is the only reported dimension, and it did not improve on the parent.

## Hurt

- hit-rate: The hypothesis ported the one-bit second-sighting admission filter into the parent. It was meant to add to the parent's gains, but hit-rate came out marginally below the parent's. Skipping insertion on a key's first miss added no value on top of what the parent already gets from its own admission and eviction behavior.
- Unmeasured: no metadata-size or per-op-time dimension was reported, so the cost of the per-segment bit array is not visible. I could not read the overlay sources, so the diff is inferred from the hypothesis text.

## Takeaway

Two changes that each improved hit-rate on workload-a did not stack. Here the filter did not add to the parent's changes, and it cost a little. The hypothesis assumed the filter's benefit was independent of the parent's change, and that assumption failed. Before porting the filter into another variant, test whether that variant already blocks one-hit-wonders by other means.
