# Variant v001 Insight

## Helped

(none)

## Hurt

- hit-rate: Tightening the frequency-cohort threshold to 15th percentile over-restricts the cold-item pool, making it too small to consistently find viable eviction candidates in the first pass and forcing more frequent fallback scans that settle for worse victims.

## Takeaway

The 25th-percentile threshold was closer to optimal than 15th percentile. While the intent to reduce contamination of warm-but-old items was sound, the tighter threshold sacrifices practical victim availability—the cold cohort becomes so selective that fallback behavior dominates, offsetting any precision gain. This suggests the original 25th-percentile balance was already near the precision-recall sweet spot for victim selection.
