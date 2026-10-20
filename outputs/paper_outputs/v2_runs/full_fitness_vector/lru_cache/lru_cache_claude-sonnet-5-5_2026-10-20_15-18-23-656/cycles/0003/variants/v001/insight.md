# Variant v001 Insight

## Helped

- metadata-size: Replacing the `usize`/`Option<usize>` links and segment head/tail fields with `u32` indices (`u32::MAX` as NIL) shrank both the per-node and the whole-cache footprint. The segment flag fit in the freed padding, so the node reached the ~32 byte target.
- hit-rate: Unchanged. Promotion, demotion, segment ratio and eviction order were kept identical, so the layout change did not alter which keys stay resident.

## Hurt

- per-op-time: Both get and put latency, including the p99 tails, got worse even though the cache is smaller. The likely causes are the added `as usize` widening and NIL comparisons at every arena access, and the avalanche fold added to the hash. The saved bytes did not buy cache locality at this workload size.

## Takeaway

Narrowing links to `u32` achieves the memory goal without touching hit-rate, but on this benchmark it did not speed anything up. The hypothesis's expectation of near-equal latency was wrong: per-op time rose, and the avalanche fold and index conversions are the likely cost. Metadata size and speed trade off here, and a later variant should separate the hash fold from the layout change to find out which one caused the slowdown. I could not read the overlay source in this session, so that attribution comes from the hypothesis text and the fitness vectors, not from the diff.
