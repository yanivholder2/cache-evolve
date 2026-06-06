# Variant v002 Insight

## Helped

- hit-rate: Splitting the single LRU list into a recency list (T1) and a frequency list (T2), with re-access promoting keys to T2 MRU, protects reused keys from the recency-driven eviction that plain LRU applies — lifting hit-rate over the baseline parent.
- hit-rate: The B1/B2 ghost lists steer the target `p` from both recency and frequency miss feedback, so the T1/T2 boundary self-tunes to the trace instead of staying fixed as in the seed's single ordering.

## Hurt

- (no measured dimension regressed; the fitness vector tracks only hit-rate, and ARC's extra ghost arena and bookkeeping carry no measured cost here.)

## Takeaway

Adding ARC's frequency dimension (T2) and ghost-driven adaptation on top of the baseline's single LRU ordering exposes a real, exploitable structural axis on this trace: separating recency from frequency keeps hot keys resident through recency churn, which plain LRU could not do.
