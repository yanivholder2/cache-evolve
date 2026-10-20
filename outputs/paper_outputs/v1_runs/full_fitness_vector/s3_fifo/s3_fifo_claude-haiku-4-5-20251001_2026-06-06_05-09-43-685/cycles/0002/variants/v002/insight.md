# Variant v002 Insight

## Hurt

- **hit-rate**: Expanding the small queue from 25% to 30% of capacity increases the eviction rate from this queue without improving frequency accumulation on the workload, causing more items to be ghosted or evicted before reaching the freq > 2 promotion threshold.
- **per-op-time**: Linear search through the longer small queue during get operations increases lookup cost; more entries in the small queue means longer chain traversal in find_entry_mut and get, adding ~14% latency despite the stripe having a stricter admission policy than pure LRU.

## Helped

(none)

## Takeaway

Increasing the small queue from 25% to 30% of capacity backfired on this workload. While the hypothesis expected longer frequency-accumulation windows to boost hit-rate, the stricter freq > 2 threshold—combined with higher eviction churn from the larger queue—prevented items from accumulating enough hits to promote. The latency cost of linear search scales with queue size, and it outweighed any hypothetical benefit. This variant teaches that queue-size is not a direct lever for hit-rate improvement in S3-FIFO: the interaction between small-queue size, eviction frequency, and the promotion threshold is tighter than expected. A meaningful hit-rate improvement likely requires either loosening the threshold or better preserving hot items during small-queue eviction, not just expanding the queue.
