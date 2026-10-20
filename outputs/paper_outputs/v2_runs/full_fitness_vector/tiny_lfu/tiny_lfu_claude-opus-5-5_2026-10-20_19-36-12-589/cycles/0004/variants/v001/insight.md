# Variant v001 Insight

## Helped

- per-op-time (get p50/p99): `increment_only` loads each of the 4 counters once into locals, takes the min branch-free, and writes back from registers. This replaces v003's two-pass loop, which stored indices and then reloaded each counter for a masked write. `get` runs this on every call, so `get` gains the most. The p99 drop as well as the p50 drop suggests the reload/branch structure caused part of v003's tail, and that the gain isn't just noise.
- per-op-time (put p50/p99): `put` gets the same fused `increment_hash`. It also gets the unrolled, unchecked `estimate_hash` for the victim lookup, which no longer has the dead `depth == 0` branch. The eviction path now runs two cheaper 4-row passes, and both typical and tail latency drop.
- metadata-size (TinyLfuCache struct): The redundant `depth` field is gone from `CountMinSketch` and `SKETCH_DEPTH` is used instead, so the cache struct shrinks by one word. The headline per-node size and the sketch's heap footprint don't change, so the summary stays flat.

## Hurt

- hit-rate: Nothing got worse; hit-rate is bit-for-bit identical to v003, as the hypothesis required. Writing back from locals matches v003's reload because the 4 flat indices fall in disjoint row ranges and can't alias. The new `increment_only_matches_increment_hash` test backs this up.

## Takeaway

v003's `get` regression came from how its conservative update was written, not from conservative update itself. With one load per counter, a branch-free min/mask, and unchecked indexing, v001 keeps v003's best hit-rate and recovers most of the per-op cost. `get` is still a little slower than the cited plain fused increment (~33). That gap is about the size of the per-op drift seen before, so the remaining cost of conservative update is small enough that later work should look elsewhere, such as hashing or list updates.
