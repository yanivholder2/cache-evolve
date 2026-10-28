# Variant v001 Insight

## Helped

- hit-rate (128K, `web_0`): This cell rose a little, though by an amount that may be noise. At the larger size, residents survive long enough to be hit while sitting at LRU. When a gated stash promotion or a probing promotion then lands at MRU with its bit clear, `make_room_clock` demotes that newcomer and keeps the proven LRU resident. That is a small dose of scan resistance for long-lived keys.
- hit-rate (128K, `hm_0`): This cell rose marginally, by the same mechanism. Probing hits now set the bit, so a re-referenced probing resident survives the next target→probing promotion instead of being the LRU victim.

## Hurt

No cell regressed in a real way. The targeted cell is the miss:

- hit-rate (16K, `web_0`): This is the cell the hypothesis aimed at, and it is flat. In a 2-slot bucket, CLOCK makes a different choice from LRU only in one state: the LRU item has its bit set and the MRU item has its bit clear. That state is rare here, for three reasons:
  - A hit on a non-hottest target item sets its bit and also moves it to MRU, which leaves the other item, unreferenced, at LRU. That is the same victim LRU would pick.
  - A hit on the target's MRU item promotes it to probing, so it never carries a bit in the target.
  - At 16K, residents turn over before they collect a second hit.

  So the scan almost always picks the LRU item anyway.
- hit-rate (16K, `hm_0`): Unchanged, for the same reason. Under saturation the scan almost always finds bit 0 at index 0 and matches LRU.

## Takeaway

A 1-bit CLOCK reference in 2-slot normal buckets mostly agrees with LRU. ClassicLRU's move-to-MRU on a hit already puts the referenced item out of the victim slot. The hottest-in-target hit leaves the bucket entirely. CLOCK only overrides LRU when a fresh, unreferenced item has just been placed at MRU ahead of a referenced LRU resident. That only matters in the larger cache, so the bit buys almost nothing where `web_0` 16K is stuck. To change victim choice in a saturated 2-slot bucket, a later attempt needs one of two things. It could use a signal that LRU order does not already encode, such as reuse that survives across stash stays or ghost-derived frequency. Or it could stop moving hit items to MRU, so that the reference bit becomes the only recency signal.
