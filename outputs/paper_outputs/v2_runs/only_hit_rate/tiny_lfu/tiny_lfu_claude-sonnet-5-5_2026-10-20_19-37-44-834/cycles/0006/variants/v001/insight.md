# Variant v001 Insight

## Helped

- None measurable. The change targeted per-op time, but the fitness vector reports only hit-rate, so any speedup from touching one cache line per sketch op is unmeasured.

## Hurt

- hit-rate: Putting all rows for a key into one shared 64-byte block makes the rows' slots correlated, since they come from bit ranges of the same hash and a block collision hits every row at once. The independence that count-min's min-over-rows depends on is weakened, so estimates are noisier. The result is a hit-rate drop from the parent, larger than the parent's own tie-admit gain, even though counter count and update rule were unchanged.

## Takeaway

Blocking the sketch keeps total counters equal but not collision quality. Cache-line locality costs estimate accuracy, and on workload-a at size 2048 the admission filter is sensitive to that. A locality optimization like this can only be judged if the eval also measures per-op time. With hit-rate alone it shows only the cost, so it would need a different blocking scheme or a per-op-time dimension to justify it.
