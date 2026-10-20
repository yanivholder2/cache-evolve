# Variant v003 Insight

## Helped

- hit-rate: `stash_insert_with_rescue` puts each stash-overflow victim into a free slot in its own target bucket, or failing that its probing bucket, instead of dropping it. Holes left by target→probing promotions (`remove_at` on a hottest-target hit) now hold items that would have been lost. Hit rate rose above the parent's and cleared the hypothesis's 0.702 bar, so H03 is confirmed. The worry that rescued cold items would come back through stash-Back as fake "demotees" and push out reused items did not outweigh the gain from keeping them longer.

## Hurt

- per-op-time: `put` p50 and p99 both rose. Every fresh put that overflows its stash bucket now runs a `get_index` hash, up to two `is_full` checks and a second `put_at`, where the parent stopped after one stash insert. `get` p50 and p99 also rose, even though `promotion_get` was not touched. Two things fit that: stash hits and promotions cascade through `demote_from_normal_bucket`, which now runs the same rescue path, and normal buckets hold more items, so `get_position` scans fuller buckets on every target and probing lookup, misses included.

(metadata-size did not change. The rescue is a free function plus a compile-time flag, with no new fields, so `Item` stays 40 bytes.)

## Takeaway

On this workload the normal tier has a lot of idle capacity, and promotion holes are not refilled fast enough by stash→target promotions alone. Using those holes as a fallback home for stash victims recovers hit rate without evicting anything. Placing rescued items at `Front` (LRU end) keeps them first to leave, so they don't block real promotions. The cost shows up as latency, not memory. Rescue makes stash overflow a multi-bucket operation and leaves buckets fuller, so `put` and `get` both slow down. Any later variant that keeps rescue should weigh this small hit-rate gain against a clear per-op-time regression.
