# Variant v001 Insight

## Helped

- **per-op-time**: The stricter promotion threshold (divisor 2 → threshold 3 vs divisor 3 → threshold 2) reduces the count of items eligible for promotion from stash to probing tier, lowering the per-operation overhead of managing the promotion state machine and tier transitions; both p50 and p99 latencies improve by ~25% on get and ~13-15% on put.

## Hurt

(none)

## Takeaway

Stricter promotion gating recovers latency on the 80/20 geometry without sacrificing hit-rate. The hit-rate plateaus at 0.7166 (matching v002), indicating that the additional items promoted by divisor 3 contribute no incremental cache benefit. By filtering them out with divisor 2, the variant eliminates their overhead while preserving the same coverage, demonstrating that optimal capacity utilization on 80/20 geometry favors selectivity over permissiveness.
