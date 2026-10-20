# Variant v003 Insight

## Helped

- per-op-time (`get`/`put` p99 only): Both tail latencies dropped, but the change cannot explain it. `get` is byte-for-byte the same as in the parent. On `put`, the only difference is the grow guard in `evict_small_inner`, which now compares against `small_ceil()` (a constant divide plus a `max`) instead of `capacity / 2`. That guard only runs on the rare path where a small-queue graduate arrives while main is full, and it short-circuits behind `want_grow`. The p50 values, including the `get` summary, did not move. So the tail improvement should be read as run-to-run drift, not as a benefit of the ceiling.

## Hurt

- (none observed) hit-rate: Unchanged to the fourth decimal, which is the "never binds" outcome the hypothesis said would settle the question. `try_new` already starts `small_size` at `capacity / 4`, so the new ceiling equals the starting split, and the parent's `capacity / 2` headroom was the only thing removed. A bit-identical hit-rate means that on workload-a the parent's grow-small rule never pushed small past its starting quarter in a way that changed any eviction. Ghost-hit shrinks (bounded by the `capacity / 64` floor) outweigh graduate-driven grows, so small stays at or below its initial size, and the cap has nothing to block.
- (none observed) metadata-size: The new `ceil_div` override field and the `ceil_blocks`/`max_small` stats are all `#[cfg(test)]`, so the release `S3Fifo` header and the 8-byte `Entry` are unchanged.

## Takeaway

On workload-a, the adaptive split only moves downward from its initial `capacity / 4`. The upper clamp is dead code at the evaluated size, so tightening it from `capacity / 2` to `capacity / 4` changes nothing. Together with the floor probe in 0005, this settles both bounds of the split. The remaining hit-rate lever is how the split moves (the shrink/grow trigger asymmetry, or the ghost size pinned to the initial main size), not where it is clamped. I couldn't confirm from the eval artifacts that the `h01_split_report` `ceil_blocks` count is zero on the Zipf sweeps. Running that report would show directly whether the ceiling ever binds on any workload.
