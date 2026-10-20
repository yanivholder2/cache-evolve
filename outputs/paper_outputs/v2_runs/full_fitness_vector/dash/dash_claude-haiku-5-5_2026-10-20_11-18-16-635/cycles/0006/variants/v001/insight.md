# Variant v001 Insight

## Helped

## Hurt

- per-op-time (put): The cached `min_slot` has to be refreshed on every insert, counter increment, and eviction, so put pays for the bookkeeping on top of v002's full-scan victim search instead of saving it. The hypothesis expected the opposite, which means the cache maintenance cost outweighed the scan it was meant to skip.
- per-op-time (get): The hypothesis kept the get path out of scope, but the saturating-counter increment on a target hit is also a cache write, so the added invalidation and refresh logic reaches the get-hit branch and shows up as a worse tail latency with median effectively flat.

## Takeaway

Caching the eviction victim per bucket did not pay off: hit-rate and metadata size stayed identical to v002, and both put latency and get tail latency got worse. The lesson is that for buckets of this size, a full scan is cheap enough that maintaining an incremental minimum on every counter write costs more than it saves, so future work on victim selection should target the scan itself or skip the extra state.
