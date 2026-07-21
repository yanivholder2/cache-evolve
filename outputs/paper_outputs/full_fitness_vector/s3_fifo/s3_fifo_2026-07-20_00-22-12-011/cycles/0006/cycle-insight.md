# Cycle 0006 Insight

## Cohort outcome

All three variants explored sizing and layout levers that cycle 0005 left open: pushing the small-queue fraction below 1/40, widening the ghost queue beyond `main_size`, and decomposing the unified `resident_map` into parallel hashmaps. None found a positive outcome — every variant registered a regression on at least one primary dimension — making this a clean falsification cycle. All three variants nonetheless appear in the Winners list, continuing the anomaly first observed in cycle 0003.

## Patterns

- **The small-queue floor has been reached at 1/40**: Tightening to 1/80 produced a marginal hit-rate uptick but a significant latency regression from more frequent `evict_small()` calls — the overhead of faster small-queue turnover now outweighs the additional main-queue residency benefit. The crossover that cycle 0004 attributed to `position()` scan length has a separate instance at this fraction boundary, independent of the scan.
- **Ghost capacity beyond `main_size` is load-bearing only when the cold tail has recency**: Doubling the ghost window accumulated useless tombstones rather than recapturing evicted keys, hurting both hit-rate and put latency via the lazy-deletion drain loop. The Zipfian cold tail simply does not re-request within the extended window; the original `main_size` bound is effectively tight.
- **The unified `resident_map` layout is structurally optimal for this workload**: Splitting it into `data<K,V>` and `freq_map<K,u8>` to improve read-path cache-line density was offset by doubled write overhead on every admission and eviction. The cycle 0005 v003 consolidation is confirmed as the right data-structure choice, not a candidate for decomposition.
- **All three known optimization levers are now exhausted or at their floor**: scan elimination (cycle 0005), queue fraction (1/40 confirmed optimal here), ghost sizing (confirmed optimal at `main_size`), and map layout (consolidated `resident_map` confirmed optimal). No incremental improvement was found on any of them.

## Open questions

- What is the current dominant per-operation cost now that all the previously identified bottlenecks have been resolved? Hash-probe collision behavior, eviction deque memory layout, and ghost-set `HashSet` membership tests are candidates but have not been profiled.
- The Winners list again includes variants with net-negative fitness profiles — is the fitness function weighting a dimension (e.g., a specific latency percentile or aggregate score) that these variants happened to pass, and should that weighting be revisited?

## Continuation

The three axes that have driven all gains since cycle 0001 — small-queue fraction, ghost capacity, and per-operation data structure layout — are now at or past their practical optima on this Zipfian workload. The next cycle must pivot to identifying a genuinely new bottleneck rather than probing exhausted levers. The most promising approach is to instrument or reason about what the current dominant per-operation cost is: candidates include the `evict_main()` deque memory-access pattern under frequency churn, `ghost_set` HashSet probe overhead on every `put()`, and hash-collision behavior at high load factors. The baseline for all future cycles should remain cycle 0005/v003's consolidated `resident_map` with a 1/20 small-queue fraction; do not revisit 1/40 or smaller fractions, the `2 × main_size` ghost bound, or the split-map layout — all three have been conclusively ruled out.
