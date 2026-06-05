# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Adding Stage 2 — evicting `lfu_counter == 1` items before falling back to LRU position — demoted single-touch residents to victim priority alongside never-accessed items, causing the cache to expel items that would have been hit again on workload-a's Zipfian distribution before exhausting truly cold alternatives.

## Takeaway

On this workload, a single in-tier access is a reliable enough reuse signal that raising the eviction guard threshold from one to two accesses is counterproductive: the items evicted under the new Stage 2 (lfu_counter == 1) were not transient noise — they were part of the Zipfian working set. The binary cold/warm split at the zero-access boundary from v001 was already correctly positioned; tightening it further by requiring two confirmed accesses for protection shrinks the protected working set too aggressively and produces a substantial hit-rate regression.
