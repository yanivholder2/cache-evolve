# Variant v001 Insight

## Helped

- hit-rate: Widening each bucket from 64 to 256 slots (factor 4→16) produced a small further reduction in premature LRU evictions on the LIRS workload-a trace, confirming the direction is still positive.

## Hurt

## Takeaway

Pushing the bucket-widening factor from 4 to 16 (8×256 instead of 32×64) yields only a marginal additional hit-rate gain — roughly one-third the size of the gain delivered by the first widening step. The sharply diminishing return indicates that the LIRS workload-a hot working set largely fits within 64-slot buckets, and most of the benefit from escaping the narrow-bucket regime was already captured at factor=4. Further increases along this axis are unlikely to produce meaningful gains without a different structural change.
