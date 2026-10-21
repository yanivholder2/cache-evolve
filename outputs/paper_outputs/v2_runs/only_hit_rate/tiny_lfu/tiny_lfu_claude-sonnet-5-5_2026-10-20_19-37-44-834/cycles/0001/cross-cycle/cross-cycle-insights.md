## Recurring wins

- Structural variants (window + segmented main region, compact sketch) have so far moved hit-rate only marginally; the baseline's sketch-gated admission already captures most of the benefit.
- Narrower counters and cheaper hashing appear to cost no accuracy, so the sketch looks over-provisioned (cost savings unconfirmed).

## Recurring failures

- Infrastructure fragility: the simulator build failed on a network rate limit (HTTP 429) fetching a dependency jar, so an idea was lost to the environment rather than on merit.
- Fitness output reports hit-rate only; metadata-size and per-op-time are unmeasured, so cost-oriented ideas can't be judged.

## Open questions

- Does periodic halving (aging) help? Never evaluated.
- Do window/segmentation and sketch compaction help or hurt on metadata size and per-op time?
- Is the small window/segmentation gain real or noise? Check other traces and cache sizes.
- Does compact sketch combined with aging or a window compound?

## Per-policy notes

- TinyLFU (sketch-gated admission): baseline is strong on workload-a; window+segmented main gives a marginal gain; single-hash u8 sketch with conservative update is roughly neutral on hit-rate.
- Aging variant: untested; re-attempt first, ensuring dependencies are cached or retried, and that fitness emits size and time metrics.
