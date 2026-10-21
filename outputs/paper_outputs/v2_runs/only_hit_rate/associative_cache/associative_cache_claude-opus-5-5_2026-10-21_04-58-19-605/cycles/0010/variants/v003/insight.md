# Variant v003 Insight

## Helped

(No measured dimension improved. As in v002, the fitness vector has only `hit-rate`. H03 is a cost-only change aimed at per-op-time, and that dimension was not evaluated, so the hypothesis can be neither confirmed nor refuted.)

- hit-rate (held, not improved): no movement, which is what H03 required. The tag array only filters which slots get a full key compare. `tag_position` walks candidate bytes in index order and returns the first `items[i].key == *key`, so it yields the same index as the linear scan. The tags are kept in lockstep with `items` on push, in-place GCLOCK replacement and `remove_at` (`copy_within` plus zeroing the last slot). Moving the ghost check onto the shared hash `x` does not change the fingerprint either: `ghost_fp(x)` is the same top-16-bit value that v002 computed from its own hash. So the hit/miss and victim sequences on workload-a at 2048 are identical to v002's.

## Hurt

(No measured dimension regressed.)

## Takeaway

Putting a SWAR tag filter in front of the bucket scan is behaviour-preserving on the GCLOCK/ghost-ring line. The `debug_assert_eq!` against `linear_position`, together with the new `tag_lookup_matches_linear_scan` and `evict_item_keeps_tags_in_sync` tests, makes it a safe base for later cost work. Whether it pays off is still open, and the trade is not one-sided:
- **New cost on hits.** Every clock-mode lookup now runs FNV plus the 64-bit mix, including `get` hits that v002 served with a bare key scan.
- **Not "one hash per op".** The miss path still re-hashes the evicted key in `record_ghost`.
- **Extra cost for large buckets.** In clock mode with `bucket_size > 16`, `put` still computes the hash through `get_position_hashed` even though no tag probe uses it.

With `u64` keys, the win depends on byte-tag rejection beating up to 16 direct 8-byte compares by more than the cost of one hash. Only adding per-op-time to the fitness vector can settle that.
