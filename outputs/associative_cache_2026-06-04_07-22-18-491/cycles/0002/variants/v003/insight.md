# Variant v003 Insight

## Helped

(none)

## Hurt

(none)

## Takeaway

Replacing the lexicographic `(lfu_counter, position)` eviction criterion with the product score `(lfu_counter + 1) * (position + 1)` produced no change in hit rate on workload-a at size 2048. The two scoring functions must agree on which item to evict in the vast majority of bucket states this trace generates — the cases where they diverge (a high-frequency stale item sitting at a low position competes with a newly inserted item at a high position) either occur too rarely or cancel out across the trace. The predicted benefit of shielding newly inserted items proportionally to bucket occupancy did not surface as a measurable effect on this workload.
