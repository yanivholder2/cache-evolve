## Recurring wins

- Node-size reductions are the most dependable lever. Packing flags into spare index bits and narrowing arena links both shrank nodes without giving up hit-rate.
- Bounding the second-chance eviction scan gives a predictable miss-path cost, at the price of a small hit-rate loss. It is the clearest per-operation gain seen so far.
- Reuse-aware eviction improves hit-rate on the LIRS workload-a trace, so it remains the right policy direction.
- Deferring recency work to eviction time helps the put path more than the get path.

## Recurring failures

- Extra per-node or per-cache metadata for policy bookkeeping has cost more in memory than its hit-rate gain justified.
- Segmenting the recency structure adds pointer work to get and put, which erodes the hit-rate benefit.
- Removing relinks on the get path has not measurably improved get. Hit-path list surgery is not the dominant cost, so it should not be a goal on its own.
- Narrowing arena indices saves memory but slows get, most likely from width conversions on every arena access rather than from the policy itself.
- Attributions are often inferred rather than verified. Several gains were credited to a mechanism that the measurements do not clearly support, so claimed causes need source-level checks.
- A variant was rejected for an unrelated build or dependency-fetch failure, so its strategy has never been evaluated.

## Open questions

- Do the hit-rate gains from reuse-aware eviction hold on workloads without one-hit scans, where segmentation should protect less?
- Can the protected-segment benefit be kept while removing its per-operation pointer work?
- Is the get regression from narrowed indices caused by casts at hot call sites, and can those casts be removed while keeping the smaller links?
- Does the hit-rate loss from deferred referenced-bit handling on re-referenced keys shrink with a different recency approximation?
- Are the node-size and get-path gains from the bounded-scan variant real, or are they layout effects or measurement noise?
- Can the bounded second-chance scan keep its put-tail gain without further hit-rate cost?

## Per-policy notes

- Segmented LRU (probationary/protected): hit-rate improved, but per-operation cost rose from promotion and demotion on get and put. Worth revisiting only if the extra pointer work can be removed.
- Referenced-bit / second-chance: improves hit-rate and makes overwrite cheaper when list surgery is avoided. Bounding the eviction scan has been the most useful refinement, though it trades a little hit-rate.
- Deferred referenced-bit with a single rescue: keeps hit-rate roughly intact and wins on smaller nodes and cheaper operations, but loses some hit-rate on re-referenced keys. The rescue may need a different recency approximation.
- Policy-agnostic layout work (narrow indices, packed flags): not a policy change, but the most reliable source of wins. Its get-path regression is a separate issue to fix.
