# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Reducing the hybrid frequency coefficient from 100× to 25× weakens frequency discrimination relative to recency, causing the cache to evict still-useful frequent items in favor of recent-but-rarely-accessed items on this workload.

## Takeaway

The hypothesis that frequency weighting was over-aggressive with decay enabled was incorrect for this workload. Pure frequency-based eviction (v001) outperforms hybrid scoring that weights recent position equally with frequency. This suggests that on this access pattern, discriminating by frequency of use—even with decay—is more predictive of reuse than recency alone. The 25× coefficient redistributes eviction pressure too heavily toward recency, causing the cache to lose track of frequently-accessed items.
