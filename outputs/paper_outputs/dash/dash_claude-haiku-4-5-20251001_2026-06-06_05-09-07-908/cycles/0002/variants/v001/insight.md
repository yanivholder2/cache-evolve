# Variant v001 Insight

## Helped

- **hit-rate**: Back-insert promotion (`InsertPosition::Back`) keeps recently promoted items at the MRU end of their target bucket longer, delaying demotion and reducing the likelihood they are evicted back to the stash before accumulating enough evidence to promote to the probing tier.

## Hurt

- **per-op-time**: Back-insert insertion requires updating more list positions than front-insert, adding a small overhead to get operations (2 ns p50 increase) that outweighs any latency benefit from the improved promotion stability.

## Takeaway

Combining back-insert promotion with the geometry rebalancing improves hit-rate by 0.26%, but at the cost of a 1.7% latency penalty. The trade-off shows that while back-insert reduces eviction pressure on recently-promoted items, the insertion cost itself becomes the bottleneck. For this workload, the hit-rate improvement is modest relative to the latency regression, suggesting that front-insert promotion remains the better choice despite geometry rebalancing's benefit.
