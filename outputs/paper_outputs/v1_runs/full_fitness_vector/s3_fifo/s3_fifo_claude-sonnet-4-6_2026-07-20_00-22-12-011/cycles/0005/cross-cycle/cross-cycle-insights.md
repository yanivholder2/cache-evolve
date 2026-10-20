## Recurring wins

- **Eliminating O(n) scans is the highest-leverage lever**: Every cycle's biggest gains came from replacing a linear scan with an O(1) structure — ghost-check HashSet (c0001), lazy tombstoning to defer deque cleanup (c0002), and the `resident_map` that eliminated `position()` entirely (c0005). Each fix delivered outsized latency reductions relative to its code complexity.
- **Amortizing per-event cleanup across future evictions pays off reliably**: Lazy ghost tombstoning (c0002) — removing a key from the HashSet immediately on hit but deferring deque compaction to the capacity guard — cut put tail latency by 5× and introduced no observable correctness risk. The pattern of deferring cheap-to-skip work generalizes.
- **Structural consolidation multiplies gains**: Collapsing separate data structures into a single `resident_map<K,(V,u8)>` (c0005 v003) simultaneously eliminated two separate deques, an Entry value field, and multiple redundant probes — the only variant across all cycles to win on all three fitness axes at once.
- **Orthogonal changes compose cleanly**: Queue sizing, ghost-check complexity, and entry layout have touched disjoint code paths across cycles and stacked without interaction penalty. The composite v001 in c0002 confirmed this explicitly.
- **Small-queue shrinkage has been monotonically beneficial through 1/20**: Reducing the small-queue fraction from 1/4 → 1/10 → 1/20 improved hit-rate and get latency at each step with no penalty, indicating the baseline dramatically over-invested in admission filtering.

## Recurring failures

- **Bounding the second-chance scan cap was falsified**: The c0003 continuation hypothesized that a fixed scan cap would recover hit-rate quality from the two-random-choices approach while limiting deque churn. c0004 showed the opposite: on Zipfian workloads, cold entries naturally settle at the tail so the unbounded scan already terminates early, and a hard cap only forces eviction of still-hot entries beyond the budget.
- **Frequency step acceleration increases total eviction work**: Draining frequency faster (step=2, c0004 v003) caused moderately-warm entries to leave main prematurely, raising the ghost-hit rate and the total number of `evict_main()` calls — scan-length reduction and total-eviction-work reduction are decoupled and can oppose each other.
- **Lowering the ghost-promotion threshold (freq > 0) degrades hit-rate**: c0003 v003 confirmed that the `freq > 1` rule is not conservative bookkeeping — it enforces that main receives only keys re-requested *after* ghost-listing, not merely re-accessed while still in small. Relaxing it floods main with weakly-warm entries. This threshold should be treated as fixed.
- **Small-queue shrinkage past 1/20 hit a scan-driven crossover at 1/40**: The latency penalty from a deeper main queue cancelled the hit-rate gain when the `position()` scan was still present. This crossover may not recur now that `position()` is eliminated, but the dynamic is worth monitoring.

## Open questions

- **What is the new dominant per-operation cost after `position()` is gone?** Eviction-loop deque churn, ghost-channel overhead, and HashMap hash-and-probe cost at large working sets are all candidates — their relative weight has not been characterized with the c0005 baseline.
- **Is sub-1/20 small-queue fraction viable again?** The c0004 crossover was explicitly scan-driven. With `position()` removed, the cost model for deeper main queues has changed; 1/40 or smaller may now be free again on Zipfian workloads.
- **How large does the ghost tombstone backlog grow under dense ghost-hit workloads?** Lazy deletion defers deque compaction to the capacity guard, and the growth rate of the transient backlog before the guard drains it has not been measured.
- **Does the miss-path scan fraction (quantified as ~27% of accesses paying two O(n) sweeps in c0005) differ significantly across non-Zipfian workloads?** All evaluations have used Zipfian; workloads with flatter access distributions would have higher miss rates and amplify any remaining O(n) paths differently.

## Per-policy notes

- **S3-FIFO (all cycles)**: The baseline left headroom on multiple independent dimensions simultaneously — queue sizing, ghost-check complexity, entry layout, and deque scan patterns — all of which yielded wins without interaction. The evolved baseline after five cycles: 1/20 small-queue fraction, HashSet ghost oracle with lazy tombstoning, and a unified `resident_map<K,(V,u8)>` replacing per-entry value+frequency fields and all O(n) deque lookups. The paper's `freq > 1` ghost-promotion threshold is load-bearing and should remain fixed. The `evict_main()` second-chance loop is the last major structural loop, but its effective cost on Zipfian workloads is low because cold entries naturally accumulate at the tail — probing it is lower priority than characterizing the new cost profile introduced by the `resident_map` HashMap at large working sets.
