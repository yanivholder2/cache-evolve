## Recurring wins

- **Bucket widening as a first-order geometry fix.** Repartitioning capacity into fewer, larger buckets produced the first clear hit-rate gain (cycle 0001). The effect was large initially but saturated quickly — one widening step captured most of the benefit.
- **Cold-first eviction (`counter == 0`) as the dominant scan-pollution defence.** Shielding any item with at least one access from eviction delivered the largest single hit-rate jump across all cycles (cycle 0002). The mechanism has a clear correctness story and remains the backbone of the best-found configuration.
- **Wide-bucket geometry as an enabler for cold-first eviction.** The two mechanisms are synergistic: wider buckets supply enough cold-item candidates so that cold-first selection rarely falls back to plain LRU. Stacking them produced the best absolute hit-rate seen to date.
- **Mid-point insertion as a reliable incremental win.** Landing new items at the bucket midpoint rather than the MRU tail consistently improved hit-rate when added on top of other strategies (cycles 0002–0003), even if the gain was modest.

## Recurring failures

- **O(1) LRU promotion shortcuts.** Swapping an accessed item to the back in O(1) instead of relying on full recency ordering corrupted the eviction gradient and produced a clear hit-rate regression (cycle 0001). This angle has not resurfaced but the lesson generalises: any shortcut that flattens the recency ordering is suspect.
- **Pure geometry pushing beyond the first widening step.** Increasing the bucket factor well past the initial gain (e.g., 4× → 16×) yielded rapidly diminishing returns (cycle 0002). The hot set fits comfortably in moderately wide buckets; further expansion does not materially grow the cold-item pool.
- **Raising the cold threshold above `counter == 0`.** Requiring two or more accesses before shielding an item caused genuine hot-set members to be prematurely evicted on workload-a, where a single access is already a reliable reuse signal (cycle 0003). Threshold-tightening variants should not be revisited for this trace.

## Open questions

- **Per-op latency for cold-first eviction and mid-point insertion is still unmeasured.** Cold-first requires a linear scan across bucket slots; mid-point insertion requires `vec.insert` (O(n)). Neither has been evaluated against a `per-op-time` evaluator. The hit-rate gains may not survive a latency-weighted objective.
- **Are geometry widening and mid-point insertion truly substitutable?** Cycle 0003 showed cold-first eviction working on narrow buckets when mid-point insertion was present, suggesting these two mechanisms solve the same underlying problem — keeping enough cold candidates available. Whether the substitution is exact or breaks down under different workloads is unconfirmed.
- **Cold pool exhaustion under warm-up.** As the cache approaches steady state on a long trace, resident cold items thin out and cold-first eviction increasingly falls back to plain LRU. Whether the hit-rate win degrades significantly over extended traces has not been measured.
- **Workload generalisability.** Every confirmed result comes from the LIRS workload-a trace. It is unknown whether the `counter == 0` boundary, bucket-sizing conclusions, or mid-point substitution hold under different locality profiles (e.g., Zipf-heavy, scan-heavy, mixed).
- **Minimum viable bucket size for mid-point insertion.** There is likely a slot-count floor below which neither mid-point insertion nor cold-first eviction can maintain a sufficient cold-item pool. That floor has not been probed.

## Per-policy notes

- **ClassicLRU:** Strict O(n) recency ordering is load-bearing for this policy — any promotion shortcut that skips intermediate slots degrades eviction quality measurably. The timestamp metadata field is structurally redundant (ClassicLRU never reads it) but removing it has not been evaluated on latency or memory dimensions due to missing evaluators.
- **Cold-first eviction (frequency-gated ClassicLRU variant):** The `counter == 0` boundary is well-calibrated for workload-a and should be treated as settled. Attempts to widen the threshold (counter ≤ 1) regressed; attempts to narrow it have not been tried but are unlikely to help given the already-strong results. The fallback to plain LRU when no cold candidates exist is a correctness safety net but its frequency in steady state is unknown.
- **Mid-point insertion variant:** Functions as both a standalone scan-pollution mitigation and a geometry substitute. Its benefit is weakly additive over cold-first eviction on wide-bucket configurations (insertion site becomes a tiebreaker), but stronger on narrow-bucket configurations where it compensates for fewer cold candidates. The O(n) insertion cost means it should be evaluated on latency before being considered a default.
