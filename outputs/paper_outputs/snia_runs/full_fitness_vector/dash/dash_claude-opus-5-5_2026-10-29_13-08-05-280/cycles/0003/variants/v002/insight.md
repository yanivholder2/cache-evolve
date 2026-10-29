# Variant v002 Insight

## Helped

- per-op-time: The measured get and put latencies both went down, but the code change doesn't explain it well. `Item<u64,u64>` is still 40 bytes, so bucket scans read the same number of cache lines. The only difference on the hot path is a 1-byte load, `saturating_add` and store on stash hits, plus a `u8` compare against `PROMOTE_THRESHOLD`. `put` was not touched, yet it improved the most. Most of this movement is probably run-to-run noise or a side effect of code generation, not a result of the layout work.

## Hurt

- metadata-size: This dimension didn't move. The hypothesis aimed to reduce it and failed. `Instant` on Linux takes 16 bytes and has align 8 (an `i64` of seconds, a `u32` of nanoseconds and 4 padding bytes inside the type). So key(8) + value(8) + timestamp(16) + counter(1) comes to 33 bytes, which rounds back up to 40. The 7 bytes saved by narrowing the counter turn into trailing padding, and reordering the fields can't reclaim that padding or the padding inside `Instant`. The implementer saw this and marked the `< 40` size test `#[ignore]`.

## Takeaway

On its own, narrowing `lfu_counter` to `u8` can't shrink `Item`. Of the 24 metadata bytes, `timestamp: Instant` uses 16, and any field of 8 bytes or less fits inside the alignment slack. To actually cut metadata-size, `timestamp` has to change: for example a `u32` tick or epoch counter, or no timestamp at all for policies like ClassicLRU that never read it. With a ≤4-byte timestamp next to the `u8` counter, the item would fit in 24 bytes. The `u8` counter itself was a safe change, since hit-rate is identical to v001. It can stay as a prerequisite for that next step.
