# Variant v003 Insight

## Helped

- **hit-rate**: Sketch decay at 500k-operation intervals refreshed frequency estimates, reducing mispredicted evictions and allowing the frequency-aware selection to better identify cold victims.

## Hurt

- **per-op-time (get)**: Decay's counter-halving operation adds latency overhead; p50 increased by 60ns and p99 spiked by 738ns due to periodic full-table scans.
- **per-op-time (put)**: Decay overhead is more severe on puts (p50 +185ns, p99 +1217ns) because put operations already perform victim selection, so they pay both decay and scanning costs.

## Takeaway

Sketch decay successfully improves hit-rate by keeping frequency estimates responsive to workload phase changes, but introduces significant latency tail variance. The lightweight N=2 victim selection was unable to offset the decay overhead; the periodic full-sketch traversal every 500k operations creates unpredictable latency spikes, especially visible in p99 measurements. A trade-off emerges: temporal freshness in frequency signals improves cache effectiveness, but synchronous decay during normal operations is too expensive for a latency-sensitive cache.
