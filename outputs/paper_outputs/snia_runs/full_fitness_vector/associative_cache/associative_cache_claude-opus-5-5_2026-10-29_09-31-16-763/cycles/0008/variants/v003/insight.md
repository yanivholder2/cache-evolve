# Variant v003 Insight

> Note: the insight writer could not open the overlay source files or any eval file except `fitness-vector.json`, because there was no way to list directories in this session. This reading is based on the H03 hypothesis (the intended diff), the two fitness vectors and the parent insight. It has not been checked against the code that actually landed.

## Helped

- hit-rate: Not a gain, but the invariant held. All four trace/size cells match the parent exactly. That fits the hypothesis's rule of taking SWAR matches in ascending index order, so `get_position_tagged` returns the same slot as the old byte loop even when tags repeat. Eviction and hit-move behaviour therefore did not change.
- metadata-size: Held flat as designed. The SWAR scan reads the existing dense `tags` array through `u64` loads and adds no fields, so `Item<u64,u64>` and the cache header keep the parent's sizes.

## Hurt

- per-op-time: Get got clearly slower at both p50 and p99, the opposite of what H03 predicted. On the benchmarked lookups the old byte loop seems to have been cheap already. It can stop at the first matching byte, and the compiler may already vectorize it. The SWAR path adds fixed work to every chunk: the XOR and mask arithmetic, a `trailing_zeros`/`m &= m - 1` loop that branches unpredictably, and the scalar tail loop that handles bucket sizes not divisible by 8. That overhead outweighs the drop from up to 1024 byte compares to about 128 word operations.
- per-op-time: Put p50 regressed badly and p99 got a little worse. This fits the regression being in the shared lookup: put's lookup also goes through `get_position_tagged`, so it pays the same per-chunk overhead before it inserts or rotates. The parent insight warned that p99 is noisy. But here both p50s moved a long way in the same direction, which points to a real cost of the new scan rather than variance.

## Takeaway

Swapping `get_position_tagged`'s byte-at-a-time tag scan for a hand-written SWAR scan made both get and put slower, even though the hit-rate result stayed exactly the same. Counting compares was the wrong cost model for this pre-filter. Most lookups seem to end or match early, and the compiler likely already handles the simple byte loop well, so word-level tricks add work without saving any. Before trying wider tag scans again (SIMD, or SWAR with an early-exit fast path for the first chunk), check the generated assembly of the parent's scalar loop and measure how far a typical scan goes. If that shows the scan is short, scan cost is not the bottleneck on the miss path.
