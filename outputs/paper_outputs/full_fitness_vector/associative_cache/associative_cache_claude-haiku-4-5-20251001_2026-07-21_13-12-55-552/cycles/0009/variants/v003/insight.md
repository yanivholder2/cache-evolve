# Variant v003 Insight

## Helped

- **hit-rate**: Workload characterization during the first 512 insertions successfully identified temporal access patterns by sampling the ratio of recent-hit items; the computed insertion_bias (derived from this characterization) positions new items more favorably in the eviction order, improving hit-rate by 8.1%.

## Hurt

- **metadata-size**: Tracking fields added to enable characterization (characterization_phase, hot_phase_count, insertion_bias, recent_item_insertions VecDeque, recent_hits_count, total_gets_count) increased the AssociativeCache struct size from 24 bytes to 80 bytes, a 56-byte overhead that persists across the cache lifetime.

- **per-op-time**: The insertion_bias logic incurs latency on every put operation—computing insertion position via `compute_insertion_position()` and inserting at a calculated offset (rather than push_back) is more expensive than the parent's append-only strategy. Get p99 rose from 100 ns to 300 ns, and put p50 rose from 100 ns to 200 ns. The VecDeque operations and Instant tracking during characterization also contributed to measurement overhead.

## Takeaway

Workload characterization is effective at identifying temporal reuse patterns—the variant's 8.1% hit-rate gain validates that adaptive insertion bias can tune positioning to match access locality. However, the hypothesis's promise of "zero per-operation overhead during normal execution" is undermined by the implementation: the insertion_bias logic runs on every put, not just during characterization, and the tracking fields add permanent memory cost. This variant demonstrates a high-hit-rate-vs.-latency trade-off: the characterization strategy works, but its cost structure differs from the hypothesis—ongoing computation cost rather than a one-time payoff.
