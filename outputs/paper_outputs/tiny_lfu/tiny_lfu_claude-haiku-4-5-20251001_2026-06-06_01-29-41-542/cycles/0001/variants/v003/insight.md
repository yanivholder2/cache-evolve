# Variant v003 Insight

## Hurt

- hit-rate: Relaxing the admission gate to accept tied frequencies allows marginal items to evict hot residents; the strict `>` comparison was filtering out noise that the `>=` comparison admits.

## Takeaway

Accepting items with equal frequency estimates (tied with the LRU victim) harms hit rate. The hypothesis assumed recency could break ties favorably, but tied frequency estimates represent uncertainty in the sketch; the resident victim, despite equal frequency, is more likely to be genuinely hot or less noisy. The strict `>` gate's selectivity is a feature, not a limitation—it protects hot items from eviction to marginal newcomers when the frequency signal is ambiguous.
