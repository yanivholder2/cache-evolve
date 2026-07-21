# Variant v002 Insight

## Helped

- **hit-rate**: Halving the probationary quota from `capacity / 4` to `capacity / 8` expanded the protected LRU budget from 75% to 87.5% of capacity, cutting the frequency at which hot items are demoted back to probationary and letting more of the Zipfian hot set remain in protected across eviction cycles.

## Hurt

- **per-op-time**: Both get p50 and put p99 rose relative to the parent; with a larger protected queue, a greater share of accesses follow the protected-hit path (unlink + push_back_prot) rather than the cheaper probationary-eviction path, increasing the average cost per operation.

## Takeaway

Shrinking the probationary admission cap to 12.5% confirms that on Zipfian traces the bottleneck was demotion pressure from an undersized protected queue, not admission filtering — the hit-rate hypothesis was correct and the gain was realized with no change to node layout or eviction logic. The accompanying per-op-time increase suggests that as the protected queue holds a larger fraction of entries, the proportion of operations that traverse the unlink/push_back_prot path grows, introducing a latency trade-off that should be monitored if the quota is narrowed further.
