# Variant v002 Insight

## Helped

## Hurt

- **per-op-time**: Removing the `.min(8)` cap forces `eviction_candidate_index` to walk every position in the bucket on each eviction call, directly multiplying the comparison work per `put` and pushing both put and get latency well above the parent's baseline.

## Takeaway

Expanding the second-chance scan to full bucket depth inflicted a large latency penalty with zero hit-rate return, revealing that never-accessed items (lfu_counter == 0) at positions beyond the 8-slot tail are effectively absent on this workload — the 3-swap promotion loop keeps the hot zone well-populated with touched items, so the extra scan range finds nothing to improve eviction quality and becomes pure overhead.
