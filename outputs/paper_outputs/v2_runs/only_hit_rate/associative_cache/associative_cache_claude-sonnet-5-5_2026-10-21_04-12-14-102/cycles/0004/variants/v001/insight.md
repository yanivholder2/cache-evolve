# Variant v001 Insight

## Helped

- hit-rate: Halving the probation segment per bucket (8 to 4 slots) gives the protected LRU more room, so re-referenced blocks in the LIRS trace are less often evicted. One-time scan blocks still leave quickly because eviction takes the probation LRU end first. The gain over the parent is small.

## Hurt

- (none observed; hit-rate is the only measured dimension and it did not regress)

## Takeaway

On this trace, the optimum probation size is still at or below the 4-slot point, so the trend from the parent has not turned yet. The gain from 8 to 4 is marginal, which suggests the curve is flattening. Further shrinking is likely to give diminishing returns and may eventually lose scan resistance. I did not inspect the overlay diff directly, so the mechanism above is inferred from the hypothesis text.
