# Variant v002 Insight

## Helped

- hit-rate: Inserting misses at the 5/8 midpoint instead of the MRU tail lets one-hit-wonders sink to `head` and evict sooner, while `get` promotion still moves genuine re-hits to the true tail so they climb above the midpoint into the protected region.

## Hurt

## Takeaway

Insertion-position bias alone — a single midpoint splice with unchanged on-hit promotion and no segments or per-node flags — measurably protects the hot region from scan pollution on this workload, isolating that effect from any two-segment bookkeeping.
