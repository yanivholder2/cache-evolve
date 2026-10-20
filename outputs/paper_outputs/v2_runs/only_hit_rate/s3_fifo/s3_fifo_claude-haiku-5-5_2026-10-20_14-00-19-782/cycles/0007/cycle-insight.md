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
