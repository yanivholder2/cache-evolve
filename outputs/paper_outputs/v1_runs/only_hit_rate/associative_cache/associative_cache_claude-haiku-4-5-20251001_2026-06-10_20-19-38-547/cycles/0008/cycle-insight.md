# Cycle 0008 Insight

## Cohort outcome

This cycle tested three directions from the Cycle 0007 asymmetric-partition baseline: further partition tuning (20% hot / 5% cold), staged multi-tier promotion with frequency gating, and simplification via unified flat stash. Only v001 succeeded with marginal hit-rate gains, confirming the two-tier asymmetric design is resilient but approaching saturation in partition space.

## Patterns

- **Partition tuning is nearing saturation**: v001's minimal improvement from the 18/7 baseline to 20/5 suggests the workload's re-access distribution is already well-matched. Further hot-tier expansion yields diminishing returns, indicating the design may be approaching an allocation boundary within this two-tier strategy.

- **Two-tier promotion-on-hit is a load-bearing mechanism**: v003's regression (consolidating to a single unified 25% stash) and v002's regression (staged cold→warm→hot promotion) both confirm that the two-tier structure is not organizational overhead. The mechanism provides the feedback signal identifying which evicted items deserve premium hot placement; removing it sacrifices this critical learning capability.

- **Gatekeeping and staged promotion filter out valuable items**: v002's failure when requiring two re-accesses for hot-tier promotion replicates the cross-cycle pattern: filtering logic—even frequency-based—delays or prevents promotion of genuinely useful single-re-access items. The workload rewards rapid escalation over observation; items demonstrating one re-access should immediately reach hot tier.

- **The design convergence is tightening**: Cycles 0006–0008 show that stash *organization* (two-tier promotion-on-hit) outperforms filtering, gatekeeping, or simplification. The 20/5 asymmetric partition with immediate promotion represents a mature equilibrium where geometric collision reduction (0.71 factor), uniform capacity provision (25% total), and adaptive organization (two-tier with asymmetric allocation) are locked in place.

## Open questions

- Has partition tuning been exhausted at 20/5, or do fractional refinements (19%, 21%, etc.) yield further gains within a narrower range?
- Can phase detection finally be coupled with effective *dynamic response*—e.g., rebalancing partition ratios during workload transitions—rather than counter manipulation?
- Are there organizational strategies orthogonal to asymmetric partitioning that exploit workload structure—e.g., frequency-based aging, time-windowed recency, multi-level tiering without staged gating?

## Continuation

v001's marginal gain confirms partition tuning is subject to diminishing returns; v002 and v003 demonstrate that multi-tier staged promotion and unified simplification are counterproductive. The two-tier asymmetric-partition design (now at 20/5) is proven and stable. Next cycle should pursue one of two paths: (1) **Fine-grained partition refinement**: test fractional adjustments (19%, 21%, etc. hot-tier allocation) to verify whether the local optimum is sharp or whether marginal gains remain, or (2) **Escape via alternative organization**: shift focus from partition tuning to orthogonal mechanisms (frequency-based aging, multi-level hierarchies without gatekeeping, or phase-aware dynamic rebalancing) that adapt the stash beyond static allocation. If partition refinement yields only noise-level gains, accept convergence at 20/5 and prioritize phase-aware dynamic response coupling as the next escape route.
