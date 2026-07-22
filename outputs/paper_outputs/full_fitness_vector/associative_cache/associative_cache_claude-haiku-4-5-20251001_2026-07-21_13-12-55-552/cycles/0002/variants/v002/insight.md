# Variant v002 Insight

## Helped

- **hit-rate**: Wider buckets (64 × 32) improved memory locality by consolidating items more densely, reducing fragmentation and collision-related misses compared to the narrower baseline geometry (128 × 16).

## Hurt

- **per-op-time**: The enlarged bucket size (32 items vs. 16) increased the cost of sequential intra-bucket searches, particularly for get operations, negating savings from the reduced hash table size and making the overall summary latency worse.

## Takeaway

Consolidating the same total capacity into fewer, wider buckets improved cache efficiency (hit-rate +0.24%) but at the cost of increased operation latency (+45% on median get time). The design reveals that for this workload, the benefit of better memory locality and fewer hash table lookups is outweighed by the penalty of scanning longer buckets during lookup operations. The trade-off suggests this cache is more latency-sensitive than capacity-sensitive under the current workload.
