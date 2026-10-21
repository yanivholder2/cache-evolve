# Variant v003 Insight

## Helped

- hit-rate: Conservative update in `CountMinSketch::increment_hash` now bumps only the counters equal to the key's current minimum. Collision noise therefore piles up more slowly in the shared counters, so a cold newcomer's estimate and a cold LRU-head victim's estimate stay apart more often. Because the admission rule is still strict `>` with ties going to the victim, fewer near-ties end with the wrong key winning. The gain is small but real, and it came without aging, so the long-term frequency gaps stay in place.
- per-op-time (put p50): `put` now uses the `min + 1` returned by `increment_hash` as `incoming_freq` and drops its separate `estimate_hash(h)` call. That saves one 4-row pass on the eviction path, which roughly cancels the extra cost of the new two-phase increment. Typical put latency stays flat.

## Hurt

- per-op-time (get p50, put p99): The old increment was a single fused read-modify-write loop. The new one makes two passes: it stores 4 indices in `idx`, takes the min, then does a masked write over each counter. `get` pays this on every call and gets nothing back, because it never uses the returned estimate. That is the likely cause of the typical-latency rise on `get`. The put tail rose too, but get p99 didn't move, so that part may be benchmark noise. As v002's insight noted, this bench's per-op numbers can shift with the environment.

## Takeaway

Conservative update sharpens the sketch enough to win a little hit-rate on workload-a, and it leaves metadata unchanged. This supports the idea that some of the ceiling comes from overestimates collapsing near-ties at the LRU end. The price is a two-pass increment, and only `put` makes up for it by reusing the returned estimate. A natural follow-up is to tighten the two-phase loop by keeping the 4 loaded counter values in registers, so the write phase doesn't reload them. That would test whether the extra cost on `get` comes from conservative update itself or just from how this version was written.
