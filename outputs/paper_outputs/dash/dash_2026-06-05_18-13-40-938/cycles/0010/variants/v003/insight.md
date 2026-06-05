# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Per-bucket saturation detection at 80% occupancy triggers premature evictions, reducing effective cache capacity and removing items before their utility window closes.

## Takeaway

Bucket-level saturation at 80% is too aggressive a signal. The proactive evictions in `promote_to_target` and `promote_to_probing` remove residents before buckets naturally fill, effectively shrinking the working set that can be retained. Unlike H01 and H02 which refined *when* and *where* items flow, H03's early-eviction strategy attacks a symptom (impending saturation) rather than addressing the root bottleneck. The parent's 0.684 plateau persists, now lower, confirming that bucket occupancy alone is not a reliable trigger for improving hit rate—the constraint lies elsewhere in how items are selected for and routed through tiers.
