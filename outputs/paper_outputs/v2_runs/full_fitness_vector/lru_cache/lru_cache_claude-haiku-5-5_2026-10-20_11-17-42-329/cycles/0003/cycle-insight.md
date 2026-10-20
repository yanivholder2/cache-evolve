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
