# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

The cycle explored three orthogonal levers: sketch counter aging, a newcomer admission window, and SLRU segmentation of the resident set. Only the SLRU variant (v003) improved hit rate and was selected as the winner; the other two degraded performance.

## Patterns

- **Resident-ordering changes beat admission-policy changes.** The sole winner restructured how residents are ordered and protected after admission; the two losers both intervened at the point of admission (frequency gate timing, sketch accuracy) and both backfired.
- **Capacity cannibalization is immediately punishing.** Reserving a permanent window segment shrank the main LRU and that structural loss was felt before any newcomer-grace benefit could materialize — the trace does not produce items that accumulate frequency fast enough inside the window to recoup the cost.
- **Sketch aging cadence matters as much as the concept.** The aging interval was too short for the workload's hot/cold cycle; counters collapsed faster than the trace refreshes frequency distinctions, making the admission gate noisy rather than adaptive.
- **The frequency gate already handles one-hit-wonders well.** SLRU's protection is partially redundant with the existing admission gate, which explains why its gain was real but narrow.

## Open questions

- Is the SLRU split ratio (80 % protected / 20 % probationary) close to optimal, or does a different ratio extract more gain on this trace?
- At what aging interval (or with what counter floor) does sketch aging shift from harmful to helpful?
- Are there complementary changes to the SLRU structure — such as adaptive segment resizing — that could amplify v003's modest improvement?

## Continuation

Build on v003's SLRU as the new baseline and probe segment-ratio sensitivity (e.g., 90/10 and 70/30 splits) since the 80/20 choice was arbitrary and the win was narrow. Sketch aging remains worth revisiting with a substantially longer reset cadence or a per-key counter floor, but not at the same interval as v001. Deprioritize the admission window: it carries a structural capacity penalty that this trace punishes immediately.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

The cycle probed three distinct levers — SLRU split-ratio tightening, sketch-aging at a much longer cadence, and a frequency-gated promotion threshold — against the SLRU baseline inherited from cycle 0001. Only the 90/10 split variant won, and only by a negligible margin; the other two produced no improvement or a slight regression.

## Patterns

- **SLRU ratio tuning has reached diminishing returns.** Moving from 80/20 to 90/10 barely moved the needle, confirming that the hot set was already well-contained in the protected segment at 80 %; the extra headroom covers only marginal-frequency items. The shrinkage of the probationary buffer caused no visible harm, reinforcing that probationary is a pass-through staging area on this workload, not a competitive contention zone.
- **Sketch aging is consistently harmful regardless of cadence.** Even at 10× the already-conservative reset interval tested in cycle 0001, periodic halving of counters erodes frequency signal more than it cleans up stale counts. The frequency distribution on this trace is stable across the entire replay window; accumulated counts are signal, not noise.
- **The promotion threshold landed in a behavioral dead zone.** At a threshold of 2, too few borderline items were affected — either they are too rare to matter or they accumulate frequency fast enough to qualify for promotion shortly after anyway. The concept is sound but the threshold value was ineffective at this workload's access rate.
- **All three axes hit a wall together.** None of the three hypotheses produced a meaningful gain, suggesting the SLRU-on-TinyLFU configuration is near a local optimum for workload-a at this cache size, and parameter-level tuning within the current structure is largely exhausted.

## Open questions

- Is there a promotion threshold high enough (e.g., 5–10) to actually reduce protected-segment churn, or does this trace simply not produce enough borderline-frequency contention for the gate to matter?
- What structural changes — beyond SLRU segmentation and sketch aging — could shift hit rate on a stable Zipfian workload? Candidates include ghost/victim caches, adaptive admission biases, or size-aware eviction.
- Is the admission frequency gate itself the tightest bottleneck, or is eviction ordering (within protected) the remaining lever?

## Continuation

SLRU split-ratio tuning and sketch aging are both exhausted as levers for this workload and should be deprioritized in future hypotheses. The next cycle should leave the 90/10 split in place (as the marginal winner) and pivot to structural rather than parametric changes: consider ghost-cache admission feedback (tracking recently evicted keys to bias re-admission), or probing whether the in-segment LRU ordering within protected could be replaced by a frequency-ordered variant. The promotion threshold concept is worth one more test at a substantially higher value, but should not consume a dedicated variant slot unless paired with a complementary change.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants targeted the protected segment from different angles — frequency-guided demotion selection, ghost-set fast-track re-admission, and promotion gating with a demotion-history bypass. Only the promotion-gate variant (v003) was selected; all three produced neutral-to-positive results, reinforcing that the configuration is near a local optimum where incremental gains are small.

## Patterns

- **Promotion-side filtering beat demotion-side selection.** Replacing the scan-based demotion picker (v001) with a threshold gate on *promotion* (v003) was the only change that cleared the winner bar. At a 90/10 split the protected segment's LRU tail is already high-frequency, so searching for a colder demotion candidate rarely overrides the default LRU choice; blocking borderline items from entering protected in the first place yields more.
- **The `was_protected` bypass is the essential partner to aggressive gating.** Raising the promotion threshold without distinguishing overflow-demoted items from frequency-weak newcomers would penalize genuinely hot items caught in capacity churn. The bypass makes the threshold safe to set high.
- **Ghost-set re-admission is real but buffer-size-constrained.** The ghost-ring mechanism produced a positive gain with no regressions, but at a buffer capped near capacity/8 the fast-track path fires infrequently because evicted items cycle out of the ring before they are re-requested. The mechanism and the SLRU changes were introduced together, leaving their individual contributions unresolved.
- **The cohort produced no regressions.** With three variants all landing flat-to-positive, the cache is operating well within a safe region, but the absence of a decisive winner signals diminishing returns from structural tuning of the protected segment alone.

## Open questions

- Does the ghost-set mechanism from v002 add further gain when layered on top of v003's promotion gating, or do the two interact and neutralize each other?
- What ghost-buffer size (e.g., capacity/4 or capacity/2) is needed for the fast-track path to fire frequently enough to matter on this trace?
- Are admission-side changes — adjusting the TinyLFU frequency gate itself — the remaining meaningful lever, given that protected-segment internals appear nearly exhausted?

## Continuation

Take v003's 90/10 SLRU with promotion threshold and `was_protected` bypass as the new baseline. The next cycle's highest-value experiment is isolating the ghost-set mechanism on top of v003 (not combined with a fresh SLRU variant) to determine whether ghost-set re-admission contributes independently. If that confirms a real gain, a follow-on probe on ghost-buffer size is warranted. Protected-segment demotion scanning should be deprioritized; admission-side or decay-side hypotheses are the better structural frontier if ghost-set isolation also disappoints.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

The cycle tested ghost-set isolation on the prior lineage, put/get promotion symmetry, and a clean-slate SLRU rebuild with frequency-sampled victim selection. Only the baseline-rooted variant — SLRU 90/10 with a k=4 LRU-tail frequency scan and unconditional promotion — produced a gain and was selected winner.

## Patterns

- **Victim-side frequency filtering opened a new positive axis.** Where cycles 0001–0003 exhausted the promotion and admission sides, scanning the k=4 probationary LRU candidates and evicting the minimum-frequency one rescued warm survivors that strict LRU would have displaced. The gain was modest, consistent with the scan overriding the default choice only at the tail of the k=4 window.
- **Ghost-set is definitively harmful on Zipfian.** Fast-tracking ghost-buffer hits into protected degrades hit rate because ghost-ring entries are precisely the items that already lost the TinyLFU admission contest — they are genuinely cold, and elevating them displaces legitimately hot residents. This mechanism has now regressed twice and the failure mode is well-understood.
- **The put-path promotion gap is a workload fiction on this trace.** Symmetric promotion in put() produced zero movement because the trace is read-dominant: hot items accumulate enough frequency via get() to qualify for promotion long before any put() could act as a trigger. The change is logically sound but structurally irrelevant here.
- **Winning from the baseline reset signals the lineage may have over-fit.** v003 (H03) was rooted at the baseline, not the accumulated cycle-0003 lineage (PROMO_THRESHOLD=8, `was_protected`, 90/10 split). That a baseline-rooted variant outcompeted two variants built on the prior winner suggests the PROMO_THRESHOLD gate and victim-scan may interact — either reinforcing or interfering — and the combined design space has not been explored.

## Open questions

- Does layering the k=4 victim scan onto the cycle-0003 lineage (with PROMO_THRESHOLD and `was_protected`) yield additive gains, or does the promotion gate reduce probationary-churn enough that the scan rarely changes the eviction choice?
- Is unconditional promotion (used in the winning variant) specifically enabling the victim-scan benefit, or is the scan effective regardless of promotion policy?
- With ghost-set ruled out, is there any remaining admission-side lever worth testing, or is the eviction path now the primary frontier?

## Continuation

The next cycle should build on the winning variant (SLRU 90/10, k=4 victim scan, unconditional promotion) and attempt to graft the PROMO_THRESHOLD gate from cycle-0003's lineage onto it — the two approaches have not been tested in combination and the interaction is unresolved. If that combination regresses, it confirms the gate and the scan are redundant on this workload and the simpler scan-only design is preferable. Ghost-set should not be revisited; it has a clear failure mechanism on Zipfian and is now closed. Put-path symmetry is also a dead end for this trace. Increasing k in the victim scan (e.g., k=8) is a low-cost probe if the combined variant disappoints.


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


### Cycle 0010

# Cycle 0010 Insight

## Cohort outcome

All three variants tested the remaining policy knobs called out by cycle 0009: further PROMO_THRESHOLD reduction, frequency-ordered protected demotion, and shrinking the protected segment to widen probationary. All three regressed; v002 (frequency-scan demotion) was selected as winner on the basis of the smallest regression, marking two cycles in a row with no net advancement.

## Patterns

- **PROMO_THRESHOLD has a hard floor at 6.** The pattern of unlocking thin Zipfian frequency slices that succeeded from 8→6 does not continue to 5; the frequency-5 cohort is sparse enough that the additional protected-segment churn it generates outweighs any admission benefit. This axis is conclusively closed.
- **Frequency-scan demotion from protected is unproductive regardless of the admission threshold.** Tried first under PROMO_THRESHOLD=8 (cycle 0009 v003) and now under PROMO_THRESHOLD=6, the scan finds no meaningfully colder candidate than the LRU head. Protected's LRU ordering already approximates frequency ordering sufficiently; the knob has no room left.
- **The SLRU split is exhausted in both directions.** Enlarging protected was probed and closed in cycles 0002–0005; shrinking protected (widening probationary by ~50%) now confirms the opposite direction is also unprofitable. The k=4 probationary victim scan already provides adequate shelter for warming items at the 10% probationary size, so the extra buffer buys nothing while the cost to hot residents is immediate.
- **Every structural policy axis identified across this run is now closed.** Sketch quality (closed 0006–0008), PROMO_THRESHOLD (floor reached 0010), victim-scan widths, demotion order, and SLRU split ratio are all exhausted.

## Open questions

- Is the remaining hit-rate gap attributable entirely to irreducible workload entropy — cold-start misses on the Zipfian long tail that no in-cache eviction policy can capture?
- Would a structurally different admission gate (e.g., a Bloom-filter-style cold-item filter at the probationary entry point) address any portion of the remaining gap, or does the gap belong to the workload rather than the policy?

## Continuation

The design has reached its ceiling for this workload and trace: all known tuning axes are closed and the last two cycles have produced no advancement. The next cycle's hypotheses-writer should either declare convergence and halt, or pivot to genuinely architectural changes (a pre-admission filter, an alternative base eviction policy) rather than re-probing closed knobs. Incremental tuning of the existing TinyLFU structure is unlikely to yield further gains.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
