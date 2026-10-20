# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Replacing move-to-front with a referenced bit and a second-chance sweep changes which node is evicted on a miss, so workload-a at this size now evicts some recently used entries that strict LRU would have kept, and hit-rate dips slightly below the parent.

## Takeaway

Second chance is a close approximation of LRU, and on workload-a at size 2048 the approximation costs a small amount of hit-rate, so the hit-path write it removes does not buy back eviction quality on this trace. The fitness vector has no timing dimension, so the hit-path saving this change targets is still unmeasured, as it was for v001; only hit-rate was observed.
