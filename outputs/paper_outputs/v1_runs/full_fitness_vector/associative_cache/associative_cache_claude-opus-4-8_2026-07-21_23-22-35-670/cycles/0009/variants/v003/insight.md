# Variant v003 Insight

## Helped

- metadata-size: Swapping the `reuse: Vec<bool>` for a `hits: Vec<u8>` saturating counter kept the protection state off the measured `Item`, so the two-hit threshold buys no metadata — `Item` holds the 16-byte floor bit-for-bit.

## Hurt

- hit-rate: Requiring a second hit before granting protection under-protects genuinely hot keys — a key must be touched twice to survive, so hot items that enter mid-scan are evicted before their second hit lands, and the rate falls back below v002's 0.7267 rather than past it.
- per-op-time: Replacing v002's unconditional bit-set with a per-hit `< PROTECT_THRESHOLD` compare-and-saturating-increment widened the get/put p99 tails, though p50 is unchanged.

## Takeaway

Raising the admission bar from one hit to two does not filter enough scan noise to break the 0.7267 ceiling — it regresses, because the same threshold that rejects one-hit-wonders also delays protection for real hot keys past the point where a scan can evict them. The one-hit binary protection of v002 sits at or above the sweet spot for this workload; the hit-count-to-earn-protection knob has no headroom above it, resolving the open "is there hit-rate room left on the in-residency protection lever" question in the negative.
