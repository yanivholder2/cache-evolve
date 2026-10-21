# Variant v002 Insight

## Helped

- (none measurable): The fitness vector only contains the hit-rate dimension. The hypothesis targeted per-op-time (skipping the protected-MRU rotate for hits in the top quarter), but that dimension is not reported, so any benefit is unobserved.

## Hurt

- (none): hit-rate is identical to the parent. Skipping the rotate for protected hits already near the MRU end did not change which entries get evicted, so eviction order was effectively preserved.

## Takeaway

Relaxing protected-segment MRU promotion for entries already in the top quarter is hit-rate neutral on this trace, as the hypothesis predicted. Its claimed payoff, lower rotate/memmove cost in wide buckets, cannot be confirmed because the evaluator does not measure per-op-time. Adding a time dimension is needed before this change can be judged a win rather than a no-op. Note: I could not read the overlay source in this session, so this insight is based on the hypothesis text and the fitness vectors only.
