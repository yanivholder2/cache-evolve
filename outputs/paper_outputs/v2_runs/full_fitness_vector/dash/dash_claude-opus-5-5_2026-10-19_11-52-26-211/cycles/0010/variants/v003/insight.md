# Variant v003 Insight

## Helped

- metadata-size (held, as required): only the stash geometry changed. `stash_geometry()` halves `stash_size` and builds stash buckets with `new_bucket_concrete(2 * bucket_size, …)`. `Item` and `Dash` are untouched. `borrowed` shrinks to `stash_size` entries, but it lives behind `SegmentCore`'s heap allocation, so it never shows up in the measured types.
- per-op-time (get p50, held within the hypothesis's ≤ 70 ns bound): the parent's inline tags absorb most of the longer stash scans, as the rationale predicted. A miss in the 2×-wide `s0`/`s1` scans more tag bytes, but still touches the item buffer only when a tag matches. So `get` barely moves at the median.

## Hurt

- hit-rate: the core claim is refuted. Hit-rate did not stay flat (the hypothesis's stated refutation case); it fell below the parent. Pooling doesn't add any slots: total stash capacity is unchanged, and each wide bucket is now shared by twice as many `s0` keys under one bucket-local eviction order. The insertion rules stayed as they were: fresh puts go in at Front (the LRU end) and demotees at Back (the MRU end). A wider bucket probably makes that asymmetry worse. A freshly stashed key is still the next victim, while demotees now have more slots behind them and survive longer. The pooled space then fills with cold items demoted from target buckets rather than the overflow keys that get re-referenced.
- per-op-time (put p50/p99): `put` got clearly slower. Every stash insert, eviction, and stash-hit promotion now works on a bucket twice as long. `put_at_tagged` runs its existence check over more tags, Front inserts shift both the items and the tag array across 2× the width, and the policy's victim search covers more entries. Put is the hot path for stash writes, so the extra length is paid on every overflow.
- per-op-time (get p99): the `get` tail grew more than the median did. Full misses still walk the whole target → probing → `s0` → `s1` chain, and the two stash legs are now twice as long. Tags cut the cost of each slot but not how many slots get scanned.

## Takeaway

Halving the stash bucket count and doubling bucket width, at the same total capacity, made things worse on every axis that moved. Hit-rate fell instead of rising, and `put` paid for longer bucket scans and shifts. That suggests stash overflow on this trace is not clustered tightly enough for pooling to help (the same conclusion as cycle 7's independent-hash result). If anything, wide buckets with Front-insert for fresh keys favour keeping demotees over overflow keys. The wins so far came from adding new places for keys to land. Merging existing places doesn't add any, so stash width should stay at `bucket_size`. A follow-up should question the Front/Back insertion asymmetry itself rather than the geometry.
