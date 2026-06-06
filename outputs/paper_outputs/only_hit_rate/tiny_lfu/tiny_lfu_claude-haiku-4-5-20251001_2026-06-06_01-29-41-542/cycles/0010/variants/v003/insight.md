# Variant v003 Insight

## Hurt

- hit-rate: Dynamic percentile adjustment based on cold-cohort size introduces variability that degrades selectivity; the fixed 25th percentile threshold performs better despite aiming to improve adaptability.

## Takeaway

The dynamic percentile mechanism, which aimed to adapt the frequency threshold to cold-cohort pressure by shifting between 15th and 35th percentiles, failed to improve hit-rate and instead degraded performance. This reveals that the fixed 25th percentile is not merely a locally optimal choice for this workload, but represents a genuine equilibrium point where the balance between availability (candidate victims) and selectivity (rejection of low-frequency arrivals) is optimized. Attempting dynamic adaptation introduces variability that disrupts this balance, suggesting that workload-aware thresholds may require deeper understanding of phase transitions rather than simple cohort-size heuristics.
