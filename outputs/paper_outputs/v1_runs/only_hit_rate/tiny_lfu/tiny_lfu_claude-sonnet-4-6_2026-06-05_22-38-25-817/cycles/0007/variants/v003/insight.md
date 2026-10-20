# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Replacing the sequential integer seeds (0, 1, 2, 3) with four diverse 64-bit hash-mixing constants did not reduce inter-row collision correlation, and instead slightly worsened the column mapping for this workload's key distribution.

## Takeaway

Swapping sequential-integer row seeds for well-separated constants from established mixing sequences produced a marginal hit-rate regression, not the expected improvement. This indicates that `DefaultHasher` already provides sufficient avalanche between rows even when seeded with small integers — the per-row independence the CMS minimum estimator requires was not meaningfully impaired by seeds 0–3, so the correlation the hypothesis targeted was not a real driver of sketch error on this workload.
