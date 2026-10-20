# Variant v002 Insight

## Helped

- per-op-time (get p50): the hypothesis's pass criterion is met, if only narrowly. `get_position_tagged` scans the inline `tags[..len]` first and loads `items[j]` only when a tag matches, so the many "not here" probes no longer follow the `Vec<Item>` pointer. These are the `s0` misses before a target hit and every probe of a full miss. This confirms the parent's diagnosis that the item-buffer miss was a real per-probe cost. Removing the `Box<dyn Bucket>` hop had not touched it.
- per-op-time (get p99): the `get` tail improved more than the median did. This fits the hypothesis's own branch: avoided item-buffer misses count for more in the tail, where a miss probes the whole 1–4 bucket chain (target → probing → `s0` → `s1`) and the parent paid one cold heap load per bucket.
- per-op-time (put p50/p99): `put` improved at both p50 and p99. `put_at_tagged` runs its "key already present?" check through the same tag filter, and when the caller already has the hash, the tag is passed in instead of recomputed. So the existence check before each insert usually skips the item buffer too. Part of this gain may just be the parent's `put` regression (which the parent insight called likely layout or variance) going away, so re-measure before crediting all of it to tags.
- hit-rate (held, as required): tags only filter. A tag match always falls through to the full key compare, and the tag array is rotated or shifted to match every items mutation (`get_from_position` rotate, `remove_at` copy_within, Front/Back insert). Victim choice and recency order are unchanged. Paths that mutate the raw items (`get_items_mut`, `retain` in `remove`) clear `tag_ok`, so lookups there fall back to a plain key scan and stay correct.
- metadata-size (held): `Item` and `Dash` are untouched. All the new state lives in the bucket header behind `SegmentCore`'s `Vec`.

## Hurt

- per-op-time (layout, latent): the code departs from the hypothesis's stride budget. It sizes the tag array at `MAX_SLOTS = 64`, not at the largest capacity the parent actually uses, and keeps `max_size` as a `u32`, not a `u8`. With `#[repr(C)]`, the header grows from the parent's 48 bytes to about 112 bytes (Vec + u32 + bool + `[u8; 64]` + fat `Box<dyn EvictionPolicy>`). That is more than double the stride, and a header now spans about two cache lines, though a 2-slot bucket only ever reads the first few tag bytes. The gains above came despite this. So the measured result is a lower bound on what tags can do, not the full win.

## Takeaway

Inline per-slot tags in the bucket header beat making the bucket concrete (v003). They attack the cost that is actually on the critical path: the dereference of the item buffer on probes that miss. `get` improved at both p50 and p99 while behaviour stayed bit-identical, and the larger p99 gain shows avoided heap loads matter most on multi-bucket miss chains. The obvious follow-up is free and still open: shrink `tags` to the real bucket or stash capacity and `max_size` to `u8`, so the header gets back to roughly the parent's stride. That would show whether the extra cache line of padding was hiding part of the gain.
