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
