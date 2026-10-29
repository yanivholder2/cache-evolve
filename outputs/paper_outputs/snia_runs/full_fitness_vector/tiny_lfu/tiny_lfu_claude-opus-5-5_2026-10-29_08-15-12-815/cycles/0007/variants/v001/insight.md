# Variant v001 Insight

## Helped

- hit-rate: No change in any cell, which is what the hypothesis promised. The reorder only moves the `increment_hashed` call after the map probe, and neither step reads state the other writes. `reordered_ops_match_parent_exactly` checks that the hit sequences, resident sets, node tables and sketch tables match the parent at capacities 1/2/7/64. So this variant is a pure latency experiment.
- per-op-time (get p99): The get tail eased a little. This fits the paired `evict_one` prefetch and the probe-overlapped sketch load trimming the occasional op where the sketch block misses cache. The parent's insight named this tail as the one part of latency left to watch. The move is small enough that it could be benchmark noise, which the hypothesis itself warned about.

## Hurt

- per-op-time (get p50, summary): The hypothesis's main target did not move. Prefetching the sketch block before the map probe did not lower the median, so v003's memory-level-parallelism gain does not stack on top of v001's `#[inline(always)]` gain. On the typical `get` the sketch block is most likely already in cache, or its load is already overlapped by out-of-order execution in the inlined straight-line code. Either way, an explicit `_mm_prefetch` has nothing left to hide.
- per-op-time (put p50): A tiny regression with no matching gain at put p99. The extra prefetch instruction, and keeping `found` live across `increment_hashed`, add work on every `put` without saving a memory stall. This is within the noise band, but it points the wrong way.
- metadata-size: No change, as expected. The node stays at 24 bytes and the sketch geometry is untouched.

## Takeaway

Once `increment_hashed` is inlined, v003's gains (prefetch + probe-before-increment + paired `evict_one` prefetch) overlap with v001's gains instead of adding to them. The two optimisations were hiding the same sketch-line latency, so the median `get` is no longer limited by the sketch load. Software prefetch is now at most a tail-latency tool on this line. Any further get p50 gain will have to come from somewhere else: the map probe, the `key.clone()` used to build the probe key, or `on_hit` list relinking.
