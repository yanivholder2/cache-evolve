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


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three variants targeted distinct axes inherited from Cycle 0001's open questions — constant-time LFU ordering, O(1) swap-based promotion, and cache-line alignment via padding — yet none cleared the Winners bar: the LFU bubble variant still regressed per-op-time on the get path, the swap variant hurt both hit rate and latency simultaneously, and the alignment variant recovered tail latency at the cost of a partial metadata-size give-back.

## Patterns

- **LFU hit-rate signal survives; the get-path overhead does not.** The bubble-sort LFU again demonstrated frequency-based eviction's advantage on scan-heavy traces, and eviction is now genuinely O(1) (tail removal). The remaining bottleneck is unconditional loop entry on every `get`, even when the accessed item's rank is already stable — a guard comparing the counter to its left neighbour before entering the loop is the natural next step.
- **Swap-to-back is a double loss at small associativity.** Replacing `remove+push` with `swap` failed on both dimensions: the bounded linear shift is not the latency bottleneck at small bucket sizes, and the ordering approximation degrades hit quality on the Zipfian workload. This axis should be abandoned.
- **Cache-line alignment is a confirmed, targeted fix for p99 spikes.** Padding the compact 24-byte struct to 32 bytes eliminated tail-latency straddling precisely as predicted, halving p99; the cost is 8 bytes of per-item footprint. At 32 bytes the struct still sits below the 40-byte baseline, making alignment padding the right structural foundation going forward even though it does not satisfy the metadata-size metric on its own.

## Open questions

- Can a neighbour-check guard (`if counter > left_neighbour.counter { bubble }`) on the LFU get path reduce unconditional overhead enough to clear per-op-time while preserving the hit-rate win?
- Does composing the 32-byte aligned struct with the guarded LFU bubble produce additive gains on both per-op-time and hit rate, or does the wider item footprint introduce new pressure that offsets the alignment benefit?
- Is there a way to reclaim the metadata-size dimension (currently 32 bytes vs. a 40-byte baseline) while holding the alignment fix — e.g., shrinking a different field rather than adding padding?

## Continuation

The next cycle should compose the two most promising pieces: the 32-byte aligned struct (confirmed p99 fix, already below baseline footprint) as the structural base, and the LFU bubble eviction with a stability guard that skips the bubble loop when the accessed item already outranks its left neighbour. This combination targets per-op-time and hit rate simultaneously without reintroducing the swap ordering regression. The swap-to-back promotion approach should be dropped entirely — it is not the latency bottleneck at these bucket sizes and actively harms hit quality.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants targeted the two open threads from Cycle 0002 — reducing get-path overhead via early-return guards and improving hit-rate quality via frequency-aware ordering — plus one structural experiment with two-zone partitioning. No variant reached the Winners bar: the position-0 guard traded latency for hit-rate, the two-zone partition regressed on both axes, and the single-swap LFU gained hit-rate without moving per-op-time.

## Patterns

- **Position-0 guard confirms the latency bottleneck but over-constrains promotion.** The early-return for already-hot items halved p99 latency as predicted, validating the write-elimination hypothesis. However, capping promotion to one swap per access prevents mid-bucket items from converging to the front quickly enough on a Zipfian workload, eroding the hit-rate the parent's full bubble had built. The useful insight is directional: the guard itself is sound, and the cap just needs to be wider than one.
- **Single-swap LFU with the aligned struct is a clean hit-rate gain at no latency cost.** Composing the 32-byte struct from Cycle 0002 with a frequency-ordered single swap lifted hit quality by steering eviction toward genuinely cold items without touching per-op-time — confirming that LFU ordering's advantage is eviction-candidate quality, not struct overhead, and that the aligned layout is a neutral-to-positive base.
- **Two-zone partitioning is a definitively closed avenue.** The hot/cold boundary shifted with every insertion, pushing boundary items back into the cold zone and degrading hit rate, while the hot-zone bypass saved no real work because baseline LRU already keeps hot items where removal is O(1). No reformulation of this design is worth pursuing at these bucket sizes.

## Open questions

- Can a bounded bubble loop (cap of 2–3 swaps) recover the hit-rate convergence that the single-swap variants sacrifice, while still short-circuiting the rare long-distance promotions that drive latency on a Zipfian workload?
- Does composing the position-0 guard with a bounded loop (rather than one swap) produce additive wins on both per-op-time and hit-rate, or does even a short loop reintroduce enough overhead to nullify the guard's p99 improvement?

## Continuation

The next cycle should compose three confirmed pieces: the 32-byte aligned struct as the structural base, the position-0 early-return guard for already-hot items, and a bounded promotion loop capped at 2–3 swaps rather than exactly one. This triplet directly addresses the failure mode identified in v001 (single-swap too conservative) while preserving v003's hit-rate quality and v001's tail-latency benefit. Two-zone partitioning should not be revisited. If per-op-time still does not improve after bounding the loop, the next hypothesis should examine whether the measurement floor itself masks the guard's contribution rather than adding further structural complexity.


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
