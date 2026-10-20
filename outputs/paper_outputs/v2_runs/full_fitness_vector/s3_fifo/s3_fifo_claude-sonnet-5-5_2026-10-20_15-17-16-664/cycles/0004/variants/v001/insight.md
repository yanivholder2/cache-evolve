# Variant v001 Insight

## Helped

- per-op-time: Replacing the linear membership scans with hash-index lookups cut the typical `put` cost sharply, and the typical `get` improved as well. Lookups no longer walk the queues. The `put` tail improved only slightly, which suggests the remaining tail cost comes from eviction and ghost-trimming work that the index does not remove. The `get` p99 got slightly worse, plausibly from hashing cost or noise.

## Hurt

- metadata-size: The headline value (per-entry size) is unchanged, because the reported accounting was held fixed. The whole-cache struct footprint nearly doubled, because the shadow indexes add hash-table state that the summary does not count. The metric hides this cost.

## Takeaway

The index preserved the hit/miss sequence exactly, so hit-rate was unchanged, as the hypothesis required. It turned the scan-dominated median `put` into a cheaper operation. The price is a larger, uncounted memory footprint and only a modest tail improvement. A fixed `metadata-size` summary will not penalize this kind of trade, so later variants should not read it as free. The `put` tail is the next place to look.
