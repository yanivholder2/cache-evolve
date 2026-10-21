# Variant v001 Insight

## Helped

## Hurt

## Takeaway

Adding a target-hit bar of `PROBING_THRESHOLD = 3` in front of the target→probing edge left hit-rate flat; the change against v003 is within noise. The hypothesis expected this edge to pay off the way v003's stash filter did, and it did not. The bar barely filters anything. The counter is never reset while an item sits in its target, so every hit counts toward it, including hits that don't come while the item is hottest. Under Front landing an item already needs at least two hits to reach probing, so the bar adds only one more for genuinely reused keys. Foreign residents pushed back from probing are pre-credited and skip the bar entirely. The cascade churn this edge causes therefore seems too small on this trace for frequency gating to change it. Future gains are more likely from how evictions are routed than from stricter admission into probing. This was measured on one LIRS trace at one cache size only.
