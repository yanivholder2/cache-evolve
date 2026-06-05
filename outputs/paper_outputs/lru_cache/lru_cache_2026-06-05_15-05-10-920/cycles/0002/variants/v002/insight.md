# Variant v002 Insight

## Helped

- hit-rate: Zone-based eviction prioritizes discarding one-time accesses (cold zone) over repeatedly-used items (hot zone), which better aligns with real-world access skew where a minority of keys account for most hits.

## Hurt

(none)

## Takeaway

The hybrid approach—partitioning the cache into a cold zone with LRU eviction and a hot zone with frequency-based eviction—successfully captures both temporal locality (cold items that arrive and leave quickly) and access skew (hot items that are accessed multiple times). By promoting items to the hot zone only after demonstrating reuse (freq ≥ 2), the cache protects truly hot items from eviction while fast-tracking cold items out of the arena, reducing churn and increasing hit rate.
