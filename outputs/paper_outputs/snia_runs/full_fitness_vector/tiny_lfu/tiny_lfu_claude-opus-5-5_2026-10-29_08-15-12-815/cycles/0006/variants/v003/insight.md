# Variant v003 Insight

## Helped

- per-op-time (get p50): The headline get median improved clearly. In `get`, an `_mm_prefetch` of the sketch block now goes out right after `key_hash`, and `increment_hashed` moved after the map probe (`found = map.get(..).copied()` first, then the increment, then `found?`). The sketch-block miss and the hashbrown bucket miss now overlap, and the increment's read-modify-write stores no longer sit in front of the map's dependent loads.
- per-op-time (put p99): Put's tail improved the most of any put metric. That fits the `evict_one` change: on the full-cache insert path, which is the slow tail, the candidate and victim blocks are both prefetched before the two `estimate_hashed` calls, so their misses run in parallel. On top of that, put also gets the same hash→prefetch→probe→increment reorder as get.
- per-op-time (put p50): A smaller gain from the same reorder in `put`. The median put is mostly an update-in-place or window insert, and link and map-insert work dominate there, not the two overlapped misses.

## Hurt

(none) Get p99 did not really move, and hit-rate and metadata-size are unchanged. Hit-rate is bit-identical in every trace/size cell, as designed: the new `reordered_ops_match_parent_exactly` test checks hits, resident sets and full sketch state against an inlined copy of the parent op order at caps 1/2/7/64. Metadata is unchanged because `prefetch` is a method, so no struct or layout changed.

## Takeaway

Overlapping the two independent per-op cache misses (prefetching the sketch block and probing the map before the increment) is a real memory-level-parallelism win that leaves admission semantics untouched. It is the first latency gain in this line that comes from instruction ordering rather than residency-layer codegen. The rationale predicted the biggest gain on get tails, but the gain showed up on the get median and the put tail instead. Get p99 likely comes from misses the prefetch doesn't touch, probably the `on_hit` node and neighbour link loads on probation→protected promotion. The paired prefetch in `evict_one` is what helped the eviction-heavy put tail. The prefetch's effect is not separated from the effect of the `#[inline(always)]` promotion of `increment_hashed`/`estimate_hashed`, because both landed together.
