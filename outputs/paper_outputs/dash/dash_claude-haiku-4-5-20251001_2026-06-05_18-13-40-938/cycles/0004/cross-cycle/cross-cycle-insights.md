# Cross-Cycle Insights

## Recurring wins

- **Stash-hit detection as acceleration signal**: Cycles 0001–0002 consistently showed that identifying stash-resident items and prioritizing their promotion (deterministically or adaptively) improves hit rate. This is the single most reliable lever tested so far.
- **MRU positioning on demotion paths**: Cycles 0002–0003 converged on back insertion for demoted items, treating re-entry as an opportunity to restore high promotion eligibility. This positioning rule consistently improved hit rate when applied to demotion cascades.
- **Fitness signals gate promotion**: Cycles 0001–0002 validated that stash-hit or recency signals must precede acceleration. Occupancy-threshold-only checks without fitness signals trigger harmful churn (cycle 0002 v003 failure).
- **Deterministic rules outperform conditional branching**: Cycles 0002–0003 showed that consistent, simple positioning rules beat adaptive occupancy-triggered destination selection, even when the conditional logic is semantically reasonable.

## Recurring failures

- **Frequency-based eviction on recency workloads**: Cycle 0001 definitively ruled out LFU; access count is a weaker signal than temporal locality on this traffic pattern.
- **Occupancy-adaptive branching**: Cycles 0002–0003 both showed that gating tier selection or insertion position based on occupancy thresholds adds complexity without hit-rate gain; the signal is noise compared to fitness checks (stash hit, proven promotion history).
- **Combining successful strategies**: Cycle 0004 revealed hard boundaries: stash-hit tier-skip, MRU back insertion, and deterministic positioning all work in isolation but interfere when combined, suggesting the baseline has reached a local optimum within the current rule set.
- **MRU positioning on fresh promotion**: Cycle 0004 v001 showed that the same back-insertion rule beneficial for demoted items destabilizes newly promoted items in higher tiers, revealing an asymmetry in how the mechanism should be applied across promotion vs. demotion paths.

## Open questions

- **Local optimum or headroom?** Cycle 0004 hit rate (0.6839) shows three separate interference patterns; is this a basin floor, or do fundamentally different levers (tier capacity, eviction policy, bucket structure, or asymmetric insertion rules) exist?
- **Why the MRU asymmetry?** Demotion-path back insertion helps; fresh-promotion back insertion hurts. Is the difference eviction cadence, promotion eligibility timing, or something about how the system filters candidates at each tier?
- **Intra-tier vs. cross-tier scope:** Cycle 0004 v002 showed that repositioning within a static tier (stash MRU ordering) yields no gain. Does the MRU principle apply only to tier-advancement transitions, or are there other cross-tier paths where it hasn't been tested?

## Per-policy notes

- **Stash-hit acceleration (promotion policy)**: Both deterministic tier-skip (cycle 0002 v001) and occupancy-adaptive tier-skip (cycle 0002 v002) work; the core mechanism is stash-hit detection, not the adaptation strategy. Occupancy checks are secondary.
- **Demotion policy**: MRU back insertion on demotion consistently improves hit rate (cycle 0003 v003). The rule suggests demoted items retain high re-promotion eligibility, validating that the system should reward survivors rather than penalize them.
- **Eviction and composition boundaries**: Simple, singular strategies (one mechanism per cycle) outperform composites (cycle 0004 all failures). Suggests the workload's hit-rate surface is dominated by a single mechanism at a time, and layering mechanisms creates conflicts that erase gains.
