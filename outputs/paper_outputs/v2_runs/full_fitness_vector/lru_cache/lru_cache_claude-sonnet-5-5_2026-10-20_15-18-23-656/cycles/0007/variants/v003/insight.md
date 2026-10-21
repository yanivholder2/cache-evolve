# Variant v003 Insight

## Helped

- per-op-time: Using `mem::replace` on the victim slot's key removes the separate evicted-key clone in `put`, and the single `let node` binding drops repeated indexing. This shows up only as a marginal put p50/p99 and get p99 shift, within run-to-run noise, and the summary does not move.

## Hurt

- None. hit-rate and metadata-size are unchanged, as intended: policy, node layout and hasher were not touched.

## Takeaway

For `K = u64`, `Clone` is a plain register copy, so the dropped clone saves almost nothing. Put cost is dominated by the hash-map remove and insert plus the list relinking, not by key copying. Micro-optimizations of this kind need a heap-allocated key type or a change to the map or list path to be measurable.
