# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants targeted independent axes — eviction policy semantics, bucket geometry, and struct metadata size — and each moved its primary metric in the right direction, yet none cleared the Winners bar because every variant regressed per-op-time, the shared blocking dimension.

## Patterns

- **Hit-rate gains are available but latency-gated.** Both the LFU-semantics variant and the wider-bucket variant improved hit rate on the LIRS trace, but both paid for it through linear scans: LFU requires scanning for min/max frequency counters, and wider buckets increase the number of entries touched per lookup. The hit-rate signal is real; the bottleneck is eviction/lookup cost.
- **Struct shrinkage does not imply speed.** Removing the unused timestamp field achieved the predicted metadata-size reduction precisely, but the changed memory layout altered cache-line packing enough to raise get latency — showing that struct-size and runtime-performance objectives can conflict even when the removed field is never read.
- **Per-op-time was the universal disqualifier.** No variant avoided a latency regression; it was the single axis that prevented all three from winning, regardless of which primary metric they were optimising.

## Open questions

- Can a constant-time LFU structure (e.g., a frequency-bucketed doubly-linked list) recover the hit-rate win from v001 without the O(n) scan cost?
- Do the hit-rate strategies (LFU semantics and wider buckets) compose — i.e., would combining them produce an additive hit-rate gain or introduce new interference?
- Is the cache-line regression in v003 layout-dependent (fixable with explicit alignment) or inherent to the smaller struct size at this bucket occupancy?

## Continuation

The next cycle should treat per-op-time as a hard constraint and pursue the hit-rate gains already demonstrated — specifically, explore a constant-time eviction structure that preserves LFU's scan-burst resilience while keeping get and put on the O(1) path. Wider-bucket geometry (32×64) remains a promising complement once the latency cost is controlled, and the metadata-size win from struct trimming is worth retaining as a no-regress starting point rather than re-litigating it.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
