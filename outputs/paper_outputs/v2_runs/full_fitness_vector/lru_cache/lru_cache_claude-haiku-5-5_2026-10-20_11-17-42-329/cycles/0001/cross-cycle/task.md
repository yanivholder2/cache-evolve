# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Both evaluated variants changed the eviction policy rather than the data layout: one split the recency list into probationary and protected segments, the other swapped move-to-MRU on hit for a referenced bit. No Winners landed. Each variant gained on hit-rate but paid for it in per-operation cost, and the third variant was rejected for a build failure unrelated to its design.

## Patterns

- Both policy changes improved hit-rate on the LIRS workload-a trace, so reuse-aware eviction looks like the right direction for this workload.
- Both changes added per-node or per-cache metadata, and that overhead showed up in the metadata-size dimension for each.
- The cost of each policy moved into the operations themselves: segment promotion and demotion on get and put in one case, and a variable-length eviction scan on misses in the other.
- The referenced-bit variant also made the put-existing path cheaper, suggesting that avoiding list surgery on overwrite is a worthwhile lever independent of the eviction scheme.
- The rejected variant (v003) failed at dependency fetch (HTTP 429 from the Maven repository), so the cycle yields no evidence about its strategy.

## Open questions

- Would the hit-rate gains hold on workloads without one-hit scans, where segmentation should have less to protect?
- Is the get p50 regression in the referenced-bit variant real or measurement noise?
- Can the second-chance eviction scan be bounded so the put tail stops worsening?
- Can the protected-segment variant keep its hit-rate while removing the extra per-operation pointer work?

## Continuation

Next cycle, keep pursuing reuse-aware eviction, and prefer variants that avoid list surgery on the hit and overwrite paths. Drop further increases in per-node fields unless they buy a clear hit-rate gain. Re-attempt the v003 strategy once the dependency fetch is reachable, since it was never actually evaluated.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
