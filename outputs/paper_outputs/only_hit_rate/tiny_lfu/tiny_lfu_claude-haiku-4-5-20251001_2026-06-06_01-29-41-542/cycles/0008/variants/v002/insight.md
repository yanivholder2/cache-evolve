# Variant v002 Insight

## Helped

- hit-rate: Age-cohort hierarchical victim selection preserves the active working set by enforcing an ordering constraint that exhausts genuinely old items before newer ones, preventing premature eviction of still-relevant entries.

## Hurt

(none)

## Takeaway

Imposing an age-based hierarchical structure on victim selection improves hit-rate by aligning eviction decisions with natural temporal distribution. Rather than scoring all items globally, restricting candidate evictions to the oldest age cohort (25% of cache) ensures that the algorithm targets stale items first, reducing the risk of evicting entries that remain part of the active working set. This modest but consistent improvement validates that age orthogonality (confirmed in prior cycles) can be operationalized through simple partitioning.
