## Recurring wins

- None established yet. Only one completed cycle exists, and no variant showed a hit-rate gain over the baseline.
- Keeping the two-hit small-queue promotion bar looks protective on the measured trace, since relaxing it cost hits. This is a single observation, not yet a recurring pattern.

## Recurring failures

- Relaxing the small-queue promotion bar to one hit displaces main-queue residents and reduces hit-rate on the measured trace.
- Single-constant tweaks to ghost-hit re-routing and small-queue sizing produced no measurable change. Repeating these at the same capacity is unlikely to pay off.

## Open questions

- Are the neutral ghost-routing and small-queue-sizing results specific to this trace and capacity, or is the workload insensitive to those axes in general?
- Would a larger structural change to small-queue sizing move hit-rate?
- What are the metadata-size and per-op-time costs of these variants? Without them, a neutral hit-rate cannot be ranked against the baseline.
- Do any results hold on a second trace or a different capacity?

## Per-policy notes

- S3-FIFO: the promotion bar is the only axis with a measured effect so far, and it is a loss when relaxed. Ghost re-admission routing and small-queue sizing are neutral on the single measured trace. Cost-side metrics have not been reported for any variant.
