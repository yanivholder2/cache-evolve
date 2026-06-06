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
