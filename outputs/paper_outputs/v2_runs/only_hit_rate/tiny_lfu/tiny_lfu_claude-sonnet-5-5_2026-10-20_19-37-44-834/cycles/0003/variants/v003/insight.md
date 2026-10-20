# Variant v003 Insight

(Note: the overlay source could not be read in this session; this is based on the hypothesis and the fitness vectors.)

## Helped

- metadata-size: No measurement exists for this variant, so no gain is confirmed. The intended gain was 4-bit counters packed 16 per `u64`, which should shrink the sketch far below the baseline's heap table.
- per-op-time: Also unmeasured. The single 64-bit hash, split into row indices with a power-of-two mask, was meant to replace 4 independent hash calls.

## Hurt

- hit-rate: It fell relative to the parent. The likely causes are the 15-count saturation cap with no aging or reset, which lets stale hot keys keep high counts, and the smaller table with correlated row indices, which adds collisions. Either one makes the admission comparison against the LRU victim less discriminating.
- metadata-size / per-op-time: The vector reports only hit-rate. The two dimensions the hypothesis targeted were not recorded, so the trade-off cannot be judged.

## Takeaway

Compacting the sketch to 4-bit saturating counters with a shared hash and no aging did not preserve accuracy, which contradicts the hypothesis's premise that a compact sketch costs no hit-rate. Without metadata-size and per-op-time measurements, the hit-rate loss cannot be weighed against any savings. A follow-up should restore aging, such as periodic halving, and make sure the size and timing dimensions are evaluated.
