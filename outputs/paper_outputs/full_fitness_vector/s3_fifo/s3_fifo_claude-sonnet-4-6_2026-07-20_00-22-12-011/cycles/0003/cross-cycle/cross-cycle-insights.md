## Recurring wins

- **Monotone small-queue shrinkage on Zipfian workloads**: Reducing the small-queue fraction from 1/4 → 1/10 → 1/20 has improved hit rate and get latency each time with no observed penalty. The original 1/4 split over-invested in admission filtering; the trend is still running clean after three cycles.
- **Amortizing cleanup cost rather than paying it on the hot path**: Replacing synchronous O(n) ghost-deque cleanup (`ghost.retain()`) with lazy tombstoning cut put tail latency by 5× and is a pattern that has generalized — deferred work that can be batched into future eviction passes consistently outperforms eager in-place maintenance.
- **O(1) data-structure upgrades for membership tests**: Replacing the linear ghost-queue scan in `put()` with a HashSet oracle was the single highest-leverage change in cycle 1; the principle (replace O(n) scan with O(1) oracle) has continued to guide wins.
- **Orthogonal structural changes compose without interaction penalty**: Queue sizing, O(1) ghost operations, parallel freq deques, and lazy deletion each touch disjoint code paths and have stacked cleanly in composition — no cross-dimension regression observed when combining them.
- **Per-instance fixed overhead is safe**: Control-struct growth (HashSet header, freq-deque headers) is constant per cache instance and has never materialized as a measurable cost at practical capacities.

## Recurring failures

- **Loosening the ghost-promotion threshold (freq > 0)**: Tried in cycle 3; backfired with a material hit-rate regression. The ghost channel enforces two independent gates — frequency evidence *and* post-eviction re-request confirmation. Bypassing the re-request gate floods main with once-warm entries lacking recency evidence. The paper's `freq > 1` rule is load-bearing; do not revisit.
- **Pure O(1) random victim selection in `evict_main()`**: Two-random-choices eliminated deque-churn latency but discarded frequency-drain protection, allowing hot entries to be evicted on a single bad draw. The hit-rate cost was material, not noise. Complexity class and eviction quality are coupled here; reducing one without preserving the other is not a net win.

## Open questions

- **Floor for small-queue shrinkage**: Three cycles of clean wins leave the optimal fraction unresolved. At what point does further shrinkage harm cold-miss handling or increase tombstone backlog pressure as `main_size` grows toward full capacity?
- **Bounded second-chance scan in `evict_main()`**: Can a shallow candidate cap (e.g., 8–16 entries) recover most of the deque-churn latency reduction observed with two-random-choices while preserving the frequency-drain quality that pure O(1) random sacrificed?
- **Tombstone backlog size under dense ghost hits**: Lazy tombstoning leaves ghost-deque slack that is drained by future capacity-guard sweeps. The transient backlog size under high ghost-hit-rate workloads is uncharacterized; it may matter as `main_size` expands.
- **Fitness-function sensitivity**: Cycle 3 designated a variant (no benefits, one regression) as a Winner. Whether this reflects legitimate latency weighting in the aggregate score or a threshold artifact is unresolved and could cause future misleading winner designations.

## Per-policy notes

- **S3-FIFO — small-queue sizing**: The original 1/4 fraction was consistently over-provisioned. Current best: 1/20. Shrinkage accelerates both read latency (shorter scan, more hot-set residency in main) and hit rate, at the cost of a mild put-latency nudge from a longer `evict_main()` traversal — a trade-off that has been dominated by other gains so far.
- **S3-FIFO — ghost channel**: Serves as a dual gate (frequency filter + demand confirmation). The `freq > 1` threshold is not bookkeeping overhead; it requires that a key be re-requested *after* ghost-listing before promotion to main. Weakening this gate has a demonstrated, material cost.
- **S3-FIFO — `evict_main()` second-chance scan**: Now the primary remaining loop cost after all O(n) hot paths were removed. Full O(n) traversal preserves frequency-drain quality; pure O(1) random preserves neither frequency nor recency ordering. A bounded scan is the most promising unexplored middle ground.
- **S3-FIFO — lazy ghost tombstoning**: Highly effective amortization pattern; deque cleanup is deferred to the eviction capacity guard. Works cleanly under low-to-moderate ghost-hit rates; behavior under sustained high ghost-hit density has not been stress-tested.
