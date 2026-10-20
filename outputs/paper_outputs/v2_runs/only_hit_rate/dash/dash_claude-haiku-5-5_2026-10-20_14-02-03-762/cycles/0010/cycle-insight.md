# Cycle 0010 Insight

## Cohort outcome

The cohort tried three changes: moving a further 128 slots from the stash to the probing tier, for a 256-slot total shift (v001), folding the seen-in-stash marker into spare bits of the slot record (v002), and gating stash promotion on a second hit that falls within a recency window (v003). None of the three improved hit-rate against its parent on the single LIRS trace at size 2048. Only v003 is listed as a Winner, yet its own insight reports a hit-rate loss, so the Winner status is not supported by this cohort's evidence.

## Patterns

- The probing-tier gain from cycle 0009 does not extend. Pushing the shift to 256 slots (v001) lost hit-rate, so the benefit of probing residency saturates at or before that point and the stash capacity given up becomes the larger cost.
- Recency is not the promotion signal. Requiring a close-in-time second hit before promotion (v003) lost hit-rate, which suggests the current promotion behaviour is doing useful work that a recency gate removes.
- The marker fold (v002) matched its parent exactly. That fits a layout-only change, but its metadata-size benefit was never measured.
- No variant recorded metadata-size or per-op-time, so the cost side of the cohort is still unknown.
- None of the three insights confirms the overlay source. Each reads the result against the stated change, so the implementations are not verified as written.

## Open questions

- Where is the probing-tier optimum? Is it at 128 slots, or is there a peak between 128 and 256 that a finer sweep would find?
- Why is v003 a Winner when its measured hit-rate fell? Is it carried forward against a different baseline, or is the designation an error?
- Does the recency gate fail on other workloads or sizes, or is it a trace-specific loss?
- Does the marker fold's metadata saving justify the change if it is never measured?

## Continuation

Next cycle should stop growing the probing tier beyond the 128-slot shift and drop the second-hit recency gate from v003. Keep the 128-slot probing split as the lead, sweep a few sizes near it on a second workload, and record hit-rate, metadata-size, and per-op-time. Re-check v003's Winner designation before building on it.
