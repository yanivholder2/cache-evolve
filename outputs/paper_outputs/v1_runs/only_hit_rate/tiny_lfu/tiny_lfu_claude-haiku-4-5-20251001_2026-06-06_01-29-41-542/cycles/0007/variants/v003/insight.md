# Variant v003 Insight

## Hurt

- hit-rate: The collision-detection heuristic (using 25th percentile when coefficient of variation > 0.15) degraded admission decisions, causing more wrong evictions and misses than the simpler minimum-based frequency estimate.

## Takeaway

Attempting to correct for hash-collision noise in the Count-Min sketch by detecting high coefficient of variation and switching to a percentile-based estimate backfired. The simpler minimum-based approach, despite potential collision artifacts, made more effective frequency comparisons for the admission gate. The heuristic's assumptions—that CV > 0.15 reliably signals collisions, and that the 25th percentile better represents true frequency—did not hold on this workload.
