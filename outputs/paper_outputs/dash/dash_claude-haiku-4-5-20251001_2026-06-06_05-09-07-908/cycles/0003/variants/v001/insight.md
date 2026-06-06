# Variant v001 Insight

## Helped

- **per-op-time**: Frequency-gated promotion (threshold of 2) to the probing tier reduces operational overhead by filtering out single-access items that would otherwise incur demotion cascades; get operations see a 10 ns latency improvement from fewer traversals through the expensive probing tier.

## Hurt

- **hit-rate**: The frequency threshold is too conservative for this workload; items that need promotion to maintain hit-rate never reach the probing tier if accessed only once or twice in the observation window, causing them to be evicted from target before achieving sufficient frequency to qualify for promotion.

## Takeaway

Frequency gating successfully reduces per-operation latency by eliminating churn from low-frequency promotions, but the fixed threshold of 2 sacrifices hit-rate. The hypothesis expected to inherit v002's hit-rate strength by operating "orthogonally to promotion order and geometry," but gating affects which items ever reach the probing tier at all. A threshold-based approach requires workload-specific tuning; this variant's conservative gate filters out too many items for the given access patterns, proving that latency and hit-rate cannot both be optimized simultaneously without adapting the threshold to the workload's frequency distribution.
