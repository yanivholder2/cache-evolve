# Variant v002 Insight

## Helped

## Hurt

- per-op-time: Lowering the promotion gate from `freq > 1` to `freq >= 1` floods main with more entries, causing `evict_main()`'s unbounded second-chance loop to run more iterations per `put()`, which lifted both get and put p50 latency.
- hit-rate: The additional freq=1 entries promoted to main failed to yield extra hits — they displaced hotter entries via increased main churn, resulting in a marginal net loss rather than the expected gain.

## Takeaway

Widening the small-to-main promotion threshold to include freq=1 entries increases main queue pressure enough to slow down `evict_main()` without recovering the hit rate: on this Zipfian workload the freq=1 segment of the hot set is either too sparse or too transient to benefit from main's second-chance protection, and the extra churn it introduces costs more in latency than it returns in cache efficiency.
