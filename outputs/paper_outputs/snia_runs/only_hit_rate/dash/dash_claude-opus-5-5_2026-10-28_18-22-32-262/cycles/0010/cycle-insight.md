# Cycle 0010 Insight

## Cohort outcome

v001 and v002 built on c9 v001. v001 replaced the CLOCK bit with a persistent, aged 2-bit reuse level. v002 made far-zone ghost credit depend on a per-segment feedback score. v003 started from c7 v002, which has plain LRU and no CLOCK, and added a count-min frequency sketch that sits outside the items and ages by halving. v003 uses the sketch to refuse a gated stash→full-target promotion when the promotee's estimate is below the LRU resident's. The Winner is v003, the first variant in several cycles to make small gains in every cell with no regressions. Its gain was not concentrated at 16K, though, and `web_0` 16K moved only slightly. v001 was flat on its target cell, with noise-level dips elsewhere. v002 helped `web_0` slightly and slipped on `hm_0` 128K, a pattern earlier ghost-reach variants also showed.

## Patterns

- The first lever that worked at all at 16K was choosing **who competes**, not how the two residents rank. v001 shows that even a persistent multi-bit level collapses back to LRU in 2-slot buckets. Contested entrants arrive almost saturated, aging per contest turns the field back into recency, hot items leave for probing, and the newcomer is never compared. v003 puts the promotee itself against the resident it would displace.
- Frequency is useful only when it is stored outside the item, survives moves and evictions, and decays. The ghost works for the same reasons. The c2 in-item counter and v001's in-item level fail because they live inside items that churn.
- Making admission stricter is safe, and making it easier keeps failing. v003 can only delay a promotion by a hit, which is why nothing regressed. That fits the lineage's lesson about crowded targets.
- The sketch's aging window scales with cache size, so at 16K it halves too often to tell candidates apart. This is the likely reason for v003's small gain at the small size.
- `hm_0` 128K rejects every ghost-credit restriction tried so far, now including outcome-based feedback (v002). Its far-credited keys pay off on later returns, not immediate re-hits. A one-stay reward signal misreads them, and fixed factor 2 is still the best setting.
- The lineage has moved to a different branch: the Winner sits on c7 v002, not on the CLOCK line. The CLOCK and level work of c8–c10 added almost nothing on top of c7 v002.
- Per-op-time and metadata-size are still unmeasured, now ten cycles in a row. v003 adds sketch memory and a sketch update on every get, and none of that cost is tracked.

## Open questions

- Would a longer or fixed aging window at 16K, or a wider sketch, let the admission check separate candidates where it currently ties?
- Should ties refuse instead of admit at the small size, or should the check use a margin?
- Can the sketch also help elsewhere: rescue, bypass, victim choice, or weighting ghost credit by estimated frequency instead of reach?
- Does stacking the c8/c9 CLOCK bit onto v003 add anything, or should it stay dropped?

## Continuation

Build on v003. Tune the sketch first: aging window at small sizes, tie policy, width. After that, extend sketch-based judgments to other displacement decisions. Drop in-item counters, CLOCK/level victim ranking, and any further ghost-reach or far-credit gating, since `hm_0` 128K defeats them all. Measure per-op-time and metadata-size, now that the Winner adds per-get work and memory.
