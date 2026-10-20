# Variant v001 Insight

## Helped

- **hit-rate**: Switching `ClassicLRU::eviction_candidate_index` from a constant `0` to a linear scan for the minimum `lfu_counter` preserved hot items that had drifted to the LRU end but still carried accumulated frequency signal, reducing false evictions on the Zipfian workload-a trace.

## Hurt

- **per-op-time**: The O(n) linear scan over all bucket items on every eviction doubled the put p99 latency (100 ns → 200 ns in the parent), adding a measurable worst-case overhead even though the p50 summary was unchanged.

## Takeaway

Frequency-minimum victim selection in `ClassicLRU` meaningfully improves hit-rate by protecting items whose access history is genuinely high from being displaced simply because recency has temporarily lapsed — a meaningful correction given MRU-end insertion. However, the improvement comes at a p99 latency cost: the scan executes on every bucket overflow and its cost is proportional to bucket occupancy, so the gain is not free. Future work should consider whether a partial scan or a maintained frequency-ordered structure can recapture the hit-rate benefit without the tail-latency regression.
