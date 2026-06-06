# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0008

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0008/hypotheses/H03.md

## Prior cycles digest

### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants pushed against the edges of the cycle-0004 winner — combining promotion gating with the victim scan, widening the scan window, and enlarging the protected segment. Only the combined promotion-gate variant (v001) was selected, and then by a vanishingly small margin; the other two produced no measurable change.

## Patterns

- **Promotion gating and victim scanning are compatible but not synergistic.** Layering PROMO_THRESHOLD + `was_protected` onto the k=4 scan did not produce additive gains — on this Zipfian trace most probationary items either accumulate enough frequency to clear the threshold quickly, or carry the `was_protected` flag and bypass the gate anyway, so the gating collapses toward unconditional promotion in practice.
- **The victim-scan window is fully saturated at k=4.** Doubling to k=8 changed no eviction decisions, confirming that the coldest reachable candidate already appears within the first few LRU-tail positions. The scan's bottleneck is not window width.
- **The SLRU split-ratio axis is conclusively closed.** The gain trend from 80/20 → 90/10 does not continue to 95/5; the hot working set fits comfortably within the existing protected budget and the additional headroom rescues nothing.
- **Three simultaneous null/marginal results signal a local optimum.** The current design — 90/10 SLRU, k=4 victim scan, PROMO_THRESHOLD gate, `was_protected` bypass, TinyLFU admission — has exhausted all parametric tuning axes that have been probed across five cycles.

## Open questions

- Is frequency-ordered eviction *within* the protected segment (replacing its internal LRU order) a meaningful axis that has not yet been tested?
- Could tuning the TinyLFU admission gate itself — rather than post-admission segment management — provide the next increment, given that the segment internals are now near-optimal?
- Does the marginal win from v001's promotion gate hold at different cache-size-to-working-set ratios, or is it purely an artifact of the tight capacity used in this benchmark?

## Continuation

Adopt v001 (90/10, k=4 victim scan, PROMO_THRESHOLD≥8, `was_protected` bypass) as the new baseline, but treat all five tuning knobs as fixed — none warrant further adjustment. The next cycle should pivot to a qualitatively different structural change: the most promising unexplored axis is frequency-aware ordering within the protected segment (evicting the cold tail of protected rather than the LRU tail). If that also disappoints, the admission gate and sketch configuration are the remaining structural frontier. All prior closed axes — ghost-set, put-path symmetry, scan-window widening, split-ratio changes — should not be revisited.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
