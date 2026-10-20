# Variant v001 Insight

## Helped

## Hurt

## Takeaway

Collapsing the hit-path key lookup into a single entry-style probe left hit-rate on workload-a exactly equal to the parent, as expected for a change that should not alter eviction order or recency order. The fitness vector only measures hit-rate, so the change's actual target, a per-op time reduction on the hit path, was not measured and remains unverified; this variant can confirm it preserved behaviour but not that it got faster.
