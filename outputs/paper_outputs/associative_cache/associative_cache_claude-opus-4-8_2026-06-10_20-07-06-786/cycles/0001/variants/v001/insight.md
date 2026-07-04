# Variant v001 Insight

## Helped

- hit-rate: Halving the bucket count and doubling bucket size in `layout` widens each bucket (e.g. 128×16 → 64×32), so hot keys that previously evicted each other in a 16-way set now share a 32-way set and survive longer — fewer collision evictions, slightly more hits.

## Hurt

- per-op-time: Doubling associativity doubles the worst-case intra-bucket scan length, so the structural cost of this change is longer per-op lookup/insert work within each wider bucket (not re-measured in this variant's vector, but inherent to the coarser split).

## Takeaway

On workload-a the cache was collision-bound at 16-way, not capacity-bound: redistributing the *same* total capacity into fewer, wider buckets buys hit rate purely by lowering per-bucket eviction pressure. The gain is small, suggesting most contention is already absorbed at 16-way and further coarsening will trade growing scan cost for diminishing hit-rate returns. The even-only guard keeps capacity exact across odd bucket-count sweeps without disturbing routing.
