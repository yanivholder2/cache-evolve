# Variant v002 Insight

## Helped

(none)

## Hurt

- **per-op-time**: FrequencyAwareLRU adds per-hit timestamp updates and composite (timestamp, frequency) eviction candidate selection, increasing latency without improving hit-rate; the additional lfu_counter tracking and comparison overhead manifests especially on put operations (+27% vs parent).

## Takeaway

Frequency-aware eviction (tracking both recency and frequency for composite candidate selection) is orthogonal overhead when paired with v003's proven 75/25 geometry. Hit-rate remains unchanged while latency degrades, confirming that frequency is not a decision-improvement axis for this workload. The 75/25 geometry and adaptive promotion threshold (from v003) are sufficient; adding frequency awareness as an eviction criterion does not help and actively hurts performance through additional tracking and comparison cost.
