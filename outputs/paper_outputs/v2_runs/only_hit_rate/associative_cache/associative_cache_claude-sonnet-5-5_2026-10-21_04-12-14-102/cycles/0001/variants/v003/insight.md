# Variant v003 Insight

## Helped

- (none): No measured dimension improved; the half-promotion did not translate into a better hit-rate.

## Hurt

- hit-rate: Moving a first-hit item only to the midpoint of its bucket left it exposed to eviction, and with small associativity the "protected" zone is too short for the policy to separate scan items from loop items, so hit-rate came out slightly below the plain move-to-back LRU.
- metadata-size / per-op-time: These dimensions are absent from the fitness vector, so no claim can be made. The change added no fields (it reuses `lfu_counter`), but each hit now does an extra counter write and a partial `rotate_left`.

## Takeaway

A 2Q-style "first hit goes halfway, second hit goes to the back" rule inside a small associative bucket did not beat full-recency LRU on this LIRS trace. Buckets are short, so the midpoint offers little extra scan protection, while demoting first-hit loop items makes them more likely to be evicted before their second reuse. Scan resistance probably needs a larger or separate probationary region, not just a shallower promotion.
