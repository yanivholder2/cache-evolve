# Variant v003 Insight

## Helped

- hit-rate: An independently hashed `s1` (`stash_alt`, salted with `STASH_ALT_SALT`) sends overflow out of a full `s0` to a bucket that isn't next to it, so it is less likely to be full too. More fresh puts and demotees land in `s1` instead of evicting `s0`'s LRU. The gain is real but small. That means stash overflow was only mildly clustered under the `s0 + 1` stride, and adjacent-bucket pressure was not a major source of lost items on this trace.

## Hurt

- per-op-time (get p50 and p99): `promotion_get` still computes `s1` lazily, but two-choice placement puts more residents in `s1`. Every `s1` hit now scans `s0`, target and probing, then pays a salted tuple `hash()`, before it finds the item. Every miss pays the same extra hash. Both paths are common at this hit rate, so the median rose by much more than the "slight" cost the hypothesis predicted.
- per-op-time (put p50 and p99): `stash_insert` calls `core.stash_indices()` up front. So every stash insert pays the second salted hash, including fresh puts and demotion cascades, even when `s0` has room and `s1` is never used. In the parent, `s1 = s0 + 1` cost nothing. The p99 also absorbs inserts that spill to `s1` and still cascade through rescue.

(metadata-size did not change: no fields were added, and `Item` stays at 40 bytes.)

## Takeaway

Swapping the adjacent stride for an independent second stash choice weakly supports H03. Clustering existed but cost little, so "power of two choices" gives only a small hit-rate gain on this stash. The real cost is the extra hash on the `s1`-hit/miss path and on every `stash_insert`. Before porting this onto another lineage, make `stash_insert` compute `s1` only after `s0.is_full()`. Also consider deriving `s1` from unused bits of the primary hash instead of a second full `hash()` call, since the hit-rate gain is too small to justify a doubled hash cost on hot paths.
