# Cross-Cycle Insights

## Recurring wins

- **Stash-based recovery is the primary productivity lever**: All six cycles validate that capturing evicted items in a bounded buffer and providing second-chance lookup outperforms broader architectural changes. The mechanism evolved from basic FIFO insertion (Cycle 0001) through capacity scaling (Cycle 0002) to organizational refinement (Cycle 0006), but remained central to every improvement.

- **Capacity scales faster than optimization within capacity**: Doubling stash size (Cycle 0002) decisively beat LRU ordering and filtering refinements. Later, stash *organization* (two-tier promotion-on-hit in Cycle 0006) outperformed *filtering* (what to reject), revealing that spare capacity is better utilized through learned prioritization than eliminated through selective insertion.

- **Complementary levers restore momentum after single-axis saturation**: Stash-only tuning plateaued by Cycle 0003. Pivoting to geometric rebalancing (Cycle 0004) unlocked fresh gains. When geometric search converged (Cycle 0006), shifting to stash-internal organization broke through again. The pattern suggests no single lever (capacity, geometry, filtering, ordering) dominates; sustained progress requires trading between orthogonal axes.

- **Workload re-use patterns favor post-eviction recovery**: Across all cycles, the fundamental finding is that evicted items exhibit temporal locality—they are re-accessed at rates high enough to justify capturing them rather than discarding immediately. This validates the stash direction over preventive approaches (like filtering low-value captures).

## Recurring failures

- **Filtering and selective insertion consistently underperform unconditional capture**: Occupancy-aware filtering (Cycle 0003), collision-aware thresholds (Cycle 0005), and phase-aware strategies (Cycle 0006) all failed or regressed. The stash's FIFO unconditional-insertion baseline captures all meaningful re-use; filtering logic either discards valuable items or adds overhead without sufficient discriminative power.

- **LRU and sophisticated eviction orderings within the stash trail simple FIFO**: Introduced in Cycle 0002, tested again in Cycle 0004, LRU ordering consistently underperformed the baseline. This suggests either the stash capacity is insufficient to show ordering benefits, or FIFO's simplicity avoids overhead costs that more complex policies incur.

- **Architectural constraints (borrow checker) block capacity exploration**: Three times across five cycles (Cycles 0003, 0005, and attempted in 0006), stash capacity scaling beyond 25% failed at the build gate. This constraint prevents direct measurement of saturation boundaries and capacity-geometry trade-offs, leaving a key design frontier unmapped.

- **Geometric refinement has sharp, narrow peaks with limited depth**: Fine-grained tuning of bucket width (Cycles 0005, 0006) revealed that collision-reduction gains saturate quickly. The 0.71 reduction factor from Cycle 0004 appears to be a hard boundary rather than a tuning zone; pushing to 0.75 (Cycle 0005) or 0.70 (Cycle 0006) both regressed, suggesting the geometric design space is highly constrained.

- **Phase detection without an effective response mechanism adds no value**: Cycle 0006's v003 correctly diagnosed workload phase transitions but pairing detection with counter resets yielded no improvement. Detection-only provides diagnostic clarity but no performance gain until coupled with a productive policy change.

## Open questions

- **What is the stash capacity saturation point?** Cycles 0002 and 0003 suggested scaling from 10% → 25% was productive, but borrow checker failures prevented testing 20% and 30% systematically. Is saturation at 25%, beyond it, or has the architectural constraint obscured a higher optimum?

- **Are there alternative stash-organization strategies beyond promotion-on-hit?** Cycle 0006's two-tier success suggests the stash has unused potential once filtered-based approaches are abandoned. Could frequency-based tiering, multi-level hierarchies, or predictive re-use indicators yield larger gains?

- **How should phase-aware detection be coupled with effective response?** Cycle 0006 identified that phase transitions occur, but counter resets alone don't help. Would dynamic tier sizing, geometric rebalancing, or stash capacity rebalancing during transitions unlock the signal's value?

- **Can the borrow checker constraint be resolved to enable systematic capacity exploration?** The recurring build failures suggest either a fundamental architectural limit in the locking model or a solvable refactoring opportunity. Resolving this would clarify whether capacity remains a productive axis beyond 25%.

- **Has the workload-cache pairing reached its exploitable margin?** After six cycles of incremental gains (0.26–1.2% per cycle at best), all single-axis refinements show diminishing returns. Is the remaining headroom in workload adaptation (e.g., phase-driven policy switching), alternative eviction semantics (beyond FIFO), or fundamentally different cache-layout strategies?

## Per-policy notes

- **FIFO insertion into the stash** (baseline and dominant): Unconditional FIFO capture at the established 25% capacity proved optimal across all cycles. No filtering signal—occupancy-based, collision-based, or phase-based—improved selectivity. The simplicity and overhead-avoidance of FIFO align with the bounded recovery model; all productive gains came from *organizing* what FIFO captured (two-tier promotion in Cycle 0006) rather than *filtering* what FIFO would insert.

- **LRU-ordered eviction within stash**: Tested twice (Cycles 0002, 0004) as a refinement to replace FIFO ordering. Consistently underperformed the baseline, suggesting either the stash capacity is insufficient to show recency benefits or FIFO's simplicity avoids overhead costs that LRU incurs. Not revisited after Cycle 0004; the signal is clear.

- **Geometric collision reduction (bucket-width adjustment)**: Emerged as a complementary lever in Cycle 0004. The 0.71 reduction factor represents a local optimum with narrow margins (0.75 and 0.70 both regressed). Unlike filtering or ordering, geometric changes compound with stash capacity gains, but exploration space is highly constrained. No policy variations beyond bucket-width tuning were tested; alternative collision-reduction strategies (e.g., cuckoo-style relocation, bucket splitting) remain unexplored.

- **Two-tier stash with promotion-on-hit** (Cycle 0006): Breakthrough winner that revealed stash *organization* as more productive than *filtering*. Partitioning the 25% capacity into hot (promoted on re-access) and cold tiers achieved measurable gains without architectural constraints. Partition ratios and aging policies within tiers remain unexplored; this direction offers depth for refinement.
