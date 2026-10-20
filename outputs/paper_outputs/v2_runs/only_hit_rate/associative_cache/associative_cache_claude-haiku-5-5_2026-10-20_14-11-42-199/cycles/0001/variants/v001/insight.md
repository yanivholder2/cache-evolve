# Variant v001 Insight

## Helped

- hit-rate: Overlaying `BucketGeometry::layout` to fix bucket width at 64 slots turns the 2048-slot budget into 32 wide buckets instead of the harness's 16-slot split, which gave a small hit-rate gain on workload-a.

## Hurt

## Takeaway

Repartitioning the same 2048 slots into fewer, wider buckets helped hit rate only marginally on workload-a, which suggests conflict misses from overloaded buckets were a minor share of misses at this size. The per-op-time cost the hypothesis predicted from scanning up to 64 entries per bucket was not measured in this variant's fitness vector, so it is unconfirmed rather than shown to be absent. The geometry change also adds a fallback that keeps the harness split for budgets not divisible by 64, instead of the `debug_assert!` the hypothesis proposed.
