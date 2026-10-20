# Variant v003 Insight

## Helped

- per-op-time: Collapsing the dual-chain SLRU into a single LRU list eliminated the conditional two-chain traversal on the `get` and `put` hot paths, cutting `get` p99 in half and dropping `put` p50 and p99 by one latency tier each.
- hit-rate: The bounded CLOCK scan gave recently-referenced nodes the same second-chance reprieve as the SLRU protected segment, matching (and fractionally exceeding) v002's hit quality without a separate list.
- metadata-size: Removing `prot_head`, `prot_tail`, and `prot_count` shrank the `TinyLfuCache` control-plane struct by 40 bytes (232 → 192), while `Node<u64,u64>` stayed at 48 bytes as expected.

## Hurt

*(no dimension regressed)*

## Takeaway

Replacing the SLRU protected segment with a CLOCK reference bit confirmed that the structural protection zone in v002 was redundant: the frequency-based admission gate already filters cold entries, so a single bit per node slot in a bounded scan can replicate SLRU's second-chance semantics at a fraction of the bookkeeping cost. The latency gains are concentrated in the eviction path — eliminating the per-eviction conditional branch over two chain heads was sufficient to measurably tighten both `get` and `put` tail latencies — while hit quality was essentially unchanged, validating the hypothesis that CLOCK and SLRU are equivalent on this workload.
