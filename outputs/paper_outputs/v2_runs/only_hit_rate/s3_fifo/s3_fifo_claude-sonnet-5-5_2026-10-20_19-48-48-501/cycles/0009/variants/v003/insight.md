# Variant v003 Insight

## Helped

- hit-rate: Shrinking the small queue to about 3% of capacity and shortening the ghost queue leaves more room in the main queue for re-referenced objects. It also filters one-hit wonders faster. Together these beat the baseline's larger small queue and ghost on this trace.

## Hurt

- metadata-size, per-op-time: Neither was reported for this variant, so no effect can be claimed. The change only altered two sizing constants and added no fields or hit-path work, so no regression is expected.

## Takeaway

On a single LIRS trace at one cache size, an even smaller small queue than the baseline's still improves hit rate when paired with a shortened ghost. Sizing the small queue and ghost is a cheap lever that leaves promotion, main eviction and freq handling untouched. The hypothesis was meant to find whether 5% is the optimum, but this variant has no 5% result to compare against. It also reports only hit-rate, so it can't show whether 3% is better or just another point on the same curve.
