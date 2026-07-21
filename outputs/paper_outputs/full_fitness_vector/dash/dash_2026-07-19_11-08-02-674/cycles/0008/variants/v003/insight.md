# Variant v003 Insight

## Helped

- **per-op-time**: The `stash_cold_evict_prepare` scan fires only on full-stash-bucket inserts — far less frequently than the target LFU-min scan added in the parent — so its O(stash-bucket-size) cost adds negligible overhead; put p99 fell from 200 ns back to 100 ns.

## Hurt

*(none)*

## Takeaway

Cold-preferring stash eviction eliminated the put p99 regression introduced by the parent's target-tier scan, because the stash tier overflows far less often than target buckets, keeping the new scan's amortized cost negligible. However, hit-rate did not improve: on workload-a at this cache size, the stash is too small a fraction of the total cache footprint for prioritising cold eviction within it to shift the aggregate miss curve, indicating that the stash tier is not the right leverage point for hit-rate gains on this Zipfian workload.
