# Variant v002 Insight

## Helped

(None — no fitness dimensions improved with this change)

## Hurt

- **hit-rate**: Declined marginally from 0.7334 to 0.7333 despite the decay mechanism targeting temporal freshness, suggesting the 500k-interval decay does not generalize the cycle 0003 breakthrough to this workload or that the added field bloat subtly impacts accuracy.
- **per-op-time**: Get latency increased from 90 to 91 ns, and put latency increased from 117 to 123 ns, indicating the conditional decay checks and the added `op_count` field introduce measurable overhead even though decay triggers only every 500k operations.
- **metadata-size**: TinyLfuCache struct grew from 152 to 160 bytes (8-byte addition of `op_count`), increasing per-instance overhead.

## Takeaway

Periodic sketch decay at 500k-operation intervals failed to reproduce cycle 0003's hit-rate improvement on this variant. The decay mechanism added latency overhead (likely from conditional checks in hot paths) and metadata bloat without delivering the expected accuracy or latency gains. This suggests that the cycle 0003 breakthrough was either specific to that cycle's workload, that the decay interval or geometry conditions must differ, or that the LRU-only victim selection (unchanged here) prevents decay from improving decision quality.
