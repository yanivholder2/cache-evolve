# Variant v001 Insight

## Helped

## Hurt

## Takeaway

Splitting `get()` into separate small and main passes to enable eager promotion of items whose `freq` reaches 2 produced no change in hit-rate on the workload-a trace. At this capacity (2048), the small queue holds roughly 200 slots — items that accumulate enough accesses to cross the promotion threshold are already being swept into main promptly by `evict_small()` before they face real eviction pressure, so moving promotion from the eviction path to the access path did not shift which items end up resident. The hypothesis correctly identified that lag in the small queue is a theoretical vulnerability, but on a Zipfian workload the hot set crosses the threshold fast enough that the lag was already negligible in practice.
