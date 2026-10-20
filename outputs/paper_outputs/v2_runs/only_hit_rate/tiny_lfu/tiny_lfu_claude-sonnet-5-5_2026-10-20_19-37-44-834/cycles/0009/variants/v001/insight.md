# Variant v001 Insight

## Helped

- hit-rate: The extra sketch row (same width, min-across-rows estimate, conservative update) lowers the chance that a cold key's estimate is inflated by collisions. The gain over the parent is tiny, well inside what one trace at one cache size could produce as noise. I could not read the overlay source in this session, so this rests on the hypothesis text and the fitness vectors.

## Hurt

- No reported dimension regressed. The only dimension is hit-rate, and it did not drop. Any extra per-access hashing and memory cost is not measured by this fitness vector.

## Takeaway

Adding a fifth hash row to the sketch barely moves hit-rate. On this workload, collision overestimates in the existing sketch were not a meaningful source of misses. Together with the parent's result, where admission-threshold changes also reached only parity, this suggests the frequency-estimation and admission layers are already near saturation here. Further gains would more likely come from the eviction or segment policy, or from other traces and sizes.
