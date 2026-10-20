# Variant v003 Insight

## Helped

- hit-rate: Selecting the lower-frequency of the two LRU-oldest entries as the eviction victim prevents discarding entries whose sketch count reflects past bursts, rescuing items that pure recency ordering would incorrectly identify as expendable.

## Hurt

- per-op-time: Adding two `sketch.estimate()` calls on every full-cache eviction path — one for the head and one for the second-oldest node — raised both put p50 and put p99 latency, and the benchmark summary (driven by the p50 floor) degraded substantially relative to baseline.

## Takeaway

Best-of-2 victim selection does lift hit rate by correcting frequency-blind LRU eviction, but the cost of the two extra sketch probes per eviction fires on every `put` when the cache is full, making the latency trade-off unfavorable for this workload. Future work on this axis should consider whether the frequency comparison can be deferred or batched to amortize the probe overhead.
