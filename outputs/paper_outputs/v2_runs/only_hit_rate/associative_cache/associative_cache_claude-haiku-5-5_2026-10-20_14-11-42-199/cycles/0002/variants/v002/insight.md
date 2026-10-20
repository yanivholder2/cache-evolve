# Variant v002 Insight

## Helped

## Hurt

- hit-rate: The 8-bit saturating age counter with lowest-index tie-breaking coarsens recency ordering relative to the baseline's full-width recency, which can only cost hit-rate; the measured effect is a slight drop, within the hypothesis's stated tolerance.

## Takeaway

The hypothesis's central claim, that a u8 age counter shrinks per-entry metadata, is not tested by this variant's evaluation: the fitness vector reports only hit-rate, with no metadata-size or per-op-time dimension, so the size benefit remains unverified. The hit-rate result suggests the recency approximation is safe on this workload, but a rerun that includes metadata-size is needed before the trade-off can be judged.
