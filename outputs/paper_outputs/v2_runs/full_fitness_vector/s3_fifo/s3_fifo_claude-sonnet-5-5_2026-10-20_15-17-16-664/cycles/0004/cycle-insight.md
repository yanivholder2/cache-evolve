# Cycle 0004 Insight

## Cohort outcome

The cycle tried hash-index membership lookups (v001), a main-eviction ghost that re-admits keys into main with a freq head start (v002), and a doubled ghost bound (v003). Only v001 has a real gain: lower typical `put` and `get` cost with hit rate unchanged. v002 won with no improved dimension. Its extra ghost scan and its lost hit rate point to noise as the reason it was picked.

## Patterns

- v001 is the only variant with a mechanism for its gain. The index removes linear scans and preserves the hit/miss sequence exactly. Its cost is a much larger whole-cache footprint that the fixed per-entry `metadata-size` figure does not show.
- v001's `put` tail barely improved. The remaining tail cost likely comes from eviction and ghost-trimming work, not membership checks.
- Both ghost-side changes (v002, v003) lowered hit rate on workload-a. A freq head start for main-evicted keys and a longer ghost history both let marginal objects into main. This is the third cycle in which anything that loosens main admission has hurt.
- v002 contradicts Cycle 0002's reading of the separate main ghost as a small gain. Its head-start variant regressed, and the extra linear scan undid scan savings. The earlier gain may have been noise or may depend on details of the design.
- Baseline ghost sizing looks near a sweet spot. Reuse beyond main capacity is rare on this trace.
- Latency shifts that have no mechanism, such as v003's tail, are noise again. Latency is still unreliable for selection unless the change has a mechanism.
- Everything still comes from one trace (workload-a) at one cache size.

## Open questions

- Does v001's index hold its gain across traces? What is its real memory cost?
- Can the `put` tail be reduced by making eviction and ghost trimming cheaper?
- Is any form of special handling for main-evicted keys worthwhile? If so, it needs an O(1) tagged-ghost design with evidence of faster reuse.
- Do the earlier small-ratio and ghost results survive repeated runs and other traces?

## Continuation

Keep v001's index-based membership as the new base. Next, target `put` tail cost in eviction and ghost trimming, and account for the index's memory footprint explicitly. Drop the main-ghost freq head start, the larger ghost bound, and further ghost-size tuning. Revisit main-evicted-key handling only as an O(1) tagged variant of the existing ghost. Validate on multiple traces and repeated runs, and judge on hit rate.
