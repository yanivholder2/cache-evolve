# Variant v003 Insight

## Helped

- **hit-rate**: Rebalancing from 128 narrow buckets to 64 wider buckets reduced hash collision probability, allowing more items to coexist in their primary bucket without triggering eviction.

## Hurt

## Takeaway

This variant demonstrates that the baseline's narrow-bucket geometry (128 × 16) exacerbates collision impact. By shifting to fewer, wider buckets (64 × 32) while preserving total capacity, the cache reduces collision-driven eviction pressure and improves hit rate. Wider buckets provide more "breathing room" for items that hash to the same bucket, shifting the system from a collision-dominated regime to one where most items survive in their primary bucket longer.
