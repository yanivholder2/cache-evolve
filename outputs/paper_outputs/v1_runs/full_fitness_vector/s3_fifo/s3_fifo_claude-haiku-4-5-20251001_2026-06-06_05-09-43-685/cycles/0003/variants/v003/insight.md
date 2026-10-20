# Variant v003 Insight

## Helped

- per-op-time: Adaptive threshold scaled by occupancy reduces small-queue evictions when capacity is available, lowering latencies across get (p50: 248→140ns, p99: 394→161ns) and put (p50: 1088→577ns, p99: 4150→1462ns) operations.

## Hurt

(none)

## Takeaway

Adaptive promotion thresholds unlock latency gains without sacrificing hit-rate. By loosening the promotion bar (freq > 1) when main-queue occupancy is below 50%, the variant reduces unnecessary small-queue evictions and improves throughput. Under contention (occupancy ≥ 80%), the threshold tightens (freq > 3) to protect against pathological latency spikes. The constant hit-rate suggests the threshold adjustment does not eject valuable entries—it merely makes promotion decisions more responsive to queue state.
