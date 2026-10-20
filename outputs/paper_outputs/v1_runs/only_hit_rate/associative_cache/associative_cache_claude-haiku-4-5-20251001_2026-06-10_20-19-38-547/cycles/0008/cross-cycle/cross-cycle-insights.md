# Cross-Cycle Insights

## Recurring wins

- **Stash buffer recovery mechanism**: Foundational across all 8 cycles. Evicted items exhibit temporal re-use; capturing them in a bounded buffer provides net hit-rate gain. The mechanism itself is load-bearing—simplification (Cycle 0008 v003) regresses.

- **Capacity scale to 25% emerges as stable optimum**: Cycles 0001–0002 established that larger stash buffers capture more working-set re-use. Capacity scaling from 10% to 25% yielded consistent gains; further scaling blocked by architectural constraints (Cycles 0003, 0005), and partition-tuning refinements (Cycles 0007–0008) did not exceed this baseline. Raw buffer size, not sophistication, is the binding constraint within the stash.

- **Two-tier organization with immediate promotion-on-hit**: Cycles 0006–0008 show that organizing captured items into hot (frequently re-accessed) and cold tiers, with immediate promotion on any re-access, outperforms filtering logic or staged escalation. The mechanism learns from workload re-access patterns without adding architectural overhead or requiring predictive logic.

- **Asymmetric partition allocation to match workload distribution**: Cycles 0007–0008 validate that tier proportionality should reflect observed re-access skew. Shifting capacity from cold (12.5%) to hot (18%→20%) captures the workload's concentrated re-access patterns. Static asymmetry avoids adaptive policy overhead while exploiting stable workload characteristics.

- **Geometric collision reduction (0.71 factor) compounds with stash**: Cycles 0004–0005 demonstrate that geometry and stash are orthogonal levers. When stash tuning alone saturates (Cycle 0005), wider buckets (0.71 factor) break the plateau and unlock fresh gains. Geometry reduces collision-driven eviction load; stash recovers what slips through. Both are necessary.

- **Unconditional capture and eager promotion outperform filtering**: Consistent across all cycles (0002–0008): occupancy-aware, collision-aware, frequency-gated, and access-count-based filtering all reduce or provide no hit-rate improvement. The workload does not benefit from selective insertion; it rewards blanket FIFO capture and rapid hot-tier escalation on any re-access.

## Recurring failures

- **Filtering at insertion or promotion**: Cycles 0002–0003 (occupancy-aware), 0005 (collision-aware), 0006–0008 (access-count and staged gating) consistently show filtering delays or prevents recovery of genuinely useful items. The workload's evicted population does not stratify into "junk" and "valuable" by collision frequency, occupancy, or access count; filtering is a false economy.

- **Staged or gated promotion**: Cycles 0006 (frequency-gated) and 0008 (dual-access requirement) show that delaying hot-tier promotion trades hit-rate for reduced churn—the wrong trade-off. Items demonstrating a single re-access should immediately reach hot tier; gatekeeping eliminates valuable single-re-access candidates.

- **Counter manipulation for phase response**: Cycles 0006–0007 establish that phase detection (identifying workload transitions via collision-distribution divergence) is reliable, but responding via counter adjustment (reset or boost) does not translate to performance gains. Phase detection is diagnostic; phase response remains mechanically disconnected.

- **Fine-grained geometric tuning beyond the 0.71 peak**: Cycles 0005–0006 show that pushing geometric aggressiveness to 0.75 or refining downward both regress. The collision-reduction optimum is a sharp boundary, not a tuning zone; geometry gains have saturated.

- **Inter-bucket redistribution and dynamic placement**: Cycle 0001's v002 failed because the cache's slot_legality invariant prohibits bucket migration. Architectural constraints make dynamic inter-bucket item movement infeasible regardless of sophistication.

## Open questions

- **Partition saturation and fine-tuning**: Is the 20/5 hot/cold split (Cycle 0008) the local optimum, or do fractional increments (19%, 21% hot allocation) yield marginal gains? Cycles 0007–0008 suggest diminishing returns but a sharp boundary has not been confirmed.

- **Phase-aware dynamic response mechanisms**: Phase detection is proven correct (Cycle 0006 identified transitions reliably). Can phase transitions trigger effective *dynamic responses*—e.g., partition rebalancing, geometry switching, or stash-capacity reallocation—rather than counter manipulation? This remains the primary untested escape route.

- **Architectural constraint resolution**: Borrow checker errors prevented stash capacity exploration beyond 25% in Cycles 0003 and 0005. Can refactoring enable testing of 15%, 20%, 30% saturation boundaries, or does the current locking model impose a hard limit?

- **Alternative stash organization strategies**: Beyond two-tier asymmetric partitioning, are frequency-based aging, time-windowed recency, multi-level tiering (without gatekeeping), or predictive prefetching strategies viable approaches that exploit workload structure?

- **Workload phase coupling mechanism**: Cycles 0006–0008 show phase detection works but response does not. What is the right coupling between phase transitions and stash/geometry policy? Dynamic partition rebalancing remains untested; adaptive geometry during phase transitions is unexplored.

## Per-policy notes

- **FIFO ordering with two-tier promotion-on-hit** (Cycles 0006–0008): The winning strategy pairs unconditional FIFO insertion at 25% capacity with immediate cold→hot promotion on any re-access. This avoids filtering overhead, respects all re-use patterns equally, and matches asymmetric allocation (20%/5% current) to observed skew.

- **LRU ordering within stash** (Cycles 0002, 0004): Tested twice; underperformed both FIFO baseline and geometric improvements. Sophisticated per-item ordering does not compound with capacity or geometric gains; simple promotion-on-hit is sufficient.

- **Filtering mechanisms tested across multiple policies** (Cycles 0002–0008): Occupancy-aware insertion filtering, collision-aware selective capture, frequency-gated promotion, and access-count thresholds all fail to improve hit-rate. No filtering policy (regardless of trigger signal) provides net benefit.

- **Static asymmetric allocation vs. adaptive partition sizing**: Cycles 0007–0008 chose static 18/7 → 20/5 asymmetry, avoiding adaptive partition-rebalancing logic. This works because workload re-access distribution appears stable within the test window; dynamic rebalancing on phase transitions remains unexplored as an orthogonal axis.
