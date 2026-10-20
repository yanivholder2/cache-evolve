# Variant v002 Insight

## Helped

- metadata-size (`S3Fifo` struct row): Dropping the `capacity` field, whose only use was `adaptive()` and the resize bounds, removes one `usize` from the struct. The summary does not move because it measures only `Entry<K,V>`, which is unchanged.

## Hurt

- hit-rate: Freezing the split at 10/90 lost a little against the parent's adaptive split, which starts at 25/75. With `freq > 1` graduation kept, a small queue of only 10% gives keys less time to collect two hits before they are ghosted. The parent's ghost-driven resizing could find a better operating point for this trace. The comparison is not single-lever, though: `ring_len = ghost_cap = main_size` now follows the 90% main, so the ghost is also larger than the parent's ghost, which was fixed at 75%.
- per-op-time (`put`): Removing the resize branches did not pay off. A thinner small queue with a strict `freq > 1` bar sends more one-hit keys to the ghost. Their returns then come in as ghost hits that go straight into a larger main, so the `evict_main` second-chance loop runs more often and over more entries. That added work outweighs the deleted branches.
- per-op-time (`get`, the summary): The `get` body is byte-identical to the parent's. So the slowdown cannot come from the removed adaptation code. It could come from the larger ghost ring and map, plus the bigger main, competing for cache lines with the interleaved `put`s, or from run-to-run noise. This variant cannot tell those apart, and the result should be re-measured before anyone acts on it.
- metadata-size (not seen by the metric): The ghost's heap allocation grew from 75% to 90% of capacity, both the ring and `ghost_map`. This cancels the small struct saving.

## Takeaway

On this trace, a fixed 10/90 split with `freq > 1` graduation loses to the adaptive split on both hit-rate and latency. The hypothesis expected the opposite. So the adaptation earns its small hit-rate margin, and its branches are not where the time goes. The test was confounded: tying `ghost_cap` to the new `main_size` changed ghost size together with the split. A cleaner repeat would pin the ghost to the parent's initial size (75% of capacity), or test 25/75 static, the parent's starting point, to isolate adaptation alone.
