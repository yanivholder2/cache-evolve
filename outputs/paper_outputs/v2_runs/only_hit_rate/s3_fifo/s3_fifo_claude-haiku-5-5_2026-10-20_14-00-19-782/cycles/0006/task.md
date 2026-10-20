# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_14-00-19-782/cycles/0006

## Winners

- v001
- v003

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_14-00-19-782/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_14-00-19-782/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_14-00-19-782/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_14-00-19-782/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_14-00-19-782/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_14-00-19-782/cycles/0006/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

The cohort tried removing a second resident-key probe on the miss path (v001), halving the main-queue frequency decay step (v002), and making the small-queue size adaptive to ghost hits (v003). Both Winners (v001, v002) are hit-rate neutral against the parent, so neither is a demonstrated gain; v003 is the only variant that lost hit-rate.

## Patterns

- Both Winners held hit-rate flat, which fits the intent of leaving eviction order alone. Neither reported metadata-size or per-op-time, so the cost claim behind v001 remains unmeasured.
- The frequency axis has now been probed again with a decay change, and it is flat, matching the earlier frequency-cap result. It looks inert on this trace.
- Making the small-queue size adaptive (v003) did not beat the fixed split. The ghost-hit signal appears too noisy to steer the target, so the sizing lever is not yet a source of gains.
- Across the recent cycles, the promotion-bar, ghost-admission, frequency, and sizing levers have all come back flat or negative on this trace.
- The overlay source could not be inspected for any variant this cycle, so none of the flat or negative results can yet be tied to the intended edits. This is the same verification gap raised in prior cycles.
- Evidence is still one trace (lirs workload-a) at one capacity (2048).

## Open questions

- Does removing the miss-path probe (v001) reduce per-op-time, and at what metadata cost? It cannot be ranked until both are reported.
- Did the v001 and v002 edits land in the built code? Until they are confirmed, their flat results say little about the intended changes.
- Would adaptive sizing behave differently with damping or a wider step, or is the ghost signal too noisy for any sizing rule on this trace?

## Continuation

Keep the v001 probe removal only as a cost candidate, and require a rerun that reports per-op-time and metadata-size, plus confirmation that the overlay edit is present, before calling it a win. Drop the frequency decay tweak (v002) as a standalone change, since the frequency axis has shown no signal across cycles. Drop the ±1 adaptive small-queue sizing (v003) as implemented. Do not spend the next cycle on further frequency or sizing tweaks on this trace; prioritize build and overlay verification plus cost reporting.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
