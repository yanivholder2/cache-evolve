# Variant v003 Insight

## Helped

- **hit-rate**: Frequency-weighted promotion scoring successfully prioritizes high-value cold-tier recoveries, ensuring items showing strongest re-access patterns (weighted by recency) gain priority access to limited hot-tier capacity, yielding +5.0% relative improvement over baseline.

## Hurt

(No regressions observed in reported fitness dimensions)

## Takeaway

The frequency-weighted promotion strategy proves effective: by computing promotion scores as `cold_access_count / (1.0 + time_since_eviction_ms / 1000.0)` and selecting the highest-scoring cold item when hot tier is at capacity, the cache dedicates its limited hot tier to items most likely to be re-accessed. This measured improvement validates that promotion prioritization matters—items accessed multiple times while cold are more valuable to recover than single-access items, and recency of eviction moderates this valuation.
