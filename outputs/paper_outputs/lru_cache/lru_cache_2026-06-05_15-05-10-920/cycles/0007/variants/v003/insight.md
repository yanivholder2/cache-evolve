# Variant v003 Insight

## Helped

- hit-rate: Hybrid scoring weighted frequency access patterns heavily (×100 multiplier), allowing frequently-accessed items to survive eviction even during quiet periods, better capturing temporal reuse patterns in workload-a.

## Hurt

(none)

## Takeaway

Frequency-recency hybrid eviction outperforms pure LRU on this workload. By computing an eviction score combining access count (dominant) and recency position (secondary), the cache avoids evicting hot items that experience periodic access gaps. The ×100 weighting ensures frequency is the primary signal, while position provides fine-grained discrimination—turning victim selection from a single-dimension decision into one that respects both temporal and spatial locality.
