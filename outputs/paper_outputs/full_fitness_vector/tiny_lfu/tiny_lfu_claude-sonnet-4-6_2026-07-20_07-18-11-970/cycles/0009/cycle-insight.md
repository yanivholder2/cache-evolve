# Cycle 0009 Insight

## Cohort outcome

This cycle focused on refining the protected-segment design from Cycle 0008: compacting the flag storage to a packed bitset, layering the segment onto the 8× sketch + amortised-gate parent, and probing a wider 30% protected fraction. The bitset compaction and the compound protected + 8× sketch variant were accepted as winners; the wider protected zone regressed on both dimensions.

## Patterns

- **The fitness model is blind to off-struct heap savings.** v001's bitset compaction reduced the `prot_flags` backing allocation 8× with no change to hit rate or per-op time, yet won by not regressing. The struct-level metrics used for evaluation cannot distinguish this from a no-op. Any further heap-footprint changes will be invisible until a separate allocation-size metric is wired into the fitness function.
- **The 20% protected fraction is near-optimal for a Zipfian trace.** v003 confirmed that raising the zone to 30% neither shields more hot keys nor reduces eviction pressure — it simply fills extra protected slots with lukewarm items while shrinking the probationary pool enough to increase churn and hurt both hit rate and latency. The optimal ratio lies at or below 20%.
- **The admission gate and the protected segment are partially redundant.** v002 combined the segment split with the amortised gate and 8× sketch, but the per-insight observation noted that the gate already shields hot keys effectively enough that the structural refuge adds only a marginal hit-rate increment while meaningfully taxing the put path through dual-list bookkeeping. The compound won, but the incremental benefit of the segment on top of a well-tuned gate is small.
- **Put-path latency is the binding cost of SLRU bookkeeping.** Both this cycle's segment variants paid elevated put p50/p99. Every full-cache put must now conditionally walk and relink two chains; this is an inherent cost of the dual-list design that is independent of the protected-zone ratio.

## Open questions

- Can the dual-list put-path overhead be reduced — e.g., by deferring promotion to `get` only and using a simpler flag check at eviction time — without sacrificing the hit-rate benefit of the segment split?
- Does combining the compact bitset (v001's representational fix) with the compound 8× sketch + protected segment (v002) produce any measurable latency improvement now that the flag-array heap is 8× smaller and more cache-friendly on the eviction path?
- With the protected-segment ratio confirmed at ≤20% and the put-path cost measured, is there a different structural lever — e.g., a clock-hand or generational eviction policy — that delivers segment-like admission filtering with lower per-operation bookkeeping?

## Continuation

The protected-segment axis is approaching its local optimum on this workload: the 20% ratio is near-correct, the bitset compaction is implemented, and the gate + segment compound has been tested. The next cycle should assess whether the put-path dual-list cost can be reduced by restricting segment promotion to the `get` path only (lazy promotion), which would eliminate most of the dual-list traversal from write operations. If that path yields no latency improvement, the protected-segment thread should be treated as complete and attention should shift to genuinely unexplored structural levers — generational or clock-based eviction — that may deliver similar hit-rate gains with lower per-operation overhead.
