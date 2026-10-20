## Recurring wins

- Representation changes that shrink nodes without costing hit-rate: packing flags into spare index bits, narrowing arena links, and replacing optional links with a sentinel index.
- Bounding the second-chance eviction scan gives a predictable miss-path and put-tail gain, at a small hit-rate cost.
- Hit-rate gains come from what is admitted or retained, not from list order. Reuse-aware eviction has improved hit-rate on the LIRS workload-a trace in more than one form.
- Deferring recency work to eviction time helps the put path more than the get path.

## Recurring failures

- Work added to every access keeps costing more than it returns. Per-access hashing and extra pointer work on the hit path both slowed get and put.
- Removing hit-path relinks did not measurably speed get, so it is not a sufficient goal on its own.
- Extra per-node or per-cache metadata has consistently shown up as overhead without a matching hit-rate gain.
- Narrowing or re-encoding arena indices saves memory but has not restored get speed. Changing the link encoding alone is not the fix.
- Trading hit-rate for put-tail gains has only been worthwhile where the trade was clearly measured.

## Open questions

- Do the hit-rate gains from reuse-aware eviction and admission hold on workloads without one-hit scans, and at other capacities?
- Is the get regression seen across several variants real, or a layout or measurement-noise effect that a re-measure would clear?
- Can per-access arena lookups and index width conversions be reduced without giving up the smaller links?
- Can admission hashing reuse a hash already computed for the map lookup, making the filter cheap enough to keep its hit-rate?
- Can the protected-side rescue's pointer work be moved off the hit path?
- The alternative eviction strategy from cycle 0001 was never evaluated, so it remains untested.

## Per-policy notes

- Segmented (probationary/protected) LRU: improves hit-rate on scan-heavy traces, but its promotion and demotion work shows up in get and put. The delayed-demotion rescue recovered part of a hit-rate loss at a visible per-operation cost.
- Referenced-bit / second-chance eviction: the overwrite path got cheaper by avoiding list surgery. The bounded scan gave the clearest put-tail gain in the cohort. The main weakness is a hit-rate loss on re-referenced keys, which a single rescue did not fully remove.
- Frequency-sketch admission filter: the clearest hit-rate gain seen so far, from rejecting one-hit keys at capacity. Its per-access hashing cost is the main obstacle, so it is only worth pursuing if that cost is removed.
