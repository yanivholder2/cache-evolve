# Variant v001 Insight

## Helped

- hit-rate: Replacing ClassicLRU with `ClassicLRUFreqProtect` across all buckets caused probing (and target) eviction to prefer zero-access newcomers over confirmed-reuse residents, retaining items with at least one in-tier hit when a never-touched alternative was available.

## Hurt

## Takeaway

Adding a single frequency guard — evict `lfu_counter == 0` first, fall back to LRU position — produced a clear improvement over the plain ClassicLRU parent. Resetting `lfu_counter` to zero on stash demotion was a necessary complement: without it, items arriving in stash with a warm counter inherited from their probing or target tier would unfairly protect stash slots from cold newcomers, undermining the guard's intent in the cold tier. The result shows that the probing boundary is a meaningful loss source in this workload — the working set is large enough that newly promoted items without confirmed reuse are being wrongly retained over items with demonstrated demand.
