# Variant v003 Insight

## Helped

- hit-rate: Replacing recency-only eviction with LIRS's inter-reference-recency split protects the ~99% LIR set from the loops and one-shot scans in workload-a. New keys enter as resident HIR in a 1% queue and are evicted first. A key re-referenced while still tracked in stack S (including as a non-resident ghost) is promoted to LIR. Strict LRU would instead cycle every key out on a loop longer than capacity.

## Hurt

(No dimension measurably regressed. The eval recorded only `hit-rate` for this variant. `metadata-size` and `per-op-time`, both present in the parent, were not measured, so the costs below are unconfirmed.)

- metadata-size (unmeasured, likely worse): `Node` now carries `Option<V>`, a status enum, two membership flags and four `u32` links for S and Q. Ghost entries also keep up to `2 * capacity` extra nodes and map entries alive, so total footprint grows well beyond the baseline's one-node-per-resident arena.
- per-op-time (unmeasured, at risk): Each access now maintains two intrusive lists plus `prune_stack`. `enforce_nonres_bound` also walks S upward from the bottom to find the oldest non-resident node. That walk is linear, not O(1), and can pass over many LIR or resident-HIR nodes on every put that exceeds the ghost bound.

## Takeaway

LIRS's main gain over LRU came from keeping loop and scan traffic out of a protected LIR set. That gain was real, but this variant did not reach the S3-FIFO hit rate that H03 targeted. On this trace, a full IRR-tracking policy with a 1% HIR queue did not beat simpler admission filtering. The next comparison should measure LIRS's metadata and per-op costs and try a larger HIR fraction. It should also replace the linear scan in the ghost-bound walk, for example with a separate FIFO of non-resident nodes, before LIRS is judged against the S3-FIFO lineage.
