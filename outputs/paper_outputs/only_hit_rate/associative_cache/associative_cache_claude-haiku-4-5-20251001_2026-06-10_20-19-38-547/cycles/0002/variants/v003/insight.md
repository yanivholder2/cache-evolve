# Variant v003 Insight

## Helped

- **hit-rate**: The selective insertion mechanism filters stash additions to only items evicted from buckets at ≥75% occupancy, avoiding pollution from low-value evictions in lightly-loaded buckets. This keeps the stash focused on items lost to actual congestion, improving the likelihood of cache recovery on subsequent accesses.

## Hurt

(No measured dimensions showed degradation.)

## Takeaway

By gating stash insertion on post-eviction bucket occupancy, this variant demonstrates that reducing stash churn through occupancy-aware filtering improves hit rate. Items evicted from congested buckets are worth preserving; those from under-utilized buckets are not, and filtering the latter out keeps the bounded stash effective.
