# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

The cohort tried three small, single-axis changes to the S3-FIFO baseline: tightening the small-queue promotion bar, re-routing ghost-hit re-admissions, and shrinking the small admission queue. Only one variant hurt hit-rate, and no variant measurably improved any dimension, so both Winners (v002, v003) are neutral rather than improvements.

## Patterns

- Nothing moved hit-rate upward. Both Winners reported no change from the parent on the single trace measured, so the Winner label reflects "did not regress" rather than a demonstrated gain.
- The two-hit promotion bar appears to be doing useful filtering. Relaxing it to one hit displaced main-queue residents and cost hits.
- The ghost re-admission axis and the small-queue sizing axis both looked inert on this workload. Changes there produced no measurable effect.
- Every variant reported only hit-rate. Metadata-size and per-op-time were not assessed in any variant, so cost-side tradeoffs are unknown.
- All evidence comes from one trace (lirs workload-a) at one capacity (2048), so none of these results generalize yet.

## Open questions

- Do the neutral results for ghost routing and small-queue sizing hold at other capacities or on other workloads, or is this trace simply insensitive to them?
- Would a larger change to the small-queue sizing, rather than a one-constant tweak, move hit-rate at all?
- What do the metadata and per-op-time costs look like for these variants? Without them, a neutral hit-rate cannot be ranked against the baseline.

## Continuation

Drop the promotion-bar relaxation; it is a measured loss on this trace. Do not spend the next cycle on small tweaks to ghost routing or small-queue ratio at this single capacity, since they showed no signal. Next cycle should test changes that report metadata-size and per-op-time alongside hit-rate, and should be checked on at least one additional trace or capacity before any variant is treated as a real gain.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

The cohort tried a cost-side refactor of ghost-membership lookup (v001), a single-constant change to the main-queue frequency cap (v002), and a freq-layout change (v003) that did not land in the overlay. Both Winners (v001, v002) are hit-rate neutral against the parent; neither shows a demonstrated gain, and v003 showed an unexplained hit-rate drop.

## Patterns

- Hit-rate stayed flat for both Winners. v001 is a pure cost refactor that preserved eviction order, and v002 raising the main-queue frequency cap showed no effect on this trace.
- Neither Winner reported metadata-size or per-op-time, so the cost claims behind v001 are unverified. This is the same gap the prior cycle flagged; it persisted into this cohort.
- v002's null result suggests few main-queue residents on this trace build up enough hits to reach the old cap, so the frequency axis looks inert here, consistent with the prior cycle's sizing and routing findings.
- v003 did not implement its hypothesis (the layout change is absent from the overlay). Its hit-rate drop cannot be attributed to the layout idea and may reflect an overlay or build mismatch rather than the change itself.
- Evidence is still one trace (lirs workload-a) at one capacity (2048).

## Open questions

- Does the v001 ghost-lookup refactor reduce per-op-time, and at what metadata cost? It cannot be ranked on cost until both are reported.
- Is the v003 drop real, or an artifact of the mismatched overlay? Does the hit-rate drop reproduce on a clean build?
- Does the frequency cap matter at other capacities or workloads, where main-queue residents may accumulate more hits?

## Continuation

Keep the ghost-lookup refactor (v001) only as a cost candidate, and require a rerun reporting per-op-time and metadata-size before treating it as a win. Drop the frequency-cap tweak (v002) on this trace; it has now shown no signal in two cycles. Before the next cycle, verify the v003 overlay and rebuild its layout variant; do not test any layout change until the build is confirmed clean.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

The cohort tried a resident-key lookup structure to skip queue scans on misses (v001), a higher small-queue promotion bar (v002), and a doubled ghost queue (v003). Both Winners (v001, v002) are hit-rate neutral against the parent, so neither is a demonstrated gain; v003 is the only variant that lost hit-rate.

## Patterns

- Hit-rate stayed flat for both Winners. v001 is intended as a cost-side change that should leave eviction order alone, and v002's promotion-bar change showed no effect on this trace.
- The promotion bar has now been probed in both directions across cycles: relaxing it to one hit lost hits in Cycle 0001, and raising it to three hits was neutral here. The axis looks saturated on this trace, not a source of gains.
- Widening the ghost window (v003) lost hit-rate. The variant's account is that extra ghost hits bypass the small queue and land directly in main, displacing resident entries. Ghost coverage alone does not appear to be the lever.
- Neither Winner reported metadata-size or per-op-time, so v001's cost claim is still unverified. This gap has now persisted across three cycles.
- v002 could not confirm from its overlay that the `freq > 2` edit landed, so its neutral result may not reflect the intended change. This echoes Cycle 0002's overlay problem.
- Evidence is still one trace (lirs workload-a) at one capacity (2048).

## Open questions

- Does the v001 resident-key structure reduce per-op-time, and at what metadata cost? It cannot be ranked until both are reported.
- Did the v002 edit actually land in the built code? Until that is confirmed, its flat result says nothing about the promotion bar.
- Does the ghost-window loss hold if ghost hits are admitted differently (for example, through the small queue rather than main)?

## Continuation

Keep v001's resident-key idea only as a cost candidate, and require a rerun that reports per-op-time and metadata-size before calling it a win. Drop the doubled ghost queue (v003) as a standalone change; any future ghost-size work must also change how ghost hits are admitted. Drop further promotion-bar tweaks on this trace. Before the next cycle, confirm that each overlay edit is present in the build so results can be attributed to the intended change.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

The cohort tried a resident-key probe with early return on hits (v001), routing ghost-hit re-admissions through the small queue (v002), and a packed single-byte bitfield for queue membership and frequency (v003). Both Winners (v001, v003) are hit-rate neutral against the parent, so neither is a demonstrated gain; v002 is the only variant that lost hit-rate.

## Patterns

- Both Winners were behavior-preserving on paper and stayed flat on hit-rate, which is consistent with unchanged eviction order. Neither reported metadata-size or per-op-time, so their cost claims remain unverified. This gap has now persisted across four cycles.
- Ghost handling has now been probed in two ways. Widening the ghost window lost hits in Cycle 0003, and routing ghost hits through the small queue lost hits here. Neither direction of the ghost-admission lever has shown a gain on this trace.
- The promotion bar appears to be the likely mechanism in v002: re-admitted keys that were recently evicted and still hot must now pass the bar before counting as main-queue residents. This connects to the promotion-bar findings of earlier cycles.
- Neither Winner's source diff was confirmed in its overlay, so the flat results may not reflect the intended edits. This is the same overlay-verification gap raised in Cycles 0002 and 0003.
- Evidence is still one trace (lirs workload-a) at one capacity (2048).

## Open questions

- Do the v001 probe and v003 bitfield reduce per-op-time or metadata-size? Neither can be ranked until both are reported on a rerun.
- Did the v001 and v003 edits actually land in the built code? Until they are confirmed, their flat results cannot be attributed to the intended changes.
- Is the v002 loss specific to the two-hit promotion bar, or would ghost re-admissions do better under a different bar?

## Continuation

Keep the v001 probe and v003 bitfield only as cost candidates, and require a rerun that reports per-op-time and metadata-size, plus confirmation that the overlay edits are present, before treating either as a win. Drop ghost-admission routing through the small queue (v002) as a standalone change, since it has now lost hits in one direction after the window-widening loss in the other. Do not spend the next cycle on further ghost-window or promotion-bar tweaks on this trace.


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
