# Variant v001 Insight

## Helped

- hit-rate: Wider sketch reduces hash collisions in frequency estimation, allowing the admission gate to better distinguish hot arrivals from cold ones, resulting in more precise eviction decisions.

## Hurt

(none)

## Takeaway

Doubling the sketch width factor (from 4 to 8) was an effective tuning that improved hit rate by reducing collision-induced errors in frequency estimation. This variant demonstrates that the baseline's sketch configuration was undersized relative to the capacity, and that collision overhead in the Count-Min sketch was a practical bottleneck limiting admission accuracy.
