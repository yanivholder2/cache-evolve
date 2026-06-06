# Task: cycle-insight-writer

Cycle: 0003
Output dir: /home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0003

## Winners

- v003
- v002
- v001

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0003/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0003/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0003/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0003/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0003/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-06-06_05-09-07-908/cycles/0003/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

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


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

This cycle tested composition of cycle-0001's orthogonal optimizations, further geometry tuning, and a new promotion-filtering dimension. All three variants proved valuable, though with distinct trade-off profiles: back-insert + 50/50 geometry trades latency for hit-rate; aggressive 62.5/37.5 geometry improves both metrics; frequency-gated promotion improves both metrics by filtering false-positive tier transitions.

## Patterns

- **Geometry allocation remains the primary lever**: The progression from baseline (25/75) through 50/50 to 62.5/37.5 shows sustained improvement, confirming that hash-indexed capacity is consistently more valuable than stash capacity on this Zipfian workload. The allocation axis has not yet saturated.

- **Promotion gating is orthogonal and cost-free**: Frequency-gated target→probing promotion (counter ≥ 2 threshold) improves both hit-rate and latency by eliminating false-positive promotions that trigger cascading demotions. This represents a new optimization axis with no apparent downside.

- **Composition can regress latency despite design orthogonality**: While back-insert promotion and 50/50 geometry are mechanically independent, combining them incurs latency overhead from increased insertion work that offsets the hit-rate gain. This suggests careful sequencing matters even when dimensions are logically distinct.

- **Reducing cache churn improves both throughput and accuracy**: v003's success demonstrates that preventing wasteful tier transitions (fewer false-hot items) simultaneously reduces eviction cascades (improving latency) and focuses capacity on genuinely hot keys (improving hit-rate). This reinforces cycle-0001's theme of eliminating unnecessary work on the hot path.

## Open questions

- Do aggressive geometry (62.5/37.5) and frequency gating compose without regression?
- Is 62.5/37.5 the asymptote for geometry allocation, or does further concentration continue to improve both metrics?
- Can the frequency threshold be tuned adaptively (e.g., learned per workload), or is counter ≥ 2 near-optimal?

## Continuation

Geometry and promotion gating emerged as independent, complementary levers. Next cycle should test their composition (v002 + v003 together) and explore adaptive or workload-specific tuning of the frequency threshold. The fact that a simple counter-based gate achieved clean wins suggests the promotion logic has further refinement room beyond just threshold tuning—consider whether adaptive thresholds, per-bucket counters, or different gating strategies for different tiers could yield additional gains.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
