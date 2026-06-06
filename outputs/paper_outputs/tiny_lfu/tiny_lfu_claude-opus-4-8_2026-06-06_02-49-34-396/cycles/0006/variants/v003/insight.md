# Variant v003 Insight

## Helped

- (none): No dimension improved. Replacing double-hashing with two independent `FastHasher` passes (`HASH_SEED` / `HASH_SEED2`) left hit-rate flat at the parent's level.

## Hurt

- per-op-time: The second full `FastHasher` pass over the key in `base_hash` doubles the per-op hashing work, lifting both `get` and `put` p50/p99 — exactly the speed cost the hypothesis flagged as the gamble.

## Takeaway

Buying back row independence from double-hashing is not worth the speed it costs on this workload: genuinely decorrelated rows produced the same per-key `min` estimate as the correlated double-hash projections, so admission and victim selection were unchanged and hit-rate did not move — while the extra hash pass made every op slower. The "double-hashing dip" the cross-cycle log tolerated is effectively zero here; the correlation between `h1` and its high-bit-derived `h2` was not the accuracy bottleneck.
