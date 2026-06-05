# Cross-Cycle Insights

## Recurring wins

- **Demotion recovery with MRU positioning**: Cycle 0003's back-insertion strategy for demoted items became the baseline through cycles 0004–0007, never improving beyond it but remaining stable as the sole working mechanism. This is the only insertion-position strategy that has held across multiple cycles.

- **Stash-hit recognition as a high-confidence signal**: Cycles 0001–0002 both validated that stash-resident items deserve acceleration through the tier hierarchy. When applied narrowly (e.g., occupancy-gated stash-hit promotion in cycle 0002 v002), modest gains persist, though broader tier-skipping proved incompatible with demotion recovery.

- **Fitness-gated promotion over occupancy-only relief**: Cycles 0002, 0005, and 0006 consistently showed that occupancy thresholds without hotness checks suppress performance, while occupancy + stash-hit gating (cycle 0002 v002, cycle 0006 v002) sustained or marginally improved the baseline.

## Recurring failures

- **Composing previously successful strategies**: Cycle 0004 demonstrated that combining stash-acceleration, MRU positioning, and tier-skipping all degraded hit rate despite each working in isolation. Cycles 0005–0007 extended this finding: every attempt to layer new mechanisms onto the 0.684 baseline either stagnated or regressed, confirming the baseline is a balanced equilibrium that destabilizes under additional constraints.

- **Occupancy-only filtering without fitness checks**: v003 variants failed in cycles 0002, 0005, and 0006 when occupancy thresholds were applied without restricting advancement to verified hot items. Gating mechanisms add complexity without unlocking gains.

- **Tier-skip architectures post-demotion-stabilization**: Cycles 0001–0002 showed stash→probing bypass worked in isolation; cycles 0004 and 0006 revealed it floods the probing tier with unvetted items, conflicting with demotion-back-insertion discipline. The incompatibility is structural, not parametric.

- **Frequency-based eviction on recency-heavy workloads**: Cycle 0001's LFU policy yielded no improvement, validating that temporal locality dominates over access count on this traffic pattern.

- **Insertion position tuning on promotion paths**: Cycle 0005 v001 showed Front vs. Back insertion on fresh promotions makes no measurable difference, indicating the asymmetry between demotion recovery and promotion paths runs deeper than landing position.

## Open questions

- **What architectural constraint defends the 0.684 hit-rate ceiling?** Cycles 0005–0007 tested insertion position, candidacy gating, occupancy filtering, eviction priority, cascade structure, and capacity reallocation—all inert or harmful. The bottleneck appears to be how tiers interact (promotion flow, eviction cascade timing, occupancy balance) rather than any single parameter.

- **Is the MRU asymmetry (beneficial for demotion, harmful for fresh promotion) resolvable or fundamental?** Cycle 0004 identified it; cycle 0005 ruled out insertion position as the cause. The constraint appears rooted in tier interaction semantics, not mechanical positioning.

- **Have we reached the true optimum, or does structural testing hold the key?** Capacity showed marginal leverage (+0.0014 in cycle 0006); alternative tier counts, non-LRU eviction orderings, and new promotion qualification criteria remain untested.

- **Why is capacity such a weak lever?** Cycle 0006's bucket rebalancing (14:14 → 18:10 toward probing) yielded only +0.0014, suggesting the workload's access distribution naturally suits the original split.

## Per-policy notes

- **LRU/MRU positioning variants dominate the search space**: Cycles 0001–0007 primarily tested Front (LRU) vs. Back (MRU) insertion within a tier-based framework. Cycles 0003–0005 validated that MRU works only for demoted items, not freshly promoted ones, establishing an asymmetric policy that persists as the working baseline.

- **Occupancy-adaptive policies show narrow utility**: When gated by stash-hit verification (cycles 0002 v002, 0006 v002), occupancy sensing yields modest gains or maintains baseline; without fitness checks (cycles 0002 v003, 0005 v003, 0006 v003, 0007 v003), it suppresses hit rate. Occupancy is a secondary signal, not a primary driver.

- **Tier-skip and demotion-recovery policies are architecturally incompatible**: No occupancy-adaptive variant, threshold tuning, or hybrid approach has reconciled these two successful-in-isolation strategies (cycles 0004, 0006). Tier-skip assumes filtering discipline that back-insertion undermines by restoring demoted items to MRU.

- **Eviction burden allocation is not a hit-rate lever**: Cycle 0007 v001 (target-first eviction priority) maintained baseline without improvement, confirming where evicted items flow does not constrain hit rate.
