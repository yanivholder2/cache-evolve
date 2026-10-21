# Cycle 0001 Insight

## Cohort outcome

All three variants started from the baseline, and each tried one separate idea: a W-TinyLFU window/segmented layout (v001), sketch aging plus conservative update (v002), and a single fast hash with a flat sketch (v003). The only Winner was v003, a speed-oriented change that is neutral on policy. Its hit-rate gain was small and probably comes from a different pattern of hash collisions, not from a better policy. Neither of the two policy changes beat the baseline in a way that mattered.

## Patterns

- Changing the resident structure (v001) did almost nothing at size 2048. The 1% window holds only about 20 slots, so admission is still decided by the same frequency contest that never ages.
- Removing frequency history (v002) hurt hit-rate. On workload-a, the baseline's long-run counts were worth more than the freshness that decay added. When counts are low and coarse, the strict-`>` contest mostly ends in a tie, and the victim stays.
- Every variant's evaluation reported hit-rate only. Per-op-time and metadata-size were never measured, so the dimension that H03 actually targeted (lower get latency) remains unverified.
- Both policy variants point to the admission contest as the real bottleneck: how estimates are compared and how ties are broken. List layout is not the bottleneck.

## Open questions

- Does aging fail because the sample size (10×capacity) is too small, or because of conservative update? v002 changed both at once.
- Would a larger window, or an adaptive one (hill-climbing), let v001's layout pay off?
- Does v003 actually lower get/put latency? This needs a benchmark run that reports per-op-time.

## Continuation

Build on v003, which keeps the fast hashing. First get a run that reports per-op-time and metadata-size, so the speedup can be confirmed. For hit-rate, aim at the admission contest. Test aging alone, without conservative update, at a much larger sample size, and/or try a tie-breaking or doorkeeper rule. Also try pairing a larger or adaptive window with that aging. Drop pure restructuring of the segments at a 1% window, and drop aggressive halving at 10×capacity.
