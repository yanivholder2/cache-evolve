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
