# Variant v003 Insight

## Helped

- hit-rate: The combination of SLRU 90/10 segmentation and the k=4 minimum-frequency probationary scan reduced premature eviction of warm items that happen to sit at the LRU head — the protected tier shields the most-accessed items from eviction pressure while the frequency tiebreaker rescues recently-demoted or high-frequency probationary entries that strict LRU would have removed first.

## Hurt

## Takeaway

Replacing the baseline's single-LRU eviction path with SLRU segmentation plus a k=4 frequency-sampled victim scan yields a small but real hit-rate gain on this Zipfian workload. The gain is modest rather than large, which suggests that on this trace the LRU-head and the minimum-frequency candidate coincide most of the time — the frequency scan is earning its keep at the tail of the k=4 window, not dramatically reordering eviction choices. Unconditional promotion on any access (no PROMO_THRESHOLD) means the protected segment fills quickly, so the primary benefit of the scan is protecting high-frequency probationary survivors rather than working around a sluggish promotion gate.
