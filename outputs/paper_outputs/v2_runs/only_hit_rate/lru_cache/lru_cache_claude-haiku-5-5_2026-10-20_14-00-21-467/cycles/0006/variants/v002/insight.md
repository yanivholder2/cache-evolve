# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Tail insertion combined with promotion only on the second hit leaves a key that has just been hit once still sitting near the eviction end, so on workload-a at size 2048 keys that would be re-referenced later are evicted before their second touch.

## Takeaway

Making MRU placement wait for a second hit, while new keys enter at the LRU tail, is a stricter scan filter than this workload tolerates: on workload-a at size 2048 it lost hit-rate relative to the parent. The first hit does not move a node, so re-referenced keys stay exposed to eviction until the second hit. The fitness vector has no per-op-time dimension, so the per-op cost of the altered hit path that the hypothesis asked to record was not measured and remains untested.
