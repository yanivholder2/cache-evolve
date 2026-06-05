# Variant v002 Insight

## Helped

- hit-rate: Expanding normal buckets from 16 to 24 (75/25 split) while shrinking stash to 8 continued to move more of workload-a's working set onto the hot-tier probing path, building on v003's 50/50 gain and confirming the gain curve had not yet peaked at 16 normal buckets.

## Hurt

## Takeaway

Pushing normal/stash from 16/16 to 24/8 yielded a further lift in hit-rate on workload-a, meaning the working set was not yet saturated in the hot tier at v003's 50/50 boundary. The stash can tolerate continued reduction to at least 8 slots without a penalty, which suggests the working set fits comfortably within 24 normal buckets and that prior over-allocation to stash was still constraining hot-tier reuse even after the 50/50 rebalance.
