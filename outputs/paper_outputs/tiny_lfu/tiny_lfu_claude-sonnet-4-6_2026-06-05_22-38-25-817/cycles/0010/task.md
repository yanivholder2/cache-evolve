# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0010

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0010/hypotheses/H03.md

## Prior cycles digest

### Cycle 0007

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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants probed the remaining sketch and admission axes: sketch depth, periodic count aging, and direct-to-protected admission for sketch-hot items. None produced a gain — the winner (v001) held flat while the other two regressed — making this the first cycle to yield zero advancement.

## Patterns

- **Sketch depth is saturated at the current width.** With 8× width already pushing per-row collision probability to a low floor, additional hash witnesses provide no further discriminating power; the 4-row minimum estimator already returns accurate estimates on this workload. The depth axis is closed.
- **Count aging actively hurts at this timescale.** Even at a very slow halving cadence the accumulated frequency signal erased by each reset outweighs the stale-count noise it removes, indicating the workload does not produce meaningful count inflation at this sketch scale. The aging/reset axis is closed.
- **Probationary is a load-bearing timing gate, not mere overhead.** Bypassing it for sketch-hot admits causes immediate protected-segment churn: direct inserts trigger premature demotions before demand is reconfirmed by a real access. The insight reinforces that the current admission path is correct in structure, not just in tuning.
- **The sketch layer is fully exhausted.** Across cycles 0006–0008 the cohort has closed conservative update, width, depth, hash-seed diversity, and aging. All sketch axes are now confirmed closed.

## Open questions

- Do the policy knobs closed in cycle 0005 (PROMO_THRESHOLD, victim-scan window, SLRU split) behave differently now that the underlying frequency estimates are substantially sharper? The earlier closures were reached with a noisier sketch baseline.
- Is the remaining hit-rate gap attributable to irreducible workload entropy (unpredictable cold items), or does a structural policy change still have room to move?

## Continuation

The sketch is fully tuned and should be treated as fixed. The most promising next probe is to revisit the policy axes closed in cycle 0005 under the now-improved sketch signal: PROMO_THRESHOLD gating and the k=4 victim-scan window are the top candidates, since their earlier conclusions were reached with higher collision noise. If both remain flat under the sharper signal, the design has reached its ceiling for this workload and trace, and further investment in tuning is unlikely to pay off.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

All three variants revisited policy axes previously closed under a noisy sketch, now re-evaluated against the fully-accurate 8×-width + conservative-update baseline. Only the PROMO_THRESHOLD reduction (v001) was selected winner, producing a small but real gain; both the wider probationary victim scan (v002) and frequency-ordered protected demotion (v003) registered directionally positive but negligible movements and were not selected.

## Patterns

- **The cycle-0008 directive was correct but modest in yield.** Recalibrating policy knobs under sharper sketch signal did unlock a real gain from PROMO_THRESHOLD — confirming that the gate was miscalibrated against the old noisy baseline — but the frequency band unlocked (freq 6–7) is a thin slice of the Zipfian access distribution, so the gain is marginal rather than structural.
- **All three variants were positive; none regressed.** This is the first cycle in which every variant moved in the right direction. It reflects that the remaining policy space has no obviously wrong directions left — only low-leverage ones — which itself signals proximity to a ceiling.
- **Probationary victim-scan width and protected demotion scan are low-leverage, not broken.** Both v002 and v003 confirmed that extending the respective scan windows under accurate estimates is sound and directionally correct, but the protected segment's tight PROMO_THRESHOLD entry bar leaves genuine frequency spread narrow, and the LRU tail of probationary is already well-characterized by k=4. Neither axis offers further meaningful tuning room.
- **PROMO_THRESHOLD is still the highest-leverage policy knob.** Even at its new value of 6 the gain is marginal; further lowering may exist but is unlikely to compound substantially given the thinning frequency bands below 6.

## Open questions

- Does PROMO_THRESHOLD=5 (or lower) unlock another thin slice, and at what threshold does protected-segment churn begin to outweigh the admission benefit?
- Is the remaining hit-rate gap now attributable to irreducible workload entropy rather than any policy choice — i.e., has the design genuinely reached its ceiling for this trace?
- Would the SLRU segment-size split (currently 90/10 probationary/protected) yield more leverage now that both admission and demotion policy are well-calibrated?

## Continuation

Adopt v001's PROMO_THRESHOLD=6 as the new baseline. The probationary scan width and protected demotion scan are closed as low-leverage. The two remaining untested structural axes are the SLRU split ratio and further PROMO_THRESHOLD reduction; of these, the split ratio is higher-risk but potentially higher-reward if the protected segment is undersized relative to the hot-item working set. If the split also disappoints, the design has reached its ceiling for this workload and trace, and further tuning investment is unlikely to pay off.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
