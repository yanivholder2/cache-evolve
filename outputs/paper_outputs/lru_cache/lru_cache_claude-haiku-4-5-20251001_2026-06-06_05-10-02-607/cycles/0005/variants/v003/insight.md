# Variant v003 Insight

## Helped

- **per-op-time (put p99)**: Saturation-conditional deferral prevents indefinite queue accumulation on puts by flushing pending promotions when saturation drops below 60%, avoiding the 297 ns spike from v002 and reducing put tail latency to 162 ns.
- **metadata-size**: At low saturation, deferral_enabled remains false, so the pending_promotions queue stays empty and wastes minimal overhead in the common case.

## Hurt

- **per-op-time (get p99)**: When saturation is below 60% (the typical state in this workload), the `deferral_enabled = false` branch forces immediate frequency updates on every get(), restoring the full pointer-chasing cost that v002 avoided by deferring, increasing tail latency from 114 ns to 226 ns.
- **per-op-time (get p50)**: Immediate frequency updates add constant overhead to median get latency, moving from 35 ns to 40 ns.
- **metadata-size**: The deferral_enabled bool flag adds 8 bytes to LruCache struct overhead (144 → 152 bytes).

## Takeaway

The saturation-conditional toggle successfully stabilizes put latency by preventing queue overflow during underutilized periods—a real win for mixed workloads. But this variant fails on the given workload because the cache never reaches the 80% saturation threshold where deferral's benefits would activate. Instead, the workload remains in the low-saturation regime where immediate updates are the fallback, undoing v002's get optimization entirely. The approach is architecturally sound but only wins if the workload exhibits the sustained high-load phases the thresholds were designed to detect.
