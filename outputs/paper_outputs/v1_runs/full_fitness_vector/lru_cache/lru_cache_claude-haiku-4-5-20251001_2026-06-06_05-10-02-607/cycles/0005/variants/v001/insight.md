# Variant v001 Insight

## Helped

- **metadata-size**: LruCache struct reduced 16.7% (192→160 bytes) by moving bucket infrastructure into a lazily-allocated `Option<Box<BucketState>>`. Eliminates the permanent 32-byte footprint of pre-allocated frequency vectors when running in simple mode.

## Hurt

- **per-op-time (get p99)**: Tail latency spiked 40× (97→3955 ns) because hysteresis-driven mode transitions became pathological. The 60% disable threshold (vs. v003's immediate switch at 75%) causes saturation to oscillate near boundaries, triggering repeated expensive allocations, deallocations, and bucket rebuilds during the same workload phase.
- **per-op-time (get p50)**: Median latency degraded 11.8% (34→38 ns) due to `Option<Box<>>` indirection and pointer dereference overhead when accessing bucket state in bucketing mode.

## Takeaway

Lazy allocation successfully reduced structural overhead but exposed a fundamental latency amplification: hysteresis mode transitions became the dominant cost. While simple mode now runs faster (no pre-allocated bucket fields to maintain), switching into and out of bucketing mode via frequent allocations/deallocations is far more expensive than v003's direct field access. On workloads near saturation boundaries—where the trace spends extended periods—this variant becomes unreliable, with p99 latency degrading by two orders of magnitude. The memory win (32 bytes) does not justify the latency loss.

