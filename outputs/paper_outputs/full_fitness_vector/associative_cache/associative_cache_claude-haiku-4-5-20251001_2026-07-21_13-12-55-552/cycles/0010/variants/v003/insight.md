# Variant v003 Insight

## Helped

- **per-op-time (p99_get)**: Early termination at 256 operations when access patterns stabilize successfully shortens the characterization phase duration, reducing the window of expensive per-operation tracking and lowering p99 get latency by 33%.

## Hurt

- **hit-rate**: Early exit at 256 operations prevents the cache from collecting the full 512-operation characterization window, limiting the working-set profile available for insertion_bias computation. The partial characterization results in weaker pattern recognition, reducing hit-rate by 0.65% relative to the parent's full-window approach.

- **metadata-size**: Two new tracking fields (stability_count and previous_recent_hits_count) add 8 bytes to the AssociativeCache struct, increasing metadata overhead from 80 to 88 bytes.

## Takeaway

The variant successfully trades hit-rate accuracy for latency reduction: early termination on pattern stability cuts p99 latency by a third, but premature characterization phase exit leaves insertion_bias underspecified. The optimization's selective timestamp comparison (only recent 50% of insertions) should reduce per-operation cost, but the simultaneous early termination strategy undermines the characterization's ability to capture sufficient temporal locality information, suggesting that reducing characterization window depth without improving characterization depth elsewhere creates a correctness vs. latency trade-off that tips negative on hit-rate.
