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


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

The cohort split into two families: a bounded second chance on the protected side of the segmented policy (v001), and a representation change that replaces optional arena links with a sentinel index (v002). Both Winners kept hit-rate intact or recovered some of it; v002 won on node size alone, while v001 bought hit-rate back at a visible per-operation cost. A third variant (v003) added a frequency-sketch admission filter, which produced the cohort's clearest hit-rate gain but paid for it on every access.

## Patterns

- Representation changes are the cheap, policy-neutral lever. Dropping the per-link discriminant shrank the node and left hit-rate unchanged, but it did not recover speed, so the `Option` match is not the hot-path cost.
- Arena indexing looks like the real per-operation cost. Both the sentinel change and the admission filter left `get` slower, which points at repeated index access and bounds-checked lookups rather than at link encoding or policy logic.
- Hit-rate gains are coming from what enters or is retained in the cache, not from list order. The admission filter gained hit-rate by rejecting one-hit keys at capacity, and the protected-side rescue recovered part of a prior loss by delaying demotion.
- Anything that runs on every access is expensive in this workload. The admission filter's per-access hashing added cost to both `get` and `put`, and the protected-side rescue's extra pointer work showed up in `get` as well as `put`, even though the hit path was meant to be untouched.
- Attributions are still partly inferred. Both v001 and v003 saw `get` movement the hypothesis did not predict, so the source of that movement is unconfirmed.

## Open questions

- Does the admission filter's hit-rate gain hold beyond workload-a and beyond a single capacity, or is it specific to the scan pattern?
- Can the admission check's hashing be made cheap enough to keep its hit-rate, for example by reusing a hash already computed for the map lookup?
- Is the `get` regression in v001 and v003 real, or is it a layout or noise effect that a re-measure would clear?
- Can the per-access arena lookups be reduced without changing link width?

## Continuation

Next cycle, combine the sentinel-link node reduction with the protected-side rescue only if the rescue's extra pointer work can be moved off the hit path, and re-measure `get` before crediting either change. Pursue admission gating as a hit-rate lever, but drop any design that hashes on every access unless the hashing cost is removed. Stop pursuing further link-encoding changes; the next gains are more likely in arena access patterns.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

The cohort tried three changes: a flag-only retry in the protected-side demotion branch (v001), hoisted arena index accessors (v002), and a single shared hash for the admission sketch (v003). Only v001 landed as a Winner, and it won on per-operation cost, giving up a small part of its hit-rate in exchange.

## Patterns

- The list surgery in the protected-side demotion path was a per-operation cost, separate from the second-chance idea itself. Keeping the rescued head in place removed that cost, which is consistent with earlier cycles finding that extra pointer work on shared paths is expensive.
- The hit-rate given back by v001 is small and plausibly comes from losing the MRU rotation that protected the rescued head. This makes the protection and the cost the same mechanism, so further gains here will likely trade against it.
- Hoisting index conversions into accessors (v002) did not speed up `get` or `put`, and both got slower. The access-count explanation from cycle 0002 does not hold on its own; codegen or layout effects are the more likely cause.
- Replacing SipHash with a cheaper shared hash (v003) reduced the admission gate's per-access cost on both paths without changing hit-rate on the one trace tested. This confirms that hashing cost, not the gate's decision logic, was the expense flagged in cycle 0003.
- Attributions are still partly inferred. v001's reading rests on the hypothesis text and fitness vectors rather than on the diff, and v002's cause is untested.

## Open questions

- Does the flag-only retry keep its per-operation gain without losing hit-rate if the rescued head is given a different recency approximation?
- Does the cheap admission hash keep its hit-rate on other workloads and capacities, since only one trace at one capacity was checked?
- Is the `get` and `put` slowdown from the hoisted accessors caused by inlining or codegen, and does comparing the parent's inline casts against the helper form in isolation settle it?

## Continuation

Next cycle, build on v001's flag-only demotion and try to recover the lost protection with a cheap recency signal that does not reintroduce list surgery. Keep the cheap admission hash and test it on another workload and capacity before stacking more on it. Drop further arena access rewrites until a codegen-only comparison explains the v002 slowdown.


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
