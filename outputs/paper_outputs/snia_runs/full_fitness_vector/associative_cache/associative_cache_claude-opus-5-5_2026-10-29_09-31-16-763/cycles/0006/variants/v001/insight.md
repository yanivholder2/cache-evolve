# Variant v001 Insight

## Helped

- hit-rate: Not a gain, but the invariant held. All four cells match the parent exactly. ClassicLRU orders by position only, and `eviction_candidate_index`/`promotion_candidate_index` for ClassicLRU never read `timestamp`. So no eviction decision changed when `Item::new` stopped calling the global `tick()` and the lip path stopped stamping. TimestampLRU still gets correct ordering from the new per-bucket `clock` (covered by `timestamp_lru_uses_bucket_clock`).
- metadata-size: Unchanged, as the hypothesis required. `Item<u64,u64>` keeps the `key, value, lfu_counter: u32, timestamp: u32` layout at 24 bytes. The new `clock: u32` lives in `AssociativeCacheBucket`, which this dimension does not measure.

## Hurt

- per-op-time: Get and put both regressed at p50 and p99, and put regressed about as much as get. The diff doesn't explain this. On the ClassicLRU path it only removes work: no `LOGICAL_CLOCK.fetch_add` in `Item::new`, and no timestamp stores in `get_from_position` or `put`. Every other part of the hot path is byte-for-byte the parent's: the tag scan, the `tags` mirroring, BIP, and quarter-depth insert. The only structural cost added is the `clock: u32` field, which likely grows each bucket by one padding slot (8 bytes) and changes the bucket stride. That is far too small to cause a slowdown this broad. The two likeliest causes are measurement drift between this run and the cycle-0005 parent run, or a layout/codegen side effect of the larger bucket. Either way, the gain the hypothesis expected didn't show up.

## Takeaway

Removing the global clock and skipping timestamp stores on ClassicLRU is safe: hit rate, `Item` size, and TimestampLRU ordering all held. In a single-threaded benchmark, though, an uncontended `Relaxed` `fetch_add` costs only a few ns, so the expected gain was small to begin with. A slowdown this large from a change that only removes work points to the measurement or the bucket layout, not the logic. Before keeping or dropping this change, rerun this variant and cycle-0005/v002 back-to-back in the same session. If the slowdown is still there, check whether getting the bucket back to the parent's size brings the speed back. `bip_tick` is only used on the lip path and `clock` only on the other path, so the two can share one `u32`.
