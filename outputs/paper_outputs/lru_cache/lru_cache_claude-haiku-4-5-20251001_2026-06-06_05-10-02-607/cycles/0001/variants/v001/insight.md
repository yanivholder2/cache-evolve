# Variant v001 Insight

## Helped

- per-op-time (put): Zone-partitioned eviction reduces the cost of put operations by approximately 19%, as the main zone eviction path is simpler and does not require examining hot-zone candidates.

## Hurt

- metadata-size: Added 8 bytes per Node for zone tracking (u8 field) and 24 bytes for LruCache struct capacity management, increasing overall memory footprint.
- per-op-time (get): Zone promotion logic adds a small overhead (~7% latency increase for get operations), as each get must check whether to promote from main to hot zone.

## Takeaway

Segmented LRU with 30/70 zone partitioning failed to improve hit rate on the Zipfian workload—the protection strategy did not benefit this access pattern. However, the zone-based structure unexpectedly improved put() performance by simplifying eviction decisions. The cost is modest memory overhead and marginal get() latency, suggesting S-LRU trades write performance for better temporal isolation without hit-rate gains on this workload.
