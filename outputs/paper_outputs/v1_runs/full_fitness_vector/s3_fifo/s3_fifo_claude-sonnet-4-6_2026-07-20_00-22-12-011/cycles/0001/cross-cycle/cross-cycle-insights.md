# Cross-Cycle Insights

## Recurring wins

- **Algorithmic complexity reduction on hot paths**: Replacing O(n) scans with O(1) hash-set lookups in the ghost-membership check yielded the single largest latency improvement seen so far; attacking linear scans in eviction-adjacent code is a reliable high-leverage move.
- **Shrinking per-entry memory footprint**: Moving per-entry metadata (frequency counter) out of the entry struct reduced cache-line pressure on scan-heavy eviction loops and delivered latency improvements beyond what the size reduction alone would predict.
- **Rebalancing small-queue fraction downward**: The default 1/4 small-queue split over-invests in admission filtering; reducing it to 1/10 improved both hit rate and latency, suggesting the baseline is conservatively biased toward cold-miss protection at the expense of main-queue efficiency.

## Recurring failures

- *(Only one cycle completed; no pattern of repeated failure has emerged yet.)*

## Open questions

- **Optimal small-queue fraction and workload sensitivity**: Is there a floor below which further shrinking the small queue hurts cold-miss handling, and does that floor shift with workload skew (uniform vs. Zipfian)?
- **Remaining O(n) tail in ghost-hit path**: `ghost.retain()` is still a linear scan executed on every ghost hit; the hot path is not yet fully O(1) — replacing this deque eviction with an O(1) structure is the last known linear-scan hotspot.
- **Composite-variant interactions**: Merging all three cycle-1 wins (queue ratio + O(1) ghost check + slimmer entry) may expose coupling — smaller small queue shifts eviction pressure and changes ghost-query frequency; the combined behavior needs empirical validation.
- **Generality of sizing heuristics**: All results so far are from a single policy (S3-FIFO); it is unknown whether the small-queue fraction finding transfers to other admission-filtering policies.

## Per-policy notes

- **S3-FIFO**: Three orthogonal improvement axes identified in cycle 1 — admission queue sizing, ghost-set data structure, and entry memory layout — all won independently over the baseline. The policy appears to have multiple simultaneously exploitable inefficiencies in its reference implementation. Next priority: a composite variant combining all three, plus O(1) ghost-hit removal.
