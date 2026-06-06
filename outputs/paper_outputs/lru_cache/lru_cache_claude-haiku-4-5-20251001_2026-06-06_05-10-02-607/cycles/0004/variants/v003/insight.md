# Variant v003 Insight

## Helped

- hit-rate: Frequency-based sampling preserves hot items better than sequential LRU eviction, improving cache effectiveness on skewed workloads.
- per-op-time (put p50): Slightly faster median put operations due to O(1) random sampling eliminating iteration overhead.
- per-op-time (get p50): Negligible improvement from streamlined access path.

## Hurt

- per-op-time (put p99): Catastrophic regression from 135ns to 383ns; random sampling and frequency tracking add unpredictable latency spikes under eviction pressure.
- per-op-time (get p99): Elevated from 83ns to 112ns due to frequency counter increment and promotion logic adding memory barriers.
- metadata-size: Node size grew from 48 to 56 bytes to accommodate the per-item frequency counter.

## Takeaway

Random sampling with frequency tracking achieved modest cache-effectiveness gains (1.3% hit-rate improvement) but inflicted severe latency penalties, especially on put p99 (2.8x worse). The frequency counter adds constant overhead to every access, and random sampling's tail latency is unpredictable when eviction candidates cluster in low-frequency regions. This suggests frequency-based selection is incompatible with latency-sensitive workloads; future iterations should prioritize deterministic, amortized eviction cost or accept trading hit-rate for stable tail latency.
