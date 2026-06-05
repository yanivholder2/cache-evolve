# Variant v003 Insight

## Helped

- hit-rate: The count-min frequency sketch (4 rows × 16K saturating counters) plus the `estimate(key) >= estimate(victim)` admission gate on the full-arena put path rejects cold one-shot/scan keys, so frequently reused residents are no longer evicted by transient traffic.

## Hurt

- (none): no other dimension is measured in this cohort; hit-rate is the only tracked dimension and it moved upward.

## Takeaway

On this workload, gating eviction on estimated frequency rather than admitting every newcomer is the decisive lever: keeping the baseline's single LRU list and arena but refusing to displace a higher-frequency victim converts scan pollution into retained hits. The cheap sketch (one `bump` per access, periodic halving at `10 * capacity`) is enough signal to make admission decisions without restructuring the cache.
