# Variant v001 Insight

## Helped

- per-op-time: Only get p99 moved, and only by noise-level amount; pre-reserving the map and slab removed no measurable cost from the common path.

## Hurt

- per-op-time: Put p99 got slightly worse, so in-place slot reuse on eviction did not shorten the put tail. Put p50 and get p50 are unchanged.

## Takeaway

Pre-sizing the key->index map and node slab, and reusing the evicted slot in place, left every dimension effectively unchanged. The parent already pays little for growth or allocation in steady state (the cache is filled to capacity early, so rehash/grow is a one-time cost), so put latency is bounded by something else, likely the sketch update and admission comparison. Hit-rate and metadata-size were unchanged, as expected from leaving the sketch, admission rule and Node layout alone. Allocation churn is not the bottleneck to target next.
