# Variant v002 Insight

## Helped

- hit-rate: Flipping `DashSegment::get` to `promotion_get(..., InsertPosition::Back)` lands stash→target and target→probing arrivals at the MRU end, so a single follow-up access advances a hot key up the ladder instead of forcing it to re-climb from LRU first — yielding a marginal hit-rate gain on the skewed trace.

## Hurt

- (none): Only the insert-position argument flipped; routing, the `is_hottest` gate, and the `put` cold-insert path stayed unchanged, so capacity and admission behaviour were untouched and no dimension regressed.

## Takeaway

On this high-skew LIRS trace, accelerating promotion by inserting at MRU rather than LRU produces only a sliver of improvement over v001 — the "earn your way up" delay was barely costing hits because hot keys on a skewed workload re-touch fast enough to climb regardless. The promotion-position lever is nearly exhausted; the protected-tier geometry inherited from v001 remains the dominant driver, and further gains likely lie elsewhere than insert ordering.
