## Recurring wins

- No variant across four cycles has shown a demonstrated hit-rate gain on the measured trace. "Winner" has meant "did not regress."
- Behavior-preserving refactors (cycle 0002 v001, cycle 0004 v001 and v003) stay flat on hit-rate, as expected if eviction order is unchanged. They are candidates only on cost, which has not been measured.
- The two-hit promotion bar looks protective: relaxing it to one hit lost hits (cycle 0001).

## Recurring failures

- Ghost-admission changes have lost hits in both directions. Widening the ghost window lost hits (cycle 0003). Routing ghost re-admissions through the small queue also lost hits (cycle 0004). The ghost lever has not produced a gain in either direction.
- Promotion-bar tweaks have not helped. Relaxing it lost hits (cycle 0001). Raising it was neutral (cycle 0003). The axis looks saturated on this trace.
- Single-constant tweaks have shown no signal: ghost routing and small-queue sizing (cycle 0001), main-queue frequency cap (cycle 0002). Repeating small knob changes at this capacity has not paid off.
- Cost claims have never been verified. No cycle has reported metadata-size or per-op-time, so no variant can be ranked against the baseline on cost.
- Several overlays did not confirm their intended edits (cycle 0002 v003, cycle 0003 v002, cycle 0004 v001 and v003). Flat or negative results from these cannot be attributed to the intended change.

## Open questions

- Do the v001 resident-key probe and the v003 bitfield reduce per-op-time or metadata-size? Rerun with both cost metrics reported, and confirm each edit is in the build.
- Did the cycle 0004 v001 and v003 edits actually land? Until confirmed, their flat results say nothing about the ideas.
- Is the ghost-admission loss specific to the two-hit bar, or would re-admissions do better under a different promotion rule? Cycles 0003 and 0004 point in the same direction but were not tested together.
- Does any of the no-signal axes (sizing, frequency cap, ghost routing) matter at other capacities or on other workloads? All evidence is one trace at one capacity.
- Would a larger change to small-queue sizing, rather than a one-constant tweak, move hit-rate?

## Per-policy notes

- S3-FIFO: the small-queue promotion bar is the most consistently informative knob. Two-hit appears to be doing useful filtering; relaxing it hurt, and raising it was neutral.
- S3-FIFO: ghost-queue handling has lost in both directions tried (larger window, small-queue admission). Treat ghost changes as unproductive on this trace until admission and size are changed together.
- S3-FIFO: small-queue sizing and main-queue frequency cap have shown no signal at capacity 2048 on lirs workload-a.
- S3-FIFO: all cycles report hit-rate only; cost-side behavior of the policy is unknown.
