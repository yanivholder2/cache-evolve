# Cycle 0007 Insight

## Cohort outcome

All three variants probed sketch-quality axes: sketch width, promotion threshold recalibration against the sharper conservative-update estimates, and hash-seed diversity. Only the width increase (v001) produced a gain and was selected winner; the other two regressed or held flat.

## Patterns

- **Width and conservative update are complementary, not redundant.** Doubling the sketch width cut residual collision noise that conservative update alone could not eliminate, yielding an additional real but modest hit-rate gain. The two mechanisms address distinct error sources — estimate inflation versus row independence — so their benefits stack rather than overlap.
- **Conservative update did not deflate legitimate hot-item counts.** The hypothesis that sharper estimates would strand medium-frequency items below the PROMO_THRESHOLD=8 gate was refuted: genuinely hot items on this Zipfian trace accumulate counts well above 4 through get volume alone, so lowering the gate to 4 only promoted second-tier churn candidates. The threshold is correctly calibrated for the current sketch quality and should not be adjusted.
- **Per-row hash independence from sequential seeds is already sufficient.** Replacing 0–3 integer seeds with diverse mixing constants produced a marginal regression, indicating that `DefaultHasher` already provides adequate avalanche even from small seeds. Row-independence failure was not a real error driver.
- **Diminishing returns are visible at the sketch layer.** The v001 gain is noticeably smaller than the cycle-0006 conservative-update gain, consistent with the collision profile being pushed toward its floor; further width increases are likely to yield progressively less.

## Open questions

- Does increasing sketch depth (number of rows) reduce the minimum-estimator variance in a way that width cannot, or have the two dimensions converged to the same bottleneck?
- Does the aging/reset interval interact with the now-narrower collision profile — does faster forgetting expose more or less residual noise at 8× width?
- At what sketch quality level does the remaining hit-rate gap stop being attributable to estimation error, and does that shift the primary frontier back to structural policy?

## Continuation

Adopt v001's 8× sketch width as the new baseline alongside conservative update. The two remaining unexplored sketch axes are depth (number of hash rows) and reset/aging interval; probe them next since they are low-cost and the sketch layer is still the live frontier. If both disappoint, the error floor has been reached and attention should shift to frequency decay policy — how the sketch ages historical counts — as the last structural axis identified at the end of cycle 0006.
