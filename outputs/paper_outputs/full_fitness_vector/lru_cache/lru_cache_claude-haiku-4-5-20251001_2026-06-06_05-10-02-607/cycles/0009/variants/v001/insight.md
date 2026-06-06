# Variant v001 Insight

## Hurt

- **per-op-time (get p50)**: Median get latency regressed 35% (34→46 ns) despite the sampling strategy. The counter increment on every operation, coupled with stale saturation state that only updates every 64 operations, apparently introduces sufficient overhead or causes inaccurate bucketing mode decisions (staying in simple mode when bucketing would be beneficial) that undermine the intended optimization.
- **per-op-time (put p50)**: Median put latency regressed 11% (72→80 ns) for the same reason—the sampled saturation approach keeps the cache in suboptimal modes more often than the parent's real-time saturation checks.
- **metadata-size**: LruCache struct increased 8 bytes (192→200 bytes) to add the saturation sampling state (saturation_sample_count, saturation_sample_threshold, saturation_state).

## Helped

- **per-op-time (put p99)**: Tail latency improved 12% (177→156 ns), though this appears to be a side effect of inaccurate saturation tracking rather than a targeted benefit.

## Takeaway

Sampling saturation to amortize its computation cost failed because the delayed saturation feedback caused poor adaptive bucketing decisions. Rather than recovering the parent's p99 gains while fixing the p50 regression, the variant made both median latencies worse. The core issue is that caching saturation state stales the bucketing decision: the cache no longer responds immediately when actual saturation changes, leading it to operate in inefficient modes. Counter increments and stale-state branch penalties offset any savings from deferring the division, revealing that for this workload, real-time saturation feedback is more valuable than the micro-optimization of async sampling.
