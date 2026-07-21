# Variant v003 Insight

## Helped

- **hit-rate**: Shrinking the small queue from 1/4 to 1/10 of capacity enlarges the main queue by the same margin, giving the Zipfian hot set more resident slots and reducing the rate at which frequently accessed entries are displaced.
- **per-op-time**: The smaller small queue shortens the linear scan that `get()` must complete before reaching main, where the majority of hits occur on a skewed workload — halving the p50 get latency relative to the parent.

## Hurt

- **per-op-time** (put): A larger main queue means `evict_main()`'s second-chance loop must cycle through more entries before finding a zero-frequency victim, nudging put p50 upward.

## Takeaway

Combining the 1/10 small-queue fraction with the parallel-deque layout stacks both benefits cleanly: the sizing change improves hit rate by giving the hot set more main-queue space, and it simultaneously accelerates reads by reducing the mandatory small-queue traversal before reaching the denser hot region — confirming that queue sizing and entry layout are orthogonal levers whose gains compose without interference on Zipfian traces.
