# Variant v001 Insight

## Helped

- hit-rate: The only change was to `with_layout`. Paired buckets now get a 15/16 protected cap (16 → 15, 32 → 30, 64 → 60 instead of 14, 28, 56), which cuts probation roughly in half in buckets of 32 or more slots. More re-referenced keys stay shielded from the probation-LRU eviction path, and hit rate landed exactly on the target the hypothesis took from cycle 0007 v003. The cap gain therefore carried over intact onto the tag-first bucket, which confirms that tags never change a replacement decision.
- per-op-time (put tail): The hypothesis did not predict this. Put's p99 improved while its median held. The likely cause is that a full-bucket cold insert does `rotate_left` over `items[0..=len0/2]` and the matching `tags` slice. With a smaller probation segment, that rotate is shorter, and so is the `len0 / 2` midpoint. This is an inference from the code, not a measured breakdown.

## Hurt

None. get p50 and p99 and put p50 held at the parent's values, as the hypothesis predicted. A larger protected segment could in principle lengthen the protected-hit `rotate_left(1)` over `items[pos..]`/`tags[pos..]`, but no cost showed up at this bucket size. metadata-size is unchanged because neither `Item` nor the cache header was touched.

## Takeaway

The protected-cap lever and the tag-first lookup lever really are orthogonal. The ratio rule moved over in isolation, as a single `with_layout` edit, and it reproduced v003's hit rate on top of v001's latency with no interaction penalty. The parent's open concern was the protected-segment rotate tail. Growing protected to 15/16 did not make that tail worse, and shrinking probation actually shortened the cold-insert rotate on put. So, at this size, pushing the cap further toward all-protected adds no latency cost.
