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
