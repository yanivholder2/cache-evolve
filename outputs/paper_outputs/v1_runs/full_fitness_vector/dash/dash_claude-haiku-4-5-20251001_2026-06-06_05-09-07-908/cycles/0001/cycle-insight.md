# Cycle 0001 Insight

## Cohort outcome

All three variants—back-insert promotion, geometry rebalancing, and Item struct compaction—proved successful with no negative tradeoffs. Together they reveal three independent optimization axes: promotion ordering strategy, hash-indexed capacity allocation, and memory layout efficiency. All three winners focused on hot-path efficiency improvements aligned with Zipfian workload characteristics.

## Patterns

- **Orthogonal improvements across layers**: The winners operate at three distinct levels (promotion logic, bucket geometry, struct packing) with no apparent conflicts, suggesting they could potentially compose.
- **Common theme of reducing unnecessary work**: v001 eliminates redundant LRU climbs; v002 eliminates unnecessary stash traversals; v003 eliminates memory pressure from sparse Item structs. All three avoid expensive operations on the hot path.
- **Strong alignment with workload skew**: All improvements exploit the Zipfian nature of the trace, where a small set of hot keys dominates. Back-insert keeps hot items resident longer; geometry rebalance catches cold misses more often; packing improves cache efficiency for repeated accesses.
- **Latency and hit-rate move together**: Rather than trading one for the other, improvements span both dimensions—wider bucket allocation improves hit-rate _and_ reduces traversal latency; tighter Item packing helps _both_ get and put operations.

## Open questions

- Do these three optimizations compose, or do they interact in ways that diminish returns?
- How do these strategies perform on non-Zipfian workloads (uniform or bimodal distributions)?
- Is the 16/16 bucket split the optimal sweet spot, or is there further geometry tuning to explore?

## Continuation

The first cycle's clean wins across distinct optimization axes suggest the baseline left room on multiple fronts. Next cycle should test whether these optimizations stack (e.g., v001 + v002 + v003 together) and explore other orthogonal dimensions: alternative eviction policies, different promotion thresholds, or adaptive geometry tuning. Confirm whether these patterns hold under diverse workload distributions.
