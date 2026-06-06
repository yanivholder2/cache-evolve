# Variant v002 Insight

## Helped

(none)

## Hurt

- **per-op-time**: Conditional decay checks on every operation introduce 3–6 ns latency overhead (get +3.3%, put +5.1%) even though the 1M interval rarely triggers decay, making the periodic conditional a pure cost with no hit-rate benefit.
- **metadata-size**: Adding an 8-byte `op_count` field to track operations since last decay increases TinyLfuCache struct size from 152 to 160 bytes.

## Takeaway

Adding a conservative 1M-operation decay interval to reduce per-operation overhead yields the opposite effect: the repeated conditional checks on every get/put add measurable latency without improving hit-rate on this workload. The sketch does not saturate on the 2048-entry cache with this workload pattern, so decay never meaningfully triggers; the interval is so conservative that it only adds fixed overhead for zero accuracy benefit.
