# Variant v002 Insight

## Helped

- **hit-rate**: Zone-based partitioning with a 2-access promotion threshold effectively separates single-access items from repeatedly-accessed ones, protecting hot-zone items from eviction by cold-zone LRU churn and improving hit rate by 8.3%.

## Hurt

(None observed)

## Takeaway

Partitioning the cache into a protected hot zone (top 1/3 capacity) and evicting preferentially from a cold zone successfully reduces thrashing on mixed-access patterns. By requiring 2 accesses before promotion to hot and then evicting cold-zone items first, the cache preserves items that demonstrate reuse while quickly discarding one-time accesses, yielding a significant hit-rate improvement over plain LRU.
