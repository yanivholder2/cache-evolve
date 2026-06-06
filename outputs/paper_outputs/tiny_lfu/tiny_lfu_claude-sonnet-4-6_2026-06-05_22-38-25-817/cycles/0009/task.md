# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0009

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

The cohort tested three distinct axes: extending the frequency-victim scan to the protected segment, sharpening CountMinSketch estimates via conservative update, and raising the admission floor. Only the sketch-quality change (v002) produced a gain and was selected winner — the first improvement in several cycles to originate from the frequency-estimation layer rather than from structural cache policy.

## Patterns

- **Sketch noise was still a meaningful drag after five cycles of policy tuning.** Conservative CMS update — incrementing only the rows already at the minimum — reduced collision-inflated frequency estimates and let the existing k=4 victim scan and PROMO_THRESHOLD gate act on sharper signal. The win came with no structural policy change at all, confirming that data-quality improvement was a live frontier the earlier policy work could not surface.
- **The protected segment's LRU tail is frequency-homogeneous.** Mirroring the probationary k=4 victim scan into protected produced zero movement: the PROMO_THRESHOLD gate pre-filters entrants, leaving the LRU end of protected with a narrow frequency spread; the scan rarely finds a colder candidate than the default LRU head. The asymmetry between the two segments is now well-understood and closed.
- **The TinyLFU admission floor is already captured by the existing strict inequality.** Raising the minimum admission frequency to 2 was redundant on a warm Zipfian trace — the edge case it targets (freq-1 item evicting a freq-0 resident) is a cold-start artifact averaged away over a long run. This axis is closed.

## Open questions

- Do other sketch parameters — width, depth, or reset interval — offer further signal-quality gains now that conservative update is the baseline?
- Does conservative update alter how quickly frequencies decay under the aging/reset mechanism, and could that interaction suppress or amplify the gain at different cache-warmth levels?
- Is the conservative-update gain robust across non-Zipfian workloads, or does it depend on the heavy-tail concentration of this trace?

## Continuation

Take v002's conservative CMS update as the new baseline. The highest-value next probe is sketch configuration: width/depth sizing and reset frequency have not been tuned and may limit how much the sharper estimates translate into correct decisions. Protected-segment victim scanning and admission-floor variants are definitively closed. If sketch configuration also disappoints, the remaining structural frontier is frequency decay/aging policy — how quickly the sketch forgets past frequency — rather than any further segment-management change.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
