## Recurring wins

- Soft, relative filtering beats hard rules. SLRU probation (c1), W-TinyLFU's tie-admitting duel against the current victim (c2), and the tightened sketch (c3) each won. Every hard gate, bypass or override lost.
- Strict one-shot filtering is the main source of hm_0 gains. Dedup on miss-then-put, the doorkeeper, the long reset period and a large protected segment have all been confirmed as load-bearing (c3, c4, c7).
- An adaptive front window is the only lever that has meaningfully lifted small-size web_0 (c3 hinted at it; c4 confirmed it with the largest single gain of the run).
- Scale-aware parameters work. Capping climber knobs at absolute values (c4) and gating changes by cache size (c7 large-only step) removed the losses at one size extreme that capacity-proportional settings kept causing.
- Changing one lever per variant gives results with a clear cause (c4 onward). Bundled changes (c3) could not be attributed.

## Recurring failures

- Loosening admission has lost in every form tried: unconditional in-window bypass (c4), in-window reuse duel bonus (c5), random admission of warm duel losers (c7), faster sketch aging (c4). Small-size hm_0 always pays the most.
- In-window reuse as an extra signal counts the sketch's evidence twice and tilts close duels toward recency. It adds no independent information.
- Promoting ghost or non-resident history straight to protected (c2) overweights a weak signal on web_0.
- Fixed capacity-proportional fractions (1% window or HIR share, capacity-sized ghost list, climber knobs scaled linearly) fail at one end of the size range (c2, c3).
- Climber tuning has run out (c5–c7): ceiling, shorter or longer period, larger fixed step, two-drop hysteresis and capacity-tied period were all neutral or worse. Further initial-step increases are also used up.
- Global main-area split changes (shrinking protected) hurt hm_0 at both sizes (c7).
- LIRS and HIR-share tuning only trades hm_0 against web_0 (c2, c3).
- Measurement debt: for seven cycles `metadata-size` and `per-op-time` have never been reported, and `win_cap` has never been read. Cost-axis claims and climber diagnoses remain inferred.

## Open questions

- What still limits small-size web_0? The doorkeeper's cold first-touch estimates is the remaining untested suspect, raised since c4. Test it by disabling or weakening it at small capacity only.
- Can probation be lengthened at large size only, keeping c7's large-size web_0 signal without exposing hm_0's protected set?
- Can a separate shrink step, or decay that depends on cache size, recover the small large-size hm_0 loss from the c7 step change?
- Where does `win_cap` settle in each cell, and does it oscillate at small size?
- How do the sketch, doorkeeper, window and climber compare with plain SLRU or CLOCK once the cost axes are measured?

## Per-policy notes

- LRU (baseline): beaten in every cycle; serves only as the reference.
- SLRU: a reliable base (c1 Winner). Gains grow with capacity. A large protected segment is essential for hm_0. Global split changes hurt; large-size-only split changes are untested.
- CLOCK: slight large-size web_0 loss because it decays toward FIFO when bits stay set. Its expected cost advantage is still unmeasured.
- TinyLFU over plain LRU: a strict-greater gate hurts recency traffic. Do not retry it.
- W-TinyLFU over SLRU: the current lineage (c2 → c3 v002 → c4 v002 → c7 v001). Keep: tie-admitting deterministic duel, wide sketch with dedup, doorkeeper, long reset, capped hill-climbed window with floor, large-size-only bigger initial step.
- Ghost-list SLRU: ghost-to-protected promotion hurt. Ghost history may only be useful as input to the admission comparison.
- LIRS: competitive with LRU but never beat W-TinyLFU. HIR tuning is a trace trade-off. Shelved.
