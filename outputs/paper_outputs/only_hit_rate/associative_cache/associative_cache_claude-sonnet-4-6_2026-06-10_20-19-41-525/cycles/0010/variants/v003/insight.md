# Variant v003 Insight

## Helped

## Hurt

- hit-rate: The post-decay cold re-scan displaced the LRU-front eviction target in warm-lock events, and selecting a positionally-interior item that decay happened to render cold (counter == 0) instead of the oldest item in the bucket (index 0) marginally degraded hit-rate on workload-a.

## Takeaway

Adding a re-scan after the bulk `saturating_sub(2)` decay to pick the first counter-0 item — rather than blindly evicting index 0 — did not improve cold-first precision in practice. The LRU front carries independent signal (it is the oldest insertion in the bucket) that is lost when a mid-point-inserted, counter-0 item is selected instead. The scenario the hypothesis targeted — where index 0 has a meaningfully higher counter than a mid-array item after decay — is apparently rare or short-lived enough on workload-a that the correction offers no benefit, and the cases where it fires instead discard a genuinely stale item in favour of a fresher one, costing the cache temporal locality.
