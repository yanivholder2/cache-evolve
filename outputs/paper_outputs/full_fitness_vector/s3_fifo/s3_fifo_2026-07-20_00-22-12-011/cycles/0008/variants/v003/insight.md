# Variant v003 Insight

## Helped

## Hurt

- per-op-time: Replacing the single `evict_small()` call with a while-loop that drains small to 75% capacity on each overflow doubled put p50 and pushed put p99 up by roughly 8×; concentrating 13 evictions into one put call was more expensive than spreading them one-per-put, and get p50 also regressed — likely because the burst eviction changes which entries occupy small at any given moment, altering the effective working set seen by subsequent gets.

## Takeaway

The batch-drain hypothesis assumed that eviction cost scales linearly with count so that concentrating 13 evictions into one put leaves 12 puts eviction-free and lowers p50. In practice the burst-drain put proved disproportionately expensive: the while-loop overhead, repeated `evict_small()` call overhead, and potential cache/branch-predictor pressure from draining 13 entries in sequence outweighed any savings in the other 12 puts. For this implementation, fine-grained one-at-a-time eviction distributes cost more evenly and keeps both p50 and p99 lower than batching.
