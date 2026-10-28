# Variant v001 Insight

## Helped

- hit-rate (128K, `web_0` and `hm_0`): Both cells moved up by a rounding-level amount, which is best read as noise and not as evidence for the mechanism. At this size the state where the carried bit can matter does occur a little more often: a ghost-origin newcomer at MRU next to a referenced LRU resident. Even there, the bit doesn't protect the newcomer. It only changes which resident's bit is cleared before the fallback runs (see Hurt).

## Hurt

No cell regressed. The cell the hypothesis targeted is the miss:

- hit-rate (16K, `web_0`): Flat. The change is wired correctly: the flag sets on a ghost hit, stays masked in the gate, carries into the target as bit 1, and is cleared on every other entry. But in a 2-slot bucket a set bit on the MRU item cannot save that item:
  - The promotee always lands at `STASH_PROMOTION_TARGET_POS = Back`, the MRU slot.
  - `make_room_clock` scans LRU→MRU and stops at the first clear bit. If the LRU resident has bit 0, it is evicted, which is what LRU does anyway. If the LRU resident has bit 1, the scan clears it, then clears the newcomer's bit, finds no victim, and falls back to plain LRU eviction.
  - So carrying bit 1 never protects the newcomer. Its only effect is to cancel the protection the parent gave a referenced LRU resident, sending that bucket back to LRU behaviour.
  - The newcomer also tends to lose the bit before it can matter. A hit on it, as the MRU item, promotes it to probing, and `promote_to_probing` resets the counter to 0.
- hit-rate (16K, `hm_0`): Unchanged, for the same structural reason. Under saturation the scan almost always finds bit 0 at index 0.

## Takeaway

Setting the reference bit on an item that is also placed at MRU adds nothing in a 2-slot CLOCK bucket. Position already protects the MRU item, and CLOCK's LRU-first scan with all-referenced fallback means a bit set on the MRU item can only cancel CLOCK's protection for the LRU item. The ghost's reuse signal is real, but to affect victim choice it has to come with a placement that position does not already protect. One option is a ghost-origin promotee landing at the LRU end with bit 1, so CLOCK spares it over an unreferenced MRU neighbour. Another is a multi-bit counter that survives more than one scan.
