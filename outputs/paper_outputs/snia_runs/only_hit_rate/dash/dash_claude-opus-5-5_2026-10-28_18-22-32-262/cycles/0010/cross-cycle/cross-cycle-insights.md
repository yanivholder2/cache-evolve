# Cross-Cycle Insights (through cycle 0010)

## Recurring wins

- **Promotee placement away from the victim slot.** An item that just proved itself should land at the MRU end, not the eviction end. This held in c1 (stash demotion), c3 (target promotion) and c5/c6 (probing promotion). It is the most reliable lever in the lineage.
- **History kept outside the item.** The global FIFO ghost ring (c5) and the count-min sketch that ages by halving (c10) are the only reuse signals that consistently moved hit-rate. Both survive moves and evictions, and both decay. Whatever beat them was signal stored inside items that churn.
- **Making admission stricter is safe.** The two-hit gate (c1–c3) and sketch-vetoed promotion into a full target (c10) can at worst delay an item by one hit, so they rarely regress. The c10 sketch check is the first lever in several cycles to gain in every cell.
- **Comparing the promotee with the resident it would displace.** At 16K, deciding who competes (c10) worked where ranking the two existing residents (c8–c10) did not.
- **Moves into free slots only, with no cascade.** These are safe but gain only at 128K. The rescue (c7) and the two-free-slot bypass (c4) both have no regressions and are settled.
- **The ghost lifecycle is settled.** Global FIFO, factor 2, consumed on put. Every attempt to change reach, partitioning or credit has lost somewhere.

## Recurring failures

- **Easier routes into a crowded target.** One-hit return (c3), the last-free-slot shortcut (c3), ghost credit on demotion (c6) and early probing triggers (c5) all lose, worst at 16K, because each sets off an eviction cascade. The exception: on `hm_0` 128K, displacing residents sometimes pays (c8).
- **Counters and bits stored inside items.** Counters reset on moves stay near zero (c2). Counters that never decay pin items permanently (c4). A 1-bit CLOCK (c8/c9) and an aged 2-bit level (c10) both collapse back to LRU in 2-slot buckets.
- **Tuning ghost reach or far-zone credit.** Longer rings (c6), a shorter ring (c7), occupancy-conditioned credit (c8), per-bucket sub-rings (c9) and outcome feedback (c10) each trade `web_0` against `hm_0` 128K or lose outright.
- **Changing the target→probing trigger in either direction.** Delaying it (c4) and speeding it up (c5) both lost. The trigger is near its optimum.
- **Moving the stash away from plain LRU.** Victim choice by counter (c4) and inserting fresh puts at the LRU end (c2) both lost.
- **Cost-only variants.** Per-op-time and metadata-size have not been reported in any of the 10 cycles. Every cost claim is unverified, including the c10 sketch's per-get work and memory.

## Open questions

- Sketch tuning at 16K. Does a longer or fixed aging window, a wider sketch, a margin, or refusing on ties let the check tell candidates apart where it currently ties?
- Can the sketch's estimates drive other decisions: rescue, bypass, victim choice, or weighting ghost credit by frequency instead of reach?
- What actually limits `web_0` 16K? It has been flat through nearly every lever, and only the sketch has nudged it. Candidates are stash size relative to the segment, or saturation that no admission rule can fix.
- The source of the `web_0` 128K giveback after probing `Back` placement (c6) is still unknown. Foreign retries, stash→full-target promotion and ghost reach are ruled out. Ghost churn caused by probing evictions has not been tested.
- Could a value-aware rule that sometimes allows displacement capture the `hm_0` 128K upside without breaking the lineage rule that crowded targets should not be easier to enter?
- What does the stack actually cost per operation and in metadata? This needs an evaluation that reports both.

## Per-policy notes

- **Normal / target buckets (2-slot LRU):** Position already encodes recency, so 1-bit and multi-bit victim ranking adds nothing. Place promotees at `Back`. Bypass the gate only when the target is empty, since the two-free-slot threshold is exhausted.
- **Stash:** Plain LRU, with `Back` demotion and the two-hit exit gate. Gated items that land in a full target can be vetoed by the sketch (current Winner, c10, which descends from c7 v002, not from the CLOCK line).
- **Probing tier:** Place promotees at `Back` (helps `hm_0`). Leave the promotion trigger alone. Displaced foreign residents already handle their own retry.
- **Ghost ring:** A global FIFO keyed on stash evictions, factor 2, consumed on put. The optimum depends on the trace (`web_0` prefers shorter, `hm_0` 128K needs the tail), but no fixed or adaptive variant has beaten factor 2.
- **Frequency sketch (count-min, aging by halving):** The newest lever that works. Its aging window scales with cache size, so it is likely too aggressive at 16K. Tune it before extending it.
- **Per-trace/size:** `hm_0` rewards protecting residents and is sensitive at 128K. `web_0` 128K is driven by admission (returning keys). `web_0` 16K is capacity-saturated and has resisted nearly every lever tried.
