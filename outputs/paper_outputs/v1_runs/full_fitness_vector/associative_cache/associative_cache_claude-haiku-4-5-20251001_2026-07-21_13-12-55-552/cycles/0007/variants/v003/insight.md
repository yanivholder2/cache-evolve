# Variant v003 Insight

## Helped

- hit-rate: Robin Hood insertion reorders items by probe distance, causing frequently-accessed items to cluster earlier in the bucket sequence, which improved ClassicLRU's ability to retain hot items and increased hit-rate by 8.6%.

## Hurt

- per-op-time (get): The robin_hood_insert scanning loop and the recalibration of probe_distances in get_from_position() add computational overhead that dominates the hot path, increasing get p50 latency by ~45%.
- per-op-time (put): The Robin Hood insertion algorithm's item scanning, distance comparison, and swap operations are significantly more expensive than simple appends, nearly doubling put p50 and p99 latency (143→300ns, 204→400ns).

## Takeaway

Robin Hood hashing's probe-distance-balancing approach successfully improved hit-rate by better ordering items for the eviction policy, but the implementation cost is prohibitively high. The scanning and swapping overhead on every insertion makes both get and put operations much slower, and put p99 latency actually worsened (204→400ns) rather than improving, defeating the hypothesis's goal of reducing variance. For this workload, the algorithmic complexity of Robin Hood insertion outweighs the variance reduction benefit—a simpler insertion strategy is needed to keep latency low while maintaining the hit-rate gains.
