# Variant v001 Insight

## Helped

- None. hit-rate, the only reported dimension, did not improve over the parent. I could not inspect the overlay sources, so this rests on the hypothesis text and the two fitness vectors.

## Hurt

- hit-rate: Widening the reuse counter from 2 to 3 bits makes an entry take several scans to age back to zero after a burst of hits. Stale formerly-hot keys therefore survive in their buckets and block fresher keys. Defining "hot" as counter >= 2 also means the all-hot drop rule now fires on weakly reused buckets, so more incoming keys are dropped.

## Takeaway

Giving the reuse counter more resolution (saturating at 7, decrement-by-one aging) cost hit-rate on workload-a instead of improving it. On this trace, recency-responsive aging matters more than separating frequent from occasional reuse. A slow-decaying counter combined with a drop rule keyed on a low threshold over-protects stale entries.
