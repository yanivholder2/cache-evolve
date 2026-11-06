# Variant v001 Insight

## Helped

- hit-rate: Inserting new items at `len / 4` from the LRU end, not at the MRU tail, means a block seen once is evicted after about a quarter of a bucket's turnover unless it is hit again. One-touch scan blocks in `web_0`/`hm_0` therefore stop flushing the reused working set. Hit-rate went up on every trace×size cell. The gain was clearly bigger at 131072 (1024-slot buckets) than at 16384 (128-slot buckets) on both traces. At 16384, a 25% probation window of about 32 slots is often too short for a real second touch to arrive before eviction, so reused blocks get thrown out along with the scan blocks.

## Hurt

(No measured dimension regressed. This variant's eval reports only `hit-rate`. `metadata-size` and `per-op-time` were not measured, so this run says nothing about the cost of the extra mid-vector `insert` memmove on misses. The parent's `remove(0)` eviction already pays a similar cost.)

## Takeaway

LRU-insertion-point admission reliably beats pure MRU insertion on these MSR block traces. How much it helps depends on bucket capacity, because the probation window is a fixed fraction of the bucket, not a fixed number of slots. Small buckets get a short window that cuts off reuse. This makes `INSERT_FRACTION_DIV` (or a probation depth that scales with bucket size) the obvious knob to tune next. A deeper insertion point (smaller divisor) is the likeliest way to recover more hits at 16384 without losing scan resistance at 131072.
