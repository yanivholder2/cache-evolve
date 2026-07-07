# Variant v002 Insight

## Helped

- **hit-rate**: Partitioning the stash into hot and cold tiers with promotion-on-hit concentrates re-accessed items in the hot tier, preserving cache capacity for items demonstrating repeated utility rather than spreading it uniformly across all evicted items.

## Hurt

(None measured in this evaluation)

## Takeaway

Stratifying the stash by promotion on secondary-lookup hit improves hit rate by creating an implicit priority layer. Items evicted once and then re-accessed (proven candidates for reuse) move to the hot tier, while newly evicted items remain in the cold tier. This learned differentiation outperforms uniform FIFO treatment, showing that organizing stash capacity orthogonal to insertion order—rather than filtering what to capture—yields measurable gains on workloads with re-access patterns.
