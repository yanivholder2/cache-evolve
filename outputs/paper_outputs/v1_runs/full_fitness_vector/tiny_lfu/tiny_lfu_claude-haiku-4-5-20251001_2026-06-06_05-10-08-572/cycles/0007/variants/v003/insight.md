# Variant v003 Insight

## Helped

- per-op-time: Reducing the width factor from 4 to 3 improved get latency by ~17% (132→109 ns p50) and put latency by ~18% (166→136 ns p50), likely due to improved cache locality or reduced hash collision overhead in the sketch computation despite the smaller factor.

## Hurt

(None detected)

## Takeaway

Reducing the sketch width factor from 4 to 3 delivers significant latency improvements without harming hit-rate or the actual heap footprint (both round to 4096 entries via `next_power_of_two()`). The hypothesis predicted metadata savings that didn't materialize due to power-of-two alignment, but the unexpected per-operation speedup suggests the leaner computation path or better memory access patterns more than compensate. Width-3 establishes a faster baseline while maintaining identical hit-rate and practical memory usage.
