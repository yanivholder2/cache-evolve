# Variant v001 Insight

## Helped

- hit-rate: On web_0 at the large size, splitting main into an 80% protected segment and a 20% probation segment moved the hit rate clearly above v003's level, which H01 had set as the refutation threshold. Entries hit at least once since entering main are now shielded in protected. Both the lazy-rebalance victim and the contest victim are now taken from probation LRU rather than from a single main LRU, so the eviction pressure that v003's climber could not relieve at this size now falls on main entries that have never been re-referenced. web_0 at the small size also gained a little, which matches the hypothesis that SLRU victim choice adds to the adaptive window instead of overlapping with it.

## Hurt

- hit-rate: hm_0 dipped slightly at both sizes. Probation holds only 20% of main, so a newly admitted or newly demoted entry faces the next contest against a window candidate after a much shorter stay than in v003's single main LRU. For hm_0's more recency-driven reuse, losing entries that would have been re-referenced a bit later costs more than protecting the hit-once set gains. The never-aged u32 sketch probably makes this worse. When a stale protected entry is demoted to probation, it keeps its old high count, so it can still beat fresher window candidates in the contest.

## Takeaway

SLRU and the hill-climbing window do stack where v003 was stuck. Protecting re-referenced main entries is the lever that moved web_0 at large capacity, which the window climber alone could not do. The fixed 80/20 split is a trade-off and not a free win: it slightly hurts hm_0. This suggests the protected fraction (or sketch aging, so that demoted stale entries lose their contest advantage) is the next thing to adapt, rather than leaving it constant.
