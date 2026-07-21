# Variant v001 Insight

## Helped

- **hit-rate**: Switching stash-to-target insertion from `Front` to `Back` places a freshly promoted item immediately at the MRU end, making it an instant promotion candidate for the probing tier on the very next access — so items that are genuinely hot escape the stash-to-probing warm-up in two hits instead of requiring a full LRU-to-MRU climb first.
- **per-op-time**: Items reaching the sticky probing tier sooner reduces churn through the stash-promote-demote cascade; more gets terminate in a shallow probing-bucket lookup rather than triggering a multi-step stash-removal + target-insertion sequence.

## Hurt

*(no dimensions moved negatively)*

## Takeaway

Changing the single `InsertPosition` argument in `DashSegment::get` from `Front` to `Back` confirms that the baseline's LRU-end placement was the bottleneck preventing recently promoted items from stabilising quickly in the probing tier. The compound improvement in both hit-rate and per-op-time on workload-a shows that MRU-end insertion accelerates not just the individual item's warm-up path but also reduces overall eviction churn, because hot items stop cycling back through stash before earning residency in probing.
