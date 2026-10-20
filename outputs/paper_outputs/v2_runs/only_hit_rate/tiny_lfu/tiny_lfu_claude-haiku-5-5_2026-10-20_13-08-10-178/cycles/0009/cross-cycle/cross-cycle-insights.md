## Recurring wins

- Collision error in the frequency sketch is the reliable lever. Widening the sketch (0004), adding a row (0007), and conservative update (0005) all helped, while removing depth (0005) hurt. Gains from the estimator appear to overlap rather than stack cleanly (0006).
- Victim selection in probation by sketch estimate is the most consistent recent gain. Comparing the LRU-tail entries and evicting the coldest helped at two candidates (0008), and the gain carried through at three (0009).
- Protected-segment sizing matters, but the optimum sits near the incumbent ratio. A modest increase helped (0006), while pushing further reversed it (0007) and halving the cap also lost (0004).

## Recurring failures

- Table-wide or fixed-schedule aging of the sketch discards admission history the comparison needs. This held for halving on an access-count trigger (0001, 0005) and for halving on saturation (0009).
- Gating what the sketch or admission sees costs hit-rate. Filtering first-touch keys out of updates (0006), gating promotion on the sketch estimate (0007), and a recency window ahead of the LRU with admission at exit (0002) all lost ground.
- Layout-only changes preserve hit-rate by construction, and their claimed gains (footprint, per-op time) are not measured. Their Winners advance by not regressing (0002, 0003). Narrower counters were neutral (0002, 0009).
- Tie-handling changes that admit equal-estimate newcomers hurt (0008). Ghost-history frequency bonuses were marginal and their metadata cost is unmeasured (0008).
- Key-mapping and hashing rewrites produced a small, unresolved hit-rate dip with no clear benefit (0001, 0003).

## Open questions

- Does any winning change hold on other workloads and cache sizes? Every result so far comes from one trace at one size.
- The fitness vector has no memory or timing dimension. Until it does, the cost of wider or deeper sketches, the extra conservative-update pass, and narrower counters cannot be weighed.
- Attribution is uncertain for several Winners. v001 versus v002 in 0007 is unresolved, and the sources behind 0006 and 0008 could not be inspected.
- Does the victim scan keep improving past three candidates? Is the gain from the rule itself, or from how the scan reorders the queue?
- Does 16-bit saturation create ties on hotter or more skewed traces?
- Is there a per-key or per-row aging rule that bounds counter growth without breaking admission ordering?

## Per-policy notes

- Admission (frequency sketch): width and depth increases and conservative update are the productive directions. Fixed-schedule, table-wide, and saturation-triggered halving are not. Counter narrowing is neutral on the current measurement.
- Admission timing and gating: filtering first touches, recency windows in front of the LRU, and sketch-gated promotion have all lost hit-rate. Avoid this family unless the fitness vector changes.
- Main region (probation/protected SLRU): the protected share has a peak near the incumbent ratio. Promotion on every hit beats a sketch-gated rule. Demotion to the back of probation beats push-to-front.
- Victim selection (probation tail): comparing several tail entries and evicting the lowest sketch estimate is the active thread. Tie-breaking by queue position protects older cold entries and should be kept.
- Ghost history: the frequency bonus for recently evicted keys is marginal and should not be pursued in its current form.
