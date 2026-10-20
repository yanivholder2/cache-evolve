# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Replacing uniform decay + LRU-front eviction with selective decay + first-cold eviction degraded hit-rate, suggesting that zeroing only the minimum-counter items removes fewer items from contention per warm-lock event and that the LRU-front (index-0) tie-breaker in the parent was providing temporal-locality benefit that the frequency-ordered cold scan does not replicate.

## Takeaway

Selective decay preserves relative frequency ranking among non-floor items as intended, but the net effect on this workload is a slight drop in hit-rate compared to uniform decay. The parent's strategy of decrementing all counters by one and then evicting the oldest-inserted item (index 0) is implicitly an aging + recency hybrid; the trace's temporal locality rewards that recency signal. Selective decay discards the recency tie-breaker — evicting whichever item first hits counter zero regardless of insertion age — and that loss outweighs any benefit from tighter frequency ordering among the warmer items.
