# Variant v001 Insight

## Hurt

- per-op-time: LFU's frequency-counter bookkeeping adds overhead to every access (get p50 +45%, put p50 +40%), without compensating hit-rate gains.

## Takeaway

Switching from ClassicLRU to LFU in this cache did not improve hit-rate on the Zipfian workload—both maintained 0.6611—while introducing measurable latency penalties across all operations. The result suggests that recency-based eviction already aligns well enough with the access pattern, or that the LFU implementation's per-access counter maintenance dominates any frequency-awareness benefit at this cache size and workload intensity.
