# Variant v002 Insight

## Helped

- per-op-time (get): Routing ghost hits through small queue instead of main reduces main queue population, shortening the search chain during get operations.
- per-op-time (put): Smaller main queue reduces eviction cost, as fewer items need to be processed when main capacity is reached.

## Hurt

- hit-rate: Ghost hits re-entering via small queue with freq=0 must prove hotness (freq > 1) to graduate to main; items accessed infrequently after re-admission get evicted from small without ever reaching main, causing subsequent re-misses.

## Takeaway

This variant trades hit rate for operation latency. By filtering ghost hits through the small queue rather than promoting them directly to main, it reduces main queue overhead and achieves ~40% faster get/put operations. However, the stricter re-admission path results in an ~8% hit rate decline, suggesting that direct main promotion of ghost hits was actually protective for moderately-accessed items that benefit from being preserved in the resident queue.
