# Variant v002 Insight

## Helped

- hit-rate: The rewrite kept the order of sketch increments, the hand movement and the admission rule exactly as they were, so hit rate matches v001 exactly. This confirms the `entry`-based restructure changes behaviour in no way, as the hypothesis required.

## Hurt

- per-op-time: The `put` p50 did not drop and is flat to marginally worse. The single-probe `entry` path only covers the fill phase, which runs for the first `capacity` (2048) inserts and then never again. In steady state the trace runs the full-phase path, and that path still does `map.get`, then `map.remove`, then `map.insert`, just as before. Also, with `u64` keys the dropped `key.clone()` was a register copy that cost nothing.
- per-op-time: The `put` and `get` p99 both rose, and the `get` p99 rose a lot, even though `get` was not touched. That rise cannot come from the code change itself. It points to benchmark noise or a code-layout/inlining shift caused by the reshaped `put`. Either way, tail numbers at this level are not a reliable signal for small edits.

## Takeaway

Removing a map probe only pays off on the path the trace actually spends its time on. Here the saved probe sat in the fill phase, which is a one-time warm-up for a 2048-entry cache, so a steady-state benchmark cannot see it. To cut `put`'s map cost, a future change has to target the full-phase miss path: the rejected-newcomer `get` probe, and the `remove`+`insert` pair on admission. For example, it could reuse the precomputed hash `h` through a raw-entry/hash-aware API so the key is not re-hashed for each of those three map operations.
