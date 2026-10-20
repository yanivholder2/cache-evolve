## Recurring wins

- Layout and representation changes are the cheapest, policy-neutral lever. Packing flags into spare index bits, narrowing arena links, and dropping per-link discriminants all shrank nodes without costing hit-rate.
- Deferring or bounding recency work helps the put and miss paths. Bounded second-chance scans and a flag-only retry in demotion gave predictable per-operation gains, the clearest in the cohort.
- Keeping list surgery off the hit and overwrite paths is consistently preferred. Removing it on the put-existing path was a cost win.
- Cheap hashing in the admission gate cut its per-access cost on both paths without changing hit-rate on the tested trace.
- Reuse-aware eviction looks like the right direction for this workload. Segmentation and admission gating both gained hit-rate by protecting reused keys or rejecting one-hit keys.

## Recurring failures

- Extra per-node or per-cache metadata keeps adding overhead, and the hit-rate gains have not always justified it.
- Work on every access is expensive. Per-access hashing and extra pointer work on shared paths added cost to both get and put, even where the hit path was meant to be untouched.
- Removing relinks on hit did not measurably speed get, so the hit-path relink is not the dominant cost. Do not pursue it as a goal on its own.
- Arena index rewrites have not sped up get or put. Narrowed indices, sentinel links, and hoisted accessors all left or made get slower, so the cost seems to lie in index access and codegen rather than in link encoding.
- Trades of hit-rate for put-tail gains have been made without clear measurement.
- Several attributions remain inferred rather than verified, and some get movement is unexplained and may be noise.
- One variant was never evaluated because its dependency fetch failed (HTTP 429). That was an infrastructure failure, not a design result.

## Open questions

- Does the get regression from index width and arena access come from casts at hot call sites, and can the smaller links be kept without them?
- Do the hit-rate gains from segmentation and admission gating hold on workloads without one-hit scans and at other capacities?
- Is the get slowdown in the protected-side and arena variants real, or layout or measurement noise? A re-measure or codegen-only comparison would settle it.
- Can the protected segment keep its protection without list surgery on demotion, using a cheap recency signal?
- Can the second-chance scan be bounded so the put tail stops worsening without giving up hit-rate?

## Per-policy notes

- Segmented (probationary and protected) recency: improved hit-rate on the scan-heavy trace. Its promotion and demotion surgery added per-operation cost. A flag-only retry removed most of that cost but gave back some protection, probably the rescued head's MRU position.
- Referenced-bit or second-chance eviction: deferring recency to eviction time helps the put path, and bounding the scan gives a predictable miss-path cost. It loses some hit-rate on re-referenced keys, and that loss has not yet been addressed.
- Admission gating with a frequency sketch: produced the largest hit-rate gain in the cohort by rejecting one-hit keys at capacity. Its hashing cost was the main drawback, now reduced by a shared hash. It has been tested on only one trace and one capacity.
- Layout changes apply across policies: node-size and index work has been policy-neutral in hit-rate, so it can be combined with any eviction scheme.
