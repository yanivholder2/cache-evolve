## Recurring wins

- (Single cycle so far, so nothing is confirmed as recurring yet.) Making admission into the scarce normal tier stricter is the strongest signal. The stash→target edge needed two stash hits before promotion, and that variant beat every variant that changed placement.
- Filtering at the stash edge appears cheap because the stash is larger than the normal tier and can hold unproven keys while they wait.

## Recurring failures

- (Single cycle; provisional.) Changes that loosen admission hurt hit-rate. This includes doing it indirectly: MRU landing moves an item to a hotter position, and that lowers the bar for promotion to probing.
- Placement that relies on a free slot (spilling into an empty probing slot) rarely triggers, because normal buckets are almost always full at the same time in steady state.
- Anything that adds normal-tier moves sets off cross-bucket demotion cascades and hurts neighbouring buckets too.

## Open questions

- Only hit-rate has been measured. Metadata-size and per-op-time are unknown, including the claimed get-latency benefit of the stash-hit filter.
- Is two the best stash-hit threshold, or would an adaptive threshold do better?
- Does a hit-count or recency filter on the target→probing edge give the same benefit as the stash filter?
- Can MRU landing help if it is paired with a separate hit counter, so the bar for entering probing doesn't drop?

## Per-policy notes

- Tiered bucket policy (stash → target → probing/normal), the only policy explored so far:
  - Which items get into the tier matters more than where they are placed inside a bucket or which bucket they go to.
  - The "is hottest" promotion check reads bucket position. Any change to where an item lands also changes its promotion threshold, so these must be evaluated together.
  - The current baseline to build on is the two-stash-hit admission filter.
