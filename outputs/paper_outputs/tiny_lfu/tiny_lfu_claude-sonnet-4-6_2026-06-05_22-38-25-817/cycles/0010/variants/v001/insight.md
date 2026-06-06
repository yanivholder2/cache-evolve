# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Lowering `PROMO_THRESHOLD` from 6 to 5 admitted frequency-5 items into the protected segment, but the extra churn this caused — displacing genuinely hot items that had already earned their protected seats — outweighed any benefit from capturing the new frequency slice.

## Takeaway

The frequency-5 cohort on this Zipfian workload is too thin to justify the additional protected-segment turnover it creates: the items newly eligible for promotion at threshold 5 are not reused often enough to recoup the eviction pressure they place on higher-frequency incumbents. This suggests that `PROMO_THRESHOLD = 6` sits at the productive floor for the current sketch accuracy; the threshold-reduction strategy that succeeded from 8→6 (cycle 0009) does not continue to yield gains at 6→5, indicating diminishing returns as the frequency band being unlocked grows increasingly sparse.
