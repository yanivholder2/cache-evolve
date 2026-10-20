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
