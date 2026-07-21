## Recurring wins

- **O(n) → O(1) algorithmic substitutions deliver the largest single-cycle gains**: replacing the ghost membership scan with a HashSet (cycle 1→2) and replacing the `position()` deque scan with a `resident_map` side-lookup (cycle 4→5) each produced step-change improvements that dwarfed all incremental tuning in adjacent cycles.
- **Lazy / amortized cleanup outperforms eager per-hit work**: tombstoning ghost entries on hit and deferring deque cleanup to the eviction capacity guard cut put tail latency by 5× in cycle 2 and remained the dominant gain in the composite baseline; the amortized model has never regressed.
- **Structural consolidation beats parallel structures**: merging value and frequency into a single `resident_map<K,(V,u8)>` (cycle 5/v003) achieved fewer hash probes, smaller per-entry footprint, and lower write overhead than any split-map alternative tested in cycle 6 — confirmed optimal.
- **Orthogonal changes stack cleanly**: queue sizing, ghost-set algorithm, and entry layout all touch disjoint code paths; every time independent wins were composed they combined without interaction penalty.
- **Small-queue shrinkage from the default 1/4 was monotonically beneficial through 1/20**: cycles 1–3 each confirmed a free gain from halving the fraction; the optimal resting point on Zipfian workloads is 1/20.

## Recurring failures

- **Bounded second-chance scan cap (SCAN_CAP)**: proposed after cycle 3's two-random-choices result, falsified in cycle 4 — Zipfian workloads front-load cold entries at the tail of main, so the unbounded scan terminates in the first few iterations anyway; a fixed cap only forces premature eviction of still-hot entries.
- **Lowering the ghost promotion threshold (freq > 0)**: tested in cycle 3, backfired — the `freq > 1` gate enforces demand confirmation (re-request *after* ghost-listing), not just frequency; relaxing it floods main with once-warm entries lacking recency evidence. Rule is load-bearing; do not revisit.
- **Frequency step acceleration (step=2 decrement)**: tested in cycle 4 — decrementing faster caused warm entries to exit main prematurely, raising the ghost-hit rate and total eviction work even as individual scan lengths shortened; scan-length reduction and total-work reduction are decoupled and opposed here.
- **Small-queue shrinkage below 1/20**: 1/40 showed a latency crossover in cycle 4 (pre-scan-elimination) and did not clearly recover in cycle 6; 1/80 produced a significant latency regression from `evict_small()` overhead. The floor is confirmed near 1/20.
- **Ghost capacity beyond `main_size`**: doubling the ghost window (cycle 6) accumulated tombstones without recapturing cold-tail re-requests on Zipfian workloads — the cold tail simply does not re-request within an extended window.
- **Splitting `resident_map` into parallel hashmaps**: cycle 6 confirmed that read-path cache-line benefits are offset by doubled write overhead on every admission and eviction.

## Open questions

- **What is the current dominant per-operation cost?** With O(n) scans and layout inefficiencies eliminated, the bottleneck is uncharacterized; candidates are hash-probe collision behavior at high load factors, `ghost_set` HashSet probe overhead on every `put()`, and eviction-deque memory-access pattern under frequency churn.
- **Fitness-function false positives**: variants with net-negative fitness profiles have appeared in the Winners list in cycles 3 and 6 — the evaluation function appears to weight a latency dimension or aggregate score that allows hit-rate regressions to be offset; this anomaly is unresolved and may be distorting hypothesis selection.
- **Tombstone backlog under dense ghost-hit workloads**: raised in cycle 2 and never profiled — how large does the ghost deque's tombstone accumulation grow before the capacity guard drains it, and does it become significant under non-Zipfian access patterns?
- **New structural axes**: all known levers (queue sizing, ghost capacity, scan elimination, map layout) are at or past their optima on Zipfian workloads; the next gains likely require a qualitatively different hypothesis — e.g., workload-adaptive sizing, clock-hand eviction order, or tiered ghost channels.

## Per-policy notes

- **S3-FIFO ghost channel**: the `freq > 1` promotion threshold is non-negotiable — it is a dual gate enforcing both frequency quality and demand confirmation. The paper's default is empirically validated here and must not be lowered.
- **S3-FIFO small queue**: optimal fraction for Zipfian workloads is 1/20; the floor (latency regression from `evict_small()` churn) appears near 1/40–1/80. Default 1/4 was significantly over-provisioned.
- **S3-FIFO `evict_main()` second-chance loop**: theoretical O(n) complexity is not realized on Zipfian workloads — cold entries accumulate at the deque tail naturally, so practical scan depth is low. Loop complexity is not the bottleneck; deque memory-access churn was, and lazy tombstoning addressed it.
- **S3-FIFO data layout**: `resident_map<K,(V,u8)>` is the confirmed optimal structure — O(1) hit detection, frequency update, value retrieval, and miss short-circuit in a single hash probe, with no separate freq deque or entry value field required.
- **Ghost lazy deletion**: tombstoning on hit + capacity-guard drain is strictly better than `retain()` on Zipfian workloads; the amortized cost model has remained sound across four cycles of composition.
