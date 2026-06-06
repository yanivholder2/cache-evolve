# Variant v001 Insight

## Helped

- **per-op-time (get)**: The adaptive promotion threshold (1 instead of 2) accelerates items into the probing tier after a single hot access, reducing the number of lookups that stall in the target bucket and achieving faster get latency on the concentrated access pattern.

## Hurt

- **hit-rate**: The aggressive 85/15 geometry paired with a lowered promotion threshold (1) increases both eviction pressure on the smaller stash and the promotion of marginal items that aren't truly part of the hot-set, degrading the cache's ability to preserve high-frequency keys.

- **per-op-time (put)**: The faster promotion of items to the probing tier triggers more frequent cascading evictions and demotion chains, offsetting any gains from the geometry change and adding latency to put operations as items churn between tiers.

## Takeaway

Lowering the promotion threshold to match aggressive geometry (85/15) trades accuracy for speed: while get latency improves via faster tier climbing, the combined effect is insufficient promotion selectivity, allowing too many marginal items into the hot tier and overwhelming the stash with eviction cascades. The geometry and threshold changes diverge under this workload—one optimizes latency at the cost of hit-rate decay, while the other amplifies that decay through increased churn.
