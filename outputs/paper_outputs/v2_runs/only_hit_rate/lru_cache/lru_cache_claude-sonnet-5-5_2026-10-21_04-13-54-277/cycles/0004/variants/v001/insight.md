# Variant v001 Insight

## Helped

- hit-rate: Growing the protected segment from 90% to 95% (`capacity * 19 / 20`) kept more re-referenced entries safe from scan pollution, which gave a small gain over the parent on the scan-heavy trace.

## Hurt

- (none) The only dimension, hit-rate, did not regress.

## Takeaway

The protected-share gain is nearly flat past 90%. Moving to 95% produced only a marginal improvement, so the curve is close to its plateau and the probation segment is already close to its minimum useful size. Pushing further would leave probation with very few slots, and the one-slot floor from `.min(capacity - 1)` would then start to bind. Further tuning of this one knob is unlikely to pay off.
