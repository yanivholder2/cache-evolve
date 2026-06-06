# Variant v003 Insight

## Helped

- per-op-time: Frequency-aware demotion gate dramatically reduces cascading evictions by bypassing cold items (lfu_counter < 2) directly to stash, eliminating unnecessary insertion overhead in the target tier and lowering contention on hot item operations.

## Hurt

- hit-rate: Aggressive cold-item eviction policy trades marginal hit-rate loss (0.14%) for latency gains, as items with proven frequency < 2 are evicted more quickly rather than given a second chance in the target tier.

## Takeaway

Frequency-aware demotion gating successfully demonstrates a new independent axis for cache tuning orthogonal to geometry and promotion thresholds. By leveraging the `lfu_counter` maintained by `FrequencyAwareLRU`, the variant gates the "one-more-chance" retry to only truly hot items, eliminating cascade depth for cold residents. This reduces per-operation latency by ~57% on get and ~31% on put while sacrificing less than 0.2% hit-rate, indicating that the frequency signal is a reliable predictor of demotion value on this workload.
