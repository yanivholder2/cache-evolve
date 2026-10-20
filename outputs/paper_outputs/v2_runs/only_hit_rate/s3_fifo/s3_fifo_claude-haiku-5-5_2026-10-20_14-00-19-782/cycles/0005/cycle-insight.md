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
