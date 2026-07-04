# Variant v001 Insight

## Helped

- **hit-rate**: Increasing hot-tier allocation to 20% provides additional overflow capacity for frequently re-accessed items, enabling slightly better retention and reducing eviction churn compared to the 18% baseline.

## Hurt

(None measured in this evaluation)

## Takeaway

The more extreme asymmetric partition (20% hot, 5% cold) continues the success of its predecessor (18% hot, 7% cold) with a marginal improvement in hit-rate. However, the minimal gain suggests diminishing returns: the workload may be approaching the optimal allocation boundary within this two-tier stash strategy, where further hot-tier expansion yields negligible benefits.
