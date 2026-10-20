# Variant v003 Insight

## Helped

- **hit-rate**: Partitioning into hot (LRU) and cold (FIFO) segments allows frequently accessed items to be preserved in the working set, with promotion based on frequency count enabling fast learning of hot items in Zipfian workloads.

## Hurt

- **per-op-time**: Maintaining dual index vectors (hot_indices/cold_indices), searching both lists on every get, frequency decay every 32 operations, and promotion logic with vector adjustments add per-hit overhead that exceeds the savings from reducing bookkeeping on cold items.

- **metadata-size**: Adding the frequency_count u8 field to Item struct increases memory overhead per entry, despite attempts to make cold items cheaper.

## Takeaway

The adaptive hot/cold partitioning strategy improves hit-rate by isolating the working set in a low-latency hot segment (LRU) while using cheap FIFO eviction for cold items—a sound algorithmic fit for Zipfian access patterns. However, the implementation underestimated the per-operation cost of maintaining separate index lists, frequency tracking, and promotion logic. The complexity of managing two segments with adaptive transitions incurs more overhead than the original ClassicLRU's single-vector approach, demonstrating that algorithmic insight alone does not guarantee latency improvement without careful implementation cost optimization.
