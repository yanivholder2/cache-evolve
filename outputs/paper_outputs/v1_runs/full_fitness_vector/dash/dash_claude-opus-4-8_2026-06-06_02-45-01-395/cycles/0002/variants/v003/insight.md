# Variant v003 Insight

## Helped

- hit-rate: Replacing v001's single-MRU gate with the `max(1, len/4)` top-quarter window in `is_in_promotion_window` lets consistently-2nd/3rd-hottest target keys reach the probing tier, protecting a few more warm keys — enough to edge hit-rate above v001 without the churn that sank v002's ungated promotion.
- per-op-time: Promoting a wider slice of warm keys out of target buckets into probing keeps the target buckets shorter, so the per-hit `get_position` scans get cheaper — get and put p50/p99 all dropped relative to v001.

## Hurt

- (none) — metadata-size is untouched; the window test reads existing bucket length and ordering and adds no per-item state.

## Takeaway

The single-item promotion gate was indeed too tight: a proportional top-quarter window, expressed against the bucket's own `promotion_candidate_index` rather than a hardcoded LRU end, is the middle ground cycle-0001 asked for — it lifts hit-rate over v001 while shorter target buckets also trim latency. The gain is small, suggesting the bulk of protectable warm mass on this trace is still near the MRU head and a quarter-window captures most of what a tight gate misses.
