# Cross-Cycle Insights

## Recurring wins

- **Hot-path efficiency improvements**: All successful variants (back-insert promotion, geometry rebalancing, struct compaction) eliminated unnecessary work on the critical path—redundant climbs, stash traversals, and memory pressure—suggesting the baseline left ample room for hot-path optimization.
- **Exploiting Zipfian workload skew**: Winners aligned with the trace's skewed access distribution; strategies that keep hot items resident longer or improve cache locality for repeated accesses saw both latency and hit-rate gains.
- **Orthogonal optimization axes**: Improvements operated at distinct layers (promotion logic, bucket geometry, memory layout) with no apparent conflicts, hinting at potential for composition across improvements.

## Recurring failures

- None observed yet.

## Open questions

- Do the three independent optimization axes compose, or do interactions diminish returns when combined?
- How do hot-path efficiency strategies perform on non-Zipfian workloads (uniform or bimodal distributions)?
- Is the 16/16 bucket split an optimal sweet spot, or does further geometry tuning yield additional gains?
- What other orthogonal dimensions remain unexplored (e.g., adaptive geometry, alternative eviction policies, promotion thresholds)?

## Per-policy notes

- **Promotion-based eviction**: Back-insert ordering keeps hot keys resident longer by deferring promotion until genuine reuse; aligns well with Zipfian patterns where repeated accesses to popular keys dominate.
- **Geometry-based access patterns**: Wider bucket allocation reduces traversal latency on misses and improves cold-hit detection; both latency and hit-rate improved together rather than trading off.
- **Memory layout**: Item struct compaction improved cache efficiency on both get and put operations, suggesting memory layout is a substantial factor in repeated-access performance.
