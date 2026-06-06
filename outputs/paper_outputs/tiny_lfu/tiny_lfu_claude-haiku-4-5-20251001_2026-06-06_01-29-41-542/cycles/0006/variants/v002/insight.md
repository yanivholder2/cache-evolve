# Variant v002 Insight

## Helped

(No dimensions improved)

## Hurt

- hit-rate: Size penalty in victim selection degraded cache correctness by allowing small, hot items to be evicted ahead of large, cold ones, undermining the frequency-age signal.

## Takeaway

Incorporating item size into victim scoring via a multiplicative penalty conflicts with the frequency-age logic that preserves hot items. The approach penalizes large items for eviction (lower scores), but when a large item is cold and a small item is hot, the size penalty can override frequency, causing the cache to evict the hot small item instead. This directly hurts hit-rate and suggests that size optimization must be decoupled from or subordinate to the frequency signal.
