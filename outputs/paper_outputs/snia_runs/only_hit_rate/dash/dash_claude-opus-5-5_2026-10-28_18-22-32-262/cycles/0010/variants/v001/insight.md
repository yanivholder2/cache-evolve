# Variant v001 Insight

## Helped

No cell improved. The targeted cell (`web_0` 16K) is exactly flat, and the other three cells changed only at rounding level.

## Hurt

- hit-rate (`web_0` 128K, `hm_0` 16K and 128K): Tiny dips, best read as noise. They come from the few buckets where the min-level rule overrides LRU order. The usual case is a low-level MRU item next to an LRU item that built up a higher level earlier: a bypass promotee entering with level 1, a rescued item, or a foreign retry. Sparing the older item is about as often wrong as right, because the old item's level reflects hits that may already be stale.
- hit-rate (`web_0` 16K): Flat because, in a 2-slot bucket, the new victim rule almost always picks the same item LRU picks:
  - `make_room_clock` runs before the newcomer is placed, so the contest is always between the two current residents, never newcomer vs resident.
  - The MRU resident got there by a promotion that carried its stash hits (level ≥ 2 through the two-hit gate) or by a resident hit (which raises the level). Each contest also ages the spared LRU resident by one. So the LRU item's level is usually ≤ the MRU item's, and ties go to the lower index, which is the LRU slot.
  - With `LEVEL_MAX = 3`, almost every contested item enters at 2 and saturates after one more hit. The field has little range to separate items, and aging by one per contest turns it back into a recency order.
  - A hit on the hottest target item moves it to probing, so target buckets never keep a resident that is both high-level and frequently hit. That is the item the level was meant to protect.

## Takeaway

A persistent, aged 2-bit level does not change victim choice in 2-slot buckets. The gate's hits load every contested entrant with nearly the maximum level, aging by one per contest turns the field back into recency, and the newcomer never takes part in the comparison. So "lowest level, ties to LRU" is the same as LRU in effect. The c8 and c9 problem was not that the counter reset or never decayed. With only two residents and hot items leaving for probing, there is too little difference in reuse history between the two candidates for any frequency signal to use. A lever would have to change who competes, for example letting the newcomer's level decide whether it enters at all, rather than refine the ranking of the two residents.
