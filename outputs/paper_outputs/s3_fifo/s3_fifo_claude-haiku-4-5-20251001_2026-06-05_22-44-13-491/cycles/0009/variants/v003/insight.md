# Variant v003 Insight

## Helped

- hit-rate: Deeper frequency protection (MAX_FREQ=7 vs 3) allows frequently-accessed items in the main queue to survive longer through more second-chance cycles, offsetting the tighter admission gate and improving overall hit-rate.

## Hurt

(none detected)

## Takeaway

Relaxing structural selectivity while strengthening frequency retention proved effective. The stricter 1/12 small-queue ratio combined with deeper MAX_FREQ=7 protection discovered a beneficial trade-off: by admitting fewer items initially, the cache reserves main-queue real estate for items that survive the frequency threshold and can then accumulate more tenure before eviction. This suggests the optimization surface has a peak where moderate structural gating complements aggressive frequency depth, rather than maximal structural constraint alone.
