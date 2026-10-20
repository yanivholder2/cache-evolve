# Variant v001 Insight

## Helped

- **hit-rate**: Skewing the two-tier stash partition toward hot (18% vs 12.5%) creates more room for frequently re-accessed items, reducing their eviction pressure and enabling more successful secondary lookups.

## Hurt

(None measured in this evaluation)

## Takeaway

The asymmetric partition (18% hot, 7% cold) outperforms the symmetric split by allocating proportionally more capacity to items proven to have repeated utility. This refinement reflects a key insight: the cold tier's role is to capture the *initial* eviction, while the hot tier's role is to *retain* items deemed worth keeping; on workloads with skewed re-access patterns, it is more valuable to protect the hot tier from overflow than to hedge with extra cold-tier capacity.
