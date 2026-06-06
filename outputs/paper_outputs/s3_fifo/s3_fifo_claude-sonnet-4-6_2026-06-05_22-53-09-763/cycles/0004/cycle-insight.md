# Cycle 0004 Insight

## Cohort outcome

All three variants probed retention depth and admission quality: raising the second-chance frequency ceiling (v001), widening small-queue admission (v002), and granting ghost-re-admitted items a head-start credit (v003). Only the ceiling raise won, and the gain was real but modest.

## Patterns

- **MAX_FREQ = 4 yields a genuine but incremental gain.** A non-trivial minority of items reach the old ceiling and benefit from the extra retention pass, confirming the upward direction is viable — but the small magnitude of the improvement suggests this minority is limited, and proportional returns from further ceiling increases are unlikely.
- **The freq > 0 admission gate is definitively closed.** Tried twice (neutral solo in cycle 0003, harmful here in combination with ghost precision), it never helps. Diluting main with single-access items costs more in churn than the marginal ghost-hit promotions can recover; the two mechanisms are orthogonal rather than synergistic on this workload.
- **Ghost re-admission credits are counterproductive.** Seeding re-admitted items at freq=1 displaces incumbents that earned their slots through genuine accesses. The ghost signal confirms re-use potential, but not enough to justify claiming a residency advantage over established main-queue residents.

## Open questions

- Is there a useful stopping point between MAX_FREQ = 4 and a higher ceiling (e.g., 5 or 6), or does the frequency distribution of hot items make any further step negligible?
- With admission thresholds and ghost sizing now well-characterised, are there structural axes entirely unexplored — such as eviction ordering within main, hit-count handling on ghost re-admission at freq=0, or adaptive queue sizing under phase changes?

## Continuation

Next cycle should adopt MAX_FREQ = 4 as the new baseline and make one cautious test of MAX_FREQ = 5, but with low expectations given the already-small delta from 3 → 4. The admission threshold and ghost credit directions are exhausted and should not be revisited. If the MAX_FREQ = 5 probe is also incremental or flat, the highest-yield path forward is likely a genuinely new structural axis — the queue-sizing and admission-threshold knobs have been mapped to their effective limits.
