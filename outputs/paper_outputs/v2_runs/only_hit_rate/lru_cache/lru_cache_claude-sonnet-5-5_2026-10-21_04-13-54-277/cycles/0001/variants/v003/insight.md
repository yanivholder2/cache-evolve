# Variant v003 Insight

## Helped

- hit-rate: Inserting new keys into a probation list and promoting them to a protected list only on a second hit keeps one-time scan keys from displacing re-referenced keys. Eviction always takes the probation LRU, so the scan-heavy LIRS trace loses fewer hot entries than strict single-list LRU does.

## Hurt

- metadata-size: The variant's fitness vector has no entry for this dimension, so it is unmeasured. The change adds a `segment` flag to every `Node`, which pushes it past the baseline's pointer-packed layout. It also adds a second list plus `protected_cap` to `LruCache`. Both should cost bytes, but that is inferred from the code, not measured.
- per-op-time: This dimension is also missing from the fitness vector, so nothing was measured. Each hit now does an unlink and a push into the protected list, and a probation hit can trigger a demotion loop. That is more pointer work per hit than the baseline's single move-to-front.

## Takeaway

Segmenting the LRU into probation and protected lists, with promotion on the second hit, is a cheap way to gain scan resistance without changing the cache API. The invariant gate did not reject the deviation from strict LRU order. The gain was measured on one trace and one cache size. The vector reports only hit-rate, so the memory and latency cost of the extra bookkeeping is unknown. Any claim that this variant dominates the baseline needs those two dimensions evaluated.
