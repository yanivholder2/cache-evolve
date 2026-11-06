# Variant v003 Insight

## Helped

- hit-rate: Not a gain, but the invariant held. All four cells match the parent exactly. This is expected because the ClassicLRU (lip) path still uses the single counter, now named `clock`, only as the BIP miss counter, with the same increment-then-`% BIP_MRU_PERIOD` sequence. ClassicLRU never reads `timestamp`, so leaving it at 0 cannot change eviction order.
- metadata-size: Held flat as designed. `Item<u64,u64>` is still 24 bytes, and the bucket keeps one `u32` counter field instead of adding a second. The new `bucket_size_matches_parent` test pins the struct at 80 bytes, which rules out bucket stride as a confounder in this variant.
- per-op-time: Get latency improved at the tail and edged down at p50. Nothing on the lip get path changed, though: `get_position_tagged` and the lip branch of `get_from_position` are the parent's code, and the only new get-side work (`next_stamp()`) runs only for non-lip policies. So this looks like the parent's get-p99 spike going away, not a benefit of this diff.

## Hurt

- per-op-time: Put got slower at p50 and noticeably slower at p99, the opposite of what the hypothesis predicted. On the benchmarked ClassicLRU path, the only behavioural change is that `Item::new` no longer does a relaxed `fetch_add` on `LOGICAL_CLOCK`. Removing an operation cannot plausibly add put latency. So this regression is either run-to-run variance, or a layout/inlining side effect of the changed `put` signature (`mut item`) and the new non-lip stamping code. It is not a cost of removing the clock.

## Takeaway

With bucket stride held fixed, removing the global atomic `tick()` from `Item::new` gave no measurable put-latency win on the ClassicLRU path. On this single-threaded benchmark, an uncontended relaxed `LOCK XADD` on a line that stays in L1 does not limit put latency. Most of the cost is still the fused quarter-length rotation and the `tags` mirror. The change is still worth keeping for correctness and future concurrency: every timestamp now comes from a per-bucket clock, and `timestamp_lru_uses_bucket_clock` checks that it survives wrap-around. But it should not be counted as a speed win. In this variant and its parent, the p99 numbers swing in both directions with no matching code change, so judge put/get tail changes only from repeated runs in the same session.
