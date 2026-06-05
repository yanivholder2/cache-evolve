# Variant v003 Insight

## Hurt

- **hit-rate**: Expanding stash capacity at the expense of normal buckets reduced cache effectiveness, suggesting the baseline allocation is better-suited to this workload's access patterns.

## Takeaway

Increasing stash buckets from 4 to 6 while shrinking normal buckets from 28 to 26 degraded hit-rate, indicating that the workload does not benefit from a larger filtering window at the entry tier. The baseline bucket allocation appears well-tuned; extending the stash does not improve hotness detection enough to compensate for the reduced capacity in the main promotion tiers.
