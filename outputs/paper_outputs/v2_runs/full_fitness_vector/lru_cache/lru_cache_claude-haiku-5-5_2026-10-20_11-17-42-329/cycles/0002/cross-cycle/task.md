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


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

The cohort moved away from pure policy changes toward layout and bookkeeping: a deferred referenced-bit scheme with one bounded rescue, a capped second-chance scan, and narrower arena indices. All three Winners kept hit-rate roughly intact or gave up only a small amount, and each won on smaller nodes or cheaper operations rather than on hit-rate.

## Patterns

- Node-size reductions were the most reliable lever. Packing a flag into spare index bits and narrowing arena links both shrank the node, and neither cost hit-rate.
- Deferring recency work to eviction time helped the put path more than the get path. Removing relinks on hit did not measurably improve get, so the hit-path relink is not the dominant cost.
- Bounding the second-chance scan traded a small hit-rate loss for a predictable miss-path cost, and that tail was the clearest per-operation gain in the cohort.
- Narrowing indices saved memory but made get slower, which points at the width conversions on every arena access rather than at the policy. Memory wins and speed wins came apart here.
- Several attributions are inferred rather than verified. One variant's node-size result was larger than its hypothesis predicted, and one get-path change was not directly targeted, so those gains should not be credited to the mechanism named in the hypothesis until checked against the source.

## Open questions

- Is the get regression from narrowed indices caused by the casts at hot call sites, and can they be removed while keeping the smaller links?
- Does the referenced-bit scheme's hit-rate loss on re-referenced keys shrink if the single rescue is replaced with a different recency approximation?
- Were the get-path and node-size gains in the bounded-scan variant caused by layout effects or by measurement noise?

## Continuation

Next cycle, combine the cheap node-size reductions with the bounded eviction scan, and pursue ways to remove the index-conversion cost on get without giving back the smaller links. Keep avoiding list surgery on the hit path, since get did not respond to it. Drop further hit-path relink removal as a goal on its own, and avoid trading more hit-rate for put-tail gains unless the trade is clearly measured.


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
