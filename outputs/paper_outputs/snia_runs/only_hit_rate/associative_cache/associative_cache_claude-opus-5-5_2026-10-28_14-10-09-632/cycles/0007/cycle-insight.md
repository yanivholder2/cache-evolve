# Cycle 0007 Insight

## Cohort outcome

All three variants built on cycle 0006 v003 (GCLOCK main with ceiling 7 + floored probation FIFO). v001 added a recent-hit bit and halving decay to main. v002 capped the promotion sweep and rejected promotion when main was hot. v003 made the one allowed depth test, raising the small-bucket floor between 64 and 96. v001 is the Winner. It gained most on web_0 at the large size, which had been stuck, and only just cleared the bar on its target, hm_0 at the large size. It cost a little on hm_0 at the small size. v002 gave bit-identical results to the parent, and v003 regressed both small-bucket cells.

## Patterns

- Recency decay in main helps by clearing out stale blocks, not by keeping near-miss blocks. Halving gated on recency cleared once-hot web_0 blocks faster and never hurt long-lived ones. But it barely moved hm_0, so the cycle 0006 "missing recency signal" theory explains only a small part of hm_0's large-size gap. Small buckets gained nothing, because their short main sweep already decays stale counters quickly.
- Coarser ranking costs capacity in small buckets. Bit-length ranking merges counter tiers, which undoes part of what ceiling 7 gained in the small main region. This matches the cohort-wide finding that small buckets depend on main's capacity and how finely it ranks, and they lose whenever either one shrinks.
- With reset-to-0, main always has cold victims. Every promotion puts a counter-0 block into main, and most web_0 promotions are never hit again. So the "hot main under promotion pressure" case almost never happens, which refutes H02's premise. Admission gates keyed on main being full of hot blocks are a dead end. The cycle 0006 worry about worst-case sweep cost was also mostly theoretical.
- Probation depth is closed. v003 met both refutation conditions. With ceiling 7, smaller main did not become more space-efficient, so 64 is a local optimum. This makes three cycles of evidence that probation depth is used up.
- Main-side levers now look nearly used up too: depth, admission strength, counter ceiling, admission gating and recency weighting have each been tried. The remaining hm_0 large-size gap most likely sits outside main: which blocks get promoted, or the near-miss history that cycle 0005's ghost captured.

## Open questions

- Can a small-bucket fix bring back fine tiers under the recency rule, for example ranking by recent bit plus raw counter instead of bit length, without giving up the web_0 large-size gain?
- Among main's counter-0 blocks, would choosing the victim by age or by whether it was a fresh promotion beat choosing the first one from the hand, given that main's victims are almost always fresh promotions?
- Can near-miss history be used for something other than admission (cycle 0005 refuted admission because it re-admits web_0 single-return blocks), for example as a tie-break among probation evictions?
- Did v002's evaluation really run the overlay? The code analysis predicts identical results, but a build-reuse check is worth doing once.

## Continuation

Make cycle 0007 v001 (recency bit + halving decay) the parent, and keep reset-to-0 on promotion. Next, try to recover small-bucket tier resolution under the recency rule, and look at which counter-0 block main evicts or which probation hits get promoted. Do not test probation depth again, or admission gates conditioned on main being hot. Higher ceilings, ghost admission and non-zero promotion counters stay retired. Per-op-time is still unmeasured, though v001 reduces the worst-case sweep.
