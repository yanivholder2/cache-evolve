# Cycle 0004 Insight

## Cohort outcome

The cohort tried a second-hit protected tier on top of probationary-first eviction (v001), a sampled probationary victim scan capped at 8 rotating slots (v002), and a single shared frequency counter gating eviction-time admission (v003). Only v003 showed a hit-rate gain on workload-a; v002 was flat to slightly negative and v001 was negative. The recorded Winner is v002, which does not match the hit-rate evidence in its own insight.

## Patterns

- The only positive hit-rate movement came from the shared-frequency admission gate (v003), which compares incoming and resident keys on one scale. This supports the Cycle 0002 suspicion that the earlier admission loss came from a scale mismatch rather than from admission itself.
- The Winner label and the evidence disagree. v002's insight reports a negligible hit-rate dip that misses its own guardrail, while v003 is the variant with a gain. Either the Winner selection used a dimension not shown in the insights, or the label was applied in error. This should be checked before the Winner is treated as a precedent.
- Bounding the victim scan (v002) left hit-rate essentially unchanged, consistent with Cycle 0003's finding that exact-minimum eviction buys little. Its per-op-time benefit remains unmeasured.
- Stacking a second reuse signal (v001) lowered hit-rate. The insight points at the protection cap and demote-on-promotion path rather than the promotion trigger, so the failure is plausibly in the mechanism, not the idea of a second tier.
- No variant measured per-op time or metadata size, and all three insights say the overlays could not be inspected in this run. Flat or negative hit-rate cannot yet be read as a null result about the mechanism, and the cost side is still unknown, as in Cycles 0001 to 0003.

## Open questions

- Which dimension made v002 the Winner, given that v003 carries the only hit-rate gain? Is the Winner rule using a metric missing from the insights?
- Is the v003 gain reproducible across more than workload-a, and does its shared counter add measurable metadata or per-op cost?
- Would a protected tier survive if the cap did not force demotion on each promotion?
- Are the v002 and v001 overlays active under the harness? Without that, their results are ambiguous.

## Continuation

Next cycle should build on the shared-frequency admission gate from v003, and add per-op time and metadata size to the fitness vector before accepting any change. Reconfirm the Winner selection for this cycle before treating v002 as a precedent. Drop the bounded-scan direction as a hit-rate lever, and retry the protected tier only with a changed cap or demotion rule rather than the same promotion path.
