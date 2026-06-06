# Variant v001 Insight

## Helped

- hit-rate: Frequency-cohort filtering ensures victim selection targets only the coldest items first (25th percentile of frequencies), preventing warm cache residents from being evicted while cold candidates remain.

## Hurt

(none)

## Takeaway

Replacing the burstiness-factor scoring with explicit frequency-cohort filtering—where victim candidates are partitioned by the 25th percentile of frequency estimates before scoring—produces a marginal hit-rate improvement. This suggests that deterministic cold-item prioritization outweighs the added complexity of burstiness factor tracking, and that the simpler two-pass approach (cold cohort first, fallback to full scan) is more effective than attempting to embed temporal clustering directly into the eviction score formula.
