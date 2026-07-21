# Variant v003 Insight

## Helped

- metadata-size: Halving `SKETCH_DEPTH` from 4 to 2 directly halved the CountMinSketch heap allocation — the dominant variable-size structure in the cache — while leaving Node struct and cache struct sizes untouched.

## Hurt

- hit-rate: With only 2 rows, the min estimator has fewer independent hash rows to suppress collisions, producing noisier frequency estimates that cause the admission gate to make more incorrect accept/evict decisions on this Zipfian trace.
- per-op-time: Latency increased at both p50 and p99 for get and put despite the reduction in per-sketch-call hash and counter operations, suggesting that the degraded frequency signal led to more cold-item evictions and correspondingly more cache misses that dominated the operation cost.

## Takeaway

The 4-row depth in the baseline CountMinSketch is load-bearing for this trace: the two extra rows provide collision suppression that materially improves admission accuracy, and removing them hurts hit rate enough to also inflate per-operation latency. The sketch heap saving is real, but at a sketch width derived from capacity-2048 the collision probability at depth 2 is high enough that accuracy, not redundancy, is what the additional rows deliver.
