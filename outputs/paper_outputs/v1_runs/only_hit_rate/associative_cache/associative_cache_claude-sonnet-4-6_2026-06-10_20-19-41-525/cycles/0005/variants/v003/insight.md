# Variant v003 Insight

## Helped

## Hurt

## Takeaway

Capping `lfu_counter` at 4 produced no measurable change relative to v001's unbounded increment on this trace. The warm-lock recovery problem the hypothesis targeted — counters accumulating so high that many decay events are needed before an item becomes cold again — does not manifest at a scale that affects hit-rate on workload-a at size 2048. Either the decay pass fires frequently enough to keep counters in check already, or the trace's access pattern never drives a bucket's hot-item counters high enough to delay cold-pool restoration in a way the eviction path can distinguish. The counter ceiling is safe to carry forward but offers no leverage here.
