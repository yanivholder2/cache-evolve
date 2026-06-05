# Cross-Cycle Insights

## Recurring wins

- **Tier simplification (architecture, not parametrics)**: Cycles 0008–0009 broke the 0.684 plateau for the first time by collapsing the three-tier hot-side to a unified tier, eliminating intermediate gatekeeping. This succeeded where seven cycles of insertion-position tuning, occupancy thresholds, eviction priority, and capacity reallocation all failed. The bottleneck was topology, not parameters.

- **MRU back-insertion for demotion recovery**: Cycles 0003–0009 validate that demoted items—those with prior tier-advancement history—benefit consistently from MRU positioning upon re-insertion. This mechanism persists as the stable foundation across architectural changes, suggesting it reflects a real principle: items earning demotion recovery deserve immediate re-promotion eligibility.

- **Stash capacity is a filtering bottleneck, not a superfluous tier**: Cycle 0010-v002 shows that expanding stash from 4 to 8 buckets improves hit-rate on the baseline three-tier architecture, revealing that the filtering window was under-provisioned. This contradicts the assumption (from cycles 0002–0007) that stash allocation was balanced and reopens capacity allocation as a primary lever alongside tier topology.

- **Gated tier-skip reconciles acceleration with recovery discipline**: Cycle 0008-v003 shows that conditional tier-skip—gated to proven stash residents with recovery flags enforcing target re-entry on eviction—works, directly refuting cycles 0004 and 0006's findings that tier-skip was "fundamentally incompatible" with demotion recovery. The key is automatic recovery structure, not elimination of the skip mechanism.

## Recurring failures

- **Occupancy-based thresholds as standalone levers**: Cycles 0002, 0005, 0006, 0007, 0009 all tested occupancy-gated promotion, occupancy-aware destination selection, and occupancy-triggered demotion. None improved hit-rate; many suppressed it. Occupancy is not a fitness signal without a deeper hotness check (e.g., stash-resident status).

- **Frequency-based eviction and LFU selection**: Cycle 0001 shows LFU underperforms on recency-dominated workloads; cycle 0010 confirms eviction ordering (LRU vs. FIFO) is orthogonal to hit-rate. The bottleneck is upstream in tier structure and promotion candidacy, not downstream in eviction selection criterion.

- **Stacking previously successful mechanisms without structural alignment**: Cycle 0004 demonstrates that strategies succeeding in isolation (MRU back-insertion for fresh promotions, tier-skip to probing) interfere when combined. The failure exposed an asymmetry: MRU helps demoted items but harms fresh promotion. Simple, singular strategies outperform composite approaches on this workload.

- **Occupancy-driven relief without fitness checks**: Cycles 0002-v003, 0005-v003, 0007 show that promoting items purely because stash occupancy exceeds a threshold (without confirming they're hot) causes churn rather than relief. Occupancy thresholds must gate a fitness signal; occupancy alone is insufficient.

- **Bucket-level saturation as an eviction trigger**: Cycle 0010-v003 shows that per-bucket saturation detection triggers premature eviction, shrinking the effective working set before items' utility windows naturally close. Saturation monitoring adds complexity without discovering latent cache utility.

## Open questions

- **Two-tier vs. three-tier with expanded stash**: Cycle 0010-v002 (8-bucket stash on three-tier) improved hit-rate, while cycles 0008–0009 achieved breakthrough via two-tier collapse. Are these equivalent paths, or does one outrank the other? If equivalent, which is more maintainable?

- **What constrains the ceiling on a given architecture?** Cycles 0002–0007's 0.684 plateau was defended across insertion position, occupancy, eviction priority, cascade structure, and capacity tuning. What is the precise architectural factor: tier occupancy balance, eviction cadence, bucket saturation dynamics, or a feedback loop between promotion and demotion?

- **Can structural and parametric improvements compound?** The two-tier unified architecture improved hit-rate (cycles 0008–0009), but cycle 0010's stash expansion suggests the three-tier baseline had untapped capacity leverage. Can the two-tier structure be enhanced with the stash-capacity discovery?

- **Are there orthogonal architectural levers still untested?** Alternative tier counts, bucket saturation thresholds, non-LRU eviction orderings within tiers, or stash-depth variations remain unexplored. Do they unlock headroom, or does the workload's access distribution naturally suit the discovered architectures?

- **Why does the same MRU mechanism benefit demotion but harm fresh promotion?** Cycle 0004 raises this as a core asymmetry. Understanding this mechanical difference clarifies whether the system has found a balanced equilibrium or if the asymmetry reveals further optimization opportunity.

## Per-policy notes

- **Insertion position and MRU policy**: Back insertion is a universal win for demoted items (proven survivors benefiting from immediate re-promotion eligibility), but remains inert or harmful for freshly promoted items in higher tiers. The asymmetry suggests demoted items earn their MRU status by surviving tier discipline, while freshly promoted items must earn residency at higher tiers. Front insertion (LRU) for fresh promotion and back insertion for demotion recovery represents a stable equilibrium across cycles 0003–0009.

- **Occupancy-based routing and gating**: Nine cycles show occupancy thresholds do not improve hit-rate in isolation. Occupancy-coordinated destination selection, occupancy-gated promotion, and occupancy-triggered cascades all failed or degraded performance. Occupancy awareness must gate a prior fitness signal (e.g., "promote stash hits when stash > 75%"), but occupancy alone is not a constraint. Once architectures are simplified (two-tier, cycle 0009), even occupancy-gated conditional routing adds complexity without gain.

- **Tier-skip with and without recovery structure**: Unconditional tier-skip (stash-to-probing bypass) floods the hot tier with unvetted items, conflicting with demotion-back-insertion discipline (cycles 0004, 0006). Conditional tier-skip gated to proven residents with automatic recovery enforcement (skip_history_flag forcing target re-entry on eviction) works, reconciling acceleration with filtering discipline (cycle 0008-v003). The difference is structural: recovery-enforced skip prevents unvetted hot-tier residents.

- **Tier architecture and capacity allocation**: The three-tier baseline (4-bucket stash, 14-bucket target, 14-bucket probing) plateaued at 0.684 across cycles 0002–0007 despite orthogonal parametric tuning. Cycles 0008–0009's breakthrough came from two-tier collapse (eliminating intermediate gatekeeping). Cycle 0010-v002's stash expansion (4→8 buckets) on the three-tier also improved hit-rate, suggesting capacity allocation to the filtering tier was a bottleneck parallel to tier topology. The optimal architecture may involve both simpler topology and better capacity distribution.

- **Eviction policy (LRU/FIFO ordering, priority, cascade structure)**: Eviction selection criterion (LRU vs. FIFO) is orthogonal; ordering does not constrain hit-rate (cycle 0010-v001). Eviction priority rebalancing (target-first, cycle 0007-v001) yields flat performance. Cascade structure is well-balanced: removing the target-tier retry step for foreign residents degrades hit-rate by ~3% (cycle 0007-v002), confirming the two-stage safety valve is load-bearing. Proactive eviction at bucket saturation triggers prematurely shrinks working set (cycle 0010-v003). Eviction mechanics are not primary levers; architectural structure and tier capacity are.
