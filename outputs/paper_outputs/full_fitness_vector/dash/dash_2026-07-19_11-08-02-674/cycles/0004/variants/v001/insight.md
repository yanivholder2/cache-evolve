# Variant v001 Insight

## Helped

*(no dimensions moved positively)*

## Hurt

- **metadata-size**: v001's overlay covers only `dash_segment.rs`, so v003's `item.rs` structural reduction (which brought `Item<u64,u64>` down to 24 bytes) is absent here — the Item struct reverts to its baseline 40-byte footprint, making this variant strictly worse on memory overhead.
- **hit-rate**: Raising the promotion gate from ≥ 2 to ≥ 3 caused a fractional drop relative to v003, confirming that the stricter threshold filters out a small cohort of genuinely reusable non-hottest items that the ≥ 2 gate was promoting correctly.
- **per-op-time**: The summary p50 is unchanged, but `put` p99 doubled (100 ns → 200 ns) compared to v003, suggesting that the threshold change interacts with the promotion/eviction path in a way that introduces occasional tail-latency spikes on writes.

## Takeaway

Combining the vacancy guard with a ≥ 3 frequency threshold failed to reproduce the hit-rate selectivity advantage observed in the prior ≥ 3 experiment (cycle/0003/v002), instead yielding a marginal hit-rate loss and a put p99 regression versus v003. The dominant finding is structural: because each variant's overlay is applied independently against the base (not stacked on its parent's overlay), carrying forward only `dash_segment.rs` silently dropped v003's Item struct compaction, revealing that hit-rate and metadata-size gains from a parent can be lost when a child hypothesis omits files its parent changed.
