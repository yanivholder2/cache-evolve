# Variant v001 Insight

## Hurt

- per-op-time: Cache-line alignment (`#[repr(align(64))]`) and early-exit optimization increased both p50 and p99 latencies for get and put operations on the baseline 128×16 bucket geometry, contrary to the hypothesis that these micro-optimizations would generalize from narrower bucket configurations.

## Takeaway

Cache-line alignment and early-exit scanning, which the hypothesis claimed succeeded on 32×64 geometry, did not improve latency on the baseline's 128×16 configuration. The optimizations increased get latency by ~45% (p50) and doubled p99 latency; put operations also deteriorated. The micro-optimizations appear to be geometry-sensitive: benefits observed elsewhere do not automatically transfer to wider, shallower bucket layouts.
