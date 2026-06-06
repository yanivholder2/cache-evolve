# Variant v001 Insight

## Helped

- **hit-rate**: Doubling sketch width (SKETCH_WIDTH_FACTOR 4→8) reduced hash collisions in the count-min sketch, improving frequency estimate accuracy for admission decisions and boosting hit rate on Zipfian workloads.
- **per-op-time**: Wider sketch layout improved CPU cache locality during frequency lookups and reduced collision-induced hash chain traversals, yielding faster get/put latencies across p50 and p99.

## Hurt

- **metadata-size**: Sketch heap memory doubled due to the 2× width increase, as sketch capacity scales linearly with the width factor.

## Takeaway

Doubling the sketch width traded a predictable 2× metadata-size cost for dual wins: improved hit rate (through better frequency distinction) and faster operations (through better cache behavior). The per-op speedup suggests that collision-free lookups in the admission decision path are faster enough to benefit overall cache performance at this scale, making the memory cost acceptable for workloads where Zipfian access patterns dominate.
