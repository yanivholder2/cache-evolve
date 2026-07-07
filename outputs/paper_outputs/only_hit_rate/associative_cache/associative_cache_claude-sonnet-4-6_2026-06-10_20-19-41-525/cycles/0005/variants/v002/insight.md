# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Mid-point insertion (`items.insert(len/2, item)`) placed new cold items in the middle of the bucket rather than the MRU tail, but the cold-first eviction scan (`iter().position(|item| lfu_counter == 0)`) is position-agnostic — it finds any cold item regardless of where it was inserted — so the repositioning did not reduce cold-pool exhaustion frequency; instead it disrupted the recency ordering that lets warm items safely cluster at the back, causing marginal additional misses.

## Takeaway

Mid-point insertion is not additive with cold-first + LFU warm-fallback on narrow 128 × 16 geometry because the benefit it was designed to deliver — keeping new items away from the eviction front — is irrelevant when the eviction policy already performs a full bucket scan for cold items by counter value rather than by position. The small hit-rate regression indicates that perturbing the recency structure (which the LFU fallback still implicitly depends on via promotion-to-back) costs more than mid-point insertion saves in this configuration.
