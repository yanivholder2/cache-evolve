# Variant v001 Insight

## Helped

## Hurt

- hit-rate: The protected tier makes entries hit twice immune to victim selection while probationary entries exist, but the half-bucket cap forces demotion of the least recently hit protected entry whenever a new promotion lands, so the working set keeps cycling back into the eviction pool instead of staying shielded.

## Takeaway

Adding a second-hit protection tier on top of v003's probationary-first eviction lowered hit-rate on workload-a, so a second reuse signal did not separate the working set from one-shot entries under this cap. The likely lever is the protection cap and demote-on-promotion path rather than the promotion trigger itself. This is inferred from the hypothesis text: the variant's source overlay and manifest could not be inspected in this run, so the mechanism is not confirmed against the actual diff.
