## Recurring wins

- **Frequency-aware eviction improves hit quality on scan-heavy and Zipfian traces.** LFU-ordered eviction has produced measurable hit-rate gains in every cycle it was tried; the signal is consistent and robust across struct variants.
- **32-byte cache-line aligned struct eliminates p99 tail-latency straddling.** Padding the compact struct to 32 bytes halved p99 in Cycle 2 and was confirmed as a neutral-to-positive structural base in Cycle 3; it should be treated as the default starting layout.
- **Position-0 early-return guard demonstrably reduces get-path write overhead.** The guard that skips promotion for already-hot items validated the write-elimination hypothesis in Cycle 3 (p99 halved), even though the accompanying one-swap cap sacrificed hit-rate convergence.
- **Struct metadata size can be reduced below the 40-byte baseline** while staying aligned — the 32-byte padded form achieves both goals simultaneously and should not be re-litigated.

## Recurring failures

- **Per-op-time has blocked every variant across all three cycles.** It is the sole axis that has prevented every cohort from reaching the Winners bar; all other metric improvements have been negated by a latency regression on at least one path.
- **Linear/unconditional eviction scans cause consistent latency regression.** Whether from LFU counter traversal, wide-bucket lookup, or unbounded bubble-sort, any O(n) path on the hot `get` code shows up as a per-op-time regression regardless of the hit-rate benefit it enables.
- **Swap-to-back promotion is a confirmed dead end.** It regressed both hit rate and latency simultaneously in Cycle 2; at small associativity it is neither a latency win nor an ordering improvement.
- **Two-zone (hot/cold) partitioning is definitively closed.** The boundary drifts with insertions, actively demoting recently accessed items, and saves no real work because baseline LRU already achieves O(1) removal for hot items.
- **Struct shrinkage without alignment awareness backfires.** Removing an unused field reduced metadata size but changed cache-line packing enough to raise get latency (Cycle 1); size and performance objectives conflict unless alignment is explicitly managed.

## Open questions

- Can a bounded promotion loop (cap of 2–3 swaps) composed with the position-0 guard recover hit-rate convergence while preserving the guard's p99 benefit — i.e., does bounding rather than capping at one produce additive wins on both axes?
- If per-op-time still does not improve after the bounded-loop composition, does the measurement floor mask the guard's contribution, or is there a structural bottleneck that additional complexity cannot resolve?
- Does composing all three confirmed pieces (32-byte aligned struct + position-0 guard + bounded LFU bubble) produce interference between the wider item footprint and the alignment benefit, or are they genuinely orthogonal?

## Per-policy notes

- **LFU variants:** Hit-rate signal is consistent and strong across all three cycles. The recurring bottleneck is unconditional loop entry on every `get`, not eviction cost — O(1) tail-removal for eviction is already solved; the remaining work is guarding the promotion loop against stable items.
- **LRU (baseline):** Serves as the latency floor; hot items already sit at removal-cost O(1). Hybrid approaches that borrow LFU ordering for eviction while keeping LRU's O(1) get path are the most promising synthesis direction.
- **Two-zone (hot/cold):** Closed. Do not revisit at these bucket sizes.
- **Struct/layout policy:** The 32-byte aligned form is the confirmed structural base. Further field-size reductions should target fields other than those affecting alignment, and any candidate must be validated against cache-line packing at realistic bucket occupancy before being declared a win.
