# Variant v003 Insight

## Helped

- hit-rate: Wider bucket geometry reduces primary-bucket collisions, allowing items that would otherwise be evicted to remain accessible; the 15% FIFO stash with secondary lookup recovers evicted items, shifting the binding constraint from raw stash capacity to collision frequency.

## Hurt

(none observed)

## Takeaway

Geometry-driven collision reduction is a viable path to hit-rate improvement with smaller stash overhead. By reshaping the bucket layout to use fewer, wider buckets while maintaining total capacity, and pairing this with a smaller (15%) FIFO stash and secondary lookup, the cache achieves better hit rates than the baseline. This confirms the hypothesis that reducing primary-bucket collision frequency—rather than maximizing stash capacity—is the binding constraint for hit-rate performance in this workload.
