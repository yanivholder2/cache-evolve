# Cycle 0009 Insight

## Cohort outcome

All three variants built on cycle 0008 v003 (GCLOCK ceiling 7 + recent bit + halving, floored probation, correlated-reference filter, reset-to-0) and tested the three combinations that cycle 0008 proposed. v001 folded in linear decay, gated to small main regions, and is the Winner. It lifted both 16384 cells, including web_0 at the small size, which had been v003's weak cell, and left the 131072 cells bit-identical. v003 (the veteran grace bit on top of the filter) helped a little everywhere and hurt nothing. v002 (a wider small-bucket window) traded web_0 for hm_0 at the small size and was not confirmed.

## Patterns

- The size-gated split now works as intended. Small buckets want fine linear tiers and large buckets want halving. Under the filter, linear decay gained more than it did on its own: once burst-only blocks are kept out of main, the finer tiers are spent ranking blocks that really are reused.
- The two changes aimed at counter-0 promotions overlap. The veteran grace still adds something on top of the filter, but only part of what it gained alone, because the filter has already removed many of the fresh promotions the grace used to outrank. It still costs nothing, though, so it is worth keeping.
- The correlated-reference window does not scale as an absolute width. Sixteen slots is a small fraction of the large probation ring but a quarter of the small one, and at that width it starts to filter hm_0's real short-gap reuse. The window is close to what it can do in small buckets.
- Small-bucket web_0 is no longer limited mainly by bursts. The lever that moved it this cycle was main-side ranking (linear tiers), not the promotion filter.
- Large buckets moved only through v003's grace. The large-size hm_0 gap is still largely unexplained.

## Open questions

- Does v003's veteran bit stack with v001's linear decay? In small buckets both act on main ranking, so the grace's small-size gain may vanish. Its large-size gain should survive, because v001 does not touch large buckets.
- Is there a small-bucket window strictly between the current narrow one and v002's quarter-ring width that filters web_0 bursts without catching hm_0 reuse, or is the window used up at that size?
- What still limits hm_0 at the large size? Main ranking, recency and promotion filtering have each moved it only a little.

## Continuation

Make cycle 0009 v001 the parent, and first add v003's veteran bit on top of it. Expect gains mainly at 131072, and check whether the small-size gain survives under linear tiers. Stop widening the small-bucket correlated window. At most, test one intermediate width, and do not use an absolute 16. Keep reset-to-0. Probation depth, hot-main admission gates, higher ceilings, ghost admission and non-zero promotion credit all stay retired. Look for a new lever for large-size hm_0 rather than tuning main ranking further. Per-op-time is still unmeasured, and linear decay brings back a longer worst-case sweep in small buckets.
