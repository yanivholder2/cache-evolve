# Variant v003 Insight

## Helped

- hit-rate: hm_0 at the small size (16384) is the one cell that improved. At this size `reset_period = 10 * capacity` is short compared with the trace, so `halve()` runs many times per run. Old high counts on keys that are no longer hot decay, and recency-driven window candidates on this recency-heavy trace can now beat probation residents that used to be unbeatable. This is the recency effect the hypothesis was aiming for.
- hit-rate: web_0 at the large size (131072) did not lose ground, so the "don't reopen the admission bar" part of the claim held. Ceiling halving (`(c>>1)+(c&1)`) keeps a once-seen resident at 1. Under single counting, a once-seen newcomer is also at 1, so it still loses under strict `>` after a reset. No burst of free admissions appeared, unlike the floor-halving and double-counting versions described in the hypothesis rationale.

## Hurt

- hit-rate: web_0 at the small size (16384) dropped by about as much as hm_0 small gained, so the net summary is flat to slightly down. web_0's gains in the parent came from frequency-based selection: the coldest of the probation head sample loses to a warmer candidate. Frequent halving at small capacity squashes the long-run frequency differences that this contest depends on. Single counting also drops a first-miss candidate's estimate from 2 to 1. Both changes make window candidates win less often against the sampled victim, exactly where web_0 had been gaining.
- hit-rate: both large-size cells moved down slightly rather than up. With a 10×capacity period at 131072, very few resets happen per trace, so aging brings almost no recency benefit there. The small cost of single counting (fewer candidates win) is still paid on every miss.

## Takeaway

Single counting plus ceiling halving is a safe combination: it does not reopen admission. But it is close to a zero-sum swap rather than a gain. It moves hit-rate from web_0 to hm_0 only at the small cache size, and does almost nothing at the large size. That happens because the aging period scales with capacity, so how often resets occur, not how they round, decides the effect. This variant cannot separate the two coupled changes. The next step is to test aging alone, keeping double counting so the candidate-side boost stays, or to give aging a period that does not scale with capacity, to find out whether the web_0 small loss comes from halving or from single counting.
