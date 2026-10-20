## Recurring wins

- **O(n) → O(1) data structure upgrades pay off immediately and durably**: replacing the ghost membership scan with a HashSet (cycle 1), then eliminating `ghost.retain()` with lazy tombstoning (cycle 2), each produced the largest single latency gain in their respective cycles. The pattern — find the dominant linear scan, replace with a constant-time structure — has never failed to win.
- **Small-queue fraction reduction is a consistent, risk-free lever on Zipfian workloads**: four successive halvings (1/4 → 1/10 → 1/20) each improved hit-rate and read latency with no observed downside until the 1/20 → 1/40 step in cycle 4, where the main-queue `position()` cost began to cancel the gain. The validated optimum is currently 1/20.
- **Orthogonal changes compose cleanly**: across cycles 1–2, queue sizing, ghost-check complexity, entry layout, and lazy deletion all touched disjoint code paths and stacked without interaction penalty. Composite variants uniformly matched or exceeded the sum of isolated gains.
- **Amortizing cleanup work across future operations**: deferring ghost-deque compaction to the capacity guard (lazy tombstoning) was more effective than eager cleanup, and the pattern is likely generalizable to other deferred-maintenance opportunities in the data structure.
- **Per-entry struct compaction yields unexpected latency gains**: moving frequency counters out of the entry struct (cycle 1) reduced cache-line pressure during eviction scans — size reductions that look like space optimizations also affect scan-path performance.

## Recurring failures

- **Bounded second-chance scan cap in `evict_main()`**: proposed in cycle 3, falsified in cycle 4. On Zipfian workloads, cold entries naturally accumulate at the tail of main, so the unbounded scan already terminates quickly; a fixed budget adds no churn savings and forces premature eviction of still-hot entries beyond the cap.
- **Lowering the ghost promotion threshold (freq > 0)**: tested in cycle 3. Relaxing from `freq > 1` floods main with once-warm entries that lack recency confirmation, producing a material hit-rate regression with no compensating benefit. The paper's threshold is a quality gate, not a conservative default.
- **Frequency step acceleration (decrement > 1)**: tested in cycle 4. Draining frequency faster causes moderately-warm entries to exit main early, increasing ghost-hit-driven `evict_main()` call rates. Scan-length reduction and total-eviction-work reduction are decoupled and can oppose each other — the net effect was higher latency.
- **O(1) random victim selection (two-random-choices)**: tested in cycle 3. Eliminated deque-churn latency but discarded the frequency-drain protection that keeps hot entries in main, causing a material hit-rate regression. Complexity class and eviction policy cannot be separated here.

## Open questions

- **Can the `position()` linear scan in `get()` and `put()` be eliminated?** Two independent variants in cycle 4 identified this as the dominant remaining latency source. O(1) alternatives — intrusive node pointers, a side hashmap into the deque, or an index structure — are unprobed and are now the top hypothesis priority.
- **Is there a lower bound on the safe small-queue fraction?** The 1/20 → 1/40 step entered diminishing-returns territory driven by `position()` cost. Eliminating `position()` may reopen further shrinkage; or 1/20 may be close to the Zipfian optimum regardless.
- **Tombstone backlog under dense ghost-hit workloads**: lazy tombstoning can allow transient deque bloat between capacity-guard sweeps. The backlog size distribution under adversarial or ghost-heavy access patterns has never been characterized.
- **Fitness function sensitivity**: cycle 3's v003 (hit-rate regression, no benefits) was still classified as a Winner. Whether the scoring system systematically overweights latency or uses a threshold that allows hit-rate regressions warrants investigation before fitness-driven selection is trusted for ambiguous results.

## Per-policy notes

- **S3-FIFO — small queue**: The paper's 1/4 default is substantially over-provisioned for Zipfian workloads. The empirically validated optimum across cycles 1–3 is 1/20; 1/40 shows continued hit-rate benefit but at the cost of worsened `position()` scan latency that now offsets the gain.
- **S3-FIFO — ghost queue**: The `freq > 1` promotion threshold is load-bearing and must not be relaxed. Lazy tombstoning (HashSet removal on ghost hit, deque compaction deferred to capacity guard) is the correct deletion strategy. Tombstone backlog behavior under dense ghost-hit workloads remains uncharacterized.
- **S3-FIFO — main queue**: The second-chance eviction loop terminates quickly on Zipfian inputs due to natural cold-entry accumulation at the deque tail — `evict_main()` scan length is not the bottleneck. The `position()` key-lookup scan in `get()` and `put()` is now the primary per-operation latency cost and the sole target worth pursuing next.
