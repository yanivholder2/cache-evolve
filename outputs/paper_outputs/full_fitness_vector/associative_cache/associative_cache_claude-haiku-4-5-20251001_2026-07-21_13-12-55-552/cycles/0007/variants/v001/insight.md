# Variant v001 Insight

## Hurt

- **hit-rate**: Reducing the hot partition from 60% to 12.5% of cache capacity caused a 3.1% drop, revealing that v003's hit-rate improvement depends critically on the larger working-set buffer, not solely on the frequency-aware promotion algorithm.

- **per-op-time**: The reduction in hot-segment capacity increased latency across all operations (get p99 2x, put p99 3x). With fewer hot slots, items remain longer in the cold FIFO segment, forcing more dual-index searches and promotion attempts per access, eroding the benefit of the adaptive partitioning strategy.

## Takeaway

The frequency-aware promotion mechanism cannot sustain v003's performance gains with severely constrained hot-segment capacity. This variant demonstrates that adaptive partitioning is inherently **capacity-bound**—the algorithm's overhead (dual indices, frequency tracking, promotion logic) only pays for itself when the hot segment is large enough to hold the working set. At 12.5% hot capacity, the system thrashes through cold items and incurs high promotion overhead without achieving improved hit-rate, proving that v003's algorithmic contribution is inseparable from its allocation of buffer space.
