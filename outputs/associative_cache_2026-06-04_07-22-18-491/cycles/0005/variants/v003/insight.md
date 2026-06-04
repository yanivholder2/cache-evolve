# Variant v003 Insight

## Helped

- hit-rate: Routing all new inserts through a half-capacity FIFO probe zone prevented one-time-access items from displacing primary residents; only keys that received a confirming `get` hit were promoted, leaving primary populated almost exclusively by items with demonstrated reuse on the Zipfian workload.

## Hurt

## Takeaway

Probationary admission control is a strong lever on Zipfian-like workloads: the probe zone absorbs the cold-insert churn that ClassicLRU could not distinguish from hot traffic, and the FIFO eviction of unconfirmed probe entries costs nothing in primary capacity. The size of the gain over baseline shows that a large fraction of primary evictions under ClassicLRU were caused by one-time-access items that this design silently discards before they ever compete for a primary slot.
