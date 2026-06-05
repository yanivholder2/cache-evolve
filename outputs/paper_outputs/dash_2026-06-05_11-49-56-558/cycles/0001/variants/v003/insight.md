# Variant v003 Insight

## Helped

- hit-rate: Doubling normal buckets from 8 to 16 (at stash's expense) expanded the hot-tier probing space, keeping frequently-accessed items on the fast lookup path longer and reducing their displacement into — or eviction from — the stash.

## Hurt

## Takeaway

For workload-a at capacity 2048, the baseline's 8-normal/24-stash split over-provisions stash relative to what the workload's working set actually needs. Rebalancing to a 50/50 split shows that the hot tier was the binding constraint: more normal-bucket slots absorbed more of the working set, yielding a clear hit-rate gain with total capacity and eviction policy unchanged. The added `.max(2)` clamp in the implementation also reveals that the edge-case validity guard is necessary before this change can generalise safely to smaller harness-provided capacities.
