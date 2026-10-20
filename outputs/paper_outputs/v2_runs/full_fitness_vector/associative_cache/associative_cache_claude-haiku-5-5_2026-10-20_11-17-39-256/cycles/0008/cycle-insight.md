# Cycle 0008 Insight

## Cohort outcome

The cohort tried three changes: a second-sighting doorkeeper on the put-miss path (v001), a Fibonacci multiplicative hash in place of the bucket index (v002), and moving the access counters out of the item into a parallel array (v003). No variant produced a clear hit-rate gain, and the listed Winners rest on small or single-run timing movement; v003's own insight records regressions.

## Patterns

- The second-sighting admission rule barely moved hit-rate, and its cost landed on the put-miss path. Across Cycles 0006, 0007 and this one, the doorkeeper has gained, lost, and now barely moved hit-rate, so the admission gate is not a dependable lever.
- The hash change (v002) is the only variant with a plausible timing movement, and it cost a small amount of hit-rate that may be noise. It partly re-measures Cycle 0007's timing claim and suggests that claim was partly real and partly noise. Bucket placement looks like a modest timing lever, not a hit-rate one.
- Moving the counters out of the item (v003) removed the get-tail movement seen in Cycle 0006's store-skip variant, while leaving retention unchanged. That supports the skipped counter store as the cause, and points against the struct layout as the explanation. The cost is an extra memory touch per get hit and a small put-side shift.
- Winners status should be reconciled. v003 is listed as a Winner, but its insight records get-tail and put regressions and no helped dimensions. Its Winner label is not supported by this cycle's evidence.
- Single runs dominate the evidence here. Several effects are within the run-to-run spread the parent insights flag, so none of the timing or hit-rate movements should be treated as settled.
- Mechanisms are inferred from diffs and variant insights, not confirmed from overlay code, because overlays were not readable this cycle.

## Open questions

- Does the hash change's timing gain hold across repeated runs, and is its small hit-rate drop real?
- Is the get-tail gain from skipping the saturated-counter store reproducible with the counter kept in the item?
- Does the doorkeeper's hit-rate effect depend on the parent or on the exact reject rule, given the conflicting results in Cycles 0006 and 0007?
- What is the put-tail source on full-bucket puts? Five cycles of shortcuts to the scan and aging have not moved it.

## Continuation

Next cycle should keep the counter in the item and retest the saturated-counter store skip from Cycle 0006 with repeated runs, rather than moving the counter. Treat the hash change (v002) as a timing candidate only, pending a three-run spread, and drop it as a hit-rate lever. Drop the second-sighting doorkeeper as a hit-rate lever, and drop the separate counter array (v003).
</content>
