# Variant v002 Insight

## Helped

No dimension moved. All four hit-rate cells (`web_0` and `hm_0`, at 16K and 128K) match the parent v001 to the reported precision.

## Hurt

- hit-rate (16K, `web_0`): This is the cell the hypothesis targeted, and it is flat. The extra-hit gate only fires when a full 2-slot target has both reference bits set, and v001's mechanics make that state very rare:
  - A hit on the target's MRU ("hottest") item sends it to probing instead of setting its bit.
  - A hit on the LRU item sets its bit and also moves it to MRU, which makes it the next one to leave on a hit.
  - So both bits end up set only after alternating LRU hits on both residents, with no hit on the MRU item in between.
  - Every way into the target (stash promotion, foreign retry, rescue) starts the item with its bit at 0, and each promotion's `make_room_clock` clears the bits it passes, so the state resets after every admission.
  - v001's insight already found that 16K residents turn over before they collect a second hit, so the precondition is rarest exactly where the gate was aimed.
- hit-rate (128K, `hm_0` / `web_0`; 16K, `hm_0`): Unchanged, as the design intended. Wherever a cold resident exists, `required` falls back to `STASH_PROMOTION_HITS`, so behaviour matches v001. Since the all-referenced state almost never occurs, that is effectively always the case.

## Takeaway

Using "every resident of the full target has its CLOCK bit set" as the admission signal does nothing in this design. Hit handling (MRU hit → probing, LRU hit → move to MRU) and the bit clearing on entry and on `make_room_clock` almost never let both bits in a 2-slot target be set at once, so the extra-hit gate is close to dead code. A value-aware gate needs a "resident value" signal that builds up under this hit-handling scheme. One option is a per-resident hit count that is not cleared on entry or by the CLOCK sweep. Another is ghost-derived reuse. A single reference bit is not enough: it gets cleared, or its item gets moved, before the next stash hit can ever read it.
