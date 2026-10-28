# Variant v003 Insight

## Helped

- hit-rate: web_0 at the small size improved slightly. At 16384 the ghost horizon is short (about 8K rejections), so the +1 bonus mostly fires for keys that really do recur at short reuse distances. Breaking their ties with cold probation-head entries in their favour is a small, real gain on this trace. The gain is small enough that it may be noise.

## Hurt

- hit-rate: the hypothesis's target, hm_0 small, went slightly *down* rather than up, so the claimed mechanism is refuted. The bonus only changes the outcome on an exact tie, because the comparison is still strict and adds just +1. This sketch never ages, and under the harness's get-miss-then-put double counting a returning key has already gained frequency on its own. Exact ties therefore happen mostly at low counts, between a cold returning key and a cold probation-head victim. In that case the bonus just swaps one cold entry for another, with no real recency signal behind the swap, and on hm_0 that swap loses slightly more often than it wins.
- hit-rate: web_0 at the large size regressed, which is the clearest move in this variant. With `ghost_cap = capacity/2`, the ghost at 131072 remembers about 65K rejections. That long horizon lets scan-like or long-loop keys on web_0 return while they are still "ghosted". They then win ties against probation entries that genuinely recur, and erode the frequency filter that the parent's sampled-victim contest had strengthened. This is the exact failure the rationale claimed to avoid ("scan keys seldom return within that horizon"), and it shows the horizon is too generous at large sizes. hm_0 large was essentially flat.

## Takeaway

A +1 re-entry bonus that applies only on exact ties is too weak to act as a deliberate recency signal for hm_0. Because the sketch never decays, ties occur almost only among cold keys, so the bonus mostly reshuffles cold entries. Its one measurable effect is harmful: a ghost horizon that scales with capacity (capacity/2) lets returning scan keys past the filter on web_0 at large sizes. If an explicit re-entry signal is tried again, it needs either a fixed or much shorter ghost horizon, or a bonus that matters at non-trivial frequencies (for example, a larger or relative bonus, or sketch aging so that ties are meaningful). Re-introducing a tie-breaker by itself will not recover the hm_0 small hit-rate the lineage lost.
