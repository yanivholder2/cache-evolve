# Variant v002 Insight

## Helped

- **per-op-time (get)**: The strict freq > 2 promotion threshold reduces main queue population, enabling faster lookups across both p50 and p99 latencies.
- **per-op-time (put)**: Fewer items reaching main queue through the restrictive admission filter reduces insertion cost and contention during both p50 and p99 operations.

## Hurt

- **hit-rate**: The compound effect of strict freq > 2 threshold combined with ghost hits restarting at freq = 0 in the small queue creates an over-restrictive barrier; items require at least 3 accesses to graduate to main, causing significant eviction before re-establishing residency.

## Takeaway

Combining strict promotion (freq > 2) with ghost re-routing through the small queue achieves substantial latency gains but destroys hit-rate by making advancement to main queue prohibitively difficult. The hypothesis that "tight threshold prevents marginal re-admitted items from flooding main" proves true for latency, but the cost is rejecting too many warm ghosts before they can prove hotness. The two mechanisms—strict admission filtering and forcing ghost hits through a warmup path—work against each other rather than synergistically.
