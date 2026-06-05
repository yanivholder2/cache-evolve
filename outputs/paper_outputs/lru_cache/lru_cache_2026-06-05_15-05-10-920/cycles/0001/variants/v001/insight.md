# Variant v001 Insight

## Helped

- hit-rate: Frequency-based eviction captures access patterns better on this workload than recency alone; by keeping frequently-accessed items regardless of recency, the cache avoids evicting "hot" items that were accessed earlier.

## Hurt

## Takeaway

Adding a frequency counter to each node and evicting the lowest-frequency item (breaking ties by LRU order) improves hit rate significantly on access-skewed workloads. This variant's ~9 percentage point gain confirms that when access patterns are non-uniform, preferring frequency over pure recency is a more effective eviction strategy, even with the added O(n) scan cost at eviction time.
