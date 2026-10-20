# Cross-Cycle Insights

## Recurring wins

- **LFU hit-rate signal is durable.** Frequency-based eviction has improved hit rate on LIRS/scan-heavy traces in every cycle it was tested. The signal is consistent regardless of implementation variant; only the latency cost of acting on it changes.
- **Cache-line alignment reliably fixes tail latency.** Padding the compact struct to 32 bytes (below the 40-byte baseline) halved p99 in Cycle 0002, confirming that straddling is the root cause of tail-latency spikes and that explicit alignment is a targeted, predictable fix.
- **Struct footprint can stay below baseline while holding alignment.** 32 bytes sits comfortably under the 40-byte baseline, meaning metadata-size and alignment are not in fundamental conflict — there is headroom to reclaim the metadata-size metric without sacrificing the alignment fix.

## Recurring failures

- **Per-op-time regression has blocked every variant across both cycles.** It is the single universal disqualifier: variants that improved hit rate, reduced struct size, or fixed tail latency all still regressed mean latency enough to miss the Winners bar. Every next attempt must treat per-op-time as a hard gate, not an afterthought.
- **Unguarded ordering loops on the get path are expensive regardless of form.** Both the Cycle 0001 min/max scan and the Cycle 0002 unconditional bubble loop impose overhead on every `get`, even when no reordering is needed. The pattern of entering the loop first and checking whether work is needed second is the shared root cause.
- **Swap-to-back promotion is a dead end.** It failed on both latency and hit rate simultaneously: the linear shift is not the bottleneck at small associativity, and the ordering approximation degrades Zipfian hit quality. No further variants should explore this direction.
- **Struct shrinkage without alignment awareness backfires.** Removing an unused field in Cycle 0001 changed cache-line packing and raised get latency even though the field was never read. Raw size reduction and runtime performance are not the same objective.

## Open questions

- **Can a stability guard eliminate the LFU get-path regression?** Checking whether the accessed item already outranks its left neighbour before entering the bubble loop is the most direct path to preserving the hit-rate win while fixing per-op-time. This is the highest-priority open question.
- **Do the 32-byte aligned struct and guarded LFU bubble compose without interference?** The wider per-item footprint from alignment padding could offset the latency benefit of the guard — this interaction needs empirical validation.
- **Does wider-bucket geometry (32×64) compose additively with O(1) LFU eviction?** Wider buckets improved hit rate in Cycle 0001 but paid a scan cost. With a guarded O(1) eviction path, the lookup overhead may shrink enough to make this combination net-positive on both axes.
- **Can the metadata-size metric be reclaimed while holding the alignment fix?** Shrinking a field other than the one that was padded is the natural path; it has not been attempted.

## Per-policy notes

- **LFU**: Consistent hit-rate advantage on scan traces across both cycles. Eviction is now genuinely O(1) (tail removal via bubble sort, Cycle 0002). The outstanding problem is the unconditional loop on every `get` — a neighbour-check guard is the prescribed next step. Do not conflate eviction cost (solved) with promotion cost (unsolved).
- **LRU / standard (baseline)**: Remains the latency reference. The swap-to-back variant (Cycle 0002) confirmed that the standard remove+push path is not the latency bottleneck at small bucket sizes, so LRU promotion mechanics are not worth further optimisation.
- **Bucket geometry (32×64)**: Showed hit-rate gains in Cycle 0001 but has not been revisited since. Worth re-evaluating as a complement once the LFU get-path overhead is controlled, rather than in isolation.
