# Variant v003 Insight

## Helped

- hit-rate: The per-bucket stash catches items evicted by LRU and re-promotes them on the next access, recovering recency misses that the baseline discards permanently — the LIRS workload-a trace has enough short-cycle reuse that this second-chance buffer converts a meaningful fraction of compulsory eviction misses into hits.

## Hurt

_(no dimensions moved in the wrong direction)_

## Takeaway

Adding a FIFO stash sized at one-quarter of each bucket's primary capacity is sufficient to produce a clear hit-rate gain on a LIRS-style workload without touching the eviction policy itself; the gain comes entirely from giving recently evicted items one more opportunity to be reclaimed before they leave the cache. The `metadata-size` dimension was not evaluated for this variant, so the memory cost of the two extra `Vec` fields per bucket remains unquantified and should be measured in a follow-on variant before treating the stash as strictly additive.
