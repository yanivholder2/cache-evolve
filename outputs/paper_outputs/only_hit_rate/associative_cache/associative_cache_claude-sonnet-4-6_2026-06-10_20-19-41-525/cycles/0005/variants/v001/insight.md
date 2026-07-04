# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Replacing `remove_at(0)` with a minimum-counter scan after the decay pass caused a small regression — the scan can select a higher-index (more recently inserted) item whose counter happened to land lowest after decay, sparing the index-0 LRU front that was the stronger eviction candidate on this trace.

## Takeaway

Stacking LFU-minimum victim selection onto the decay path does not improve over the simpler positional-LRU fallback. After a single uniform decay step, counter differences across items are small and noisy enough that the minimum-counter scan overrides the recency signal encoded in bucket position — evicting items with slightly more recent insertion at the cost of retaining genuinely old items at index 0. The parent's `remove_at(0)` was already well-aligned with the mid-point insertion ordering; the LFU scan disrupts that alignment without a reliable enough frequency signal to compensate.
