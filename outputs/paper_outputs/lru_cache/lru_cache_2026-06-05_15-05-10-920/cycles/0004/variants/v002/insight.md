# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Halving the decay interval from `capacity * 16` to `capacity * 8` forces frequencies to reset twice as often, causing the cache to forget historical access patterns too aggressively and lose discrimination between hot and cold items before new patterns can properly stabilize.

## Takeaway

Doubling the frequency of periodic decay harms hit rate on this workload. The original `capacity * 16` interval provides better balance: it allows frequency values to accumulate and differentiate items sufficiently while still enabling adaptation to genuine phase shifts. More frequent decay (every `capacity * 8` operations) overshoots the workload's actual need for forgetting and causes the cache to lose its ability to reliably identify truly frequently-accessed items. The phase shifts in this workload are gradual enough that less aggressive decay is preferable.
