# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Hybrid frequency-recency scoring (frequency × 100 + position) degrades eviction precision by mixing position weighting with frequency discrimination, causing the cache to over-protect recently-inserted items at the expense of frequently-accessed older items.

## Takeaway

Introducing position-based recency into the eviction score formula caused a measurable regression in hit-rate (0.7334 → 0.7057, -2.77%). The simple frequency-only approach from v003 is more effective at identifying cache victims than the hybrid score that weights position equally with fractional frequency differences. This suggests the workload benefits from prioritizing access frequency over insertion recency, and adding positional bias creates noise that obscures the frequency signal rather than refining it.
