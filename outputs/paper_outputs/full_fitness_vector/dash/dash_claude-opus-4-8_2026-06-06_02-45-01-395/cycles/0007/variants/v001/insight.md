# Variant v001 Insight

## Helped

- per-op-time: Pre-reserving `target_heat` to `num_of_normal_buckets * bucket_size` at `try_new` eliminated the incremental rehash-and-reinsert allocations the table previously paid as heat entries accumulated, dropping put-p99 the most and nudging put-p50 down — the put path's `target_heat.remove` and migration work no longer collides with a growing/relocating table.

## Hurt

- (none) — no dimension regressed; get-p99 moved one ns (within noise) and hit-rate/metadata-size are bit-for-bit identical to the parent as predicted.

## Takeaway

Layering capacity pre-reservation on top of the inherited `U64BuildHasher` did compose with no hit-rate or metadata cost, but the tail win landed on the put path, not the get-p99 the hypothesis named as the target. The get path was already pinned at its hasher-set floor by the parent, so the reservation had no get-tail rehash spike left to remove — its only headroom was the table-growth cost the put-side hit operations were absorbing. The two wins are additive in the sense of "both free and both present," but they act on different ops than the disjoint-cause model assumed.
