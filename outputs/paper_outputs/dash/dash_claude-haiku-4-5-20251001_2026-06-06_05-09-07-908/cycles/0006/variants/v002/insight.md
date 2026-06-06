# Variant v002 Insight

## Helped

- **hit-rate**: Allocating 80% of buckets to the normal pool reduces hash collisions for hot items, allowing more working-set entries to coexist without eviction. The improved from 0.6634 to 0.7132 reflects better capacity utilization through aggressive bucket distribution.
- **per-op-time (get)**: Fewer normal buckets collide with each other; the 26-bucket pool reduces search-chain depth for lookups. Get latency drops from 183 ns to 115 ns as most operations resolve on the first or second probe in the now-larger normal bucket pool.

## Hurt

- **per-op-time (put)**: The reduced stash (6 buckets vs. baseline's implied 16) forces more put operations to compete for overflow capacity. Put latency increases from 142 ns to 160 ns as stash contention rises under insertions that overflow the normal bucket pool.

## Takeaway

The 80/20 geometry validates that the hit-rate/latency frontier within the saturation zone favors aggressive normal bucket allocation. This variant demonstrates that swinging from a balanced split toward 80% normal buckets yields 7.5% better cache hit-rate and 37% faster get operations—establishing that the optimal allocation sits closer to the aggressive end of the explored range, not the conservative middle. The modest increase in put latency is a reasonable trade-off given the substantial wins on hit-rate and get performance.
