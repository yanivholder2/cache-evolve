# Variant v003 Insight

## Helped

- hit-rate: Replacing LRU with SIEVE adds "quick demotion". New keys enter at the newest end, behind the hand's direction of travel, and stay unvisited, so one-touch keys from scans are evicted before keys that have been reused. Under LRU, every new key was promoted to the MRU position and pushed reused keys out. The variant clears the hypothesis's bar (the cycle-1 SLRU figure), but only narrowly.
- metadata-size: The `u32` links with a `NIL = u32::MAX` sentinel make `Node<u64,u64>` smaller than the baseline's `usize` links. The extra `visited: bool` fits in the existing padding, so the SIEVE bookkeeping adds no bytes per node. The cache struct also shrinks even though it gains a `hand` field, because `head`, `tail` and `hand` are all `u32`.
- per-op-time: A `get` hit now only sets a bit and does no unlink/push-to-front. An update `put` also writes in place without relinking. That removes the pointer writes and cache-line misses on neighbouring nodes that LRU paid on every hit, and the smaller nodes improve arena locality as well.

## Hurt

- per-op-time: No dimension regressed, but the speedup is smaller at the `put` tail than at the median. When a long run of visited nodes sits ahead of it, the eviction hand has to clear bits across that run, so an occasional insert does O(run) work. LRU always evicts in O(1).

## Takeaway

On this trace, one queue with one visited bit and a hand that leaves survivors in place beats strict LRU on all three dimensions at once. Quick demotion of new keys does the work for hit-rate, and hits that never relink do it for speed. The hit-rate gain over the threshold in the hypothesis is small, though. The clearer win is that SIEVE matches a probation-style policy's hit-rate with a smaller node and cheaper hits. The remaining cost is the hand sweep's effect on insert tail latency, which this variant does not bound.
