# Variant v001 Insight

## Helped

- hit-rate: Replacing move-to-MRU with S3-FIFO admission stops loop and scan blocks larger than the cache from flushing the reused working set. New keys land in a small probation FIFO, about 10% of capacity. Keys never re-referenced there are evicted without touching `main`. Keys that were re-referenced (`freq > 0`) are promoted to `main`, and ghost hits go straight into `main`. Together these keep the reusable blocks of LIRS workload-a resident.

## Hurt

(No dimension regressed. v001's fitness vector reports only `hit-rate`. `metadata-size` and `per-op-time` were not measured for this variant, so the expected gains from dropping `prev`/`next` and from `get` becoming a saturating byte increment are still unverified.)

## Takeaway

On workload-a, the gain comes from filtering at admission. Each new key has to prove reuse during a short probation in `small` before it can take space in `main`, which strict LRU never requires. A ghost of about `capacity - small_target` keys gives evicted keys a second chance at admission. The memory and latency case for S3-FIFO in this arena layout is still untested, because the ghost `VecDeque<K>` and `HashMap<K, ()>` add per-key overhead outside `Node` that could cancel out the smaller `Node`. A follow-up variant should measure `metadata-size` and `per-op-time` before this design is built on further.
