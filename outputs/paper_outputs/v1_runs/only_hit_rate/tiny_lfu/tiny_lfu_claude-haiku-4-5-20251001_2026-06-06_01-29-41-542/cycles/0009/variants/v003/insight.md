# Variant v003 Insight

## Helped

(none)

## Hurt

- hit-rate: Broadening the frequency-cohort threshold from 25th to 40th percentile conflates warm cache residents with genuinely cold items, reducing the selectivity of victim identification and preventing a marginal improvement.

## Takeaway

Expanding the cold-cohort partition from the 25th to 40th percentile of frequency estimates does not improve hit-rate, indicating the 25th percentile threshold is already well-calibrated for this workload. The hypothesis that a broader threshold would capture more appropriate victims by accounting for gentler frequency distributions is unsupported; instead, including items in the higher percentile ranges reduces discriminative power without capturing additional working-set misses.
