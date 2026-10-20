# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_14-00-19-782/cycles/0008

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_14-00-19-782/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_14-00-19-782/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_14-00-19-782/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_14-00-19-782/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_14-00-19-782/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_14-00-19-782/cycles/0008/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

The cohort tried folding resident-key membership into the queue-state bit (v001), giving frequency-positive main-queue victims a tail second pass instead of a head reinsert (v002), and returning early on resident hits before the ghost lookup (v003). Both Winners (v001, v003) are cost-side changes that are hit-rate neutral against the parent, so neither is a demonstrated gain; v002 is the only variant that lost hit-rate.

## Patterns

- The two Winners both target per-op-time or metadata-size while leaving eviction order alone. Both held hit-rate flat, which fits that intent, but neither reported the cost dimensions, so their claims remain unmeasured. This gap has now persisted across several cycles.
- Longer retention of warm main-queue residents (v002) lost hit-rate on this trace. This is a new lever, distinct from the promotion-bar and ghost-admission axes, and it suggests the existing head reinsert is not leaving value on the table here.
- The v001 overlay was not inspected this cycle, and the v002 and v003 insights do not confirm their edits either. The same overlay-verification gap raised in prior cycles still applies.
- Evidence is still one trace (lirs workload-a) at one capacity (2048).

## Open questions

- Do the v001 bit-folding and v003 early return reduce per-op-time or metadata-size? Neither can be ranked until a rerun reports both.
- Did the v001, v002, and v003 edits actually land in the built code? Until they are confirmed, the flat and negative results cannot be tied to the intended changes.
- The v003 insight cites v001 as its control. Confirm that the v003 comparison baseline is the intended parent, so that its neutral result is attributed correctly.

## Continuation

Keep the v001 bit-folding and the v003 early return only as cost candidates, and require a rerun that reports per-op-time and metadata-size, plus confirmation that each overlay edit is present, before calling either a win. Drop the tail-reinsert retention change (v002) as a standalone change, since it lost hits on this trace. Do not spend the next cycle on further main-queue retention or reinsert-position tweaks; prioritize build and overlay verification and cost reporting.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

The cohort tried replacing queue scans with a resident-key index (v001), removing the ghost-membership mirror set in favor of a linear ghost scan (v002, stacked on v001), and carrying a frequency of 1 into main on ghost re-admission (v003). Both Winners (v001, v002) are hit-rate neutral against the parent and are cost-side changes, so neither is a demonstrated gain; v003 is the only variant that lost hit-rate.

## Patterns

- The two Winners are structurally one line of work: v002 is explicitly equivalence-preserving on top of v001. Their hit-rate results therefore do not count as independent confirmations.
- Both Winners were built to leave eviction order alone and held hit-rate flat, which fits that intent. Neither reported metadata-size or per-op-time, so the O(1) and metadata claims are still unmeasured.
- v002 trades a hash mirror for a linear ghost scan. Its own insight flags a possible per-op-time cost on the miss path, so the cost direction it claims may not hold.
- Ghost re-admission has now been pushed in a third way. Widening the ghost window (Cycle 0003), routing ghost hits through the small queue (Cycle 0004), and giving re-admissions an extra main-queue pass (v003) all lost or failed to help. Ghost-admission levers look unproductive on this trace.
- The v001 and v002 insights do not confirm that their intended edits are present in the built code. The overlay-verification gap from prior cycles still applies.
- Evidence is still one trace (lirs workload-a) at one capacity (2048).

## Open questions

- Does the resident-key index (v001) reduce per-op-time, and at what metadata cost? Only a rerun reporting both can settle it.
- Does removing the ghost mirror (v002) help metadata-size enough to offset a linear miss-path scan? Its cost direction is unknown until per-op-time is measured.
- Did the v001 and v002 edits land in the built code? Their flat hit-rate results cannot be tied to the intended changes until this is confirmed.

## Continuation

Treat the resident-key index (v001) as the lead cost candidate, and require a rerun that reports per-op-time and metadata-size plus confirmation of the overlay edit before calling it a win. Hold the ghost-mirror removal (v002) until v001 is measured, since its linear scan may regress per-op-time. Drop giving ghost re-admissions a frequency of 1 (v003) as a standalone change. Do not spend the next cycle on further ghost-admission or frequency tweaks on this trace; prioritize build and overlay verification and cost reporting.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
