# Variant v002 Insight

## Helped

- **hit-rate**: Removing the two-tier hot-side gatekeeping (target → probing advancement) and collapsing to a single unified "hot" tier allows items to be promoted directly from stash to hot on access, reducing filtering overhead and improving cache hit performance.

## Hurt

(none)

## Takeaway

The architectural simplification from a three-tier hierarchy (stash + target + probing) to two tiers (stash + hot) proves effective for this workload. By eliminating the intermediate target filtering step and allowing direct stash-to-hot promotion, items reach the hot tier with less gatekeeping overhead, resulting in improved hit rate. This suggests the workload's access pattern benefits from a simpler promotion path rather than multi-stage filtering.
